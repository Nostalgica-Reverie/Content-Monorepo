use std::str::FromStr;

use packwand_pack::{HashFormat, Hasher, hash_bytes};

#[test]
fn matches_go_golden_vectors() {
	let vectors = [
		(
			b"".as_slice(),
			[
				("sha1", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
				(
					"sha256",
					"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
				),
				(
					"sha512",
					"cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
				),
				("md5", "d41d8cd98f00b204e9800998ecf8427e"),
				("murmur2", "1540447798"),
				("length-bytes", "0"),
			],
		),
		(
			b"packwiz".as_slice(),
			[
				("sha1", "0ea33eefe7606ab0a35bb7301e9d31f530a76a64"),
				(
					"sha256",
					"2cc884ee64ae3d5312e0ad713857d0c97b4d6b5c9f657878b489af7ccbd9a2c1",
				),
				(
					"sha512",
					"9e295fc75ab988af67b2b03bcb07a71198f23c36f0a62f61c363ea86a84724b5ba1987edb6b03622dca2f1c691efcc05b8dea1d6fa234a7f9254f583eb563fd9",
				),
				("md5", "c25cb1fdc83dc8ab3a5a8b4887a3dfe8"),
				("murmur2", "2676380970"),
				("length-bytes", "7"),
			],
		),
		(
			b"packwand: the quick brown fox jumps over the lazy dog".as_slice(),
			[
				("sha1", "ad0c069c8f6da3f7c09359f2047506a93b0ac66a"),
				(
					"sha256",
					"86c9917988ef302683f6dbcbb28b0adb34ae39d37a75dfcec575b61982987a3a",
				),
				(
					"sha512",
					"c0639fad2036a0a95999e04e3862755f7361e6a5944df6b5df6574efb14a3e85a06b22e1e3367a41bcc96325c5077936b2716ba0c89bdb0e6326696efb5fc215",
				),
				("md5", "872e9fd24efc757bdc004c0896a099d3"),
				("murmur2", "574953714"),
				("length-bytes", "53"),
			],
		),
	];
	for (input, expected) in vectors {
		for (name, expected) in expected {
			let format = HashFormat::from_str(name).unwrap();
			assert_eq!(hash_bytes(format, input), expected, "{name}");
		}
	}
}

#[test]
fn curseforge_variant_strips_only_expected_whitespace() {
	for input in ["helloworld", "hello \tworld\r\n", " h e l l o w o r l d "] {
		assert_eq!(
			hash_bytes(HashFormat::Murmur2, input.as_bytes()),
			"2824650221"
		);
	}
}

#[test]
fn incremental_writes_match_one_shot_hashes() {
	let input = b"packwand: the quick brown fox jumps over the lazy dog";
	for format in [
		HashFormat::Sha1,
		HashFormat::Sha256,
		HashFormat::Sha512,
		HashFormat::Md5,
		HashFormat::Murmur2,
		HashFormat::LengthBytes,
	] {
		let mut incremental = Hasher::new(format);
		for chunk in input.chunks(7) {
			incremental.update(chunk);
		}
		assert_eq!(incremental.finish(), hash_bytes(format, input));
	}
}

#[test]
fn format_lookup_is_case_insensitive_and_rejects_unknown_values() {
	assert_eq!(HashFormat::from_str("SHA512").unwrap(), HashFormat::Sha512);
	assert!(HashFormat::from_str("blake3").is_err());
}
