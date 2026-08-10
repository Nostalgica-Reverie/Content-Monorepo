use fancy_regex::Regex;
use url::Url;

use crate::ProviderError;

pub const DEFAULT_ASSET_PATTERN: &str = r"^.+(?<!-api|-dev|-dev-preshadow|-sources)\.jar$";

pub(crate) fn repository_reference(
	input: &str,
	default_instance: &str,
	required_instance: Option<&str>,
) -> Result<(String, String), ProviderError> {
	let input = input.trim();
	let (instance, slug) = if input.contains("://") {
		let url =
			Url::parse(input).map_err(|error| ProviderError::InvalidUrl(error.to_string()))?;
		if !matches!(url.scheme(), "http" | "https") {
			return Err(ProviderError::InvalidUrl(input.to_string()));
		}
		let host = url
			.host_str()
			.ok_or_else(|| ProviderError::InvalidUrl(input.to_string()))?;
		let instance = match url.port() {
			Some(port) => format!("{host}:{port}"),
			None => host.to_string(),
		};
		if required_instance.is_some_and(|required| !host.eq_ignore_ascii_case(required)) {
			return Err(ProviderError::InvalidUrl(input.to_string()));
		}
		let slug = url
			.path_segments()
			.ok_or_else(|| ProviderError::InvalidUrl(input.to_string()))?
			.filter(|segment| !segment.is_empty())
			.take(2)
			.collect::<Vec<_>>()
			.join("/");
		(instance, slug)
	} else {
		(default_instance.to_string(), input.to_string())
	};
	let slug = slug.trim_end_matches('/').trim_end_matches(".git");
	let parts: Vec<_> = slug.split('/').collect();
	if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) {
		return Err(ProviderError::InvalidRepository(input.to_string()));
	}
	Ok((instance, slug.to_string()))
}

pub(crate) fn selected_asset(
	names: impl IntoIterator<Item = String>,
	pattern: &str,
) -> Result<usize, ProviderError> {
	let regex = Regex::new(pattern)
		.map_err(|error| ProviderError::InvalidAssetPattern(error.to_string()))?;
	let mut matches = Vec::new();
	for (index, name) in names.into_iter().enumerate() {
		if regex
			.is_match(&name)
			.map_err(|error| ProviderError::InvalidAssetPattern(error.to_string()))?
		{
			matches.push(index);
		}
	}
	if matches.len() != 1 {
		return Err(ProviderError::AmbiguousAssets {
			pattern: pattern.to_string(),
			count: matches.len(),
		});
	}
	Ok(matches[0])
}

pub(crate) fn asset_pattern(requested: Option<&str>) -> String {
	requested.unwrap_or(DEFAULT_ASSET_PATTERN).to_string()
}

pub(crate) fn slugify_name(name: &str) -> String {
	let lower = name.to_ascii_lowercase();
	let without_brackets = match (lower.find('('), lower.rfind(')')) {
		(Some(start), Some(end)) if start < end => {
			format!("{}{}", &lower[..start], &lower[end + 1..])
		}
		_ => lower,
	};
	let without_suffix = without_brackets
		.find(" - ")
		.map_or(without_brackets.as_str(), |index| {
			&without_brackets[..index]
		});
	let mut slug = String::new();
	for character in without_suffix.chars() {
		if character.is_ascii_lowercase() || character.is_ascii_digit() {
			slug.push(character);
		} else if !slug.ends_with('-') {
			slug.push('-');
		}
	}
	slug.trim_matches('-').to_string()
}

pub(crate) fn release_channel_allowed(channels: &[crate::ReleaseChannel]) -> bool {
	channels.is_empty() || channels.contains(&crate::ReleaseChannel::Release)
}

#[cfg(test)]
mod tests {
	use super::{DEFAULT_ASSET_PATTERN, repository_reference, selected_asset, slugify_name};

	#[test]
	fn default_pattern_rejects_development_classifiers() {
		assert_eq!(
			selected_asset(
				["mod-api.jar", "mod-dev.jar", "mod-sources.jar", "mod.jar"].map(str::to_string),
				DEFAULT_ASSET_PATTERN,
			)
			.unwrap(),
			3
		);
	}

	#[test]
	fn repository_urls_and_names_match_go_conventions() {
		assert_eq!(
			repository_reference(
				"https://github.com/Owner/Repo/releases/latest",
				"github.com",
				Some("github.com"),
			)
			.unwrap(),
			("github.com".into(), "Owner/Repo".into())
		);
		assert_eq!(
			slugify_name("Example Mod (Fabric) - Releases"),
			"example-mod"
		);
	}
}
