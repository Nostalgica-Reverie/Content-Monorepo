use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

use packwand_collab::protocol::{FsRequest, PROTOCOL_VERSION, Participant, SessionInfo};
use packwand_collab::{Message, ParticipantId, Session, TransportEvent};

fn unused_address() -> std::net::SocketAddr {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	listener.local_addr().unwrap()
}

fn participant(id: u64, name: &str) -> Participant {
	Participant {
		id: ParticipantId(id),
		display_name: name.into(),
		git_name: name.into(),
		git_email: format!("{name}@example.com"),
	}
}

#[test]
fn loopback_sessions_exchange_early_phase_messages() {
	let psk = [7u8; 32];
	let host = Session::host("127.0.0.1:0", psk).unwrap();
	let (host_events_tx, host_events_rx) = mpsc::channel();
	let responder = host.clone();
	host.on_event(move |event| {
		if let TransportEvent::Message(Message::FsRequest { id, .. }) = &event {
			let _ = responder.send(Message::FsResponse {
				id: *id,
				result: Ok(serde_json::json!({ "ok": true })),
			});
		}
		let _ = host_events_tx.send(event);
	});
	let guest = Session::join(host.local_addr(), psk).unwrap();
	let (guest_events_tx, guest_events_rx) = mpsc::channel();
	guest.on_event(move |event| {
		let _ = guest_events_tx.send(event);
	});

	guest
		.send(Message::Hello {
			participant: participant(2, "guest"),
			protocol: PROTOCOL_VERSION,
		})
		.unwrap();
	assert!(matches!(
		host_events_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
		TransportEvent::Connected
	));

	let response = guest
		.request(Message::FsRequest {
			id: 44,
			request: FsRequest {
				method: "editor_fs_stat".into(),
				parameters: serde_json::json!({}),
			},
		})
		.unwrap();
	assert!(matches!(response, Message::FsResponse { id: 44, .. }));
	let hello = host_events_rx.recv_timeout(Duration::from_secs(2)).unwrap();
	assert!(
		matches!(hello, TransportEvent::Message(Message::Hello { .. })),
		"unexpected event: {hello:?}"
	);

	host.send(Message::Welcome {
		session: SessionInfo {
			pack_id: "example".into(),
			pack_name: "Example".into(),
			allow_git_write: true,
		},
		participants: vec![participant(1, "host"), participant(2, "guest")],
	})
	.unwrap();
	host.send(Message::ParticipantJoined(participant(2, "guest")))
		.unwrap();
	assert!(matches!(
		guest_events_rx
			.recv_timeout(Duration::from_secs(2))
			.unwrap(),
		TransportEvent::Connected
	));
	assert!(matches!(
		guest_events_rx
			.recv_timeout(Duration::from_secs(2))
			.unwrap(),
		TransportEvent::Message(Message::Welcome { .. })
	));
	assert!(matches!(
		guest_events_rx
			.recv_timeout(Duration::from_secs(2))
			.unwrap(),
		TransportEvent::Message(Message::ParticipantJoined(_))
	));

	guest.shutdown();
	host.shutdown();
}

#[test]
fn a_wrong_psk_fails_before_any_message_is_decoded() {
	let address = unused_address();
	let host = Session::host(address, [1u8; 32]).unwrap();
	let (events_tx, events_rx) = mpsc::channel();
	host.on_event(move |event| {
		let _ = events_tx.send(event);
	});
	let joined = Session::join(address, [2u8; 32]);
	assert!(joined.is_err());

	let event = events_rx.recv_timeout(Duration::from_secs(2)).unwrap();
	assert!(matches!(event, TransportEvent::Error(_)));
	assert!(
		events_rx
			.try_iter()
			.all(|event| !matches!(event, TransportEvent::Message(_)))
	);
	host.shutdown();
}
