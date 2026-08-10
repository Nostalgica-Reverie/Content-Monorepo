//! Blocking, Noise-encrypted collaboration transport.
//!
//! The webview never receives a network capability. A listener or dialer lives
//! here on ordinary threads and exposes only the closed [`Message`] protocol
//! to the Tauri command layer.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::invite::PSK_BYTES;
use crate::protocol::{Frame, FrameError, Message};

const NOISE_PATTERN: &str = "Noise_NNpsk0_25519_ChaChaPoly_BLAKE2s";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const MAX_NOISE_PLAINTEXT: usize = 60 * 1024;
const MAX_NOISE_RECORD: usize = MAX_NOISE_PLAINTEXT + 1024;

/// Which side of a collaboration link this process owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
	Host,
	Guest,
}

/// A lifecycle or application event produced by a session thread.
#[derive(Debug, Clone)]
pub enum TransportEvent {
	Connected,
	Disconnected,
	Message(Message),
	Error(String),
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
	#[error("address did not resolve")]
	Address,
	#[error("noise setup failed: {0}")]
	NoiseSetup(String),
	#[error("noise handshake failed: {0}")]
	Handshake(String),
	#[error("session is disconnected")]
	Disconnected,
	#[error("request timed out")]
	Timeout,
	#[error("request message has no correlation id")]
	NotARequest,
	#[error("session state lock was poisoned")]
	Poisoned,
	#[error(transparent)]
	Frame(#[from] FrameError),
	#[error(transparent)]
	Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RequestKey {
	Fs(u64),
	Git(u64),
}

type EventHandler = Arc<dyn Fn(TransportEvent) + Send + Sync + 'static>;

struct SessionInner {
	role: Role,
	local_addr: SocketAddr,
	sender: mpsc::Sender<Message>,
	stop: Arc<AtomicBool>,
	connected: Arc<AtomicBool>,
	active_stream: Arc<Mutex<Option<TcpStream>>>,
	pending: Arc<Mutex<HashMap<RequestKey, mpsc::Sender<Message>>>>,
	handler: Arc<RwLock<Option<EventHandler>>>,
	backlog: Arc<Mutex<Vec<TransportEvent>>>,
	threads: Mutex<Vec<JoinHandle<()>>>,
}

/// An active host listener or joined connection.
#[derive(Clone)]
pub struct SessionHandle {
	inner: Arc<SessionInner>,
}

/// Constructors for host and guest sessions.
pub struct Session;

impl Session {
	/// Binds a listener and starts accepting one guest at a time.
	pub fn host(
		bind_addr: impl ToSocketAddrs,
		psk: [u8; PSK_BYTES],
	) -> Result<SessionHandle, TransportError> {
		let listener = TcpListener::bind(bind_addr)?;
		listener.set_nonblocking(true)?;
		let local_addr = listener.local_addr()?;
		let (sender, receiver) = mpsc::channel();
		let handle = SessionHandle::new(Role::Host, local_addr, sender);
		let worker = handle.clone();
		let join = thread::Builder::new()
			.name("packwand-collab-host".into())
			.spawn(move || host_loop(listener, receiver, psk, worker))?;
		handle
			.inner
			.threads
			.lock()
			.map_err(|_| TransportError::Poisoned)?
			.push(join);
		Ok(handle)
	}

	/// Dials and authenticates the host before returning a usable handle.
	pub fn join(
		addr: impl ToSocketAddrs,
		psk: [u8; PSK_BYTES],
	) -> Result<SessionHandle, TransportError> {
		let address = addr
			.to_socket_addrs()?
			.next()
			.ok_or(TransportError::Address)?;
		let mut stream = TcpStream::connect_timeout(&address, HANDSHAKE_TIMEOUT)?;
		configure_handshake_stream(&stream)?;
		let noise = handshake(&mut stream, &psk, true)?;
		configure_transport_stream(&stream)?;
		let local_addr = stream.local_addr()?;
		let (sender, receiver) = mpsc::channel();
		let handle = SessionHandle::new(Role::Guest, local_addr, sender);
		handle.inner.connected.store(true, Ordering::Release);
		let worker = handle.clone();
		let join = thread::Builder::new()
			.name("packwand-collab-guest".into())
			.spawn(move || run_connection(stream, noise, receiver, worker))?;
		handle
			.inner
			.threads
			.lock()
			.map_err(|_| TransportError::Poisoned)?
			.push(join);
		Ok(handle)
	}
}

impl SessionHandle {
	fn new(role: Role, local_addr: SocketAddr, sender: mpsc::Sender<Message>) -> Self {
		Self {
			inner: Arc::new(SessionInner {
				role,
				local_addr,
				sender,
				stop: Arc::new(AtomicBool::new(false)),
				connected: Arc::new(AtomicBool::new(false)),
				active_stream: Arc::new(Mutex::new(None)),
				pending: Arc::new(Mutex::new(HashMap::new())),
				handler: Arc::new(RwLock::new(None)),
				backlog: Arc::new(Mutex::new(Vec::new())),
				threads: Mutex::new(Vec::new()),
			}),
		}
	}

	pub fn role(&self) -> Role {
		self.inner.role
	}

	pub fn local_addr(&self) -> SocketAddr {
		self.inner.local_addr
	}

	pub fn is_connected(&self) -> bool {
		self.inner.connected.load(Ordering::Acquire)
	}

	/// Installs the application callback and replays events received before it.
	pub fn on_event(&self, handler: impl Fn(TransportEvent) + Send + Sync + 'static) {
		let handler: EventHandler = Arc::new(handler);
		if let Ok(mut slot) = self.inner.handler.write() {
			*slot = Some(handler.clone());
		}
		let backlog = self
			.inner
			.backlog
			.lock()
			.map(|mut events| std::mem::take(&mut *events))
			.unwrap_or_default();
		for event in backlog {
			handler(event);
		}
	}

	pub fn send(&self, message: Message) -> Result<(), TransportError> {
		if self.inner.stop.load(Ordering::Acquire) || !self.is_connected() {
			return Err(TransportError::Disconnected);
		}
		self.inner
			.sender
			.send(message)
			.map_err(|_| TransportError::Disconnected)
	}

	/// Sends an id-bearing request and waits for its matching response.
	pub fn request(&self, message: Message) -> Result<Message, TransportError> {
		let key = request_key(&message).ok_or(TransportError::NotARequest)?;
		let (sender, receiver) = mpsc::channel();
		self.inner
			.pending
			.lock()
			.map_err(|_| TransportError::Poisoned)?
			.insert(key, sender);
		if let Err(error) = self.send(message) {
			if let Ok(mut pending) = self.inner.pending.lock() {
				pending.remove(&key);
			}
			return Err(error);
		}
		match receiver.recv_timeout(REQUEST_TIMEOUT) {
			Ok(response) => Ok(response),
			Err(mpsc::RecvTimeoutError::Timeout) => {
				if let Ok(mut pending) = self.inner.pending.lock() {
					pending.remove(&key);
				}
				Err(TransportError::Timeout)
			}
			Err(mpsc::RecvTimeoutError::Disconnected) => Err(TransportError::Disconnected),
		}
	}

	/// Stops the listener/connection and joins its supervisor thread.
	pub fn shutdown(&self) {
		self.inner.stop.store(true, Ordering::Release);
		if let Ok(mut active) = self.inner.active_stream.lock()
			&& let Some(stream) = active.take()
		{
			let _ = stream.shutdown(Shutdown::Both);
		}
		let current = thread::current().id();
		let threads = self
			.inner
			.threads
			.lock()
			.map(|mut threads| std::mem::take(&mut *threads))
			.unwrap_or_default();
		for thread in threads {
			if thread.thread().id() != current {
				let _ = thread.join();
			}
		}
	}

	fn emit(&self, event: TransportEvent) {
		let handler = self
			.inner
			.handler
			.read()
			.ok()
			.and_then(|handler| handler.clone());
		if let Some(handler) = handler {
			handler(event);
		} else if let Ok(mut backlog) = self.inner.backlog.lock() {
			backlog.push(event);
		}
	}

	fn dispatch(&self, message: Message) {
		if let Some(key) = response_key(&message)
			&& let Ok(mut pending) = self.inner.pending.lock()
			&& let Some(sender) = pending.remove(&key)
		{
			let _ = sender.send(message);
			return;
		}
		self.emit(TransportEvent::Message(message));
	}
}

impl Drop for SessionInner {
	fn drop(&mut self) {
		self.stop.store(true, Ordering::Release);
		if let Ok(mut active) = self.active_stream.lock()
			&& let Some(stream) = active.take()
		{
			let _ = stream.shutdown(Shutdown::Both);
		}
	}
}

fn host_loop(
	listener: TcpListener,
	receiver: mpsc::Receiver<Message>,
	psk: [u8; PSK_BYTES],
	handle: SessionHandle,
) {
	let receiver = Arc::new(Mutex::new(receiver));
	while !handle.inner.stop.load(Ordering::Acquire) {
		match listener.accept() {
			Ok((mut stream, _)) => {
				if handle.is_connected() {
					let _ = stream.shutdown(Shutdown::Both);
					continue;
				}
				if let Err(error) = configure_handshake_stream(&stream) {
					handle.emit(TransportEvent::Error(error.to_string()));
					continue;
				}
				match handshake(&mut stream, &psk, false) {
					Ok(noise) => {
						if let Err(error) = configure_transport_stream(&stream) {
							handle.emit(TransportEvent::Error(error.to_string()));
							continue;
						}
						run_connection_shared(stream, noise, receiver.clone(), handle.clone());
					}
					Err(error) => handle.emit(TransportEvent::Error(error.to_string())),
				}
			}
			Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
				thread::sleep(POLL_INTERVAL);
			}
			Err(error) => {
				handle.emit(TransportEvent::Error(error.to_string()));
				break;
			}
		}
	}
}

fn run_connection(
	stream: TcpStream,
	noise: snow::TransportState,
	receiver: mpsc::Receiver<Message>,
	handle: SessionHandle,
) {
	run_connection_shared(stream, noise, Arc::new(Mutex::new(receiver)), handle);
}

fn run_connection_shared(
	stream: TcpStream,
	noise: snow::TransportState,
	receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
	handle: SessionHandle,
) {
	let active = match stream.try_clone() {
		Ok(active) => active,
		Err(error) => {
			handle.emit(TransportEvent::Error(error.to_string()));
			return;
		}
	};
	if let Ok(mut slot) = handle.inner.active_stream.lock() {
		*slot = Some(active);
	}
	handle.inner.connected.store(true, Ordering::Release);
	handle.emit(TransportEvent::Connected);

	let noise = Arc::new(Mutex::new(noise));
	let connection_stop = Arc::new(AtomicBool::new(false));
	let reader_stream = match stream.try_clone() {
		Ok(stream) => stream,
		Err(error) => {
			handle.emit(TransportEvent::Error(error.to_string()));
			return;
		}
	};
	let reader_handle = handle.clone();
	let reader_noise = noise.clone();
	let reader_stop = connection_stop.clone();
	let reader = thread::spawn(move || {
		let mut reader = NoiseReader::new(reader_stream, reader_noise);
		while !reader_handle.inner.stop.load(Ordering::Acquire)
			&& !reader_stop.load(Ordering::Acquire)
		{
			match Frame::decode(&mut reader) {
				Ok(message) => reader_handle.dispatch(message),
				Err(FrameError::Io(error))
					if matches!(
						error.kind(),
						std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
					) => {}
				Err(FrameError::Io(error))
					if matches!(
						error.kind(),
						std::io::ErrorKind::UnexpectedEof
							| std::io::ErrorKind::ConnectionReset
							| std::io::ErrorKind::ConnectionAborted
							| std::io::ErrorKind::BrokenPipe
					) =>
				{
					break;
				}
				Err(error) => {
					reader_handle.emit(TransportEvent::Error(error.to_string()));
					break;
				}
			}
		}
		reader_stop.store(true, Ordering::Release);
	});

	let writer_handle = handle.clone();
	let writer_stop = connection_stop.clone();
	let writer_noise = noise;
	let writer = thread::spawn(move || {
		let mut stream = stream;
		while !writer_handle.inner.stop.load(Ordering::Acquire)
			&& !writer_stop.load(Ordering::Acquire)
		{
			let next = receiver
				.lock()
				.ok()
				.and_then(|receiver| receiver.recv_timeout(POLL_INTERVAL).ok());
			let Some(message) = next else {
				continue;
			};
			if let Err(error) = write_message(&mut stream, &writer_noise, &message) {
				writer_handle.emit(TransportEvent::Error(error.to_string()));
				break;
			}
		}
		writer_stop.store(true, Ordering::Release);
		let _ = stream.shutdown(Shutdown::Both);
	});

	let _ = reader.join();
	connection_stop.store(true, Ordering::Release);
	if let Ok(active) = handle.inner.active_stream.lock()
		&& let Some(stream) = active.as_ref()
	{
		let _ = stream.shutdown(Shutdown::Both);
	}
	let _ = writer.join();
	if let Ok(mut active) = handle.inner.active_stream.lock() {
		active.take();
	}
	handle.inner.connected.store(false, Ordering::Release);
	if let Ok(mut pending) = handle.inner.pending.lock() {
		pending.clear();
	}
	handle.emit(TransportEvent::Disconnected);
}

fn configure_handshake_stream(stream: &TcpStream) -> Result<(), std::io::Error> {
	stream.set_nonblocking(false)?;
	stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT))?;
	stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT))?;
	stream.set_nodelay(true)
}

