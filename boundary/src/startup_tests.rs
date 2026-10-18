//! Start-up logic of the worker binary over injected streams: manifest DTO
//! validation and round trip, exit codes, diagnostics kept out of stdout,
//! and equivalence with the direct worker. Run individually through
//! `scripts/test_safe.py --test-target boundary`.
use residuum::experiments::exploration::*;
use residuum::experiments::worker::*;
use residuum::typed::*;
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::PathBuf;

use crate::manifest::*;
use crate::startup::*;
use crate::transport::{Line, TransportLimits, decode_line};
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
        episode_id: "riggs-startup-001".into(),
        manifest_hash: "h-startup".into(),
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
        episode_id: "riggs-startup-001".into(),
        action_id: action.into(),
        manifest_hash: "h-startup".into(),
        request,
    }
}

fn write_manifest(name: &str, wire: &WireManifest) -> PathBuf {
    let dir = root().join("target/exploration/startup");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, serde_json::to_string_pretty(wire).unwrap()).unwrap();
    path
}

fn args(path: &PathBuf) -> Vec<String> {
    vec!["--manifest".into(), path.to_string_lossy().into_owned()]
}

fn script() -> Vec<Envelope> {
    vec![
        envelope("a1", Request::InspectTheory),
        envelope(
            "a2",
            Request::BeginForward {
                dossier: "historicalRiggs".into(),
                theory: None,
            },
        ),
        envelope(
            "a3",
            Request::AdvanceSession {
                session: "s1".into(),
                max_steps: 8,
            },
        ),
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
    ]
}

fn stdin_of(lines: &[Envelope]) -> Cursor<Vec<u8>> {
    let mut bytes = Vec::new();
    for e in lines {
        bytes.extend_from_slice(encode_envelope(e).unwrap().as_bytes());
        bytes.push(b'\n');
    }
    Cursor::new(bytes)
}

fn stderr_lines(err: &[u8]) -> Vec<String> {
    std::str::from_utf8(err)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn startup_serves_injected_streams_matching_the_direct_worker() {
    let manifest = manifest("startup");
    let path = write_manifest("manifest.json", &from_manifest(&manifest, None));
    let mut stdin = stdin_of(&script());
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let exit = run(&args(&path), &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(exit, Exit::Eof);
    assert_eq!(exit.code(), 0);

    let out = std::str::from_utf8(&stdout).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), script().len(), "one protocol line per request");
    assert!(!out.contains("worker:"), "no diagnostics on stdout");
    let mut direct = Worker::new(manifest).unwrap();
    for (line, env) in lines.iter().zip(script()) {
        let decoded = decode_line(line.as_bytes(), 32 * 1024 * 1024).unwrap();
        let Line::Response(w) = decoded else {
            panic!("{decoded:?}");
        };
        assert_eq!(Response::try_from(w).unwrap(), direct.handle(env));
    }
    let err = stderr_lines(&stderr);
    assert!(err.iter().all(|l| l.starts_with("worker: ")), "{err:?}");
    assert!(err[0].contains("episode `riggs-startup-001` manifest `h-startup` loaded; 6 dossiers"));
    assert_eq!(
        err.last().unwrap(),
        "worker: stopped: eof; lines 6 responses 6 faults 0; exit 0"
    );
}

