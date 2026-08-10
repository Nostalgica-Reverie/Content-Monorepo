//! Content-addressed metadata freshness, end to end over a real HTTP stub.
//!
//! The claim being tested is specifically about *requests*, not about
//! correctness of the bytes: a version document whose parent still vouches
//! for the same sha1 must cost zero round trips on a second boot, and a
//! document whose declared digest changed must be refetched even though
//! nothing expired.

use packwand_minecraft::http::HttpClient;
use packwand_minecraft::{MetadataClient, MetadataEndpoints, UreqClient};
use packwand_net::testing::{Reply, StubServer};

fn sha1_of(bytes: &[u8]) -> String {
	packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha1, bytes)
}

const VERSION_DOC: &[u8] = br#"{"id":"1.21.1","mainClass":"net.minecraft.client.main.Main"}"#;
const CHANGED_DOC: &[u8] = br#"{"id":"1.21.1","mainClass":"net.minecraft.client.main.Other"}"#;

fn endpoints(server: &StubServer) -> MetadataEndpoints {
	MetadataEndpoints {
		version_manifest_url: server.url("/manifest.json"),
		..MetadataEndpoints::default()
	}
}

fn manifest_body(server: &StubServer, doc: &[u8]) -> Vec<u8> {
	format!(
		r#"{{"latest":{{"release":"1.21.1","snapshot":"1.21.1"}},
           "versions":[{{"id":"1.21.1","type":"release","url":"{}","sha1":"{}"}}]}}"#,
		server.url("/1.21.1.json"),
		sha1_of(doc)
	)
	.into_bytes()
}

#[test]
fn a_version_document_whose_digest_still_matches_costs_no_request() {
	let cache_dir = tempfile::tempdir().unwrap();
	// Two stubs, because a manifest has to name the document server's URL and
	// that is only known once it has bound a port. Splitting them also makes
	// the hit count unambiguous: it counts document fetches only.
	let docs = StubServer::start([(
		"/1.21.1.json".to_string(),
		Reply::body(VERSION_DOC.to_vec()),
	)]);
	let meta = StubServer::start([(
		"/manifest.json".to_string(),
		Reply::body(manifest_body(&docs, VERSION_DOC)),
	)]);

	let http = UreqClient::new().with_document_cache(cache_dir.path());
	let client = MetadataClient::new(&http, endpoints(&meta));

	let manifest = client.fetch_manifest().unwrap();
	let entry = manifest.find("1.21.1").unwrap();
	let first = client.fetch_version(entry).unwrap();
	assert_eq!(first.value.id, "1.21.1");
	assert_eq!(docs.hits("/1.21.1.json"), 1, "the first fetch downloads");

	// Second boot: same digest in the manifest, so the document is not asked
	// for again — not even conditionally.
	let second = client.fetch_version(entry).unwrap();
	assert_eq!(second.value.id, "1.21.1");
	assert_eq!(
		docs.hits("/1.21.1.json"),
		1,
		"a document the parent still vouches for was refetched"
	);
}

#[test]
fn a_changed_digest_refetches_even_though_nothing_expired() {
	let cache_dir = tempfile::tempdir().unwrap();
	let server = StubServer::start([(
		"/1.21.1.json".to_string(),
		Reply::body(VERSION_DOC.to_vec()),
	)]);
	let http = UreqClient::new().with_document_cache(cache_dir.path());

	// Prime the cache with the original body and digest.
	let url = server.url("/1.21.1.json");
	let bytes = http
		.get_child_document(&url, Some(&sha1_of(VERSION_DOC)))
		.unwrap();
	assert_eq!(bytes, VERSION_DOC);
	assert_eq!(server.hits("/1.21.1.json"), 1);

	// The parent now declares a different digest. Nothing about the cache
	// entry expired; the digest alone is what makes it stale.
	let refetched = http
		.get_child_document(&url, Some(&sha1_of(CHANGED_DOC)))
		.unwrap();
	assert_eq!(
		server.hits("/1.21.1.json"),
		2,
		"a document whose declared digest changed was served from cache"
	);
	// The stub still serves the old body, which no longer matches — the
	// caller's own verification is what rejects it, and the mismatched body
	// must not have been cached.
	assert_eq!(refetched, VERSION_DOC);
	let again = http
		.get_child_document(&url, Some(&sha1_of(CHANGED_DOC)))
		.unwrap();
	assert_eq!(again, VERSION_DOC);
	assert_eq!(
		server.hits("/1.21.1.json"),
		3,
		"a body that failed its declared digest was cached anyway"
	);
}

#[test]
fn without_a_declared_digest_nothing_is_cached_or_skipped() {
	let cache_dir = tempfile::tempdir().unwrap();
	let server = StubServer::start([("/free.json".to_string(), Reply::body(VERSION_DOC.to_vec()))]);
	let http = UreqClient::new().with_document_cache(cache_dir.path());
	let url = server.url("/free.json");
	http.get_child_document(&url, None).unwrap();
	http.get_child_document(&url, None).unwrap();
	assert_eq!(
		server.hits("/free.json"),
		2,
		"a document with no declared digest must not be served from cache"
	);
}
