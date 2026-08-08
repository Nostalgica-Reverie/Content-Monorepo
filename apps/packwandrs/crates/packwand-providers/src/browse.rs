//! Searching a provider's catalogue, as opposed to resolving one known project.
//!
//! [`ProviderResolver`](crate::ProviderResolver) answers "give me this exact
//! project"; browsing asks "what is there?". They are separate traits because
//! the shapes genuinely differ — a resolve returns one project with a chosen
//! file, a search returns a page of projects with no file decision made — and
//! because a provider can reasonably support one and not the other.
//!
//! This exists because the provider websites cannot be shown inside the app:
//! `modrinth.com` sends `X-Frame-Options: DENY` and both CurseForge sites send
//! `SAMEORIGIN` plus a Cloudflare challenge to non-browser clients, so an
//! iframe or a proxy is impossible. Rendering search results natively is the
//! only way to browse without leaving the window — and it is also faster,
//! themeable, and one click from "add to pack".

use serde::{Deserialize, Serialize};

use crate::ProviderError;

/// What to search for.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseQuery {
    /// Free text. Empty means "show me anything", which providers answer with
    /// their default ordering.
    #[serde(default)]
    pub text: String,
    /// Restrict to these loaders. Empty means no loader filter.
    #[serde(default)]
    pub loaders: Vec<String>,
    /// Restrict to these Minecraft versions. Empty means no version filter.
    #[serde(default)]
    pub game_versions: Vec<String>,
    /// Which kind of project. `None` means the provider's default (mods).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
    /// Zero-based offset, in projects.
    #[serde(default)]
    pub offset: u32,
    /// How many to return. Clamped by [`BrowseQuery::limit_or_default`].
    #[serde(default)]
    pub limit: u32,
}

impl BrowseQuery {
    /// The page size to actually request.
    ///
    /// Clamped rather than passed through: a caller asking for thousands would
    /// spend a provider's request budget on a page nobody scrolls, and both
    /// APIs cap it server-side anyway.
    #[must_use]
    pub fn limit_or_default(&self) -> u32 {
        match self.limit {
            0 => 20,
            other => other.min(100),
        }
    }
}

/// One project in a page of results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseProject {
    /// Provider-native id, which [`crate::ResolveRequest`] can be built from.
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub author: String,
    pub downloads: u64,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// The project's page on the provider's own site.
    pub page_url: String,
    /// The same project on Legacy CurseForge, when that applies.
    ///
    /// Legacy CurseForge is the same catalogue behind a different front end
    /// rather than a separate source, so it is a link on a result rather than
    /// a third provider to search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_page_url: Option<String>,
}

/// A page of results.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsePage {
    pub projects: Vec<BrowseProject>,
    /// Total matches the provider reports, for paging. Providers are
    /// inconsistent about how exact this is, so it drives "next page" but is
    /// not presented as a precise count.
    pub total: u64,
    pub offset: u32,
}

/// How a project's long description is encoded.
///
/// Providers disagree: Modrinth stores markdown, CurseForge serves rendered
/// HTML. Carried alongside the body rather than normalized here, because
/// turning either into safe markup is the *display* layer's job — a CLI
/// consumer wants neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BodyFormat {
    Markdown,
    Html,
}

/// Everything needed to read about a project without leaving the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseDetail {
    pub project: BrowseProject,
    /// The long description, unrendered and unsanitized.
    pub body: String,
    pub body_format: BodyFormat,
    #[serde(default)]
    pub gallery: Vec<GalleryImage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issues_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wiki_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryImage {
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
}

/// Searching a provider's catalogue, and reading one project from it.
pub trait ProviderBrowser {
    fn search(&self, query: &BrowseQuery) -> Result<BrowsePage, ProviderError>;

    /// The full record for one project, by its provider-native id.
    fn project(&self, id: &str) -> Result<BrowseDetail, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_size_is_clamped_at_both_ends() {
        assert_eq!(BrowseQuery::default().limit_or_default(), 20);
        assert_eq!(
            BrowseQuery {
                limit: 5,
                ..BrowseQuery::default()
            }
            .limit_or_default(),
            5
        );
        assert_eq!(
            BrowseQuery {
                limit: 10_000,
                ..BrowseQuery::default()
            }
            .limit_or_default(),
            100
        );
    }
}
