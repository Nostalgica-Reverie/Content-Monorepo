//! Turning game output into structured log lines.
//!
//! Minecraft emits two shapes depending on how it was configured. With the
//! log4j XML layout — which vanilla enables through
//! `-Dlog4j.configurationFile` — each entry is a `<log4j:Event>` element
//! carrying its logger, level, thread and timestamp. Without it, output is
//! plain text in the familiar `[12:34:56] [Render thread/INFO]: …` form.
//! Both arrive on the same pipe, sometimes in the same run, so one parser
//! reads both.
//!
//! The property worth naming is **sticky level**: a Java stack trace is a
//! block of lines of which only the first is labelled. Treating the
//! continuation lines as their own unlabelled entries would file the actual
//! exception under INFO and hide it from anyone filtering for errors, so a
//! line that does not announce its own level inherits the previous one's.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Severity of one log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
	Trace,
	Debug,
	#[default]
	Info,
	Warn,
	Error,
	Fatal,
}

impl LogLevel {
	/// Parses a log4j level name, case-insensitively.
	pub fn parse(name: &str) -> Option<Self> {
		match name.trim().to_ascii_uppercase().as_str() {
			"TRACE" | "FINEST" => Some(Self::Trace),
			"DEBUG" | "FINE" => Some(Self::Debug),
			"INFO" | "CONFIG" => Some(Self::Info),
			"WARN" | "WARNING" => Some(Self::Warn),
			"ERROR" | "SEVERE" => Some(Self::Error),
			"FATAL" => Some(Self::Fatal),
			_ => None,
		}
	}

	/// Whether this level denotes a problem.
	pub fn is_problem(self) -> bool {
		self >= Self::Warn
	}
}

/// One parsed entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogLine {
	/// Emitting logger, when the entry named one.
	pub logger: Option<String>,
	pub level: LogLevel,
	/// Milliseconds since the Unix epoch, when the entry carried one.
	pub timestamp_ms: Option<u64>,
	/// Emitting thread, when the entry named one.
	pub thread: Option<String>,
	pub message: String,
	/// Whether the level was inherited from the preceding entry rather than
	/// stated. Lets a UI show a stack trace as one block.
	pub inherited_level: bool,
}

/// Incremental parser over a game's output stream.
///
/// Feed it whatever arrives; it holds back only a partial XML element and
/// returns every entry it can complete.
#[derive(Debug, Default)]
pub struct LogParser {
	pending: String,
	last_level: Option<LogLevel>,
}

const EVENT_OPEN: &str = "<log4j:Event";
const EVENT_CLOSE: &str = "</log4j:Event>";

impl LogParser {
	/// A parser with no buffered input.
	pub fn new() -> Self {
		Self::default()
	}

	/// Consumes a chunk of output and returns the entries it completes.
	pub fn feed(&mut self, chunk: &str) -> Vec<LogLine> {
		self.pending.push_str(chunk);
		let mut out = Vec::new();
		loop {
			let Some(start) = self.pending.find(EVENT_OPEN) else {
				// No XML in sight: everything whole is plain text.
				let text = std::mem::take(&mut self.pending);
				let (complete, rest) = split_complete_lines(&text);
				self.pending = rest;
				self.push_plain(&complete, &mut out);
				break;
			};
			// Plain text before the element is still log output.
			if start > 0 {
				let head = self.pending[..start].to_string();
				self.push_plain(&head, &mut out);
			}
			let Some(end) = self.pending[start..].find(EVENT_CLOSE) else {
				// Element still arriving; keep it for the next chunk.
				self.pending = self.pending[start..].to_string();
				break;
			};
			let element = self.pending[start..start + end + EVENT_CLOSE.len()].to_string();
			self.pending = self.pending[start + end + EVENT_CLOSE.len()..].to_string();
			if let Some(line) = self.parse_event(&element) {
				self.last_level = Some(line.level);
				out.push(line);
			}
		}
		out
	}

