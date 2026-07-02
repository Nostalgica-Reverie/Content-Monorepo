use std::{
    io::{self, BufRead, BufReader},
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{ensure, Context};
use closure::closure;
use json::{object, JsonValue};
use lazy_static::lazy_static;
use rand::Rng;
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

fn main() {
    println!("curseforge_webview {}", env!("CARGO_PKG_VERSION"));

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
    *cookie = rand::thread_rng().gen::<f64>();
    webview.evaluate_script(&format!(
        r#"window.ipc.postMessage(JSON.stringify({{...{},cookie:{},value:confirm({})}}))"#,
        json::stringify(ipc_data),
        json::stringify(*cookie),
        json::stringify(message)
    ))
}

struct File {
    id: u32,
    url: String,
}

fn start() -> anyhow::Result<()> {
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
                let (id, url) = parse_url_line(&line).context("Failed to read URL")?;
                files.push(File {
                    id,
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
            "curseforge_webview version {}",
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
            "(1/{num_files}) curseforge_webview {}",
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
		.with_html(&format!(
			"{}<script>window.location.href = {};</script>",
			include_str!("loading.html"),
			json::stringify(format!("{}/files/{}", files[0].url, files[0].id))
		))
		.context("Failed to load HTML")?
		.with_web_context(&mut webcontext)
		.with_navigation_handler(closure!(clone proxy, clone files, clone cur_file, |uri: String| {
			let (evt, nav) = handle_uri(uri.clone(), false, files[*cur_file.lock().unwrap()].id);
			if cfg!(debug_assertions) {
				eprintln!("Navigating {} -> {:?} (allow: {})", &uri, evt, nav);
			}
			_ = proxy.send_event(evt);
			nav
		}))
		.with_new_window_req_handler(closure!(clone proxy, clone files, clone cur_file, |uri: String| {
			let (evt, nav) = handle_uri(uri.clone(), true, files[*cur_file.lock().unwrap()].id);
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
					if (num - *ipc_cookie.lock().unwrap()).abs() < f64::EPSILON {
						if let (Some(t), Some(uri), Some(JsonValue::Boolean(value))) = (obj.get("type"), obj.get("uri"), obj.get("value")) {
							if t == "nonhttp" && *value {
								_ = proxy.send_event(NavigationEvent::NonHTTPNavigationConfirmed(uri.to_string()));
							}
						}
					}
				}
			}
		}));

    let webview = webview
        .build()
        .attach_os_ctx()
        .context("Failed to create webview")?;
    let licenses_webview: Arc<Mutex<Option<WebView>>> = Arc::new(Mutex::new(None));

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
                    *licenses_webview.lock().unwrap() = None;
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
                update_page(f, &webview, &files, num_files, control_flow);
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
                r#"Wrong link opened:
Please click the correct download button, below "File Details""#,
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
                            *licenses_webview.lock().unwrap() = Some(view);
                            Ok(())
                        }
                        Err(err) => Err(err),
                    },
                    id if id == source_menu_id => open::that(env!("CARGO_PKG_REPOSITORY"))
                        .context("Failed to open source link"),
                    id if id == reload_menu_id => {
                        let f = cur_file.lock().unwrap();
                        update_page(f, &webview, &files, num_files, control_flow);
                        Ok(())
                    }
                    id if id == skip_menu_id => {
                        let mut f = cur_file.lock().unwrap();
                        *f += 1;
                        update_page(f, &webview, &files, num_files, control_flow);
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

fn parse_url_line(line: &str) -> Result<(u32, &str), anyhow::Error> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new("^https?://(?:(?:www|beta)\\.)?curseforge\\.com/[^/]+/[^/]+/[^/]+$")
                .unwrap();
    }

    let split: Vec<_> = line.split(' ').collect();
    ensure!(
        split.len() == 2,
        "Invalid line format (requires ID, then space, then base project URL)"
    );
    ensure!(
        RE.is_match(split[1]),
        "Invalid URL (must be a CurseForge project URL)"
    );
    let id: u32 = split[0].parse().context("Failed to parse file ID")?;
    Ok((id, split[1]))
}

#[derive(Debug)]
enum NavigationEvent {
    // External protocols, like curseforge:// URI (can allow, just with a prompt)
    NonHTTPNavigation(String),
    // External protocols after confirming by user
    NonHTTPNavigationConfirmed(String),
    // edge.forgecdn.net/media.forgecdn.net
    DownloadUrl(String),
    // File page, file download page, download/file page
    Navigation(String),
    // Wrong file/download page, general download page
    BadNavigationWrongPage,
    // Anything from Navigation, but when opened as a new window
    BadNavigationNewWindow,
    // Anything else
    ExternalNavigation(String),
}

fn handle_uri(uri: String, is_new_window: bool, file_id: u32) -> (NavigationEvent, bool) {
    lazy_static! {
        static ref DL_URL_REGEX: Regex = Regex::new("^https?://(?:edge|media)\\.forgecdn\\.net/files/.+$").unwrap();
        // Note: + after / due to bad path normalisation in beta redirect
        static ref BAD_NAV_REGEX: Regex = Regex::new("^https?://(?:(?:www|beta)\\.)?curseforge\\.com/+[^/]+/[^/]+/[^/]+/(?:files/[0-9]+|download)").unwrap();
    }
    let nav_regex = Regex::new(&format!(
        "^https?://(?:(?:www|beta)\\.)?curseforge\\.com/+[^/]+/[^/]+/[^/]+/(?:files|download)/{}",
        file_id
    ))
    .unwrap();

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
    if DL_URL_REGEX.is_match(&uri) {
        return (NavigationEvent::DownloadUrl(uri), false);
    }
    if nav_regex.is_match(&uri) {
        if is_new_window {
            return (NavigationEvent::BadNavigationNewWindow, false);
        }
        return (NavigationEvent::Navigation(uri), true);
    }
    if BAD_NAV_REGEX.is_match(&uri) {
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
        .with_title("curseforge_webview licenses")
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
) {
    if *f >= num_files {
        // No more files: quit!
        *control_flow = ControlFlow::Exit;
    } else {
        // Load next file
        webview.load_url(&format!("{}/files/{}", files[*f].url, files[*f].id));
        webview.window().set_title(&format!(
            "({}/{num_files}) curseforge_webview {}",
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
        self.context("WebKitGTK is required for this application");
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    fn attach_os_ctx(self) -> anyhow::Result<T> {
        self.context("A webview is required for this application");
    }
}
