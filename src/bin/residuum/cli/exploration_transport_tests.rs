//! Line transport: the same DTOs and results reach the caller intact through
//! framing, in order, with faults separated from results. In-memory streams
//! only; no process or socket. Run individually through
//! `scripts/test_safe.py --test-target cli`.
use residuum::experiments::exploration::*;
use residuum::experiments::transport::*;
use residuum::experiments::wire::*;
use residuum::experiments::worker::*;
use residuum::typed::*;
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::PathBuf;

const HEAD: &str = "hypotheticalSuccessionImpediment";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn manifest(workdir: &str) -> Manifest {
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
        episode_id: "riggs-transport-001".into(),
        manifest_hash: "h-transport".into(),
        imports: vec![
            root().join("examples/experiments/legal_geometry/riggs_problem_manifest.res"),
            root().join("examples/experiments/legal_geometry/riggs_cases.res"),
        ],
        workdir: root().join(format!("target/exploration/{workdir}")),
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
        episode_id: "riggs-transport-001".into(),
        action_id: action.into(),
        manifest_hash: "h-transport".into(),
        request,
    }
}

fn input(lines: &[String]) -> Cursor<Vec<u8>> {
    let mut bytes = Vec::new();
    for l in lines {
        bytes.extend_from_slice(l.as_bytes());
        bytes.push(b'\n');
    }
    Cursor::new(bytes)
}

fn output_lines(out: &[u8]) -> Vec<Line> {
    let text = std::str::from_utf8(out).unwrap();
    assert!(text.ends_with('\n') || text.is_empty());
    text.lines()
        .map(|l| decode_line(l.as_bytes(), 32 * 1024 * 1024).unwrap())
        .collect()
}

fn response(line: &Line) -> Response {
    match line {
        Line::Response(w) => Response::try_from(w.clone()).unwrap(),
        other => panic!("expected a response, got {other:?}"),
    }
}

fn fault(line: &Line) -> &Fault {
    match line {
        Line::Fault(f) => f,
        other => panic!("expected a fault, got {other:?}"),
    }
}