	/// Returns whatever is still buffered, for the end of a stream.
	pub fn flush(&mut self) -> Vec<LogLine> {
		let text = std::mem::take(&mut self.pending);
		let mut out = Vec::new();
		if !text.trim().is_empty() {
			self.push_plain(&text, &mut out);
		}
		out
	}

	fn push_plain(&mut self, text: &str, out: &mut Vec<LogLine>) {
		for raw in text.lines() {
			if raw.trim().is_empty() {
				continue;
			}
			let line = self.parse_plain(raw);
			self.last_level = Some(line.level);
			out.push(line);
		}
	}

	/// Reads `[12:34:56] [Render thread/INFO]: message`, falling back to an
	/// unlabelled line that inherits the level above it.
	fn parse_plain(&self, raw: &str) -> LogLine {
		if let Some(rest) = raw.strip_prefix('[')
			&& let Some(close) = rest.find(']')
		{
			let after_time = rest[close + 1..].trim_start();
			if let Some(tagged) = after_time.strip_prefix('[')
				&& let Some(tag_end) = tagged.find(']')
			{
				let tag = &tagged[..tag_end];
				if let Some((thread, level_name)) = tag.rsplit_once('/')
					&& let Some(level) = LogLevel::parse(level_name)
				{
					let message = tagged[tag_end + 1..]
						.trim_start()
						.trim_start_matches(':')
						.trim_start();
					return LogLine {
						logger: None,
						level,
						timestamp_ms: None,
						thread: Some(thread.to_string()),
						message: message.to_string(),
						inherited_level: false,
					};
				}
			}
		}
		LogLine {
			logger: None,
			// The stack-trace case: no level of its own, so it keeps the
			// level of the entry it belongs to.
			level: self.last_level.unwrap_or_default(),
			timestamp_ms: None,
			thread: None,
			message: raw.trim_end().to_string(),
			inherited_level: self.last_level.is_some(),
		}
	}

	fn parse_event(&self, element: &str) -> Option<LogLine> {
		let level = attribute(element, "level")
			.and_then(|v| LogLevel::parse(&v))
			.unwrap_or_else(|| self.last_level.unwrap_or_default());
		let mut message = cdata_of(element, "log4j:Message").unwrap_or_default();
		// A throwable is part of the same entry; keeping it attached is what
		// makes the level stick to the whole stack trace.
		if let Some(throwable) = cdata_of(element, "log4j:Throwable")
			&& !throwable.trim().is_empty()
		{
			if !message.is_empty() {
				message.push('\n');
			}
			message.push_str(throwable.trim_end());
		}
		Some(LogLine {
			logger: attribute(element, "logger"),
			level,
			timestamp_ms: attribute(element, "timestamp").and_then(|v| v.parse().ok()),
			thread: attribute(element, "thread"),
			message,
			inherited_level: false,
		})
	}
}

/// Splits text into the part ending at the last newline and the remainder.
fn split_complete_lines(text: &str) -> (String, String) {
	match text.rfind('\n') {
		Some(at) => (text[..=at].to_string(), text[at + 1..].to_string()),
		None => (String::new(), text.to_string()),
	}
}

/// Reads `name="value"` from an element's opening tag.
fn attribute(element: &str, name: &str) -> Option<String> {
	let needle = format!("{name}=\"");
	let start = element.find(&needle)? + needle.len();
	let end = element[start..].find('"')? + start;
	Some(unescape_xml(&element[start..end]))
}

/// Reads the CDATA payload of a child element.
fn cdata_of(element: &str, tag: &str) -> Option<String> {
	let open = format!("<{tag}>");
	let close = format!("</{tag}>");
	let start = element.find(&open)? + open.len();
	let end = element[start..].find(&close)? + start;
	let inner = &element[start..end];
	let inner = inner
		.trim()
		.strip_prefix("<![CDATA[")
		.and_then(|s| s.strip_suffix("]]>"))
		.unwrap_or(inner);
	Some(unescape_xml(inner))
}

fn unescape_xml(value: &str) -> String {
	value
		.replace("&lt;", "<")
		.replace("&gt;", ">")
		.replace("&quot;", "\"")
		.replace("&apos;", "'")
		.replace("&amp;", "&")
}

