//! Independent check of a supervised interactive episode: the worker's
//! answers recorded in the controller's ledger for the dossier no recording
//! covers are compared with `close_program` on that dossier.
//!
//! Two explicit paths. The regression reads the certified copy archived in
//! the repository (`docs/legal-geometry/exploration/interactive-001/ledger`)
//! and therefore works on a clean checkout. The check of a *new* run reads
//! `target/exploration/interactive/ledger` and fails with an explicit
//! message when no run exists; it is selected by name, never by default.
//! Run individually through `scripts/test_safe.py --test-target boundary`.
use residuum::typed::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::wire::{WireLimits, decode_response};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[derive(Default)]
struct CountingClock(usize);
impl residuum::DerivationClock for CountingClock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}

fn events(path: &PathBuf) -> Vec<serde_json::Value> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("ledger missing at {}", path.display()))
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn interactive_archived_ledger_forward_on_uncovered_dossier_matches_close_program() {
    check_ledger(root().join("docs/legal-geometry/exploration/interactive-001/ledger"));
}

/// Opt-in path for a fresh supervised run. The runner cannot select
/// `#[ignore]` tests, so the explicit marker is the environment variable
/// `RESIDUUM_CHECK_FRESH_LEDGER=1`: without it the test reports that it
/// was not opted in and does nothing, so a full selection never depends on
/// a run that a clean checkout does not have.
#[test]
fn interactive_fresh_ledger_forward_on_uncovered_dossier_matches_close_program_opt_in() {
    if std::env::var("RESIDUUM_CHECK_FRESH_LEDGER").as_deref() != Ok("1") {
        eprintln!(
            "not opted in: set RESIDUUM_CHECK_FRESH_LEDGER=1 to check a fresh supervised run"
        );
        return;
    }
    let fresh = root().join("target/exploration/interactive/ledger");
    assert!(
        fresh.join("events.jsonl").is_file(),
        "no fresh supervised run under {}; run scripts/test_safe.py --interactive-manifest first",
        fresh.display()
    );
    check_ledger(fresh);
}

fn check_ledger(ledger: PathBuf) {
    let all = events(&ledger.join("events.jsonl"));
    let artifact = |digest: &str| {
        std::fs::read_to_string(ledger.join("artifacts/sha256").join(digest)).unwrap()
    };
    const DOSSIER: &str = "killingWithoutAcquisitionPurpose";
    // Session opened by begin_forward on the uncovered dossier.
    let mut session = None;
    let mut results: BTreeMap<String, String> = BTreeMap::new();
    let mut requests: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for e in &all {
        let p = &e["payload"];
        match e["kind"].as_str().unwrap() {
            "action.validated" => {
                requests.insert(
                    p["action_id"].as_str().unwrap().into(),
                    p["request"].clone(),
                );
            }
            "action.result" => {
                results.insert(
                    p["action_id"].as_str().unwrap().into(),
                    p["artifact"].as_str().unwrap().into(),
                );
            }
            _ => {}
        }
    }
    for (id, req) in &requests {
        if req["begin_forward"]["dossier"] == DOSSIER {
            let line: serde_json::Value = serde_json::from_str(&artifact(&results[id])).unwrap();
            session = Some(
                line["response"]["payload"]["session_opened"]["session"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
        }
    }
    let session = session.expect("the policy opens a forward session on the uncovered dossier");
    // Every advance on that session, decoded through the wire DTOs.
    let mut known: Vec<Event> = Vec::new();
    let mut derivations: Vec<String> = Vec::new();
    let mut complete = false;
    for (id, req) in &requests {
        if req["advance_session"]["session"] == session.as_str() {
            let text = artifact(&results[id]);
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            let mut inner = v["response"].clone();
            inner["wire_version"] = serde_json::Value::String("residuum-wire/1".into());
            let response =
                decode_response(inner.to_string().as_bytes(), &WireLimits::default()).unwrap();
            if let residuum::experiments::worker::Payload::Forward {
                rounds,
                known: k,
                complete: c,
                ..
            } = response.payload
            {
                for r in rounds {
                    derivations.extend(
                        r.derivations
                            .iter()
                            .map(|d| format!("{:?}\t{:?}", d.event, d.record)),
                    );
                }
                known = k;
                complete = c;
            } else {
                panic!("not a forward payload");
            }
        }
    }
    assert!(
        complete,
        "the policy advances the uncovered dossier to completion"
    );
    // Independent closure of the same dossier with the same theory.
    let cases = residuum::load_formal_module(
        root().join("examples/experiments/legal_geometry/riggs_cases.res"),
    )
    .unwrap();
    let seeds = cases
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == DOSSIER => Some(e.input.clone()),
            _ => None,
        })
        .unwrap();
    let full = residuum::close_program(seeds, &cases, &mut CountingClock::default()).unwrap();
    let wire_known: BTreeSet<String> = known.iter().map(|e| format!("{e:?}")).collect();
    let full_known: BTreeSet<String> = full.known.iter().map(|e| format!("{e:?}")).collect();
    assert_eq!(wire_known, full_known, "events of the uncovered dossier");
    let mut expected: Vec<String> = full
        .derivations
        .iter()
        .map(|d| format!("{:?}\t{:?}", d.event, d.record))
        .collect();
    derivations.sort();
    expected.sort();
    assert_eq!(
        derivations, expected,
        "justifications of the uncovered dossier, with multiplicity"
    );
    assert!(!derivations.is_empty());
}
