use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use packwand_installer::index::RemotePack;
use packwand_installer::plan::{InstallSide, build};
use packwand_pack::{HashFormat, Index, IndexFile, Mod, hash_bytes};
use packwand_providers::{HttpRequest, Transport, TransportError};

struct MappingTransport(BTreeMap<String, Vec<u8>>);

impl Transport for MappingTransport {
	fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
		self.0.get(&request.url).cloned().ok_or(TransportError {
			url: request.url,
			message: "missing fixture".into(),
			status: Some(404),
			body_snippet: None,
		})
	}
}

#[test]
fn loads_plans_and_applies_current_pack_format() {
	let artifact = b"verified mod jar".to_vec();
	let artifact_hash = hash_bytes(HashFormat::Sha256, &artifact);
	let metadata = Mod {
		name: "Example".into(),
		// Real pack metadata stores a bare filename; the mod's directory comes
		// from where its metafile (`mods/example.pw.json`) lives, not from
		// this field.
		filename: "example.jar".into(),
		download: packwand_pack::Download {
			url: "https://cdn.example/example.jar".into(),
			hash_format: "sha256".into(),
			hash: artifact_hash,
			..packwand_pack::Download::default()
		},
		..Mod::default()
	}
	.to_json_bytes()
	.unwrap();
	let index = serde_json::to_vec(&Index {
		hash_format: "sha256".into(),
		files: vec![IndexFile {
			file: "mods/example.pw.json".into(),
			hash: hash_bytes(HashFormat::Sha256, &metadata),
			metafile: true,
			..IndexFile::default()
		}],
	})
	.unwrap();
	let pack = format!(
		"name = \"Fixture\"\npack-format = \"packwand:27\"\n\n[index]\nfile = \"index.json\"\nhash-format = \"sha256\"\nhash = \"{}\"\n",
		hash_bytes(HashFormat::Sha256, &index)
	)
	.into_bytes();
	let transport = MappingTransport(BTreeMap::from([
		("https://packs.example/pack.toml".into(), pack),
		("https://packs.example/index.json".into(), index),
		(
			"https://packs.example/mods/example.pw.json".into(),
			metadata,
		),
		("https://cdn.example/example.jar".into(), artifact.clone()),
	]));
	let remote = RemotePack::load("https://packs.example/pack.toml", &transport).unwrap();
	let root = tempfile::tempdir().unwrap();
	let plan = build(&remote, root.path(), InstallSide::Client, &transport).unwrap();
	packwand_installer::download::apply(&plan, root.path(), &transport).unwrap();
	assert_eq!(
		std::fs::read(root.path().join("mods/example.jar")).unwrap(),
		artifact
	);
}

#[test]
fn native_binary_installs_a_pack_over_http() {
	let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
	listener.set_nonblocking(true).unwrap();
	let origin = format!("http://{}", listener.local_addr().unwrap());
	let artifact = b"binary-installed mod jar".to_vec();
	let metadata = Mod {
		name: "Binary Example".into(),
		filename: "binary-example.jar".into(),
		download: packwand_pack::Download {
			url: format!("{origin}/artifact.jar"),
			hash_format: "sha256".into(),
			hash: hash_bytes(HashFormat::Sha256, &artifact),
			..packwand_pack::Download::default()
		},
		..Mod::default()
	}
	.to_json_bytes()
	.unwrap();
	let index = serde_json::to_vec(&Index {
		hash_format: "sha256".into(),
		files: vec![IndexFile {
			file: "mods/binary-example.pw.json".into(),
			hash: hash_bytes(HashFormat::Sha256, &metadata),
			metafile: true,
			..IndexFile::default()
		}],
	})
	.unwrap();
	let pack = format!(
		"name = \"Binary Fixture\"\npack-format = \"packwand:27\"\n\n[index]\nfile = \"index.json\"\nhash-format = \"sha256\"\nhash = \"{}\"\n",
		hash_bytes(HashFormat::Sha256, &index)
	)
	.into_bytes();
	let fixtures = BTreeMap::from([
		("/pack.toml".to_owned(), pack),
		("/index.json".to_owned(), index),
		("/mods/binary-example.pw.json".to_owned(), metadata),
		("/artifact.jar".to_owned(), artifact.clone()),
	]);
	let server = thread::spawn(move || serve_http_fixture(listener, fixtures, 4));
	let instance = tempfile::tempdir().unwrap();
	let status = Command::new(env!("CARGO_BIN_EXE_packwand-installer"))
		.args(["--side", "client", "--game-dir"])
		.arg(instance.path())
		.arg(format!("{origin}/pack.toml"))
		.status()
		.unwrap();
	assert!(status.success());
	assert_eq!(server.join().unwrap(), 4);
	assert_eq!(
		std::fs::read(instance.path().join("mods/binary-example.jar")).unwrap(),
		artifact
	);
}

