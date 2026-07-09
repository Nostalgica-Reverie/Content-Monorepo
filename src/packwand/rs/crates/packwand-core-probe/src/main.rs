//! Diagnostic CLI over the packwand-rs core crates.
//!
//! This is the probe from `packwandrs.md`, not a product CLI. Beyond the
//! four Phase 1 commands it exposes the Phase 2 launcher capabilities:
//! Java runtime discovery (`runtime list`) and real instance
//! bootstrapping (`instance bootstrap`), plus launch-time controls used
//! by the Phase 3 disposable-boot flow.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use packwand_devboot::bootstrap;
use packwand_instance::{FsInstanceRepository, InstanceRepository, InstanceSpec, ListEntry};
use packwand_launch::{build_launch_plan, launch, LaunchEvent, LaunchOptions};
use packwand_minecraft::MetadataEndpoints;

#[derive(Parser)]
#[command(
    name = "packwand-core-probe",
    about = "Test/diagnostic CLI over the packwand-rs core crates (spike; not a product CLI)"
)]
struct Cli {
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    /// Instance record operations.
    #[command(subcommand)]
    Instance(InstanceCommand),
    /// Launch plan and supervision operations.
    #[command(subcommand)]
    Launch(LaunchCommand),
    /// Java runtime discovery.
    #[command(subcommand)]
    Runtime(RuntimeCommand),
}

