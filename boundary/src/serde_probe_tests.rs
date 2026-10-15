//! Minimal serde_json round trip of a boundary DTO. Measurement probe for
//! the transport decision; not the worker's serialisation.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProbeEnvelope {
    protocol_version: String,
    action_id: String,
    steps: usize,
    pending: Vec<String>,
}

#[test]
fn serde_probe_round_trips_a_boundary_dto() {
    let e = ProbeEnvelope {
        protocol_version: "residuum-worker/0.1".into(),
        action_id: "a1".into(),
        steps: 3,
        pending: vec!["x".into(), "ñ \"q\" \\ \u{1F600}".into()],
    };
    let text = serde_json::to_string(&e).unwrap();
    let back: ProbeEnvelope = serde_json::from_str(&text).unwrap();
    assert_eq!(back, e);
    assert!(serde_json::from_str::<ProbeEnvelope>("{\"protocol_version\": 1}").is_err());
    assert!(serde_json::from_str::<ProbeEnvelope>("{\"protocol_version\": \"v\"").is_err());
}