#[test]
fn curseforge_no_distribute_mod_is_manual_but_does_not_block_the_rest() {
	let allowed_artifact = b"allowed mod jar".to_vec();
	let allowed_metadata = Mod {
		name: "Allowed".into(),
		filename: "allowed.jar".into(),
		download: packwand_pack::Download {
			url: "https://cdn.example/allowed.jar".into(),
			hash_format: "sha256".into(),
			hash: hash_bytes(HashFormat::Sha256, &allowed_artifact),
			..packwand_pack::Download::default()
		},
		..Mod::default()
	}
	.to_json_bytes()
	.unwrap();
	let blocked_artifact = b"blocked mod jar".to_vec();
	let blocked_metadata = Mod {
		name: "Blocked".into(),
		filename: "blocked.jar".into(),
		side: "both".into(),
		download: packwand_pack::Download {
			mode: "metadata:curseforge".into(),
			hash_format: "sha256".into(),
			hash: hash_bytes(HashFormat::Sha256, &blocked_artifact),
			..packwand_pack::Download::default()
		},
		update: BTreeMap::from([(
			"curseforge".into(),
			serde_json::json!({"project-id": 111, "file-id": 222})
				.as_object()
				.unwrap()
				.clone(),
		)]),
		..Mod::default()
	}
	.to_json_bytes()
	.unwrap();
	let index = serde_json::to_vec(&Index {
		hash_format: "sha256".into(),
		files: vec![
			IndexFile {
				file: "mods/allowed.pw.json".into(),
				hash: hash_bytes(HashFormat::Sha256, &allowed_metadata),
				metafile: true,
				..IndexFile::default()
			},
			IndexFile {
				file: "mods/blocked.pw.json".into(),
				hash: hash_bytes(HashFormat::Sha256, &blocked_metadata),
				metafile: true,
				..IndexFile::default()
			},
		],
	})
	.unwrap();
	let pack = format!(
		"name = \"Fixture\"\npack-format = \"packwand:27\"\n\n[index]\nfile = \"index.json\"\nhash-format = \"sha256\"\nhash = \"{}\"\n",
		hash_bytes(HashFormat::Sha256, &index)
	)
	.into_bytes();
	let cf_file =
		br#"{"data":{"id":222,"fileName":"blocked.jar","displayName":"blocked.jar","releaseType":1,"downloadUrl":null}}"#
			.to_vec();
	let cf_project = br#"{"data":{"id":111,"name":"Blocked","slug":"blocked-mod"}}"#.to_vec();
	let transport = MappingTransport(BTreeMap::from([
		("https://packs.example/pack.toml".into(), pack),
		("https://packs.example/index.json".into(), index),
		(
			"https://packs.example/mods/allowed.pw.json".into(),
			allowed_metadata,
		),
		(
			"https://packs.example/mods/blocked.pw.json".into(),
			blocked_metadata,
		),
		(
			"https://cdn.example/allowed.jar".into(),
			allowed_artifact.clone(),
		),
		(
			"https://api.curseforge.com/v1/mods/111/files/222".into(),
			cf_file,
		),
		("https://api.curseforge.com/v1/mods/111".into(), cf_project),
	]));
	let remote = RemotePack::load("https://packs.example/pack.toml", &transport).unwrap();
	let root = tempfile::tempdir().unwrap();
	let plan = build(&remote, root.path(), InstallSide::Client, &transport).unwrap();
	packwand_installer::download::apply(&plan, root.path(), &transport).unwrap();

	// The blocked mod didn't stop the allowed one from installing.
	assert_eq!(
		std::fs::read(root.path().join("mods/allowed.jar")).unwrap(),
		allowed_artifact
	);
	assert_eq!(plan.manual.len(), 1);
	assert_eq!(plan.manual[0].name, "Blocked");
	assert_eq!(plan.manual[0].target, root.path().join("mods/blocked.jar"));
	assert_eq!(
		plan.manual[0].page_url.as_deref(),
		Some("https://www.curseforge.com/minecraft/mc-mods/blocked-mod/files/222")
	);
	assert_eq!(
		plan.manual[0].hash,
		hash_bytes(HashFormat::Sha256, &blocked_artifact)
	);

	// A tester drops the file in by hand (Prism-style manual install); a
	// rebuild recognizes it and stops asking.
	std::fs::write(root.path().join("mods/blocked.jar"), &blocked_artifact).unwrap();
	let rebuilt = build(&remote, root.path(), InstallSide::Client, &transport).unwrap();
	assert!(rebuilt.manual.is_empty());
}

fn serve_http_fixture(
	listener: TcpListener,
	fixtures: BTreeMap<String, Vec<u8>>,
	expected_requests: usize,
) -> usize {
	let deadline = Instant::now() + Duration::from_secs(10);
	let mut served = 0;
	while served < expected_requests && Instant::now() < deadline {
		match listener.accept() {
			Ok((mut stream, _)) => {
				let mut request = [0_u8; 4096];
				let count = stream.read(&mut request).unwrap();
				let target = std::str::from_utf8(&request[..count])
					.unwrap()
					.lines()
					.next()
					.and_then(|line| line.split_whitespace().nth(1))
					.unwrap();
				let body = fixtures.get(target).expect("requested fixture");
				write!(
					stream,
					"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
					body.len()
				)
				.unwrap();
				stream.write_all(body).unwrap();
				stream.flush().unwrap();
				served += 1;
			}
			Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
				thread::sleep(Duration::from_millis(10));
			}
			Err(error) => panic!("fixture server failed: {error}"),
		}
	}
	served
}
