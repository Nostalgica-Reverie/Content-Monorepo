//! Offline end-to-end test of `instance bootstrap`: a local HTTP server
//! serves synthetic version metadata, and the resulting instance is
//! planned and run with the fake-java helper. No real network, Java, or
//! Minecraft is involved.

mod common;

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use sha1::{Digest, Sha1};

fn sha1_hex(bytes: &[u8]) -> String {
	Sha1::digest(bytes)
		.iter()
		.map(|b| format!("{b:02x}"))
		.collect()
}

/// Serves fixed routes on a random local port for the lifetime of the test
/// process.
fn serve(routes: BTreeMap<String, Vec<u8>>) -> u16 {
	let server = tiny_http::Server::http("127.0.0.1:0").expect("bind fixture server");
	let port = server.server_addr().to_ip().expect("ip addr").port();
	let routes = Arc::new(routes);
	std::thread::spawn(move || {
		for request in server.incoming_requests() {
			let path = request.url().to_string();
			match routes.get(&path) {
				Some(body) => {
					let _ = request.respond(tiny_http::Response::from_data(body.clone()));
				}
				None => {
					let _ = request.respond(tiny_http::Response::empty(404));
				}
			}
		}
	});
	port
}

#[test]
fn bootstrap_plan_and_run_offline_fixture() {
	let client_jar = b"client jar bytes".to_vec();
	let lib_jar = b"library jar bytes".to_vec();
	let asset = b"asset object bytes".to_vec();
	let asset_hash = sha1_hex(&asset);

	let asset_index = format!(
		r#"{{"objects": {{"icons/icon.png": {{"hash": "{asset_hash}", "size": {}}}}}}}"#,
		asset.len()
	)
	.into_bytes();

	// The port is not known until the server starts, but URLs must be
	// absolute. Serve everything under stable paths and register the
	// version document twice: it references the port only via its own URL.
	let mut routes = BTreeMap::new();
	routes.insert("/client.jar".to_string(), client_jar.clone());
	routes.insert(
		"/libraries/com/example/fixture/1.0/fixture-1.0.jar".to_string(),
		lib_jar.clone(),
	);
	routes.insert(
		format!("/resources/{}/{}", &asset_hash[..2], asset_hash),
		asset.clone(),
	);
	routes.insert("/assets/17.json".to_string(), asset_index.clone());
	let port = serve(routes);
	let base = format!("http://127.0.0.1:{port}");

	let version_doc = format!(
        r#"{{
        "id": "fixture-1.0",
        "type": "release",
        "mainClass": "fixture.Main",
        "arguments": {{
            "game": [
                "--username", "${{auth_player_name}}",
                "--gameDir", "${{game_directory}}",
                "--assetsDir", "${{assets_root}}",
                "--assetIndex", "${{assets_index_name}}",
                "--uuid", "${{auth_uuid}}",
                "--accessToken", "${{auth_access_token}}"
            ],
            "jvm": ["-Djava.library.path=${{natives_directory}}", "-cp", "${{classpath}}"]
        }},
        "libraries": [
            {{
                "name": "com.example:fixture:1.0",
                "downloads": {{
                    "artifact": {{
                        "path": "com/example/fixture/1.0/fixture-1.0.jar",
                        "url": "{base}/libraries/com/example/fixture/1.0/fixture-1.0.jar",
                        "sha1": "{lib_sha1}",
                        "size": {lib_size}
                    }}
                }}
            }}
        ],
        "assetIndex": {{"id": "17", "url": "{base}/assets/17.json", "sha1": "{index_sha1}", "size": {index_size}}},
        "assets": "17",
        "downloads": {{"client": {{"url": "{base}/client.jar", "sha1": "{client_sha1}", "size": {client_size}}}}},
        "javaVersion": {{"component": "java-runtime", "majorVersion": 21}}
    }}"#,
        lib_sha1 = sha1_hex(&lib_jar),
        lib_size = lib_jar.len(),
        index_sha1 = sha1_hex(&asset_index),
        index_size = asset_index.len(),
        client_sha1 = sha1_hex(&client_jar),
        client_size = client_jar.len(),
    )
    .into_bytes();

	let manifest = format!(
		r#"{{
        "latest": {{"release": "fixture-1.0", "snapshot": "fixture-1.0"}},
        "versions": [
            {{"id": "fixture-1.0", "type": "release", "url": "{base2}/fixture-1.0.json", "sha1": "{doc_sha1}"}}
        ]
    }}"#,
		base2 = "{BASE2}",
		doc_sha1 = sha1_hex(&version_doc),
	);

	// Second server carries the version document and manifest so the
	// manifest can embed that server's own port.
	let mut meta_routes = BTreeMap::new();
	meta_routes.insert("/fixture-1.0.json".to_string(), version_doc);
	let meta_port = serve(meta_routes);
	let meta_base = format!("http://127.0.0.1:{meta_port}");
	let manifest = manifest.replace("{BASE2}", &meta_base).into_bytes();
	let mut manifest_routes = BTreeMap::new();
	manifest_routes.insert("/manifest.json".to_string(), manifest);
	let manifest_port = serve(manifest_routes);

	let dir = tempfile::tempdir().unwrap();
	let root = dir.path().join("root");

	let output = Command::new(common::probe_bin())
		.args(["instance", "bootstrap", "--id", "boot-fixture"])
		.arg("--root")
		.arg(&root)
		.args(["--minecraft", "fixture-1.0"])
		.arg("--java")
		.arg(common::fake_java())
		.args(["--workers", "2", "--json"])
		.args([
			"--manifest-url",
			&format!("http://127.0.0.1:{manifest_port}/manifest.json"),
		])
		.args(["--resources-url", &format!("{base}/resources")])
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"bootstrap failed:\nstdout: {}\nstderr: {}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr)
	);
	let record: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
	assert_eq!(record["id"], "boot-fixture");
	assert_eq!(record["main_class"], "fixture.Main");
	assert_eq!(
		record["session_placeholders"],
		serde_json::json!(["auth_access_token"])
	);
	assert_eq!(
		record["identity_placeholders"],
		serde_json::json!(["auth_player_name", "auth_uuid"])
	);
	let game_args = record["game_args"].as_array().unwrap();
	assert!(game_args.iter().any(|a| a == "${secret:auth_access_token}"));
	// Neither the token nor the account is baked in: this record is shared by
	// every pack on this version, and by every account.
	assert!(!game_args.iter().any(|a| a == "offline"));
	assert!(!game_args.iter().any(|a| a == "Tester"));
	assert!(
		game_args
			.iter()
			.any(|a| a == "${identity:auth_player_name}")
	);

	// Installed files exist with verified content.
	let installed = |rel: &str| root.join(Path::new(rel));
	assert_eq!(
		std::fs::read(installed("versions/fixture-1.0/fixture-1.0.jar")).unwrap(),
		client_jar
	);
	assert_eq!(
		std::fs::read(installed(
			"libraries/com/example/fixture/1.0/fixture-1.0.jar"
		))
		.unwrap(),
		lib_jar
	);
	assert_eq!(
		std::fs::read(installed(&format!(
			"assets/objects/{}/{}",
			&asset_hash[..2],
			asset_hash
		)))
		.unwrap(),
		asset
	);
	assert!(installed("assets/indexes/17.json").exists());
	assert!(installed("versions/fixture-1.0/fixture-1.0.json").exists());

	// The plan resolves layout vars and keeps the secret redacted.
	let output = Command::new(common::probe_bin())
		.args(["launch", "plan", "--instance", "boot-fixture", "--json"])
		.arg("--root")
		.arg(&root)
		.output()
		.unwrap();
	assert!(output.status.success());
	let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
	assert_eq!(
		plan["session"]["auth_access_token"],
		"${secret:auth_access_token}"
	);
	let plan_args: Vec<String> = plan["game_args"]
		.as_array()
		.unwrap()
		.iter()
		.map(|a| a.as_str().unwrap().to_string())
		.collect();
	assert!(
		!plan_args.iter().any(|a| a.contains("${game_directory}")),
		"layout var must be resolved in the plan: {plan_args:?}"
	);
	// Classpath ends with the client jar, so the fixture library precedes it.
	let classpath = plan["classpath"].as_array().unwrap();
	assert!(
		classpath
			.last()
			.unwrap()
			.as_str()
			.unwrap()
			.ends_with("fixture-1.0.jar")
	);

	// The instance runs: fake-java exits 0 and the secret resolves.
	let output = Command::new(common::probe_bin())
		.args([
			"launch",
			"run",
			"--instance",
			"boot-fixture",
			"--json-events",
		])
		.arg("--root")
		.arg(&root)
		.args(["--username", "Tester"])
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"run failed: {}",
		String::from_utf8_lossy(&output.stdout)
	);
	let events: Vec<serde_json::Value> = String::from_utf8(output.stdout.clone())
		.unwrap()
		.lines()
		.map(|line| serde_json::from_str(line).unwrap())
		.collect();
	assert_eq!(events.last().unwrap()["event"], "exited");
}
