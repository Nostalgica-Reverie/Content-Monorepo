use std::{
    cell::RefCell,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use std::sync::LazyLock;

use anyhow::{Context, bail, ensure};
use closure::closure;
use json::{JsonValue, object};
use regex::Regex;
use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
        menu::{MenuBar, MenuItemAttributes},
        window::WindowBuilder,
    },
    webview::{WebContext, WebView, WebViewBuilder},
};

/// The mod-hosting site to browse. Selected with `--provider <name>`;
/// defaults to CurseForge for backward compatibility with curseforge_webview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    CurseForge,
    Modrinth,
}

impl Provider {
    fn from_args() -> anyhow::Result<Provider> {
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--provider" {
                let value = args
                    .next()
                    .context("--provider requires a value (curseforge or modrinth)")?;
                return match value.as_str() {
                    "curseforge" => Ok(Provider::CurseForge),
                    "modrinth" => Ok(Provider::Modrinth),
                    other => bail!("unknown provider {other:?} (expected curseforge or modrinth)"),
                };
            }
        }
        Ok(Provider::CurseForge)
    }

    /// Validates a base project page URL for this provider.
    fn project_url_valid(self, url: &str) -> bool {
        static CF: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new("^https?://(?:(?:www|beta)\\.)?curseforge\\.com/[^/]+/[^/]+/[^/]+$").unwrap()
        });
        static MR: LazyLock<Regex> =
            LazyLock::new(|| Regex::new("^https?://(?:www\\.)?modrinth\\.com/[^/]+/[^/]+$").unwrap());
        match self {
            Provider::CurseForge => CF.is_match(url),
            Provider::Modrinth => MR.is_match(url),
        }
    }

    /// Validates a file/version identifier for this provider.
    fn id_valid(self, id: &str) -> bool {
        static CF_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[0-9]+$").unwrap());
        static MR_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-zA-Z0-9]+$").unwrap());
        match self {
            Provider::CurseForge => CF_ID.is_match(id),
            Provider::Modrinth => MR_ID.is_match(id),
        }
    }

    /// The page opened for a requested file.
    fn file_page(self, base_url: &str, id: &str) -> String {
        match self {
            Provider::CurseForge => format!("{base_url}/files/{id}"),
            Provider::Modrinth => format!("{base_url}/version/{id}"),
        }
    }

    /// Matches a direct download URL served by the provider's CDN.
    fn download_url_valid(self, uri: &str) -> bool {
        static CF_DL: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new("^https?://(?:edge|media)\\.forgecdn\\.net/files/.+$").unwrap()
        });
        static MR_DL: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new("^https?://cdn\\.modrinth\\.com/data/[^/]+/versions/.+$").unwrap()
        });
        match self {
            Provider::CurseForge => CF_DL.is_match(uri),
            Provider::Modrinth => MR_DL.is_match(uri),
        }
    }

    /// Matches navigation to the page(s) belonging to the requested file.
    fn nav_regex(self, file_id: &str) -> Regex {
        match self {
            Provider::CurseForge => Regex::new(&format!(
                "^https?://(?:(?:www|beta)\\.)?curseforge\\.com/+[^/]+/[^/]+/[^/]+/(?:files|download)/{}",
                regex::escape(file_id)
            ))
            .unwrap(),
            Provider::Modrinth => Regex::new(&format!(
                "^https?://(?:www\\.)?modrinth\\.com/+[^/]+/[^/]+/version/{}",
                regex::escape(file_id)
            ))
            .unwrap(),
        }
    }

    /// Matches navigation to a *different* file's page (the wrong download).
    fn bad_nav_valid(self, uri: &str) -> bool {
        // Note: + after / due to bad path normalisation in beta redirect
        static CF_BAD: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                "^https?://(?:(?:www|beta)\\.)?curseforge\\.com/+[^/]+/[^/]+/[^/]+/(?:files/[0-9]+|download)"
            )
            .unwrap()
        });
        static MR_BAD: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new("^https?://(?:www\\.)?modrinth\\.com/+[^/]+/[^/]+/version/[a-zA-Z0-9.+-]+")
                .unwrap()
        });
        match self {
            Provider::CurseForge => CF_BAD.is_match(uri),
            Provider::Modrinth => MR_BAD.is_match(uri),
        }
    }
}

fn main() {
    println!("mod_browser_webview {}", env!("CARGO_PKG_VERSION"));

    if let Err(e) = start() {
        println!("ERROR");
        println!("{:?}", e);
    }
}

fn create_alert(webview: &WebView, message: &str) -> wry::Result<()> {
    webview.evaluate_script(&format!("alert({});", json::stringify(message)))
}

