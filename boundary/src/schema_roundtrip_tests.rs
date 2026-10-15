//! End of the schema round trip: envelopes produced from strict-schema tool
//! calls by the Python adapter, local validation and dispatcher are decoded
//! by the wire DTOs and re-encoded to the same JSON. Reads
//! `target/exploration/schema-roundtrip/envelopes.jsonl`, written by
//! `test_provider_openai.SchemaRoundTripTests`; fails explicitly when absent.
//! Run individually through `scripts/test_safe.py --test-target boundary`.
use std::path::PathBuf;

use crate::wire::{WireLimits, decode_envelope, encode_envelope};

#[test]
fn schema_conforming_envelopes_decode_and_reencode_unchanged() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/exploration/schema-roundtrip/envelopes.jsonl");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing {}: run the python test SchemaRoundTripTests first",
            path.display()
        )
    });
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