/// A bounded window over the most recent log lines.
///
/// Fixed capacity on purpose: a modpack that logs a warning per tick will
/// produce gigabytes over an afternoon, and a launcher that keeps all of it
/// in memory to render a scrollback is a launcher that dies before the game
/// does.
#[derive(Debug)]
pub struct LogBuffer {
	lines: VecDeque<LogLine>,
	capacity: usize,
	dropped: usize,
}

impl LogBuffer {
	/// A buffer holding at most `capacity` lines.
	pub fn new(capacity: usize) -> Self {
		Self {
			lines: VecDeque::with_capacity(capacity.min(1024)),
			capacity: capacity.max(1),
			dropped: 0,
		}
	}

	/// Appends one line, evicting the oldest when full.
	pub fn push(&mut self, line: LogLine) {
		if self.lines.len() == self.capacity {
			self.lines.pop_front();
			self.dropped += 1;
		}
		self.lines.push_back(line);
	}

	/// The retained lines, oldest first.
	pub fn lines(&self) -> impl Iterator<Item = &LogLine> {
		self.lines.iter()
	}

	/// How many lines were evicted, so a UI can say so rather than implying
	/// the log started where the buffer does.
	pub fn dropped(&self) -> usize {
		self.dropped
	}

	/// Number of retained lines.
	pub fn len(&self) -> usize {
		self.lines.len()
	}

	/// Whether nothing has been retained.
	pub fn is_empty(&self) -> bool {
		self.lines.is_empty()
	}
}

/// Parses `<game_dir>/logs/latest.log`, when it exists.
///
/// Reading the file rather than only the live pipe is what makes a log
/// available after a crash the launcher did not witness — an instance
/// launched externally, or a session from before the app was opened.
pub fn read_latest_log(logs_dir: &Path) -> Option<Vec<LogLine>> {
	let text = std::fs::read_to_string(logs_dir.join("latest.log")).ok()?;
	let mut parser = LogParser::new();
	let mut lines = parser.feed(&text);
	lines.extend(parser.flush());
	Some(lines)
}