fn create_confirm<T: Into<JsonValue>>(
    webview: &WebView,
    ipc_cookie: &mut Arc<Mutex<f64>>,
    ipc_data: T,
    message: &str,
) -> wry::Result<()> {
    let mut cookie = ipc_cookie.lock().unwrap();
    *cookie = rand::random::<f64>();
    webview.evaluate_script(&format!(
        r#"window.ipc.postMessage(JSON.stringify({{...{},cookie:{},value:confirm({})}}))"#,
        json::stringify(ipc_data),
        json::stringify(*cookie),
        json::stringify(message)
    ))
}

struct File {
    id: String,
    url: String,
}

fn start() -> anyhow::Result<()> {
    let provider = Provider::from_args()?;
    let mut data_dir: Option<PathBuf> = None;
    let files = {
        let mut files: Vec<File> = Vec::new();
        let reader = BufReader::new(io::stdin());
        for line in reader.lines() {
            let line = line?;
            if line == "DONE" {
                break;
            } else if line.starts_with("DATA ") {
                data_dir = Some(line.strip_prefix("DATA ").unwrap().into())
            } else {
                let (id, url) = parse_url_line(&line, provider).context("Failed to read URL")?;
                files.push(File {
                    id: id.to_string(),
                    url: url.to_string(),
                });
            }
        }
        files
    };

    let num_files = files.len();
    if num_files == 0 {
        return Ok(()); // Nothing to do!
    }

    let cur_file: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let event_loop: EventLoop<NavigationEvent> = EventLoop::with_user_event();

    // Allow sharing between threads with EventLoopProxy / Arc
    let proxy = event_loop.create_proxy();
    let files = Arc::new(files);

    // Prevent malicious webpage from creating erroneous IPC calls
    let mut ipc_cookie = Arc::new(Mutex::new(0f64));

    let mut menu = MenuBar::new();
    let reload_menu_id = menu.add_item(MenuItemAttributes::new("Reload")).id();
    let skip_menu_id = menu.add_item(MenuItemAttributes::new("Skip")).id();
    let mut about_menu = MenuBar::new();
    about_menu.add_item(
        MenuItemAttributes::new(&format!(
            "mod_browser_webview version {}",
            env!("CARGO_PKG_VERSION")
        ))
        .with_enabled(false),
    );
    let source_menu_id = about_menu
        .add_item(MenuItemAttributes::new("Source code... (on GitHub)"))
        .id();
    let licenses_menu_id = about_menu
        .add_item(MenuItemAttributes::new("Licenses..."))
        .id();
    menu.add_submenu("About", true, about_menu);

    let window = WindowBuilder::new()
        .with_title(format!(
            "(1/{num_files}) mod_browser_webview {}",
            env!("CARGO_PKG_VERSION")
        ))
        .with_focused(true)
        .with_menu(menu)
        .build(&event_loop)
        .context("Failed to create webview window")?;
    let main_window_id = window.id();

    let mut webcontext = WebContext::new(data_dir);
    let webview = WebViewBuilder::new(window)
		.context("Failed to create webview")?
		.with_html(format!(
			"{}<script>window.location.href = {};</script>",
			include_str!("loading.html"),
			json::stringify(provider.file_page(&files[0].url, &files[0].id))
		))
		.context("Failed to load HTML")?
		.with_web_context(&mut webcontext)
		.with_navigation_handler(closure!(clone proxy, clone files, clone cur_file, |uri: String| {
			let (evt, nav) = handle_uri(uri.clone(), false, &files[*cur_file.lock().unwrap()], provider);
			if cfg!(debug_assertions) {
				eprintln!("Navigating {} -> {:?} (allow: {})", &uri, evt, nav);
			}
			_ = proxy.send_event(evt);
			nav
		}))
		.with_new_window_req_handler(closure!(clone proxy, clone files, clone cur_file, |uri: String| {
			let (evt, nav) = handle_uri(uri.clone(), true, &files[*cur_file.lock().unwrap()], provider);
			if cfg!(debug_assertions) {
				eprintln!("New window: {} -> {:?} (allow: {})", &uri, evt, nav);
			}
			_ = proxy.send_event(evt);
			nav
		}))
		.with_ipc_handler(closure!(clone proxy, clone ipc_cookie, |_window, data| {
			if let Ok(JsonValue::Object(obj)) = json::parse(&data) {
				// Check for IPC cookie
				if let Some(JsonValue::Number(num)) = obj.get("cookie") {
					let num: f64 = (*num).into();
					if (num - *ipc_cookie.lock().unwrap()).abs() < f64::EPSILON
					&& let (Some(t), Some(uri), Some(JsonValue::Boolean(value))) = (obj.get("type"), obj.get("uri"), obj.get("value"))
					&& t == "nonhttp"
					&& *value
					{
						_ = proxy.send_event(NavigationEvent::NonHTTPNavigationConfirmed(uri.to_string()));
					}
				}
			}
		}));

    let webview = webview
        .build()
        .attach_os_ctx()
        .context("Failed to create webview")?;
    let licenses_webview: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        let res = match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                // If main window, quit; otherwise close licenses webview
                if window_id == main_window_id {
                    *control_flow = ControlFlow::Exit;
                } else {
                    *licenses_webview.borrow_mut() = None;
                }
                Ok(())
            }
            // Do nothing: handled by caller!
            Event::UserEvent(NavigationEvent::Navigation(_)) => Ok(()),
            // Launch default browser to handle URI
            Event::UserEvent(NavigationEvent::ExternalNavigation(uri)) => {
                open::that(uri).context("Failed to open external link in new window")
            }
            Event::UserEvent(NavigationEvent::DownloadUrl(uri)) => {
                let mut f = cur_file.lock().unwrap();
                // Return file to client
                println!("{} {}", f, uri);
                *f += 1;
                update_page(f, &webview, &files, num_files, control_flow, provider);
                Ok(())
            }
            Event::UserEvent(NavigationEvent::NonHTTPNavigation(uri)) => {
                if uri.starts_with("curseforge://") {
                    create_confirm(
                        &webview,
                        &mut ipc_cookie,
                        object! {
                            type: "nonhttp",
                            uri: uri
                        },
                        r#"curseforge:// link opened:
This link is intended to open the CurseForge launcher, and will not let you download this file.
Use the "Download" button instead to download this file in your current program.
Do you want to continue opening the CurseForge launcher anyway?"#,
                    )
                } else {
                    create_confirm(
                        &webview,
                        &mut ipc_cookie,
                        object! {
                            type: "nonhttp",
                            uri: uri.clone()
                        },
                        &format!(
                            r#"External link opened: {uri}
This link is intended to open an external program, and will not let you download this file.
Use the "Download" button instead to download this file in your current program.
Do you want to continue opening the external program anyway?"#
                        ),
                    )
                }
                .context("Failed to create non-http URI prompt")
            }
            Event::UserEvent(NavigationEvent::NonHTTPNavigationConfirmed(uri)) => {
                open::that(uri).context("Failed to open non-http navigation in new window")
            }
            Event::UserEvent(NavigationEvent::BadNavigationWrongPage) => create_alert(
                &webview,
                match provider {
                    Provider::CurseForge => {
                        r#"Wrong link opened:
Please click the correct download button, below "File Details""#
                    }
                    Provider::Modrinth => {
                        r#"Wrong link opened:
Please use the Download button on the requested version's page"#
                    }
                },
            )
            .context("Failed to display wrong link message"),
            Event::UserEvent(NavigationEvent::BadNavigationNewWindow) => create_alert(
                &webview,
                r#"Link opened in new window:
Please use the primary mouse button to open this link"#,
            )
            .context("Failed to display new window message"),
            Event::MenuEvent { menu_id, .. } => {
                // Handle menu buttons: licenses, source, reload, skip
                match menu_id {
                    id if id == licenses_menu_id => match show_licenses(event_loop) {
                        Ok(view) => {
                            *licenses_webview.borrow_mut() = Some(view);
                            Ok(())
                        }
                        Err(err) => Err(err),
                    },
                    id if id == source_menu_id => open::that(env!("CARGO_PKG_REPOSITORY"))
                        .context("Failed to open source link"),
                    id if id == reload_menu_id => {
                        let f = cur_file.lock().unwrap();
                        update_page(f, &webview, &files, num_files, control_flow, provider);
                        Ok(())
                    }
                    id if id == skip_menu_id => {
                        let mut f = cur_file.lock().unwrap();
                        *f += 1;
                        update_page(f, &webview, &files, num_files, control_flow, provider);
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        };

        if let Err(e) = res {
            println!("ERROR");
            println!("{:?}", e);
            *control_flow = ControlFlow::ExitWithCode(1);
        }
    });
}

