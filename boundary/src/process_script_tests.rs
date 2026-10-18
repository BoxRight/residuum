//! Finite script for the bounded process check. This test writes, under
//! `target/exploration/process/`, the manifest file, the request lines and
//! the expected protocol output produced by the in-memory service, after
//! asserting that the in-memory service equals the direct worker line by
//! line. The runner's worker mode then feeds the same lines to the
//! `residuum-worker` binary, and its stdout is compared with
//! `expected.jsonl` outside this test. Run individually through
//! `scripts/test_safe.py --test-target boundary`.
use residuum::experiments::exploration::*;
use residuum::experiments::worker::*;
use residuum::typed::*;
use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::PathBuf;

use crate::manifest::from_manifest;
use crate::transport::{Line, TransportLimits, decode_line, serve};
use crate::wire::*;

const HEAD: &str = "hypotheticalSuccessionImpediment";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn fixture(name: &str) -> Module {
    residuum::load_formal_module(
        root().join(format!("examples/experiments/legal_geometry/{name}.res")),
    )
    .unwrap()
}

fn experiment<'a>(m: &'a Module, name: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}

const PURPOSEFUL: &str = "rule purposefulImpediment(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
";

fn manifest() -> Manifest {
    let base = fixture("riggs_cases");
    let attr = experiment(&base, "historicalRiggs").goal.clone();
    let mut imp = attr.clone();
    imp.verb = HEAD.into();
    let dossiers: BTreeMap<String, Vec<Event>> = base
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) => Some((e.name.clone(), e.input.clone())),
            _ => None,
        })
        .collect();
    Manifest {
        episode_id: "riggs-process-001".into(),
        manifest_hash: "h-process".into(),
        imports: vec![
            root().join("examples/experiments/legal_geometry/riggs_problem_manifest.res"),
            root().join("examples/experiments/legal_geometry/riggs_cases.res"),
        ],
        workdir: root().join("target/exploration/process/work"),
        profile: Profile::p1().with_problem(
            [HEAD],
            [
                "operativeDisposition",
                "deathRecorded",
                "survivorRecorded",
                "intentionalKilling",
                "deathCausedBy",
                "acquisitionPurposeRecorded",
                "testamentaryEntitlement",
                "acquisitionLinkedToWrong",
                "purposefulAcquisition",
                "patrimonialAsset",
            ],
            ["intentionalKilling", "acquisitionPurposeRecorded"],
        ),
        decision: Decision {
            components: vec![attr, imp],
            criterion: Criterion::Component(1),
        },
        dossiers,
        budgets: Budgets {
            max_actions: 40,
            max_steps_per_action: 8,
            max_steps_total: 200,
            max_sessions: 10,
            max_candidates: 6,
            max_clauses: 2,
            search_limits: residuum::SearchLimits::default(),
        },
    }
}

fn envelope(action: &str, request: Request) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.into(),
        episode_id: "riggs-process-001".into(),
        action_id: action.into(),
        manifest_hash: "h-process".into(),
        request,
    }
}

