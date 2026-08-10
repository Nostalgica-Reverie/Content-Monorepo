//! The branching between "build a request" and "hand back bytes", against a
//! real socket. Every assertion here is on behaviour a parser-level unit test
//! cannot reach.

use std::time::Instant;

use packwand_net::testing::{Reply, StubServer};
use packwand_net::{Client, Freshness, MetaCache, NetError, Request, Source};

#[test]
fn a_stale_entry_revalidates_and_a_304_reuses_the_local_file() {
	let server = StubServer::start([(
		"/manifest.json".to_owned(),
		Reply::NotModifiedIfMatch {
			etag: "\"v1\"".to_owned(),
			body: b"{\"latest\":\"1.21.1\"}".to_vec(),
		},
	)]);
	let root = tempfile::tempdir().unwrap();
	let cache = MetaCache::open(root.path(), "meta").unwrap();
	let client = Client::api();
	let request = Request::get(server.url("/manifest.json"));

	// First time: nothing cached, so a full body.
	let first = client.get_cached(&request, &cache).unwrap();
	assert_eq!(first.source, Source::Network);
	assert_eq!(first.bytes, b"{\"latest\":\"1.21.1\"}");
	assert_eq!(server.hits("/manifest.json"), 1);

	// Forced revalidation: the stub only answers 304 when our If-None-Match
	// arrived, so the source proves the header was sent.
	let second = client
		.get_cached_with(&request, &cache, Freshness::AlwaysRevalidate)
		.unwrap();
	assert_eq!(second.source, Source::CacheRevalidated);
	assert_eq!(second.bytes, first.bytes, "304 reuses the stored body");
	assert_eq!(server.hits("/manifest.json"), 2);
}

#[test]
fn a_fresh_entry_is_served_without_touching_the_server() {
	let server = StubServer::start([(
		"/index.json".to_owned(),
		Reply::cacheable(b"cached".to_vec(), "\"e\"", 600),
	)]);
	let root = tempfile::tempdir().unwrap();
	let cache = MetaCache::open(root.path(), "meta").unwrap();
	let client = Client::api();
	let request = Request::get(server.url("/index.json"));

	assert_eq!(
		client.get_cached(&request, &cache).unwrap().source,
		Source::Network
	);
	let again = client.get_cached(&request, &cache).unwrap();
	assert_eq!(again.source, Source::CacheFresh);
	assert_eq!(again.bytes, b"cached");
	assert_eq!(server.hits("/index.json"), 1, "no second request at all");
}

#[test]
fn an_unreachable_server_serves_the_cached_copy_as_stale() {
	let root = tempfile::tempdir().unwrap();
	let cache = MetaCache::open(root.path(), "meta").unwrap();
	let client = Client::api();

	let url = {
		let server = StubServer::start([(
			"/manifest.json".to_owned(),
			Reply::cacheable(b"stored".to_vec(), "\"e\"", 0),
		)]);
		let url = server.url("/manifest.json");
		client.get_cached(&Request::get(&url), &cache).unwrap();
		url
	};

	// Server gone. The answer still comes back, but marked unconfirmed —
	// which is the whole point: resolving "latest release" off a week-old
	// document must be distinguishable from having checked.
	let offline = client
		.get_cached_with(&Request::get(&url), &cache, Freshness::AlwaysRevalidate)
		.unwrap();
	assert_eq!(offline.source, Source::Stale);
	assert!(!offline.source.is_current());
	assert_eq!(offline.bytes, b"stored");
}

#[test]
fn a_429_is_retried_after_the_delay_the_server_asked_for() {
	let server = StubServer::start([(
		"/limited".to_owned(),
		Reply::RetryAfter {
			times: 1,
			seconds: 1,
			body: b"eventually".to_vec(),
		},
	)]);
	let started = Instant::now();
	let bytes = Client::api()
		.get(&Request::get(server.url("/limited")))
		.unwrap();
	assert_eq!(bytes, b"eventually");
	assert_eq!(server.hits("/limited"), 2, "one refusal, one success");
	assert!(
		started.elapsed() >= std::time::Duration::from_secs(1),
		"honoured Retry-After rather than backing off on its own schedule"
	);
}

#[test]
fn a_404_fails_immediately_instead_of_being_retried() {
	let server = StubServer::start([]);
	let error = Client::api()
		.get(&Request::get(server.url("/missing")))
		.unwrap_err();
	assert_eq!(error.status(), Some(404));
	// A missing resource does not become present by asking again; retrying it
	// only delays the real error.
	assert_eq!(server.hits("/missing"), 1);
}

#[test]
fn a_mirror_takes_over_when_the_first_url_fails() {
	let broken = StubServer::start([("/lib.jar".to_owned(), Reply::Status(500))]);
	let good = StubServer::start([(
		"/lib.jar".to_owned(),
		Reply::body(b"from the mirror".to_vec()),
	)]);

	let request = Request::get(broken.url("/lib.jar")).mirror(good.url("/lib.jar"));
	let bytes = Client::downloads().get(&request).unwrap();

	assert_eq!(bytes, b"from the mirror");
	// 500 is transient, so the primary is exhausted before falling through.
	assert_eq!(broken.hits("/lib.jar"), 3);
	assert_eq!(good.hits("/lib.jar"), 1);
}

#[test]
fn a_download_that_fails_verification_leaves_the_target_alone() {
	let server = StubServer::start([("/mod.jar".to_owned(), Reply::body(b"wrong bytes".to_vec()))]);
	let root = tempfile::tempdir().unwrap();
	let target = root.path().join("mods/mod.jar");

	let checksum = packwand_net::Checksum::parse(
		"sha512",
		packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha512, b"right bytes"),
	)
	.unwrap();
	let error = Client::downloads()
		.download_to(
			&Request::get(server.url("/mod.jar")),
			&target,
			Some(&checksum),
			&mut |_, _| {},
		)
		.unwrap_err();

	assert!(matches!(error, NetError::Checksum { .. }));
	assert!(!target.exists(), "nothing unverified reaches the target");
}