fn parse_url_line(line: &str, provider: Provider) -> Result<(&str, &str), anyhow::Error> {
    let split: Vec<_> = line.split(' ').collect();
    ensure!(
        split.len() == 2,
        "Invalid line format (requires ID, then space, then base project URL)"
    );
    ensure!(
        provider.id_valid(split[0]),
        "Invalid file/version ID for provider"
    );
    ensure!(
        provider.project_url_valid(split[1]),
        "Invalid URL (must be a project URL of the selected provider)"
    );
    Ok((split[0], split[1]))
}

#[derive(Debug)]
enum NavigationEvent {
    // External protocols, like curseforge:// URI (can allow, just with a prompt)
    NonHTTPNavigation(String),
    // External protocols after confirming by user
    NonHTTPNavigationConfirmed(String),
    // edge.forgecdn.net/media.forgecdn.net or cdn.modrinth.com
    DownloadUrl(String),
    // File page, file download page, download/file page
    #[allow(dead_code)]
    Navigation(String),
    // Wrong file/download page, general download page
    BadNavigationWrongPage,
    // Anything from Navigation, but when opened as a new window
    BadNavigationNewWindow,
    // Anything else
    ExternalNavigation(String),
}

fn handle_uri(
    uri: String,
    is_new_window: bool,
    file: &File,
    provider: Provider,
) -> (NavigationEvent, bool) {
    let nav_regex = provider.nav_regex(&file.id);

    // Internal (for the loading screen)
    if uri.starts_with("data:") || uri.starts_with("http://localhost") {
        return (NavigationEvent::Navigation(uri), true);
    }
    // Ignore about:blank
    if uri == "about:blank" {
        return (NavigationEvent::Navigation(uri), true);
    }
    if !uri.starts_with("http://") && !uri.starts_with("https://") {
        return (NavigationEvent::NonHTTPNavigation(uri), false);
    }
    if provider.download_url_valid(&uri) {
        return (NavigationEvent::DownloadUrl(uri), false);
    }
    if nav_regex.is_match(&uri) {
        if is_new_window {
            return (NavigationEvent::BadNavigationNewWindow, false);
        }
        return (NavigationEvent::Navigation(uri), true);
    }
    // Allow browsing within the requested project's own pages (Modrinth shows
    // the download button on the project's version pages).
    if provider == Provider::Modrinth && uri.starts_with(&file.url) && !provider.bad_nav_valid(&uri)
    {
        return (NavigationEvent::Navigation(uri), true);
    }
    if provider.bad_nav_valid(&uri) {
        return (NavigationEvent::BadNavigationWrongPage, false);
    }

    // Disabled on linux: WebKitGTK doesn't distinguish between top-level navigation and iframe loads
    // (https://lists.webkit.org/pipermail/webkit-gtk/2017-February/002924.html)
    if cfg!(target_os = "linux") {
        (NavigationEvent::Navigation(uri), true)
    } else {
        (NavigationEvent::ExternalNavigation(uri), false)
    }
}