fn configure_transport_stream(stream: &TcpStream) -> Result<(), std::io::Error> {
	stream.set_nonblocking(false)?;
	stream.set_read_timeout(None)?;
	stream.set_write_timeout(Some(Duration::from_secs(5)))?;
	stream.set_nodelay(true)
}

fn handshake(
	stream: &mut TcpStream,
	psk: &[u8; PSK_BYTES],
	initiator: bool,
) -> Result<snow::TransportState, TransportError> {
	let params = NOISE_PATTERN
		.parse()
		.map_err(|error: snow::Error| TransportError::NoiseSetup(error.to_string()))?;
	let builder = snow::Builder::new(params)
		.psk(0, psk)
		.map_err(|error| TransportError::NoiseSetup(error.to_string()))?;
	let mut state = if initiator {
		builder.build_initiator()
	} else {
		builder.build_responder()
	}
	.map_err(|error| TransportError::NoiseSetup(error.to_string()))?;
	let mut payload = [0u8; 65535];
	if initiator {
		write_handshake(stream, &mut state, &mut payload)?;
		read_handshake(stream, &mut state, &mut payload)?;
	} else {
		read_handshake(stream, &mut state, &mut payload)?;
		write_handshake(stream, &mut state, &mut payload)?;
	}
	state
		.into_transport_mode()
		.map_err(|error| TransportError::Handshake(error.to_string()))
}

