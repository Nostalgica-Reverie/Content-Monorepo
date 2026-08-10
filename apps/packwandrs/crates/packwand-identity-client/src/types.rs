use serde::{Deserialize, Serialize};

/// Public identity information returned by the ATProto bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
	pub did: String,
	pub handle: String,
	pub pds: String,
}

/// A stable ATProto repository record reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrongRef {
	pub uri: String,
	pub cid: String,
}

/// A record returned by `com.atproto.repo.listRecords`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
	pub uri: String,
	pub cid: String,
	pub value: serde_json::Value,
}

/// One page of records and its optional continuation cursor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordPage {
	pub records: Vec<Record>,
	#[serde(default)]
	pub cursor: Option<String>,
}

/// An IPLD CID link encoded in ATProto JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CidLink {
	#[serde(rename = "$link")]
	pub cid: String,
}

/// A blob uploaded to the signed-in account's PDS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobRef {
	#[serde(rename = "$type")]
	pub kind: String,
	#[serde(rename = "ref")]
	pub reference: CidLink,
	#[serde(rename = "mimeType")]
	pub mime_type: String,
	pub size: u64,
}

/// Summary of the local manifest published with a shared pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSummary {
	pub id: String,
	#[serde(rename = "type")]
	pub project_type: String,
	pub version: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub minecraft_version: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub loader: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub environment: Option<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub variants: Vec<String>,
}

/// Content used to create a `net.nostalgica.packwand.pack` record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackShare {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	pub manifest: ManifestSummary,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tangled_repo: Option<StrongRef>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub git_remote: Option<String>,
}

/// A mutual Bluesky follow or explicit Packwand contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
	pub did: String,
	#[serde(default)]
	pub handle: String,
	#[serde(default)]
	pub display_name: String,
	#[serde(default)]
	pub avatar: String,
	#[serde(default)]
	pub pds: String,
	#[serde(default)]
	pub sources: Vec<String>,
}

/// An unexpired Packwand collaboration invitation found in a friend's repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingInvite {
	pub from: String,
	#[serde(default)]
	pub from_handle: String,
	pub invite: String,
	pub created_at: String,
	pub expires_at: String,
	pub uri: String,
	pub cid: String,
}

/// A Tangled repository record linked to an ATProto identity by Bobbin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TangledRepo {
	pub uri: String,
	pub cid: String,
	pub value: serde_json::Value,
}