fn show_licenses(event_loop: &EventLoopWindowTarget<NavigationEvent>) -> anyhow::Result<WebView> {
    let window = WindowBuilder::new()
        .with_title("mod_browser_webview licenses")
        .with_focused(true)
        .build(event_loop)
        .context("Failed to create webview window")?;
    let webview = WebViewBuilder::new(window)
        .context("Failed to create webview")?
        .with_html(include_str!("licenses.html"))
        .context("Failed to load HTML")?
        .with_navigation_handler(|uri| {
            if (uri.starts_with("http://") || uri.starts_with("https://"))
                && !uri.starts_with("http://localhost")
            {
                // Open HTTP links externally (rather than in the webview)
                open::that(uri).unwrap();
                false
            } else {
                true
            }
        });
    webview.build().context("Failed to create webview")
}

fn update_page(
    f: MutexGuard<usize>,
    webview: &WebView,
    files: &[File],
    num_files: usize,
    control_flow: &mut ControlFlow,
    provider: Provider,
) {
    if *f >= num_files {
        // No more files: quit!
        *control_flow = ControlFlow::Exit;
    } else {
        // Load next file
        webview.load_url(&provider.file_page(&files[*f].url, &files[*f].id));
        webview.window().set_title(&format!(
            "({}/{num_files}) mod_browser_webview {}",
            *f + 1,
            env!("CARGO_PKG_VERSION")
        ));
    }
}

// Add OS context for installing the right webview type
trait OsCtx<T> {
    fn attach_os_ctx(self) -> anyhow::Result<T>;
}

impl<T, E> OsCtx<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[cfg(target_os = "windows")]
    fn attach_os_ctx(self) -> anyhow::Result<T> {
        self.context(
			"Webview2 is required for this application - get it from https://go.microsoft.com/fwlink/p/?LinkId=2124703",
		)
    }

    #[cfg(target_os = "linux")]
    fn attach_os_ctx(self) -> anyhow::Result<T> {
        // (will probably fail at the point of program loading, since it is linked)
        self.context("WebKitGTK is required for this application")
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    fn attach_os_ctx(self) -> anyhow::Result<T> {
        self.context("A webview is required for this application")
    }
}
