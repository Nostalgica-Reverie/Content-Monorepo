//! Turning a provider's project description into markup the webview may render.
//!
//! Mod descriptions are arbitrary content written by third parties and served
//! by a third party. Rendering them means putting someone else's markup inside
//! the application window, so this is the one place in the app where that is
//! allowed to happen — and it happens in Rust, before the webview ever sees the
//! bytes, so there is no window in which unsanitized markup exists in the DOM.
//!
//! The Content Security Policy already blocks script execution
//! (`script-src 'self'`), but CSP is a second line rather than the first: it
//! does nothing about markup that phishes, covers the UI, or exfiltrates
//! through an image URL. Sanitizing removes those; CSP catches whatever the
//! sanitizer would have missed.

use std::collections::HashSet;

use packwand_providers::BodyFormat;

/// Hosts whose images may appear in a rendered description.
///
/// Restricted rather than allowing any `https:` image: a remote image is a
/// request, and a request is a way for a page to learn that you opened it.
/// Screenshots and badges live on these hosts anyway.
///
/// **Must stay in sync with `img-src` in `tauri.conf.json`.** These are two
/// independent gates on the same thing — this one decides what survives
/// sanitizing, the CSP decides what the webview will actually fetch. A host
/// listed here but not there produces an image that silently never loads;
/// a host there but not here is simply unreachable.
const ALLOWED_IMAGE_HOSTS: [&str; 4] = [
    "cdn.modrinth.com",
    "media.forgecdn.net",
    "raw.githubusercontent.com",
    "user-images.githubusercontent.com",
];

/// Renders a project description into sanitized HTML.
pub fn render(body: &str, format: BodyFormat) -> String {
    let html = match format {
        BodyFormat::Markdown => markdown_to_html(body),
        BodyFormat::Html => body.to_owned(),
    };
    sanitize(&html)
}

fn markdown_to_html(source: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    // Deliberately not `ENABLE_SMART_PUNCTUATION`: it rewrites quotes and
    // dashes inside code spans too, which mangles command lines in install
    // instructions.
    let mut rendered = String::with_capacity(source.len());
    html::push_html(&mut rendered, Parser::new_ext(source, options));
    rendered
}

fn sanitize(html: &str) -> String {
    ammonia::Builder::default()
        // `rel` is rewritten rather than trusted, so a link cannot opt out of
        // `noopener` and get a handle on the app window.
        .link_rel(Some("noopener noreferrer nofollow"))
        // Only http(s): no `javascript:`, no `data:` (which can carry an SVG
        // with script in it), no `file:`.
        .url_schemes(HashSet::from_iter(["http", "https"]))
        .url_relative(ammonia::UrlRelative::Deny)
        .add_tag_attributes("img", ["loading"])
        .attribute_filter(|element, attribute, value| {
            // Images are the one element that fetches on its own, so their
            // host is checked rather than merely their scheme.
            if element == "img" && attribute == "src" && !image_host_allowed(value) {
                return None;
            }
            Some(value.into())
        })
        .clean(html)
        .to_string()
}

fn image_host_allowed(url: &str) -> bool {
    url::Url::parse(url).is_ok_and(|parsed| {
        parsed.scheme() == "https"
            && parsed.host_str().is_some_and(|host| {
                ALLOWED_IMAGE_HOSTS.contains(&host.to_ascii_lowercase().as_str())
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_becomes_structured_html() {
        let html = render(
            "# Title\n\nSome **bold** text.\n\n- one\n- two",
            BodyFormat::Markdown,
        );
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<li>one</li>"));
    }

    /// The reason this module exists. A description is written by whoever
    /// uploaded the mod.
    #[test]
    fn scripts_are_removed_from_both_formats() {
        let attack = "<script>alert(1)</script><p>after</p>";
        let from_html = render(attack, BodyFormat::Html);
        assert!(!from_html.contains("<script"));
        assert!(!from_html.contains("alert(1)"));
        assert!(from_html.contains("after"));

        // Markdown passes raw HTML through, so it needs the same treatment.
        let from_markdown = render(attack, BodyFormat::Markdown);
        assert!(!from_markdown.contains("<script"));
    }

    #[test]
    fn event_handlers_and_dangerous_schemes_are_removed() {
        let html = render(
            r#"<a href="javascript:alert(1)" onclick="alert(2)">click</a>"#,
            BodyFormat::Html,
        );
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("onclick"));
        // The text survives; only the mechanism is gone.
        assert!(html.contains("click"));
    }

    #[test]
    fn links_cannot_opt_out_of_noopener() {
        let html = render(
            r#"<a href="https://example.test" rel="opener">x</a>"#,
            BodyFormat::Html,
        );
        assert!(html.contains("noopener"));
        assert!(!html.contains(r#"rel="opener""#));
    }

    /// A remote image is a request, so only the providers' own CDNs are
    /// allowed to be fetched from inside the window.
    #[test]
    fn images_are_restricted_to_known_hosts() {
        let allowed = render(
            r#"<img src="https://cdn.modrinth.com/data/x/icon.png">"#,
            BodyFormat::Html,
        );
        assert!(allowed.contains("cdn.modrinth.com"));

        let blocked = render(
            r#"<img src="https://tracker.test/pixel.gif">"#,
            BodyFormat::Html,
        );
        assert!(!blocked.contains("tracker.test"));

        // A `data:` SVG can carry script, so it is not an escape hatch either.
        let data_uri = render(
            r#"<img src="data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=">"#,
            BodyFormat::Html,
        );
        assert!(!data_uri.contains("data:image"));
    }

    #[test]
    fn iframes_and_objects_do_not_survive() {
        for attack in [
            r#"<iframe src="https://evil.test"></iframe>"#,
            r#"<object data="https://evil.test"></object>"#,
            r#"<embed src="https://evil.test">"#,
            r#"<form action="https://evil.test"><input name="p"></form>"#,
        ] {
            let html = render(attack, BodyFormat::Html);
            assert!(!html.contains("evil.test"), "survived sanitizing: {attack}");
        }
    }

    #[test]
    fn an_empty_description_renders_to_nothing() {
        assert_eq!(render("", BodyFormat::Markdown), "");
        assert_eq!(render("", BodyFormat::Html), "");
    }
}

#[cfg(test)]
mod csp_agreement {
    /// The sanitizer allowlist and the CSP are two gates on the same thing,
    /// maintained in different files and different languages. A host in one
    /// and not the other fails silently — an image that is stripped, or one
    /// that is kept and then never fetched — so they are checked against each
    /// other here rather than by remembering.
    #[test]
    fn every_allowed_image_host_is_also_permitted_by_the_csp() {
        let config = include_str!("../../tauri.conf.json");
        let csp: serde_json::Value = serde_json::from_str(config).expect("config parses");
        let csp = csp["app"]["security"]["csp"]
            .as_str()
            .expect("the app declares a CSP");
        let img_src = csp
            .split(';')
            .map(str::trim)
            .find(|directive| directive.starts_with("img-src"))
            .expect("the CSP constrains img-src");
        for host in super::ALLOWED_IMAGE_HOSTS {
            assert!(
                img_src.contains(host),
                "{host} survives sanitizing but the CSP will not load it"
            );
        }
    }
}