fn write_handshake(
	stream: &mut TcpStream,
	state: &mut snow::HandshakeState,
	buffer: &mut [u8],
) -> Result<(), TransportError> {
	let length = state
		.write_message(&[], buffer)
		.map_err(|error| TransportError::Handshake(error.to_string()))?;
	stream.write_all(&(length as u16).to_le_bytes())?;
	stream.write_all(&buffer[..length])?;
	stream.flush()?;
	Ok(())
}

fn read_handshake(
	stream: &mut TcpStream,
	state: &mut snow::HandshakeState,
	buffer: &mut [u8],
) -> Result<(), TransportError> {
	let mut length = [0u8; 2];
	stream.read_exact(&mut length)?;
	let length = u16::from_le_bytes(length) as usize;
	if length == 0 || length > buffer.len() {
		return Err(TransportError::Handshake("invalid handshake frame".into()));
	}
	let mut message = vec![0u8; length];
	stream.read_exact(&mut message)?;
	state
		.read_message(&message, buffer)
		.map_err(|error| TransportError::Handshake(error.to_string()))?;
	Ok(())
}

fn write_message(
	stream: &mut TcpStream,
	noise: &Arc<Mutex<snow::TransportState>>,
	message: &Message,
) -> Result<(), TransportError> {
	let frame = Frame::encode(message)?;
	for chunk in frame.chunks(MAX_NOISE_PLAINTEXT) {
		let mut encrypted = vec![0u8; chunk.len() + 1024];
		let length = noise
			.lock()
			.map_err(|_| TransportError::Poisoned)?
			.write_message(chunk, &mut encrypted)
			.map_err(|error| TransportError::Handshake(error.to_string()))?;
		stream.write_all(&(length as u32).to_le_bytes())?;
		stream.write_all(&encrypted[..length])?;
	}
	stream.flush()?;
	Ok(())
}

