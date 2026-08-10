//! Spawn-time secret resolution: the child process receives the raw
//! value, while records, plans, and events only ever carry the
//! `${secret:<name>}` placeholder.

mod common;

use std::collections::BTreeMap;

use packwand_auth::SecretString;
use packwand_instance::{FsInstanceRepository, InstanceRepository, InstanceSpec, MemoryLimits};
use packwand_launch::{LaunchEvent, LaunchOptions, build_launch_plan, launch};

fn secret_spec(id: &str, record_path: &std::path::Path) -> InstanceSpec {
	InstanceSpec {
		id: id.to_string(),
		name: format!("Secret fixture {id}"),
		java_executable: common::fake_java(),
		jvm_args: vec![],
		main_class: "fixture.Main".to_string(),
		classpath: vec![],
		game_args: vec![
			"--accessToken".to_string(),
			"${secret:auth_access_token}".to_string(),
		],
		env: BTreeMap::from([(
			"FAKE_JAVA_RECORD".to_string(),
			record_path.display().to_string(),
		)]),
		memory: MemoryLimits::default(),
		session_placeholders: vec!["auth_access_token".to_string()],
		identity_placeholders: Vec::new(),
	}
}

#[test]
fn secret_reaches_child_but_never_plan_or_events() {
	let dir = tempfile::tempdir().unwrap();
	let record_path = dir.path().join("child-record.json");
	let repo = FsInstanceRepository::new(dir.path().join("root"));
	let record = repo
		.create(&secret_spec("secret-e2e", &record_path))
		.unwrap();
	let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));

	// The plan (what GUIs inspect and serialize) holds only the placeholder.
	let plan_json = serde_json::to_string(&plan).unwrap();
	assert!(plan_json.contains("${secret:auth_access_token}"));
	assert!(!plan_json.contains("tok-super-secret"));

	let options = LaunchOptions {
		secrets: BTreeMap::from([(
			"auth_access_token".to_string(),
			SecretString::new("tok-super-secret"),
		)]),
		..LaunchOptions::default()
	};
	let events = launch(&plan, options).unwrap().wait();
	assert!(
		matches!(
			events.last(),
			Some(LaunchEvent::Exited { code: Some(0), .. })
		),
		"unexpected events: {events:?}"
	);
	// Lifecycle events never carry the secret.
	let events_json = serde_json::to_string(&events).unwrap();
	assert!(!events_json.contains("tok-super-secret"));

	// The child's recorded argv received the resolved value.
	let child_record: serde_json::Value =
		serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
	let args: Vec<String> = child_record["args"]
		.as_array()
		.unwrap()
		.iter()
		.map(|a| a.as_str().unwrap().to_string())
		.collect();
	assert!(
		args.windows(2)
			.any(|w| w[0] == "--accessToken" && w[1] == "tok-super-secret"),
		"child argv: {args:?}"
	);
	assert!(!args.iter().any(|a| a.contains("${secret:")));

	// The token reached the child, but not by way of the command line: argv
	// is the one place every other user on the machine can read.
	let raw: Vec<String> = child_record["raw_args"]
		.as_array()
		.unwrap()
		.iter()
		.map(|a| a.as_str().unwrap().to_string())
		.collect();
	assert!(
		!raw.iter().any(|a| a.contains("tok-super-secret")),
		"access token was on the command line: {raw:?}"
	);
	assert!(
		raw.len() == 1 && raw[0].starts_with('@'),
		"expected a single argument-file argument, got {raw:?}"
	);
}

/// Stages the fake JVM inside a synthetic installation whose `release` file
/// claims a given version, so the launcher's probe identifies it as that
/// release without the fixture needing to impersonate one at runtime.
fn fake_java_reporting(dir: &std::path::Path, java_version: &str) -> std::path::PathBuf {
	let home = dir.join(format!("jdk-{java_version}"));
	std::fs::create_dir_all(home.join("bin")).unwrap();
	let executable = home
		.join("bin")
		.join(if cfg!(windows) { "java.exe" } else { "java" });
	std::fs::copy(common::fake_java(), &executable).unwrap();
	std::fs::write(
		home.join("release"),
		format!("JAVA_VERSION=\"{java_version}\"\nOS_ARCH=\"x86_64\"\n"),
	)
	.unwrap();
	executable
}