struct FailingWriter;
impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("stdout closed"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn startup_exit_codes_and_diagnostics_are_documented_and_distinct() {
    let manifest = manifest("startup-exit");
    let good = write_manifest("good.json", &from_manifest(&manifest, None));
    let empty = Cursor::new(Vec::new());

    // Usage.
    for bad in [
        vec![],
        vec!["--manifest".to_string()],
        vec!["--other".into(), "x".into()],
    ] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(&bad, &mut empty.clone(), &mut out, &mut err),
            Exit::Usage
        );
        assert!(out.is_empty());
        assert!(stderr_lines(&err)[0].contains("usage"));
    }
    // Manifest: missing, malformed, oversized, unknown field, wrong version,
    // invalid decision, no dossiers.
    let missing = root().join("target/exploration/startup/absent.json");
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&missing), &mut empty.clone(), &mut out, &mut err),
        Exit::Manifest
    );
    assert!(stderr_lines(&err)[0].contains("cannot read manifest"));
    let malformed = root().join("target/exploration/startup/malformed.json");
    std::fs::write(&malformed, b"{\"wire_version\": ").unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&malformed), &mut empty.clone(), &mut out, &mut err),
        Exit::Manifest
    );
    assert!(stderr_lines(&err)[0].contains("manifest rejected: transport"));
    let text = std::fs::read_to_string(&good).unwrap();
    let unknown = root().join("target/exploration/startup/unknown.json");
    std::fs::write(&unknown, text.replacen("{", "{\"extra\": 1,", 1)).unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&unknown), &mut empty.clone(), &mut out, &mut err),
        Exit::Manifest
    );
    assert!(stderr_lines(&err)[0].contains("manifest rejected: validation"));
    let version = root().join("target/exploration/startup/version.json");
    std::fs::write(&version, text.replace("residuum-wire/1", "residuum-wire/9")).unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&version), &mut empty.clone(), &mut out, &mut err),
        Exit::Manifest
    );
    let mut bad_decision = from_manifest(&manifest, None);
    bad_decision.decision.criterion = WireCriterion::Component(7);
    let path = write_manifest("decision.json", &bad_decision);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&path), &mut empty.clone(), &mut out, &mut err),
        Exit::Manifest
    );
    assert!(stderr_lines(&err)[0].contains("component 7"));
    let mut no_dossiers = from_manifest(&manifest, None);
    no_dossiers.dossiers.clear();
    let path = write_manifest("dossiers.json", &no_dossiers);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&path), &mut empty.clone(), &mut out, &mut err),
        Exit::Manifest
    );
    // Worker: an import that does not exist is a start-up failure, not a
    // manifest syntax error.
    let mut bad_import = from_manifest(&manifest, None);
    bad_import.imports.push("nowhere/missing.res".into());
    let path = write_manifest("import.json", &bad_import);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&path), &mut empty.clone(), &mut out, &mut err),
        Exit::Worker
    );
    assert!(
        stderr_lines(&err)
            .iter()
            .any(|l| l.contains("worker cannot start"))
    );
    assert!(out.is_empty());
    // Incomplete line: one fault on stdout, exit 5.
    let partial = encode_envelope(&envelope("a1", Request::InspectTheory)).unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let exit = run(
        &args(&good),
        &mut Cursor::new(partial.into_bytes()),
        &mut out,
        &mut err,
    );
    assert_eq!(exit, Exit::IncompleteLine);
    assert_eq!(exit.code(), 5);
    let fault = decode_line(
        std::str::from_utf8(&out)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .as_bytes(),
        1024 * 1024,
    )
    .unwrap();
    assert!(matches!(fault, Line::Fault(_)));
    assert!(
        stderr_lines(&err)
            .last()
            .unwrap()
            .contains("stopped: incomplete_line")
    );
    // Stream: stdout refuses writes, exit 6, the request still executed.
    let mut err = Vec::new();
    let exit = run(
        &args(&good),
        &mut stdin_of(&script()[..1]),
        &mut FailingWriter,
        &mut err,
    );
    assert_eq!(exit, Exit::Stream);
    assert_eq!(exit.code(), 6);
    assert!(stderr_lines(&err).last().unwrap().contains("stream_error"));
    // Empty input is a clean EOF.
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(&args(&good), &mut empty.clone(), &mut out, &mut err),
        Exit::Eof
    );
    assert!(out.is_empty());
    assert!(
        stderr_lines(&err)
            .last()
            .unwrap()
            .ends_with("lines 0 responses 0 faults 0; exit 0")
    );
}

#[test]
fn manifest_dto_round_trips_resolves_paths_and_is_immutable_to_requests() {
    let manifest = manifest("startup-manifest");
    let limits = TransportLimits {
        max_line_bytes: 2048,
        max_output_bytes: 4096,
        wire: WireLimits {
            max_bytes: 2048,
            max_depth: 16,
        },
    };
    let wire = from_manifest(&manifest, Some(&limits));
    let text = serde_json::to_string(&wire).unwrap();
    let parsed = parse_manifest(text.as_bytes(), MAX_MANIFEST_BYTES).unwrap();
    assert_eq!(parsed, wire);
    let (back, back_limits) = to_manifest(&parsed, &root()).unwrap();
    assert_eq!(back_limits, limits);
    assert_eq!(back.episode_id, manifest.episode_id);
    assert_eq!(back.manifest_hash, manifest.manifest_hash);
    assert_eq!(back.imports, manifest.imports);
    assert_eq!(back.workdir, manifest.workdir);
    assert_eq!(back.profile, manifest.profile);
    assert_eq!(back.decision, manifest.decision);
    assert_eq!(back.dossiers, manifest.dossiers);
    assert_eq!(back.budgets, manifest.budgets);
    // Relative paths resolve against the manifest's directory.
    let mut relative = wire.clone();
    relative.imports =
        vec!["examples/experiments/legal_geometry/riggs_problem_manifest.res".into()];
    relative.workdir = "target/exploration/relative".into();
    let (resolved, _) = to_manifest(&relative, &root()).unwrap();
    assert_eq!(
        resolved.imports[0],
        root().join("examples/experiments/legal_geometry/riggs_problem_manifest.res")
    );
    assert_eq!(resolved.workdir, root().join("target/exploration/relative"));
    // Oversized manifest and zero budgets are rejected before any work.
    assert!(matches!(
        parse_manifest(text.as_bytes(), 16),
        Err(WireError::Transport(_))
    ));
    let mut zero = wire.clone();
    zero.budgets.max_actions = 0;
    assert!(matches!(
        to_manifest(&zero, &root()),
        Err(WireError::Validation(_))
    ));
    // No request can carry a manifest: an invented request kind is rejected
    // at the wire, and the worker exposes no method that replaces it.
    let good = encode_envelope(&envelope("a1", Request::InspectTheory)).unwrap();
    let forged = good.replace("\"inspect_theory\"", "\"set_manifest\":{\"budgets\":{}}");
    assert!(matches!(
        decode_envelope(forged.as_bytes(), &WireLimits::default()),
        Err(WireError::Validation(_))
    ));
}