/// The finite script: results, pending state, provenance, idempotent
/// replay, execution errors, protocol faults and an engine failure.
fn script(attr: &Event) -> Vec<String> {
    let e = |a: &str, r: Request| encode_envelope(&envelope(a, r)).unwrap();
    let advance_all = e(
        "a4",
        Request::AdvanceSession {
            session: "s1".into(),
            max_steps: 8,
        },
    );
    let inspect = e("a1", Request::InspectTheory);
    vec![
        inspect.clone(),
        e(
            "a2",
            Request::BeginForward {
                dossier: "historicalRiggs".into(),
                theory: None,
            },
        ),
        // Pending: a step limit before the fixed point.
        e(
            "a3",
            Request::AdvanceSession {
                session: "s1".into(),
                max_steps: 2,
            },
        ),
        // Complete, with every justification of the remaining rounds.
        advance_all.clone(),
        // Idempotent replay, byte-identical answer.
        advance_all,
        // Reused identifier with other content: execution error.
        e(
            "a4",
            Request::AdvanceSession {
                session: "s1".into(),
                max_steps: 1,
            },
        ),
        e(
            "a5",
            Request::ProposeCandidate {
                text: PURPOSEFUL.into(),
            },
        ),
        e(
            "a6",
            Request::ProposeCandidate {
                text: "rule broken( = <=".into(),
            },
        ),
        e(
            "a7",
            Request::BeginCandidateEvaluation {
                candidate: "c1".into(),
                dossier: "historicalRiggs".into(),
            },
        ),
        e(
            "a8",
            Request::AdvanceSession {
                session: "s2".into(),
                max_steps: 8,
            },
        ),
        // Engine failure mapped to a status: unresolved parameters.
        e(
            "a9",
            Request::BeginResidual {
                dossier: "historicalRiggs".into(),
                theory: None,
                goal: attr.clone(),
                scope: None,
                bindings: Vec::new(),
                allowed: None,
                limits: None,
            },
        ),
        e(
            "a10",
            Request::AdvanceSession {
                session: "s3".into(),
                max_steps: 8,
            },
        ),
        // Protocol faults: transport, framing, validation.
        "garbage {".into(),
        String::new(),
        inspect.replacen("{", "{\"extra\":1,", 1),
        // Execution error: unknown session.
        e(
            "a11",
            Request::AdvanceSession {
                session: "s99".into(),
                max_steps: 1,
            },
        ),
        e(
            "a12",
            Request::MinimalSupports {
                candidate: "c1".into(),
                goal: {
                    let mut imp = attr.clone();
                    imp.verb = HEAD.into();
                    imp
                },
            },
        ),
    ]
}

#[test]
fn process_script_fixture_is_written_with_in_memory_expectation() {
    let manifest = manifest();
    let base = fixture("riggs_cases");
    let attr = experiment(&base, "historicalRiggs").goal.clone();
    let dir = root().join("target/exploration/process");
    std::fs::create_dir_all(&dir).unwrap();
    let lines = script(&attr);
    let mut requests = String::new();
    for l in &lines {
        requests.push_str(l);
        requests.push('\n');
    }
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&from_manifest(&manifest, None)).unwrap(),
    )
    .unwrap();
    std::fs::write(dir.join("requests.jsonl"), &requests).unwrap();

    // In-memory service.
    let mut served_worker = Worker::new(manifest.clone()).unwrap();
    let mut reader = Cursor::new(requests.clone().into_bytes());
    let mut out: Vec<u8> = Vec::new();
    let served = serve(
        &mut reader,
        &mut out,
        &mut served_worker,
        &TransportLimits::default(),
    );
    assert_eq!(served.lines, lines.len());
    assert_eq!(served.faults, 3);
    assert_eq!(served.responses, lines.len() - 3);
    std::fs::write(dir.join("expected.jsonl"), &out).unwrap();

    // Direct worker, same envelopes, line by line.
    let mut direct = Worker::new(manifest).unwrap();
    let outputs: Vec<&str> = std::str::from_utf8(&out).unwrap().lines().collect();
    assert_eq!(outputs.len(), lines.len());
    let mut statuses = Vec::new();
    for (input, output) in lines.iter().zip(&outputs) {
        let line = decode_line(output.as_bytes(), 32 * 1024 * 1024).unwrap();
        match decode_envelope(input.as_bytes(), &WireLimits::default()) {
            Ok(env) => {
                let Line::Response(w) = line else {
                    panic!("{line:?}")
                };
                let response = Response::try_from(w).unwrap();
                assert_eq!(response, direct.handle(env));
                statuses.push(format!("{:?}", response.status));
            }
            Err(_) => {
                assert!(matches!(line, Line::Fault(_)), "{line:?}");
                statuses.push("fault".into());
            }
        }
    }
    assert_eq!(
        statuses,
        vec![
            "Complete",
            "Complete",
            "StepLimit",
            "Complete",
            "Complete",
            "Error",
            "Complete",
            "StructuralReject",
            "Complete",
            "Complete",
            "Complete",
            "UnresolvedParameters",
            "fault",
            "fault",
            "fault",
            "Error",
            "Complete",
        ]
    );
    assert_eq!(
        outputs[3], outputs[4],
        "idempotent replay is byte-identical"
    );
    std::fs::write(dir.join("statuses.txt"), statuses.join("\n") + "\n").unwrap();
}