#[derive(Subcommand)]
// The bootstrap variant is naturally flag-heavy; this is a short-lived
// dispatch value in a CLI, not a stored type.
#[allow(clippy::large_enum_variant)]
enum InstanceCommand {
    /// Create an instance under --root from a JSON spec file.
    Create {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        spec: PathBuf,
    },
    /// List instances under --root.
    List {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Download and install a real Minecraft version, then store a
    /// launchable instance record for it (offline session).
    Bootstrap {
        #[arg(long)]
        root: PathBuf,
        /// Instance id to create.
        #[arg(long)]
        id: String,
        /// Version id, or `latest-release` / `latest-snapshot`.
        #[arg(long)]
        minecraft: String,
        /// Mod loader to overlay (`fabric`, `quilt`, `forge`, or `neoforge`).
        #[arg(long)]
        loader: Option<String>,
        /// Loader version; defaults to the newest stable one.
        #[arg(long)]
        loader_version: Option<String>,
        /// Offline player name baked into the instance.
        #[arg(long, default_value = "Player")]
        username: String,
        /// Java executable to record; discovered automatically when absent.
        #[arg(long)]
        java: Option<PathBuf>,
        /// JVM maximum heap in MiB.
        #[arg(long)]
        memory_max: Option<u32>,
        /// Concurrent download workers.
        #[arg(long, default_value_t = 8)]
        workers: usize,
        #[arg(long)]
        json: bool,
        /// Override the version-manifest endpoint (tests/fixtures).
        #[arg(long, hide = true)]
        manifest_url: Option<String>,
        /// Override the Fabric meta endpoint (tests/fixtures).
        #[arg(long, hide = true)]
        fabric_meta_url: Option<String>,
        /// Override the asset resources endpoint (tests/fixtures).
        #[arg(long, hide = true)]
        resources_url: Option<String>,
    },
}

#[derive(Subcommand)]
enum RuntimeCommand {
    /// List Java installations discovered on this machine.
    List {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum LaunchCommand {
    /// Print the deterministic launch plan for an instance.
    Plan {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        instance: String,
        #[arg(long)]
        json: bool,
    },
    /// Run an instance and stream lifecycle events (one JSON object per
    /// line with --json-events). Exits with the child's exit code, 2 on
    /// failure, 3 on cancellation.
    Run {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        instance: String,
        #[arg(long)]
        json_events: bool,
        /// Offline player name used to resolve the instance's session
        /// placeholders at spawn time.
        #[arg(long, default_value = "Player")]
        username: String,
        /// Cancel the run after this many seconds (disposable boots).
        #[arg(long)]
        max_runtime_secs: Option<u64>,
        /// Cancel the run once a stdout/stderr line contains this text.
        #[arg(long)]
        stop_on_line: Option<String>,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        TopCommand::Instance(InstanceCommand::Create { root, spec }) => {
            let bytes = std::fs::read(&spec)
                .map_err(|e| format!("failed to read spec {}: {e}", spec.display()))?;
            let spec: InstanceSpec = serde_json::from_slice(&bytes)
                .map_err(|e| format!("invalid instance spec: {e}"))?;
            let record = FsInstanceRepository::new(root)
                .create(&spec)
                .map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?
            );
            Ok(ExitCode::SUCCESS)
        }
        TopCommand::Instance(InstanceCommand::List { root, json }) => {
            let entries = FsInstanceRepository::new(root)
                .list()
                .map_err(|e| e.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?
                );
            } else {
                for entry in &entries {
                    match entry {
                        ListEntry::Ok { id, record } => println!("{id}\tok\t{}", record.name),
                        ListEntry::Error { id, error } => println!("{id}\terror\t{error}"),
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        TopCommand::Instance(InstanceCommand::Bootstrap {
            root,
            id,
            minecraft,
            loader,
            loader_version,
            username,
            java,
            memory_max,
            workers,
            json,
            manifest_url,
            fabric_meta_url,
            resources_url,
        }) => {
            let mut endpoints = MetadataEndpoints::default();
            if let Some(url) = manifest_url {
                endpoints.version_manifest_url = url;
            }
            if let Some(url) = fabric_meta_url {
                endpoints.fabric_meta_url = url;
            }
            if let Some(url) = resources_url {
                endpoints.resources_url = url;
            }
            let session = packwand_auth::offline_session(&username).map_err(|e| e.to_string())?;
            let record = bootstrap::bootstrap(&bootstrap::BootstrapRequest {
                root,
                id,
                minecraft,
                loader,
                loader_version,
                session,
                java,
                memory_max_mb: memory_max,
                workers,
                endpoints,
            })?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?
                );
            } else {
                println!("bootstrapped {} ({})", record.id, record.name);
            }
            Ok(ExitCode::SUCCESS)
        }
        TopCommand::Launch(LaunchCommand::Plan {
            root,
            instance,
            json,
        }) => {
            let repo = FsInstanceRepository::new(root);
            let record = repo.get(&instance).map_err(|e| e.to_string())?;
            let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&plan).map_err(|e| e.to_string())?
                );
            } else {
                println!("instance: {}", plan.instance_id);
                println!("java: {}", plan.java_executable.display());
                println!("main class: {}", plan.main_class);
                println!("arguments: {}", plan.command_arguments().join(" "));
            }
            Ok(ExitCode::SUCCESS)
        }
        TopCommand::Launch(LaunchCommand::Run {
            root,
            instance,
            json_events,
            username,
            max_runtime_secs,
            stop_on_line,
        }) => {
            let repo = FsInstanceRepository::new(root);
            let record = repo.get(&instance).map_err(|e| e.to_string())?;
            let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));
            let mut options = LaunchOptions::default();
            if !record.session_placeholders.is_empty() {
                let session =
                    packwand_auth::offline_session(&username).map_err(|e| e.to_string())?;
                options.secrets = session.secrets();
            }
            let handle = launch(&plan, options).map_err(|e| e.to_string())?;
            if let Some(secs) = max_runtime_secs {
                let cancel = handle.cancel_token();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(secs));
                    cancel.cancel();
                });
            }
            let cancel_on_line = handle.cancel_token();
            let mut exit = ExitCode::SUCCESS;
            for event in handle.events() {
                if json_events {
                    println!(
                        "{}",
                        serde_json::to_string(&event).map_err(|e| e.to_string())?
                    );
                } else {
                    print_event(&event);
                }
                match &event {
                    LaunchEvent::Stdout { line, .. } | LaunchEvent::Stderr { line, .. } => {
                        if let Some(marker) = &stop_on_line {
                            if line.contains(marker.as_str()) {
                                cancel_on_line.cancel();
                            }
                        }
                    }
                    LaunchEvent::Exited { code, .. } => {
                        exit = match code {
                            Some(code) if (0..=255).contains(code) => {
                                ExitCode::from(u8::try_from(*code).unwrap_or(1))
                            }
                            Some(_) => ExitCode::FAILURE,
                            None => ExitCode::FAILURE,
                        };
                    }
                    LaunchEvent::Failed { .. } => exit = ExitCode::from(2),
                    LaunchEvent::Cancelled { .. } => exit = ExitCode::from(3),
                    _ => {}
                }
            }
            Ok(exit)
        }
        TopCommand::Runtime(RuntimeCommand::List { json }) => {
            let installations =
                packwand_runtime::discover(&packwand_runtime::DiscoveryConfig::from_host());
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&installations).map_err(|e| e.to_string())?
                );
            } else {
                for install in &installations {
                    println!(
                        "{}\t{}\t{}",
                        install.version,
                        install.vendor.as_deref().unwrap_or("?"),
                        install.home.display()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn print_event(event: &LaunchEvent) {
    match event {
        LaunchEvent::Starting { instance_id } => println!("starting {instance_id}"),
        LaunchEvent::Started { instance_id, pid } => println!("started {instance_id} pid={pid}"),
        LaunchEvent::Stdout { line, .. } => println!("stdout: {line}"),
        LaunchEvent::Stderr { line, .. } => println!("stderr: {line}"),
        LaunchEvent::Exited { instance_id, code } => {
            println!("exited {instance_id} code={code:?}")
        }
        LaunchEvent::Failed { instance_id, error } => {
            println!("failed {instance_id}: {error}")
        }
        LaunchEvent::Cancelled { instance_id } => println!("cancelled {instance_id}"),
    }
}