#[test]
fn transport_serves_scripted_lines_in_order_matching_direct_worker() {
    // Both workers share the work directory so that the private module paths
    // in their reports coincide; the files they write are identical.
    let mut served_worker = Worker::new(manifest("transport")).unwrap();
    let mut direct_worker = Worker::new(manifest("transport")).unwrap();
    let advance = envelope(
        "a3",
        Request::AdvanceSession {
            session: "s1".into(),
            max_steps: 8,
        },
    );
    let script = vec![
        envelope("a1", Request::InspectTheory),
        envelope(
            "a2",
            Request::BeginForward {
                dossier: "historicalRiggs".into(),
                theory: None,
            },
        ),
        advance.clone(),
        envelope(
            "a4",
            Request::ProposeCandidate {
                text: PURPOSEFUL.into(),
            },
        ),
        envelope(
            "a5",
            Request::BeginCandidateEvaluation {
                candidate: "c1".into(),
                dossier: "historicalRiggs".into(),
            },
        ),
        envelope(
            "a6",
            Request::AdvanceSession {
                session: "s2".into(),
                max_steps: 8,
            },
        ),
        // Idempotent replay and a reused identifier with other content.
        advance.clone(),
        envelope(
            "a3",
            Request::AdvanceSession {
                session: "s1".into(),
                max_steps: 1,
            },
        ),
        envelope(
            "a7",
            Request::ProposeCandidate {
                text: "rule broken( = <=".into(),
            },
        ),
        envelope(
            "a8",
            Request::AdvanceSession {
                session: "s99".into(),
                max_steps: 1,
            },
        ),
    ];
    let lines: Vec<String> = script.iter().map(|e| encode_envelope(e).unwrap()).collect();
    let mut reader = input(&lines);
    let mut out: Vec<u8> = Vec::new();
    let served = serve(
        &mut reader,
        &mut out,
        &mut served_worker,
        &TransportLimits::default(),
    );
    assert_eq!(
        served,
        Served {
            lines: script.len(),
            responses: script.len(),
            faults: 0,
            stop: StopReason::Eof,
        }
    );
    let outputs = output_lines(&out);
    assert_eq!(
        outputs.len(),
        script.len(),
        "one output line per input line"
    );
    for (i, (line, env)) in outputs.iter().zip(&script).enumerate() {
        let via_transport = response(line);
        let direct = direct_worker.handle(env.clone());
        assert_eq!(
            via_transport, direct,
            "line {i} differs from the direct worker"
        );
        assert_eq!(via_transport.action_id, env.action_id);
    }
    // The replayed line is byte-identical to the original answer; the reused
    // identifier is an execution error inside a response, not a fault.
    let texts: Vec<&str> = std::str::from_utf8(&out).unwrap().lines().collect();
    assert_eq!(texts[2], texts[6]);
    assert_eq!(response(&outputs[7]).status, Status::Error);
    assert_eq!(response(&outputs[8]).status, Status::StructuralReject);
    assert_eq!(response(&outputs[9]).status, Status::Error);
    // Provenance survived framing: the forward response carries every
    // justification of the full closure.
    match response(&outputs[2]).payload {
        Payload::Forward {
            rounds, complete, ..
        } => {
            assert!(complete);
            let full = residuum::close_program(
                experiment(&fixture("riggs_cases"), "historicalRiggs")
                    .input
                    .clone(),
                &fixture("riggs_cases"),
                &mut CountingClock::default(),
            )
            .unwrap();
            assert_eq!(
                rounds.iter().map(|r| r.derivations.len()).sum::<usize>(),
                full.derivations.len()
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn transport_faults_are_framed_and_do_not_execute() {
    let mut worker = Worker::new(manifest("transport-faults")).unwrap();
    let limits = TransportLimits {
        max_line_bytes: 4096,
        max_output_bytes: 16 * 1024 * 1024,
        wire: WireLimits {
            max_bytes: 4096,
            max_depth: 32,
        },
    };
    let good = encode_envelope(&envelope("a1", Request::InspectTheory)).unwrap();
    let unknown_field = good.replacen("{", "{\"extra\":1,", 1);
    let lines = vec![
        String::new(),
        "   ".into(),
        "garbage {".into(),
        unknown_field,
        "x".repeat(5000),
        format!("{}1{}", "[".repeat(40), "]".repeat(40)),
        good.clone(),
    ];
    let mut reader = input(&lines);
    let mut out: Vec<u8> = Vec::new();
    let served = serve(&mut reader, &mut out, &mut worker, &limits);
    assert_eq!(
        served,
        Served {
            lines: 7,
            responses: 1,
            faults: 6,
            stop: StopReason::Eof,
        }
    );
    let outputs = output_lines(&out);
    assert_eq!(outputs.len(), 7);
    assert_eq!(fault(&outputs[0]).class, FaultClass::Framing);
    assert_eq!(fault(&outputs[1]).class, FaultClass::Framing);
    assert_eq!(fault(&outputs[2]).class, FaultClass::Transport);
    assert_eq!(fault(&outputs[3]).class, FaultClass::Validation);
    let too_long = fault(&outputs[4]);
    assert_eq!(too_long.class, FaultClass::Framing);
    assert!(
        too_long.message.contains("exceeds 4096 bytes"),
        "{}",
        too_long.message
    );
    assert!(
        too_long.message.contains("904 discarded"),
        "{}",
        too_long.message
    );
    assert_eq!(fault(&outputs[5]).class, FaultClass::Validation);
    assert!(fault(&outputs[5]).message.contains("depth"));
    assert_eq!(response(&outputs[6]).status, Status::Complete);
    assert!(outputs[..6].iter().all(|l| fault(l).action_id.is_none()));
    // Only the valid line was an action.
    assert_eq!(worker.budget().actions_remaining, 39);

    // An unterminated final line is not executed and stops the service.
    let mut reader = Cursor::new(good.as_bytes().to_vec());
    let mut out: Vec<u8> = Vec::new();
    let served = serve(&mut reader, &mut out, &mut worker, &limits);
    assert_eq!(served.stop, StopReason::IncompleteLine);
    assert_eq!((served.lines, served.responses, served.faults), (1, 0, 1));
    let outputs = output_lines(&out);
    assert_eq!(fault(&outputs[0]).class, FaultClass::Framing);
    assert!(fault(&outputs[0]).message.contains("incomplete"));
    assert_eq!(worker.budget().actions_remaining, 39);

    // An over-long line without a newline at the end is also incomplete and
    // never accumulated beyond the limit.
    let mut reader = Cursor::new("y".repeat(20_000).into_bytes());
    let mut out: Vec<u8> = Vec::new();
    let served = serve(&mut reader, &mut out, &mut worker, &limits);
    assert_eq!(served.stop, StopReason::IncompleteLine);
    assert!(
        fault(&output_lines(&out)[0])
            .message
            .contains("20000 bytes")
    );
}

#[test]
fn transport_output_limit_reports_instead_of_truncating() {
    let mut worker = Worker::new(manifest("transport-output")).unwrap();
    let limits = TransportLimits {
        max_line_bytes: 1024 * 1024,
        max_output_bytes: 3000,
        wire: WireLimits::default(),
    };
    let advance = envelope(
        "b2",
        Request::AdvanceSession {
            session: "s1".into(),
            max_steps: 8,
        },
    );
    let lines = vec![
        encode_envelope(&envelope(
            "b1",
            Request::BeginForward {
                dossier: "historicalRiggs".into(),
                theory: None,
            },
        ))
        .unwrap(),
        encode_envelope(&advance).unwrap(),
        encode_envelope(&advance).unwrap(),
    ];
    let mut reader = input(&lines);
    let mut out: Vec<u8> = Vec::new();
    let served = serve(&mut reader, &mut out, &mut worker, &limits);
    assert_eq!((served.lines, served.responses, served.faults), (3, 1, 2));
    let outputs = output_lines(&out);
    assert_eq!(response(&outputs[0]).status, Status::Complete);
    let big = fault(&outputs[1]);
    assert_eq!(big.class, FaultClass::OutputLimit);
    assert_eq!(big.action_id.as_deref(), Some("b2"));
    assert!(big.message.contains("exceeds the output limit of 3000"));
    assert!(big.message.contains("nothing was truncated"));
    assert_eq!(fault(&outputs[2]), big, "the replay reports the same fault");
    for l in std::str::from_utf8(&out).unwrap().lines() {
        assert!(l.len() <= 3000);
    }
    // The action executed: the session is complete and a fresh advance
    // consumes nothing. With a wide enough output limit the full response,
    // untruncated, comes through.
    let again = envelope(
        "b3",
        Request::AdvanceSession {
            session: "s1".into(),
            max_steps: 1,
        },
    );
    let mut reader = input(&[encode_envelope(&again).unwrap()]);
    let mut out: Vec<u8> = Vec::new();
    serve(
        &mut reader,
        &mut out,
        &mut worker,
        &TransportLimits::default(),
    );
    let r = response(&output_lines(&out)[0]);
    assert_eq!(r.status, Status::Complete);
    assert_eq!(r.steps_consumed, 0);
    let mut reader = input(&[encode_envelope(&advance).unwrap()]);
    let mut out: Vec<u8> = Vec::new();
    serve(
        &mut reader,
        &mut out,
        &mut worker,
        &TransportLimits::default(),
    );
    let replay = response(&output_lines(&out)[0]);
    assert!(replay.steps_consumed > 0);
    assert!(out.len() > 3000);
}

struct FailingWriter {
    budget: usize,
    written: Vec<u8>,
}

impl Write for FailingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.written.len() + buf.len() > self.budget {
            return Err(std::io::Error::other("pipe closed"));
        }
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn transport_write_errors_stop_the_service_after_execution() {
    let mut worker = Worker::new(manifest("transport-write")).unwrap();
    let lines = vec![
        encode_envelope(&envelope("w1", Request::InspectTheory)).unwrap(),
        encode_envelope(&envelope(
            "w2",
            Request::BeginForward {
                dossier: "historicalRiggs".into(),
                theory: None,
            },
        ))
        .unwrap(),
    ];
    let mut reader = input(&lines);
    let mut writer = FailingWriter {
        budget: 10,
        written: Vec::new(),
    };
    let served = serve(
        &mut reader,
        &mut writer,
        &mut worker,
        &TransportLimits::default(),
    );
    assert!(matches!(served.stop, StopReason::WriteError(ref m) if m.contains("pipe closed")));
    assert_eq!((served.lines, served.responses, served.faults), (1, 1, 0));
    assert!(writer.written.is_empty());
    // The first request executed before the write failed; the second was
    // never read. Idempotent replay recovers the lost answer.
    assert_eq!(worker.budget().actions_remaining, 39);
    let replay = worker.handle(envelope("w1", Request::InspectTheory));
    assert_eq!(replay.status, Status::Complete);
    assert_eq!(worker.budget().actions_remaining, 39);
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
