//! Tauri boundary for host-authoritative live collaboration.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use packwand_collab::protocol::{
	ChangeKind, FsRequest, Message, PROTOCOL_VERSION, Participant, ParticipantId, RemoteError,
	Selection, SessionInfo,
};
use packwand_collab::{Invite, Role, Session, SessionHandle, TextOp, TransportEvent};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);
const HOST_ID: ParticipantId = ParticipantId(1);
const GUEST_ID: ParticipantId = ParticipantId(2);
const DOCUMENT_HISTORY_LIMIT: usize = 512;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollabIdentity {
	pub display_name: String,
	pub git_name: String,
	pub git_email: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollabRole {
	Host,
	Guest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollabConnection {
	Disconnected,
	Connecting,
	Connected,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollabState {
	pub role: Option<CollabRole>,
	pub participants: Vec<Participant>,
	pub connection: CollabConnection,
	pub session: Option<SessionInfo>,
	pub allow_git_write: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantEvent {
	pub event: String,
	pub participant: Option<Participant>,
	pub id: Option<ParticipantId>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceUpdate {
	pub origin: ParticipantId,
	pub path: Option<String>,
	pub selections: Vec<Selection>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DocumentUpdate {
	Open {
		path: String,
	},
	Close {
		path: String,
	},
	Snapshot {
		path: String,
		revision: u64,
		text: String,
	},
	Applied {
		path: String,
		revision: u64,
		ops: Vec<TextOp>,
		origin: ParticipantId,
	},
	Save {
		path: String,
	},
	FsChanged {
		path: String,
		kind: ChangeKind,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OutputLine {
	Output {
		channel: String,
		line: String,
	},
	Problems {
		snapshot: serde_json::Value,
	},
	JobEvent {
		event: String,
		payload: serde_json::Value,
	},
}

#[derive(Debug, Clone)]
struct AppliedRevision {
	revision: u64,
	ops: Vec<TextOp>,
}

#[derive(Debug, Clone, Default)]
struct DocumentState {
	revision: u64,
	text: String,
	history: VecDeque<AppliedRevision>,
}

impl DocumentState {
	fn snapshot(text: String) -> Self {
		Self {
			revision: 0,
			text,
			history: VecDeque::new(),
		}
	}

	fn apply(
		&mut self,
		base_revision: u64,
		incoming: &[TextOp],
		insert_wins_tie: bool,
	) -> CommandResult<(u64, Vec<TextOp>)> {
		if base_revision > self.revision {
			return Err(SerializableError::new(
				"collab_revision",
				format!(
					"base revision {base_revision} is ahead of {}",
					self.revision
				),
			));
		}
		if let Some(first) = self.history.front()
			&& base_revision < first.revision.saturating_sub(1)
		{
			return Err(SerializableError::new(
				"collab_revision",
				"the edit is older than the retained transform horizon",
			));
		}
		let applied = self
			.history
			.iter()
			.filter(|entry| entry.revision > base_revision)
			.flat_map(|entry| entry.ops.iter().cloned())
			.collect::<Vec<_>>();
		let transformed = incoming
			.iter()
			.flat_map(|operation| {
				packwand_collab::transform_all(operation, &applied, insert_wins_tie)
			})
			.collect::<Vec<_>>();
		for operation in &transformed {
			self.text = packwand_collab::apply(&self.text, operation);
		}
		self.revision = self.revision.saturating_add(1);
		self.history.push_back(AppliedRevision {
			revision: self.revision,
			ops: transformed.clone(),
		});
		while self.history.len() > DOCUMENT_HISTORY_LIMIT {
			self.history.pop_front();
		}
		Ok((self.revision, transformed))
	}
}

#[derive(Debug)]
struct Runtime {
	role: CollabRole,
	connection: CollabConnection,
	identity: CollabIdentity,
	session: Option<SessionInfo>,
	participants: BTreeMap<ParticipantId, Participant>,
	documents: BTreeMap<String, DocumentState>,
	edited: BTreeSet<ParticipantId>,
}

impl Runtime {
	fn state(&self) -> CollabState {
		CollabState {
			role: Some(self.role),
			participants: self.participants.values().cloned().collect(),
			connection: self.connection,
			session: self.session.clone(),
			allow_git_write: self
				.session
				.as_ref()
				.is_some_and(|session| session.allow_git_write),
		}
	}

	fn local_id(&self) -> ParticipantId {
		match self.role {
			CollabRole::Host => HOST_ID,
			CollabRole::Guest => GUEST_ID,
		}
	}

	fn remote_id(&self) -> ParticipantId {
		match self.role {
			CollabRole::Host => GUEST_ID,
			CollabRole::Guest => HOST_ID,
		}
	}

	fn co_authors(&self) -> Vec<Participant> {
		self.edited
			.iter()
			.filter(|id| **id != HOST_ID)
			.filter_map(|id| self.participants.get(id).cloned())
			.collect()
	}
}

#[derive(Clone)]
pub struct CollabHandle {
	transport: SessionHandle,
	runtime: Arc<Mutex<Runtime>>,
}

impl CollabHandle {
	fn state(&self) -> CommandResult<CollabState> {
		self.runtime
			.lock()
			.map(|runtime| runtime.state())
			.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))
	}
}

fn idle_state() -> CollabState {
	CollabState {
		role: None,
		participants: Vec::new(),
		connection: CollabConnection::Disconnected,
		session: None,
		allow_git_write: false,
	}
}

fn remote_error(error: SerializableError) -> RemoteError {
	RemoteError {
		kind: error.kind,
		message: error.message,
	}
}

fn transport_error(error: impl std::fmt::Display) -> SerializableError {
	SerializableError::new("collab_transport", error.to_string())
}

fn participant(id: ParticipantId, identity: &CollabIdentity) -> Participant {
	Participant {
		id,
		display_name: identity.display_name.clone(),
		git_name: identity.git_name.clone(),
		git_email: identity.git_email.clone(),
	}
}

fn default_identity(state: &AppState) -> CollabIdentity {
	let git = state
		.workspace()
		.ok()
		.or_else(|| std::env::current_dir().ok())
		.map(|workspace| crate::commands::git::resolved_identity(&workspace))
		.unwrap_or_default();
	let display_name = crate::commands::accounts::stored_modrinth_identity()
		.or_else(|| git.name.clone())
		.unwrap_or_else(|| "Packwand collaborator".into());
	CollabIdentity {
		git_name: git.name.unwrap_or_else(|| display_name.clone()),
		git_email: git.email.unwrap_or_default(),
		display_name,
	}
}

fn resolved_identity(state: &AppState) -> CommandResult<CollabIdentity> {
	let mut identity = state
		.collab_identity
		.lock()
		.map_err(|_| SerializableError::new("state", "identity lock was poisoned"))?;
	if identity.display_name.trim().is_empty() {
		*identity = default_identity(state);
	}
	Ok(identity.clone())
}

fn advertised_host() -> String {
	UdpSocket::bind("0.0.0.0:0")
		.and_then(|socket| {
			socket.connect("8.8.8.8:80")?;
			socket.local_addr()
		})
		.map(|address| address.ip().to_string())
		.unwrap_or_else(|_| "127.0.0.1".into())
}

fn current_handle(state: &AppState) -> CommandResult<CollabHandle> {
	state
		.collab
		.lock()
		.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?
		.clone()
		.ok_or_else(|| SerializableError::new("collab_disconnected", "no live session"))
}

pub(crate) fn broadcast_job_event(app: &AppHandle, event: &str, payload: serde_json::Value) {
	let state = app.state::<AppState>();
	let Ok(handle) = current_handle(&state) else {
		return;
	};
	if handle.transport.role() == Role::Host && handle.transport.is_connected() {
		let _ = handle.transport.send(Message::JobEvent {
			event: event.to_owned(),
			payload,
		});
	}
}

pub(crate) fn broadcast_fs_changes(app: &AppHandle, paths: &[String]) {
	let state = app.state::<AppState>();
	let Ok(handle) = current_handle(&state) else {
		return;
	};
	if handle.transport.role() != Role::Host || !handle.transport.is_connected() {
		return;
	}
	let pack_id = handle.runtime.lock().ok().and_then(|runtime| {
		runtime
			.session
			.as_ref()
			.map(|session| session.pack_id.clone())
	});
	let Some(pack_id) = pack_id else {
		return;
	};
	let prefix = pack_id.replace('\\', "/").trim_matches('/').to_owned();
	for path in paths {
		let normalized = path.replace('\\', "/").trim_start_matches('/').to_owned();
		let relative = if prefix == "." {
			Some(normalized)
		} else {
			normalized
				.strip_prefix(&format!("{prefix}/"))
				.map(str::to_owned)
		};
		if let Some(path) = relative.filter(|path| !path.is_empty()) {
			let _ = handle.transport.send(Message::FsChanged {
				path,
				kind: ChangeKind::Modified,
			});
		}
	}
}

pub(crate) fn commit_co_authors(state: &AppState) -> Vec<Participant> {
	current_handle(state)
		.ok()
		.and_then(|handle| {
			handle
				.runtime
				.lock()
				.ok()
				.map(|runtime| runtime.co_authors())
		})
		.unwrap_or_default()
}

pub(crate) fn clear_commit_co_authors(state: &AppState) {
	if let Ok(handle) = current_handle(state)
		&& let Ok(mut runtime) = handle.runtime.lock()
	{
		runtime.edited.clear();
	}
}

fn install_handler(app: &AppHandle, handle: &CollabHandle) {
	let app = app.clone();
	handle.transport.on_event(move |event| {
		handle_transport_event(&app, event);
	});
}

fn emit_state(app: &AppHandle, handle: &CollabHandle) {
	if let Ok(state) = handle.state() {
		let _ = crate::events::emit_collab_state(app, state);
	}
}

fn handle_transport_event(app: &AppHandle, event: TransportEvent) {
	let state = app.state::<AppState>();
	let Ok(handle) = current_handle(&state) else {
		return;
	};
	match event {
		TransportEvent::Connected => {
			if let Ok(mut runtime) = handle.runtime.lock()
				&& runtime.role == CollabRole::Host
			{
				runtime.connection = CollabConnection::Connected;
			}
			emit_state(app, &handle);
		}
		TransportEvent::Disconnected => {
			let mut left = None;
			if let Ok(mut runtime) = handle.runtime.lock() {
				runtime.connection = if runtime.role == CollabRole::Host {
					CollabConnection::Connecting
				} else {
					CollabConnection::Disconnected
				};
				let remote = runtime.remote_id();
				if runtime.participants.remove(&remote).is_some() {
					left = Some(remote);
				}
			}
			if let Some(id) = left {
				let _ = crate::events::emit_collab_participant(
					app,
					ParticipantEvent {
						event: "left".into(),
						participant: None,
						id: Some(id),
					},
				);
			}
			emit_state(app, &handle);
		}
		TransportEvent::Error(message) => {
			let _ = crate::events::emit_collab_output(
				app,
				OutputLine::Output {
					channel: "collaboration".into(),
					line: message,
				},
			);
		}
		TransportEvent::Message(message) => handle_message(app, &state, &handle, message),
	}
}

fn handle_message(app: &AppHandle, state: &AppState, handle: &CollabHandle, message: Message) {
	match message {
		Message::Hello {
			mut participant,
			protocol,
		} => handle_hello(app, handle, &mut participant, protocol),
		Message::Welcome {
			session,
			participants,
		} => {
			if let Ok(mut runtime) = handle.runtime.lock() {
				runtime.session = Some(session);
				runtime.participants = participants
					.into_iter()
					.map(|participant| (participant.id, participant))
					.collect();
				runtime.connection = CollabConnection::Connected;
			}
			emit_state(app, handle);
		}
		Message::ParticipantJoined(participant) => {
			if let Ok(mut runtime) = handle.runtime.lock() {
				runtime
					.participants
					.insert(participant.id, participant.clone());
			}
			let _ = crate::events::emit_collab_participant(
				app,
				ParticipantEvent {
					event: "joined".into(),
					participant: Some(participant),
					id: None,
				},
			);
			emit_state(app, handle);
		}
		Message::ParticipantLeft(id) => {
			if let Ok(mut runtime) = handle.runtime.lock() {
				runtime.participants.remove(&id);
			}
			let _ = crate::events::emit_collab_participant(
				app,
				ParticipantEvent {
					event: "left".into(),
					participant: None,
					id: Some(id),
				},
			);
			emit_state(app, handle);
		}
		Message::FsRequest { id, request } => handle_fs_request(app, state, handle, id, request),
		Message::FsChanged { path, kind } => {
			let _ =
				crate::events::emit_collab_document(app, DocumentUpdate::FsChanged { path, kind });
		}
		Message::GitRequest {
			id,
			method,
			parameters,
		} => handle_git_request(state, handle, id, method, parameters),
		Message::DocOpen { path } => {
			if validate_document_path(&path).is_ok() {
				let _ = crate::events::emit_collab_document(app, DocumentUpdate::Open { path });
			}
		}
		Message::DocClose { path } => {
			if validate_document_path(&path).is_ok() {
				let _ = crate::events::emit_collab_document(app, DocumentUpdate::Close { path });
			}
		}
		Message::DocSnapshot {
			path,
			revision,
			text,
		} => {
			if validate_document_path(&path).is_ok() {
				if let Ok(mut runtime) = handle.runtime.lock() {
					runtime.documents.insert(
						path.clone(),
						DocumentState {
							revision,
							text: text.clone(),
							history: VecDeque::new(),
						},
					);
				}
				let _ = crate::events::emit_collab_document(
					app,
					DocumentUpdate::Snapshot {
						path,
						revision,
						text,
					},
				);
			}
		}
		Message::DocEdit {
			path,
			base_revision,
			ops,
		} => handle_remote_edit(app, handle, path, base_revision, ops),
		Message::DocApplied {
			path,
			revision,
			ops,
			origin,
		} => {
			let _ = crate::events::emit_collab_document(
				app,
				DocumentUpdate::Applied {
					path,
					revision,
					ops,
					origin,
				},
			);
		}
		Message::DocSave { path } => {
			if validate_document_path(&path).is_ok() {
				let _ = crate::events::emit_collab_document(app, DocumentUpdate::Save { path });
			}
		}
		Message::Presence { path, selections } => {
			if path
				.as_deref()
				.is_none_or(|path| validate_document_path(path).is_ok())
			{
				let origin = handle
					.runtime
					.lock()
					.map(|runtime| runtime.remote_id())
					.unwrap_or(HOST_ID);
				let _ = crate::events::emit_collab_presence(
					app,
					PresenceUpdate {
						origin,
						path,
						selections,
					},
				);
			}
		}
		Message::FollowRequest { .. } => {}
		Message::Output { channel, line } => {
			let _ = crate::events::emit_collab_output(app, OutputLine::Output { channel, line });
		}
		Message::Problems { snapshot } => {
			let _ = crate::events::emit_collab_output(app, OutputLine::Problems { snapshot });
		}
		Message::JobEvent { event, payload } => {
			let _ = crate::events::emit_collab_output(app, OutputLine::JobEvent { event, payload });
		}
		Message::FsResponse { .. } | Message::GitResponse { .. } => {}
	}
}

fn handle_hello(
	app: &AppHandle,
	handle: &CollabHandle,
	participant: &mut Participant,
	protocol: u32,
) {
	if handle.transport.role() != Role::Host || protocol != PROTOCOL_VERSION {
		handle.transport.shutdown();
		return;
	}
	participant.id = GUEST_ID;
	participant.display_name = participant.display_name.trim().chars().take(80).collect();
	participant.git_name = participant.git_name.trim().chars().take(160).collect();
	participant.git_email = participant.git_email.trim().chars().take(320).collect();
	if participant.display_name.is_empty() {
		participant.display_name = "Guest".into();
	}
	let (session, participants) = match handle.runtime.lock() {
		Ok(mut runtime) => {
			runtime.participants.insert(GUEST_ID, participant.clone());
			(
				runtime.session.clone(),
				runtime.participants.values().cloned().collect::<Vec<_>>(),
			)
		}
		Err(_) => return,
	};
	let Some(session) = session else {
		return;
	};
	let _ = handle.transport.send(Message::Welcome {
		session,
		participants,
	});
	let _ = handle
		.transport
		.send(Message::ParticipantJoined(participant.clone()));
	let _ = crate::events::emit_collab_participant(
		app,
		ParticipantEvent {
			event: "joined".into(),
			participant: Some(participant.clone()),
			id: None,
		},
	);
	emit_state(app, handle);
}

fn handle_fs_request(
	app: &AppHandle,
	state: &AppState,
	handle: &CollabHandle,
	id: u64,
	request: FsRequest,
) {
	if handle.transport.role() != Role::Host {
		return;
	}
	let pinned_id = handle.runtime.lock().ok().and_then(|runtime| {
		runtime
			.session
			.as_ref()
			.map(|session| session.pack_id.clone())
	});
	let result = pinned_id
		.ok_or_else(|| SerializableError::new("collab_scope", "session has no pinned pack"))
		.and_then(|pinned_id| {
			crate::commands::editor::remote_fs_dispatch(&state.workspace()?, &pinned_id, &request)
		});
	match result {
		Ok(result) => {
			let _ = handle.transport.send(Message::FsResponse {
				id,
				result: Ok(result.value),
			});
			let _ = result.changes;
			let _ = crate::events::emit_packs_changed(app);
		}
		Err(error) => {
			let _ = handle.transport.send(Message::FsResponse {
				id,
				result: Err(remote_error(error)),
			});
		}
	}
}

fn handle_git_request(
	state: &AppState,
	handle: &CollabHandle,
	id: u64,
	method: String,
	parameters: serde_json::Value,
) {
	if handle.transport.role() != Role::Host {
		return;
	}
	let (allow_git_write, co_authors) = handle
		.runtime
		.lock()
		.map(|runtime| {
			(
				runtime
					.session
					.as_ref()
					.is_some_and(|session| session.allow_git_write),
				runtime.co_authors(),
			)
		})
		.unwrap_or_default();
	let result = state.workspace().and_then(|workspace| {
		crate::commands::git::remote_git_dispatch(
			&workspace,
			&method,
			&parameters,
			allow_git_write,
			&co_authors,
		)
	});
	if result.is_ok()
		&& method == "git_commit"
		&& let Ok(mut runtime) = handle.runtime.lock()
	{
		runtime.edited.clear();
	}
	let _ = handle.transport.send(Message::GitResponse {
		id,
		result: result.map_err(remote_error),
	});
}

fn validate_document_path(path: &str) -> CommandResult<()> {
	if path.is_empty() {
		return Err(SerializableError::new(
			"unsafe_path",
			"a document path cannot be empty",
		));
	}
	packwand_platform::validate_relative_path(path)
		.map_err(|error| SerializableError::new("unsafe_path", error.to_string()))
}

fn handle_remote_edit(
	app: &AppHandle,
	handle: &CollabHandle,
	path: String,
	base_revision: u64,
	ops: Vec<TextOp>,
) {
	if validate_document_path(&path).is_err() || handle.transport.role() != Role::Host {
		return;
	}
	let applied = handle.runtime.lock().ok().and_then(|mut runtime| {
		let origin = runtime.remote_id();
		let document = runtime.documents.get_mut(&path)?;
		let (revision, ops) = document.apply(base_revision, &ops, false).ok()?;
		runtime.edited.insert(origin);
		Some((origin, revision, ops))
	});
	let Some((origin, revision, ops)) = applied else {
		return;
	};
	let message = Message::DocApplied {
		path: path.clone(),
		revision,
		ops: ops.clone(),
		origin,
	};
	let _ = handle.transport.send(message);
	let _ = crate::events::emit_collab_document(
		app,
		DocumentUpdate::Applied {
			path,
			revision,
			ops,
			origin,
		},
	);
}

#[tauri::command]
pub fn collab_host_start(
	pack_id: String,
	allow_git_write: Option<bool>,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<String> {
	let pack_root = crate::commands::packs::pack_root(&state.workspace()?, &pack_id)?;
	let pack_name = std::fs::read_to_string(pack_root.join("pack.toml"))
		.ok()
		.and_then(|source| source.parse::<toml::Value>().ok())
		.and_then(|value| value.get("name")?.as_str().map(str::to_owned))
		.unwrap_or_else(|| pack_id.clone());
	let identity = resolved_identity(&state)?;
	let mut invite = Invite::generate("127.0.0.1", 1);
	let transport = Session::host("0.0.0.0:0", invite.psk).map_err(transport_error)?;
	invite.host = advertised_host();
	invite.port = transport.local_addr().port();
	let session = SessionInfo {
		pack_id,
		pack_name,
		allow_git_write: allow_git_write.unwrap_or(true),
	};
	let host = participant(HOST_ID, &identity);
	let runtime = Runtime {
		role: CollabRole::Host,
		connection: CollabConnection::Connecting,
		identity,
		session: Some(session),
		participants: [(HOST_ID, host)].into_iter().collect(),
		documents: BTreeMap::new(),
		edited: BTreeSet::new(),
	};
	let handle = CollabHandle {
		transport,
		runtime: Arc::new(Mutex::new(runtime)),
	};
	let previous = state
		.collab
		.lock()
		.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?
		.replace(handle.clone());
	if let Some(previous) = previous {
		previous.transport.shutdown();
	}
	install_handler(&app, &handle);
	emit_state(&app, &handle);
	Ok(invite.render())
}

#[tauri::command]
pub fn collab_host_stop(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
	let handle = state
		.collab
		.lock()
		.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?
		.take();
	if let Some(handle) = handle {
		if handle.transport.role() != Role::Host {
			return Err(SerializableError::new(
				"collab_role",
				"only a host can stop a hosted session",
			));
		}
		handle.transport.shutdown();
	}
	crate::events::emit_collab_state(&app, idle_state())
}

#[tauri::command]
pub fn collab_join(
	invite: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<CollabState> {
	let invite = Invite::parse(&invite).map_err(transport_error)?;
	let identity = resolved_identity(&state)?;
	let transport =
		Session::join((invite.host.as_str(), invite.port), invite.psk).map_err(transport_error)?;
	let local = participant(GUEST_ID, &identity);
	let runtime = Runtime {
		role: CollabRole::Guest,
		connection: CollabConnection::Connecting,
		identity,
		session: None,
		participants: [(GUEST_ID, local.clone())].into_iter().collect(),
		documents: BTreeMap::new(),
		edited: BTreeSet::new(),
	};
	let handle = CollabHandle {
		transport,
		runtime: Arc::new(Mutex::new(runtime)),
	};
	let previous = state
		.collab
		.lock()
		.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?
		.replace(handle.clone());
	if let Some(previous) = previous {
		previous.transport.shutdown();
	}
	install_handler(&app, &handle);
	handle
		.transport
		.send(Message::Hello {
			participant: local,
			protocol: PROTOCOL_VERSION,
		})
		.map_err(transport_error)?;
	emit_state(&app, &handle);
	handle.state()
}

#[tauri::command]
pub fn collab_leave(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
	let handle = state
		.collab
		.lock()
		.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?
		.take();
	if let Some(handle) = handle {
		let local_id = handle
			.runtime
			.lock()
			.map(|runtime| runtime.local_id())
			.unwrap_or(GUEST_ID);
		let _ = handle.transport.send(Message::ParticipantLeft(local_id));
		handle.transport.shutdown();
	}
	crate::events::emit_collab_state(&app, idle_state())
}

#[tauri::command]
pub fn collab_state(state: State<'_, AppState>) -> CommandResult<CollabState> {
	let handle = state
		.collab
		.lock()
		.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?
		.clone();
	handle.map_or_else(|| Ok(idle_state()), |handle| handle.state())
}

#[tauri::command]
pub fn collab_set_identity(
	display_name: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<CollabIdentity> {
	let display_name = display_name.trim().chars().take(80).collect::<String>();
	if display_name.is_empty() {
		return Err(SerializableError::new(
			"collab_identity",
			"enter a display name",
		));
	}
	let mut identity = resolved_identity(&state)?;
	identity.display_name = display_name;
	*state
		.collab_identity
		.lock()
		.map_err(|_| SerializableError::new("state", "identity lock was poisoned"))? = identity.clone();
	if let Ok(handle) = current_handle(&state) {
		let updated = if let Ok(mut runtime) = handle.runtime.lock() {
			runtime.identity = identity.clone();
			let id = runtime.local_id();
			let participant = participant(id, &identity);
			runtime.participants.insert(id, participant.clone());
			Some(participant)
		} else {
			None
		};
		if let Some(updated) = updated {
			let _ = handle
				.transport
				.send(Message::ParticipantJoined(updated.clone()));
			let _ = crate::events::emit_collab_participant(
				&app,
				ParticipantEvent {
					event: "updated".into(),
					participant: Some(updated),
					id: None,
				},
			);
			emit_state(&app, &handle);
		}
	}
	Ok(identity)
}

#[tauri::command]
pub fn collab_set_git_write(
	allow: bool,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<CollabState> {
	let handle = current_handle(&state)?;
	if handle.transport.role() != Role::Host {
		return Err(SerializableError::new(
			"collab_role",
			"only the host controls git write access",
		));
	}
	let (session, participants) = {
		let mut runtime = handle
			.runtime
			.lock()
			.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?;
		let session = runtime
			.session
			.as_mut()
			.ok_or_else(|| SerializableError::new("collab_state", "session is not ready"))?;
		session.allow_git_write = allow;
		(
			session.clone(),
			runtime.participants.values().cloned().collect(),
		)
	};
	if handle.transport.is_connected() {
		handle
			.transport
			.send(Message::Welcome {
				session,
				participants,
			})
			.map_err(transport_error)?;
	}
	emit_state(&app, &handle);
	handle.state()
}

#[tauri::command]
pub fn collab_fs_request(
	method: String,
	parameters: serde_json::Value,
	state: State<'_, AppState>,
) -> CommandResult<serde_json::Value> {
	let handle = current_handle(&state)?;
	if handle.transport.role() != Role::Guest {
		return Err(SerializableError::new(
			"collab_role",
			"filesystem proxy requests are guest-only",
		));
	}
	let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
	match handle
		.transport
		.request(Message::FsRequest {
			id,
			request: FsRequest { method, parameters },
		})
		.map_err(transport_error)?
	{
		Message::FsResponse { result, .. } => {
			result.map_err(|error| SerializableError::new(error.kind, error.message))
		}
		_ => Err(SerializableError::new(
			"collab_protocol",
			"host returned the wrong filesystem response",
		)),
	}
}

#[tauri::command]
pub fn collab_git_request(
	method: String,
	parameters: serde_json::Value,
	state: State<'_, AppState>,
) -> CommandResult<serde_json::Value> {
	let handle = current_handle(&state)?;
	if handle.transport.role() != Role::Guest {
		return Err(SerializableError::new(
			"collab_role",
			"git proxy requests are guest-only",
		));
	}
	let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
	match handle
		.transport
		.request(Message::GitRequest {
			id,
			method,
			parameters,
		})
		.map_err(transport_error)?
	{
		Message::GitResponse { result, .. } => {
			result.map_err(|error| SerializableError::new(error.kind, error.message))
		}
		_ => Err(SerializableError::new(
			"collab_protocol",
			"host returned the wrong git response",
		)),
	}
}

#[tauri::command]
pub fn collab_document_open(path: String, state: State<'_, AppState>) -> CommandResult<()> {
	validate_document_path(&path)?;
	current_handle(&state)?
		.transport
		.send(Message::DocOpen { path })
		.map_err(transport_error)
}

#[tauri::command]
pub fn collab_document_close(path: String, state: State<'_, AppState>) -> CommandResult<()> {
	validate_document_path(&path)?;
	current_handle(&state)?
		.transport
		.send(Message::DocClose { path })
		.map_err(transport_error)
}

#[tauri::command]
pub fn collab_document_snapshot(
	path: String,
	text: String,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	validate_document_path(&path)?;
	let handle = current_handle(&state)?;
	if handle.transport.role() != Role::Host {
		return Err(SerializableError::new(
			"collab_role",
			"only the host publishes document snapshots",
		));
	}
	let revision = {
		let mut runtime = handle
			.runtime
			.lock()
			.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?;
		let document = runtime
			.documents
			.entry(path.clone())
			.or_insert_with(|| DocumentState::snapshot(text.clone()));
		document.text = text.clone();
		document.revision
	};
	if handle.transport.is_connected() {
		handle
			.transport
			.send(Message::DocSnapshot {
				path,
				revision,
				text,
			})
			.map_err(transport_error)?;
	}
	Ok(())
}

#[tauri::command]
pub fn collab_document_edit(
	path: String,
	base_revision: u64,
	ops: Vec<TextOp>,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	validate_document_path(&path)?;
	let handle = current_handle(&state)?;
	if handle.transport.role() == Role::Guest {
		return handle
			.transport
			.send(Message::DocEdit {
				path,
				base_revision,
				ops,
			})
			.map_err(transport_error);
	}
	let origin = HOST_ID;
	let (revision, applied) = {
		let mut runtime = handle
			.runtime
			.lock()
			.map_err(|_| SerializableError::new("state", "collaboration lock was poisoned"))?;
		let document = runtime
			.documents
			.get_mut(&path)
			.ok_or_else(|| SerializableError::new("collab_document", "publish a snapshot first"))?;
		document.apply(base_revision, &ops, true)?
	};
	handle
		.transport
		.send(Message::DocApplied {
			path: path.clone(),
			revision,
			ops: applied.clone(),
			origin,
		})
		.map_err(transport_error)?;
	crate::events::emit_collab_document(
		&app,
		DocumentUpdate::Applied {
			path,
			revision,
			ops: applied,
			origin,
		},
	)
}

#[tauri::command]
pub fn collab_document_save(path: String, state: State<'_, AppState>) -> CommandResult<()> {
	validate_document_path(&path)?;
	current_handle(&state)?
		.transport
		.send(Message::DocSave { path })
		.map_err(transport_error)
}

#[tauri::command]
pub fn collab_presence(
	path: Option<String>,
	selections: Vec<Selection>,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	if let Some(path) = path.as_deref() {
		validate_document_path(path)?;
	}
	current_handle(&state)?
		.transport
		.send(Message::Presence { path, selections })
		.map_err(transport_error)
}

#[tauri::command]
pub fn collab_follow(target: ParticipantId, state: State<'_, AppState>) -> CommandResult<()> {
	current_handle(&state)?
		.transport
		.send(Message::FollowRequest { target })
		.map_err(transport_error)
}

#[tauri::command]
pub fn collab_output(
	channel: String,
	line: String,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	let handle = current_handle(&state)?;
	if handle.transport.role() != Role::Host {
		return Ok(());
	}
	if !handle.transport.is_connected() {
		return Ok(());
	}
	handle
		.transport
		.send(Message::Output { channel, line })
		.map_err(transport_error)
}

#[tauri::command]
pub fn collab_problems(
	snapshot: serde_json::Value,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	let handle = current_handle(&state)?;
	if handle.transport.role() != Role::Host {
		return Ok(());
	}
	if !handle.transport.is_connected() {
		return Ok(());
	}
	handle
		.transport
		.send(Message::Problems { snapshot })
		.map_err(transport_error)
}

#[tauri::command]
pub fn collab_job_event(
	event: String,
	payload: serde_json::Value,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	let handle = current_handle(&state)?;
	if handle.transport.role() != Role::Host {
		return Ok(());
	}
	if !handle.transport.is_connected() {
		return Ok(());
	}
	handle
		.transport
		.send(Message::JobEvent { event, payload })
		.map_err(transport_error)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn insert(offset: usize, text: &str) -> TextOp {
		TextOp::Insert {
			offset,
			text: text.into(),
		}
	}

	#[test]
	fn document_revisions_rebase_stale_edits_and_advance_once_per_batch() {
		let mut document = DocumentState::snapshot("ab".into());
		let (first_revision, _) = document.apply(0, &[insert(1, "H")], true).unwrap();
		let (second_revision, guest) = document.apply(0, &[insert(1, "G")], false).unwrap();
		assert_eq!((first_revision, second_revision), (1, 2));
		assert_eq!(guest, [insert(2, "G")]);
		assert_eq!(document.text, "aHGb");
	}

	#[test]
	fn a_future_revision_is_rejected() {
		let mut document = DocumentState::snapshot("text".into());
		assert_eq!(
			document
				.apply(1, &[insert(0, "x")], false)
				.unwrap_err()
				.kind,
			"collab_revision"
		);
	}

	#[test]
	fn document_history_is_bounded_and_old_edits_are_rejected() {
		let mut document = DocumentState::snapshot(String::new());
		for revision in 0..=DOCUMENT_HISTORY_LIMIT {
			document
				.apply(revision as u64, &[insert(0, "x")], true)
				.unwrap();
		}
		assert_eq!(document.history.len(), DOCUMENT_HISTORY_LIMIT);
		assert!(document.apply(0, &[insert(0, "old")], false).is_err());
	}

	#[test]
	fn unsafe_document_paths_are_rejected() {
		assert!(validate_document_path("../outside").is_err());
		assert!(validate_document_path("C:/outside").is_err());
		assert!(validate_document_path("config/settings.txt").is_ok());
	}

	fn workspace_with_two_packs() -> tempfile::TempDir {
		let workspace = tempfile::tempdir().unwrap();
		for id in ["shared", "sibling"] {
			let root = workspace.path().join(id);
			std::fs::create_dir_all(&root).unwrap();
			std::fs::write(root.join("pack.toml"), "name = \"test\"\n").unwrap();
			std::fs::write(root.join("safe.txt"), "safe").unwrap();
		}
		workspace
	}

	fn read_request(id: &str, path: &str) -> FsRequest {
		FsRequest {
			method: "editor_fs_read_file".into(),
			parameters: json!({ "id": id, "path": path }),
		}
	}

	#[test]
	fn remote_filesystem_requests_cannot_traverse_or_use_absolute_paths() {
		let workspace = workspace_with_two_packs();
		for path in ["../sibling/safe.txt", "C:/Windows/win.ini", "/etc/passwd"] {
			assert!(
				crate::commands::editor::remote_fs_dispatch(
					workspace.path(),
					"shared",
					&read_request("shared", path),
				)
				.is_err(),
				"path {path:?} escaped the session scope"
			);
		}
	}

	#[test]
	fn remote_filesystem_requests_cannot_name_a_sibling_pack() {
		let workspace = workspace_with_two_packs();
		let error = crate::commands::editor::remote_fs_dispatch(
			workspace.path(),
			"shared",
			&read_request("sibling", "safe.txt"),
		)
		.unwrap_err();
		assert_eq!(error.kind, "collab_scope");
	}

	#[test]
	fn dangerous_command_families_are_protocol_errors() {
		let workspace = workspace_with_two_packs();
		for method in [
			"accounts_state",
			"shell_exec",
			"settings_get",
			"packeater_run",
		] {
			let error = crate::commands::editor::remote_fs_dispatch(
				workspace.path(),
				"shared",
				&FsRequest {
					method: method.into(),
					parameters: json!({}),
				},
			)
			.unwrap_err();
			assert_eq!(error.kind, "collab_protocol");
		}
	}
}
