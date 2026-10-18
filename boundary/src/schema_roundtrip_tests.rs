//! End of the schema round trip: envelopes produced from strict-schema tool
//! calls by the Python adapters, local validation and the dispatcher are
//! decoded by the wire DTOs and re-encoded to the same JSON.
//!
//! The regression reads the certified copy archived in the repository
//! (`docs/legal-geometry/exploration/schema-roundtrip/envelopes.jsonl`) and
//! does not depend on test order. Checking a freshly regenerated file under
//! `target/` is opt-in through `RESIDUUM_CHECK_FRESH_SCHEMA=1`; the Python
//! test that regenerates it also compares it with the archived copy.
//! Run individually through `scripts/test_safe.py --test-target boundary`.
use std::path::PathBuf;

use crate::wire::{WireLimits, decode_envelope, encode_envelope};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn check(path: PathBuf) {
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {}", path.display()));
    let mut count = 0;
    for line in text.lines() {
        let envelope = decode_envelope(line.as_bytes(), &WireLimits::default())
            .unwrap_or_else(|e| panic!("{e}: {line}"));
        let again: serde_json::Value =
            serde_json::from_str(&encode_envelope(&envelope).unwrap()).unwrap();
        let original: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(again, original, "{line}");
        count += 1;
    }
    assert_eq!(count, 9);
}

#[test]
fn schema_conforming_envelopes_decode_and_reencode_unchanged() {
    check(root().join("docs/legal-geometry/exploration/schema-roundtrip/envelopes.jsonl"));
}

#[test]
fn schema_conforming_fresh_envelopes_decode_and_reencode_unchanged_opt_in() {
    if std::env::var("RESIDUUM_CHECK_FRESH_SCHEMA").as_deref() != Ok("1") {
        eprintln!("not opted in: set RESIDUUM_CHECK_FRESH_SCHEMA=1 to check a regenerated file");
        return;
    }
    check(root().join("target/exploration/schema-roundtrip/envelopes.jsonl"));
}