struct NoiseReader {
	stream: TcpStream,
	noise: Arc<Mutex<snow::TransportState>>,
	plain: Vec<u8>,
	offset: usize,
}

impl NoiseReader {
	const fn new(stream: TcpStream, noise: Arc<Mutex<snow::TransportState>>) -> Self {
		Self {
			stream,
			noise,
			plain: Vec::new(),
			offset: 0,
		}
	}

	fn read_record(&mut self) -> Result<(), std::io::Error> {
		let mut length = [0u8; 4];
		self.stream.read_exact(&mut length)?;
		let length = u32::from_le_bytes(length) as usize;
		if length == 0 || length > MAX_NOISE_RECORD {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("invalid encrypted record length {length}"),
			));
		}
		let mut encrypted = vec![0u8; length];
		self.stream.read_exact(&mut encrypted)?;
		let mut plain = vec![0u8; length];
		let plain_length = self
			.noise
			.lock()
			.map_err(|_| std::io::Error::other("noise lock was poisoned"))?
			.read_message(&encrypted, &mut plain)
			.map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
		plain.truncate(plain_length);
		self.plain = plain;
		self.offset = 0;
		Ok(())
	}
}

impl Read for NoiseReader {
	fn read(&mut self, buffer: &mut [u8]) -> Result<usize, std::io::Error> {
		if self.offset >= self.plain.len() {
			self.read_record()?;
		}
		let available = &self.plain[self.offset..];
		let length = available.len().min(buffer.len());
		buffer[..length].copy_from_slice(&available[..length]);
		self.offset += length;
		Ok(length)
	}
}

const fn request_key(message: &Message) -> Option<RequestKey> {
	match message {
		Message::FsRequest { id, .. } => Some(RequestKey::Fs(*id)),
		Message::GitRequest { id, .. } => Some(RequestKey::Git(*id)),
		_ => None,
	}
}

const fn response_key(message: &Message) -> Option<RequestKey> {
	match message {
		Message::FsResponse { id, .. } => Some(RequestKey::Fs(*id)),
		Message::GitResponse { id, .. } => Some(RequestKey::Git(*id)),
		_ => None,
	}
}