#[test]
fn a_java_8_child_gets_arguments_on_the_command_line() {
	// @argfile is JDK 9+. Minecraft 1.16 and earlier run on Java 8, so
	// handing one an argument file would make the game fail to start — the
	// launcher has to give up the argv protection rather than the launch.
	let dir = tempfile::tempdir().unwrap();
	let record_path = dir.path().join("legacy-record.json");
	let repo = FsInstanceRepository::new(dir.path().join("root"));
	let mut spec = secret_spec("secret-legacy", &record_path);
	spec.java_executable = fake_java_reporting(dir.path(), "1.8.0_392");
	let record = repo.create(&spec).unwrap();
	let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));

	let events = launch(
		&plan,
		LaunchOptions {
			secrets: BTreeMap::from([(
				"auth_access_token".to_string(),
				SecretString::new("tok-super-secret"),
			)]),
			..LaunchOptions::default()
		},
	)
	.unwrap()
	.wait();
	assert!(
		matches!(
			events.last(),
			Some(LaunchEvent::Exited { code: Some(0), .. })
		),
		"unexpected events: {events:?}"
	);

	let child: serde_json::Value =
		serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
	let raw: Vec<String> = child["raw_args"]
		.as_array()
		.unwrap()
		.iter()
		.map(|a| a.as_str().unwrap().to_string())
		.collect();
	assert!(
		!raw.iter().any(|a| a.starts_with('@')),
		"a Java 8 child was handed an argument file: {raw:?}"
	);
	assert!(
		raw.windows(2)
			.any(|w| w[0] == "--accessToken" && w[1] == "tok-super-secret"),
		"child argv: {raw:?}"
	);
}

#[test]
fn a_java_21_child_still_gets_an_argument_file() {
	// The other side of the same gate: a modern JVM keeps the protection.
	let dir = tempfile::tempdir().unwrap();
	let record_path = dir.path().join("modern-record.json");
	let repo = FsInstanceRepository::new(dir.path().join("root"));
	let mut spec = secret_spec("secret-modern", &record_path);
	spec.java_executable = fake_java_reporting(dir.path(), "21.0.5");
	let record = repo.create(&spec).unwrap();
	let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));

	launch(
		&plan,
		LaunchOptions {
			secrets: BTreeMap::from([(
				"auth_access_token".to_string(),
				SecretString::new("tok-super-secret"),
			)]),
			..LaunchOptions::default()
		},
	)
	.unwrap()
	.wait();

	let child: serde_json::Value =
		serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
	let raw: Vec<String> = child["raw_args"]
		.as_array()
		.unwrap()
		.iter()
		.map(|a| a.as_str().unwrap().to_string())
		.collect();
	assert!(
		raw.len() == 1 && raw[0].starts_with('@'),
		"child argv: {raw:?}"
	);
	assert!(!raw.iter().any(|a| a.contains("tok-super-secret")));
}

#[test]
fn a_token_the_game_prints_back_is_censored_before_it_becomes_an_event() {
	// The realistic leak: the launcher keeps the token off the command line,
	// and then the game prints its own arguments into the log on a crash.
	// Everything downstream — the UI, a saved log, a paste — reads these
	// events, so the value has to be gone by the time one exists.
	let dir = tempfile::tempdir().unwrap();
	let repo = FsInstanceRepository::new(dir.path().join("root"));
	let mut spec = secret_spec("secret-echo", &dir.path().join("unused.json"));
	// Resolved at spawn like any other placeholder, so the child receives —
	// and prints — the real value.
	spec.env = BTreeMap::from([(
		"FAKE_JAVA_STDOUT".to_string(),
		"Setting user: args --accessToken ${secret:auth_access_token} done".to_string(),
	)]);
	let record = repo.create(&spec).unwrap();
	let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));

	let events = launch(
		&plan,
		LaunchOptions {
			secrets: BTreeMap::from([(
				"auth_access_token".to_string(),
				SecretString::new("tok-super-secret"),
			)]),
			..LaunchOptions::default()
		},
	)
	.unwrap()
	.wait();

	let printed = events
		.iter()
		.find_map(|e| match e {
			LaunchEvent::Stdout { line, .. } if line.contains("Setting user") => Some(line.clone()),
			_ => None,
		})
		.expect("the child's line should still be reported");
	assert!(
		!printed.contains("tok-super-secret"),
		"the token survived into an event: {printed}"
	);
	// Censoring removes the value, not the line: the surrounding context is
	// what makes a log worth reading.
	assert!(printed.contains("Setting user"), "{printed}");
	assert!(printed.contains("done"), "{printed}");

	let all = serde_json::to_string(&events).unwrap();
	assert!(!all.contains("tok-super-secret"), "{all}");
}

#[test]
fn unresolved_secret_placeholder_refuses_to_spawn() {
	let dir = tempfile::tempdir().unwrap();
	let record_path = dir.path().join("never-written.json");
	let repo = FsInstanceRepository::new(dir.path().join("root"));
	let record = repo
		.create(&secret_spec("secret-missing", &record_path))
		.unwrap();
	let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));

	let events = launch(&plan, LaunchOptions::default()).unwrap().wait();
	assert!(
		matches!(
			events.last(),
			Some(LaunchEvent::Failed { error, .. }) if error.contains("auth_access_token")
		),
		"unexpected events: {events:?}"
	);
	assert!(!record_path.exists(), "the child must never have spawned");
}