/// The newest file in `<game_dir>/crash-reports`, if any.
///
/// A crash report is the artifact a user is asked for, and it is written by
/// the game rather than printed, so nothing on the output pipe points at it.
pub fn latest_crash_report(game_dir: &Path) -> Option<PathBuf> {
	let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
	for entry in std::fs::read_dir(game_dir.join("crash-reports"))
		.ok()?
		.flatten()
	{
		let path = entry.path();
		if path.extension().is_none_or(|e| e != "txt") {
			continue;
		}
		let Ok(modified) = entry.metadata().and_then(|m| m.modified()) else {
			continue;
		};
		if newest.as_ref().is_none_or(|(t, _)| modified > *t) {
			newest = Some((modified, path));
		}
	}
	newest.map(|(_, path)| path)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_log4j_event_is_parsed_into_its_fields() {
		let mut parser = LogParser::new();
		let lines = parser.feed(
			r#"<log4j:Event logger="net.minecraft.client.Minecraft" timestamp="1700000000000" level="INFO" thread="Render thread">
	<log4j:Message><![CDATA[Setting user: Notch]]></log4j:Message>
</log4j:Event>
"#,
		);
		assert_eq!(lines.len(), 1);
		let line = &lines[0];
		assert_eq!(
			line.logger.as_deref(),
			Some("net.minecraft.client.Minecraft")
		);
		assert_eq!(line.level, LogLevel::Info);
		assert_eq!(line.timestamp_ms, Some(1_700_000_000_000));
		assert_eq!(line.thread.as_deref(), Some("Render thread"));
		assert_eq!(line.message, "Setting user: Notch");
	}

	#[test]
	fn an_event_split_across_chunks_is_held_until_complete() {
		// The stream arrives in pipe-sized pieces, not element-sized ones.
		let mut parser = LogParser::new();
		assert!(
			parser
				.feed("<log4j:Event level=\"ERROR\" thread=\"main\">\n<log4j:Mes")
				.is_empty()
		);
		let lines = parser.feed("sage><![CDATA[boom]]></log4j:Message>\n</log4j:Event>\n");
		assert_eq!(lines.len(), 1);
		assert_eq!(lines[0].level, LogLevel::Error);
		assert_eq!(lines[0].message, "boom");
	}

	#[test]
	fn a_throwable_stays_attached_to_its_event() {
		let mut parser = LogParser::new();
		let lines = parser.feed(
			r#"<log4j:Event logger="mod" level="ERROR" thread="main"><log4j:Message><![CDATA[Mod failed]]></log4j:Message><log4j:Throwable><![CDATA[java.lang.NullPointerException
	at com.example.Mod.init(Mod.java:42)]]></log4j:Throwable></log4j:Event>"#,
		);
		assert_eq!(lines.len(), 1);
		assert_eq!(lines[0].level, LogLevel::Error);
		assert!(lines[0].message.contains("Mod failed"));
		assert!(lines[0].message.contains("NullPointerException"));
		assert!(lines[0].message.contains("at com.example.Mod.init"));
	}

	#[test]
	fn plain_text_levels_are_read_from_the_thread_tag() {
		let mut parser = LogParser::new();
		let lines = parser.feed("[12:34:56] [Render thread/WARN]: Something is odd\n");
		assert_eq!(lines.len(), 1);
		assert_eq!(lines[0].level, LogLevel::Warn);
		assert_eq!(lines[0].thread.as_deref(), Some("Render thread"));
		assert_eq!(lines[0].message, "Something is odd");
		assert!(!lines[0].inherited_level);
	}

	#[test]
	fn a_stack_trace_keeps_the_level_of_the_line_that_introduced_it() {
		// The point of sticky level: filtering for errors has to show the
		// whole trace, not just its first line.
		let mut parser = LogParser::new();
		let lines = parser.feed(
			"[12:34:56] [main/ERROR]: Failed to load\n\
			 java.lang.IllegalStateException: nope\n\
			\tat com.example.A.b(A.java:1)\n\
			\tat com.example.C.d(C.java:2)\n",
		);
		assert_eq!(lines.len(), 4);
		assert!(lines.iter().all(|l| l.level == LogLevel::Error));
		assert!(!lines[0].inherited_level);
		assert!(lines[1..].iter().all(|l| l.inherited_level));
	}

	#[test]
	fn the_ring_buffer_drops_the_oldest_and_says_how_many() {
		let mut buffer = LogBuffer::new(3);
		for i in 0..5 {
			buffer.push(LogLine {
				logger: None,
				level: LogLevel::Info,
				timestamp_ms: None,
				thread: None,
				message: format!("line {i}"),
				inherited_level: false,
			});
		}
		assert_eq!(buffer.len(), 3);
		assert_eq!(buffer.dropped(), 2);
		let kept: Vec<&str> = buffer.lines().map(|l| l.message.as_str()).collect();
		assert_eq!(kept, ["line 2", "line 3", "line 4"]);
	}

	#[test]
	fn latest_log_and_crash_reports_are_read_from_disk() {
		let dir = tempfile::tempdir().unwrap();
		let logs = dir.path().join("logs");
		std::fs::create_dir_all(&logs).unwrap();
		std::fs::write(
			logs.join("latest.log"),
			"[12:34:56] [main/ERROR]: from the file\n",
		)
		.unwrap();
		let lines = read_latest_log(&logs).unwrap();
		assert_eq!(lines.len(), 1);
		assert_eq!(lines[0].message, "from the file");
		assert_eq!(lines[0].level, LogLevel::Error);

		assert!(latest_crash_report(dir.path()).is_none());
		let crashes = dir.path().join("crash-reports");
		std::fs::create_dir_all(&crashes).unwrap();
		std::fs::write(crashes.join("crash-2026-01-01.txt"), b"boom").unwrap();
		assert!(latest_crash_report(dir.path()).is_some());
	}
}
