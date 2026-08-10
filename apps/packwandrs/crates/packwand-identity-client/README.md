# Packwand Identity Client

This crate keeps Packwand's Rust CLI and desktop shell behind a small typed
boundary to the local `packwand-social` process. OAuth secrets remain in the Go
helper; Rust receives public identity information and ATProto record results.
It also provides the shared pack/snippet/image publishing, friend discovery,
collaboration invite, and Tangled lookup operations used by both the CLI and
Tauri shell.
