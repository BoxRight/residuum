//! Wire DTOs: structural round trips of requests and of real worker
//! responses, full provenance preservation, and rejections. Lives in the CLI
//! test binary because the library test target has no memory margin left
//! for it under the runner (see docs/legal-geometry/exploration/verification.md).
//! Run individually through `scripts/test_safe.py --test-target cli`.
use residuum::experiments::exploration::*;
use residuum::experiments::proposal::ProposalStatus;
use residuum::experiments::wire::*;
use residuum::experiments::worker::*;
use residuum::typed::*;
use residuum::{Abducible, HornQueryScope, QueryBinding, QueryParameterScope, SearchLimits};
use std::collections::BTreeMap;
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

fn ev(verb: &str, args: Vec<Term>, time: TimeExpr) -> Event {
    Event {
        polarity: Polarity::Positive,
        verb: verb.into(),
        args,
        proposition: PropositionType {
            kind: PropositionKind::Derived,
            time,
        },
        transition: None,
    }
}

fn envelope(action: &str, request: Request) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.into(),
        episode_id: "riggs-wire-001".into(),
        action_id: action.into(),
        manifest_hash: "h-wire".into(),
        request,
    }
}

fn round_trip_envelope(e: &Envelope) -> (String, Envelope) {
    let text = encode_envelope(e).unwrap();
    let back = decode_envelope(text.as_bytes(), &WireLimits::default()).unwrap();
    (text, back)
}

#[test]
fn wire_round_trips_every_request_variant_structurally() {
    // Var versus Const, nested constructors, None, a record literal, both
    // substitution values, both scopes, both parameter scopes, a negative
    // abducible and explicit limits.
    let goal = ev(
        "compliance",
        vec![
            Term::Var("norm".into()),
            Term::Const("gAct".into()),
            Term::Constructor {
                constructor: Constructor::After,
                argument: Box::new(Term::Constructor {
                    constructor: Constructor::After,
                    argument: Box::new(Term::Var("t".into())),
                }),
            },
            Term::None,
            Term::RecordLiteral {
                entity: "Holding".into(),
                fields: vec![RecordFieldValue {
                    field: "owner".into(),
                    value: Term::Const("gSubject".into()),
                }],
            },
        ],
        TimeExpr::After(Box::new(TimeExpr::Var("t".into()))),
    );
    let mut negative = goal.clone();
    negative.polarity = Polarity::EvidentialNot;
    negative.verb = "Act".into();
    negative.proposition.kind = PropositionKind::Seeded;
    negative.transition = Some(StateTransitionType {
        input: StateTime::Unresolved,
        output: StateTime::At(TimeExpr::At("gDate".into())),
    });
    let bindings = vec![
        QueryBinding {
            scope: QueryParameterScope::Rule(RuleRef { name: "r1".into() }),
            parameter: "death".into(),
            ty: Type::Entity("Fact".into()),
            value: SubstitutionValue::Term(Term::Const("palmerDeath".into())),
        },
        QueryBinding {
            scope: QueryParameterScope::Program(ProgramRef { name: "p1".into() }),
            parameter: "when".into(),
            ty: Type::Optional(Box::new(Type::Set(Box::new(Type::PropositionTime)))),
            value: SubstitutionValue::Time(TimeExpr::At("death1882".into())),
        },
    ];
    let requests = vec![
        Request::InspectTheory,
        Request::BeginForward {
            dossier: "historicalRiggs".into(),
            theory: Some("c1".into()),
        },
        Request::BeginResidual {
            dossier: "d".into(),
            theory: None,
            goal: goal.clone(),
            scope: Some(HornQueryScope::Rules(vec![
                RuleRef { name: "r1".into() },
                RuleRef { name: "r2".into() },
            ])),
            bindings: bindings.clone(),
            allowed: Some(vec![Abducible {
                verb: "Act".into(),
                polarity: Polarity::EvidentialNot,
            }]),
            limits: Some(SearchLimits {
                max_depth: 3,
                max_nodes: 10,
                max_branches: 11,
            }),
        },
        Request::BeginActiveResidual {
            dossier: "d".into(),
            theory: Some("c2".into()),
            goal: negative.clone(),
            scope: Some(HornQueryScope::Program(ProgramRef { name: "p1".into() })),
            bindings: Vec::new(),
            allowed: None,
            limits: None,
        },
        Request::AdvanceSession {
            session: "s1".into(),
            max_steps: 7,
        },
        Request::ProposeCandidate {
            text: "rule x(a) = \n  y(a) @ t\n  <= z(a) @ t  -- with \"quotes\" and ñ".into(),
        },
        Request::BeginCandidateEvaluation {
            candidate: "c1".into(),
            dossier: "d".into(),
        },
        Request::MinimalSupports {
            candidate: "c1".into(),
            goal: goal.clone(),
        },
        Request::ComparePartition {
            candidates: vec!["c1".into(), "c2".into()],
            goal: goal.clone(),
        },
    ];
    for (i, request) in requests.into_iter().enumerate() {
        let original = envelope(&format!("a{i}"), request);
        let (text, back) = round_trip_envelope(&original);
        assert_eq!(back, original, "{text}");
        assert!(text.contains("\"wire_version\":\"residuum-wire/1\""));
    }
    // The representation distinguishes variables from constants and keeps
    // the transition metadata that Event equality ignores.
    let (text, back) = round_trip_envelope(&envelope(
        "neg",
        Request::MinimalSupports {
            candidate: "c1".into(),
            goal: negative.clone(),
        },
    ));
    assert!(text.contains("{\"var\":\"norm\"}"));
    assert!(text.contains("{\"const\":\"gAct\"}"));
    assert!(text.contains("\"polarity\":\"evidential_not\""));
    match back.request {
        Request::MinimalSupports { goal, .. } => {
            assert_eq!(goal.args, negative.args);
            assert_eq!(goal.transition, negative.transition);
            assert_eq!(goal.polarity, Polarity::EvidentialNot);
            assert_eq!(goal.proposition.kind, PropositionKind::Seeded);
        }
        other => panic!("{other:?}"),
    }
}

fn manifest() -> (Manifest, Module) {
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
    let manifest = Manifest {
        episode_id: "riggs-wire-001".into(),
        manifest_hash: "h-wire".into(),
        imports: vec![
            root().join("examples/experiments/legal_geometry/riggs_problem_manifest.res"),
            root().join("examples/experiments/legal_geometry/riggs_cases.res"),
        ],
        workdir: root().join("target/exploration/wire"),
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
            search_limits: SearchLimits::default(),
        },
    };
    (manifest, base)
}

const PURPOSEFUL: &str = "rule purposefulImpediment(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
";

const NOT_KILLING: &str = "rule innocentImpediment(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
";

fn session_of(r: &Response) -> String {
    match &r.payload {
        Payload::SessionOpened { session, .. } => session.clone(),
        other => panic!("{other:?}"),
    }
}

fn candidate_of(r: &Response) -> String {
    match &r.payload {
        Payload::Candidate {
            candidate: Some(c), ..
        } => c.clone(),
        other => panic!("{other:?}"),
    }
}

fn check_round_trip(r: &Response) -> Response {
    let text = encode_response(r).unwrap();
    let back = decode_response(text.as_bytes(), &WireLimits::default()).unwrap();
    assert_eq!(&back, r);
    // Event equality ignores transition metadata; compare it explicitly.
    assert_eq!(
        format!("{:?}", transitions(&back.payload)),
        format!("{:?}", transitions(&r.payload))
    );
    back
}

fn transitions(p: &Payload) -> Vec<Option<StateTransitionType>> {
    let mut out = Vec::new();
    let mut push = |events: &[Event]| out.extend(events.iter().map(|e| e.transition.clone()));
    match p {
        Payload::Forward { known, rounds, .. } => {
            push(known);
            for r in rounds {
                push(&r.added);
            }
        }
        Payload::Evaluation { closure, .. } => {
            if let Some(c) = closure {
                push(&c.known);
            }
        }
        _ => {}
    }
    out
}

#[test]
fn wire_round_trips_worker_responses_with_full_provenance() {
    let (manifest, base) = manifest();
    let historical = experiment(&base, "historicalRiggs");
    let attr = historical.goal.clone();
    let imp = manifest.decision.components[1].clone();
    let mut w = Worker::new(manifest).unwrap();
    let mut n = 0;
    let mut send = |w: &mut Worker, request: Request| {
        n += 1;
        let env = envelope(&format!("a{n}"), request);
        // Requests travel over the wire too.
        let (_, decoded) = round_trip_envelope(&env);
        assert_eq!(decoded, env);
        let r = w.handle(decoded);
        check_round_trip(&r)
    };

    send(&mut w, Request::InspectTheory);
    let s1 = session_of(&send(
        &mut w,
        Request::BeginForward {
            dossier: "historicalRiggs".into(),
            theory: None,
        },
    ));
    let forward = send(
        &mut w,
        Request::AdvanceSession {
            session: s1,
            max_steps: 8,
        },
    );
    assert_eq!(forward.status, Status::Complete);
    // Provenance survives the wire: records with ordered antecedents,
    // substitutions, derivedAt and multiplicity.
    let full = residuum::close_program(
        historical.input.clone(),
        &fixture("riggs_cases"),
        &mut CountingClock::default(),
    )
    .unwrap();
    match &forward.payload {
        Payload::Forward { rounds, .. } => {
            let wired: Vec<&DerivedEvent> =
                rounds.iter().flat_map(|r| r.derivations.iter()).collect();
            assert_eq!(wired.len(), full.derivations.len());
            let mut expected: Vec<String> = full
                .derivations
                .iter()
                .map(|d| format!("{:?}|{:?}", d.event, d.record))
                .collect();
            let mut got: Vec<String> = wired
                .iter()
                .map(|d| format!("{:?}|{:?}", d.event, d.record))
                .collect();
            expected.sort();
            got.sort();
            assert_eq!(got, expected);
            assert!(wired.iter().all(|d| d.derived_at.name.starts_with('d')));
            assert!(wired.iter().any(|d| d.record.antecedents.len() == 3));
        }
        other => panic!("{other:?}"),
    }

    let c1 = candidate_of(&send(
        &mut w,
        Request::ProposeCandidate {
            text: PURPOSEFUL.into(),
        },
    ));
    let c2 = candidate_of(&send(
        &mut w,
        Request::ProposeCandidate {
            text: NOT_KILLING.into(),
        },
    ));
    let rejected = send(
        &mut w,
        Request::ProposeCandidate {
            text: "rule broken( = <=".into(),
        },
    );
    match &rejected.payload {
        Payload::Candidate { report, .. } => {
            assert!(matches!(report.status, ProposalStatus::Syntax(_)))
        }
        other => panic!("{other:?}"),
    }
    let s2 = session_of(&send(
        &mut w,
        Request::BeginCandidateEvaluation {
            candidate: c1.clone(),
            dossier: "historicalRiggs".into(),
        },
    ));
    let evaluated = send(
        &mut w,
        Request::AdvanceSession {
            session: s2,
            max_steps: 8,
        },
    );
    match &evaluated.payload {
        Payload::Evaluation {
            closure: Some(closure),
            report: Some(report),
            ..
        } => {
            assert_eq!(report_label(report), "evaluated_compatible");
            assert!(
                closure
                    .derivations
                    .iter()
                    .any(|d| d.record.rule == "purposefulImpediment")
            );
            assert!(!report.rules.is_empty());
        }
        other => panic!("{other:?}"),
    }

    let literal = RuleRef {
        name: "literalTestamentaryEntitlement".into(),
    };
    let binding = |parameter: &str, ty: Type, value: SubstitutionValue| QueryBinding {
        scope: QueryParameterScope::Rule(literal.clone()),
        parameter: parameter.into(),
        ty,
        value,
    };
    let bindings = vec![
        binding(
            "death",
            Type::Entity("Fact".into()),
            SubstitutionValue::Term(Term::Const("palmerDeath".into())),
        ),
        binding(
            "deathAuthority",
            Type::Entity("Authority".into()),
            SubstitutionValue::Term(Term::Const("judicial".into())),
        ),
        binding(
            "deathEffective",
            Type::PropositionTime,
            SubstitutionValue::Time(TimeExpr::At("death1882".into())),
        ),
    ];
    let s3 = session_of(&send(
        &mut w,
        Request::BeginResidual {
            dossier: "historicalRiggs".into(),
            theory: None,
            goal: attr.clone(),
            scope: Some(HornQueryScope::Rules(vec![literal.clone()])),
            bindings: bindings.clone(),
            allowed: None,
            limits: None,
        },
    ));
    let horn = send(
        &mut w,
        Request::AdvanceSession {
            session: s3,
            max_steps: 8,
        },
    );
    assert!(matches!(
        horn.payload,
        Payload::Residual {
            result: Some(ResidualOutcome::Horn(_)),
            ..
        }
    ));
    let s4 = session_of(&send(
        &mut w,
        Request::BeginActiveResidual {
            dossier: "survivalUnestablished".into(),
            theory: None,
            goal: attr.clone(),
            scope: Some(HornQueryScope::Rules(vec![literal.clone()])),
            bindings,
            allowed: None,
            limits: None,
        },
    ));
    let active = send(
        &mut w,
        Request::AdvanceSession {
            session: s4,
            max_steps: 8,
        },
    );
    match &active.payload {
        Payload::Residual {
            result: Some(ResidualOutcome::Active(q)),
            rounds,
            ..
        } => {
            assert!(!q.explanations.is_empty());
            assert!(!q.explanations[0].hypothetical.derivations.is_empty());
            assert!(rounds.iter().any(|r| !r.branches.is_empty()));
        }
        other => panic!("{other:?}"),
    }
    // Unresolved parameters: error text travels with the rounds so far.
    let s5 = session_of(&send(
        &mut w,
        Request::BeginResidual {
            dossier: "historicalRiggs".into(),
            theory: None,
            goal: attr.clone(),
            scope: None,
            bindings: Vec::new(),
            allowed: None,
            limits: None,
        },
    ));
    let failed = send(
        &mut w,
        Request::AdvanceSession {
            session: s5,
            max_steps: 8,
        },
    );
    assert_eq!(failed.status, Status::UnresolvedParameters);
    send(
        &mut w,
        Request::MinimalSupports {
            candidate: c1.clone(),
            goal: imp.clone(),
        },
    );
    let partition = send(
        &mut w,
        Request::ComparePartition {
            candidates: vec![c1, c2],
            goal: imp,
        },
    );
    assert!(matches!(partition.payload, Payload::Partition { .. }));
    let error = send(
        &mut w,
        Request::AdvanceSession {
            session: "s99".into(),
            max_steps: 1,
        },
    );
    assert_eq!(error.status, Status::Error);
}

#[test]
fn wire_rejects_malformed_oversized_deep_and_unknown_inputs() {
    let limits = WireLimits {
        max_bytes: 4096,
        max_depth: 8,
    };
    let good = encode_envelope(&envelope("a1", Request::InspectTheory)).unwrap();
    assert!(decode_envelope(good.as_bytes(), &limits).is_ok());

    // Transport: oversized and malformed.
    let big = format!("{{\"pad\":\"{}\"}}", "x".repeat(5000));
    assert!(matches!(
        decode_envelope(big.as_bytes(), &limits),
        Err(WireError::Transport(_))
    ));
    assert!(matches!(
        decode_envelope(b"{\"wire_version\": ", &limits),
        Err(WireError::Transport(_))
    ));
    assert!(matches!(
        decode_envelope(b"not json at all", &limits),
        Err(WireError::Transport(_))
    ));
    assert!(matches!(
        json_depth(b"{\"a\": \"unterminated"),
        Err(WireError::Transport(_))
    ));

    // Depth is checked before anything is parsed or dispatched.
    let deep = format!("{}1{}", "[".repeat(9), "]".repeat(9));
    assert_eq!(json_depth(deep.as_bytes()).unwrap(), 9);
    assert!(matches!(
        decode_envelope(deep.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
    assert_eq!(json_depth(b"{\"s\":\"[[[[[[[[[[\"}").unwrap(), 1);

    // Validation: version, unknown field, unknown variant, bad identifier.
    let wrong_version = good.replace("residuum-wire/1", "residuum-wire/2");
    assert!(matches!(
        decode_envelope(wrong_version.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
    let extra_field = good.replacen("{", "{\"extra\":1,", 1);
    assert!(matches!(
        decode_envelope(extra_field.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
    let unknown_variant = good.replace("\"inspect_theory\"", "\"drop_everything\"");
    assert!(matches!(
        decode_envelope(unknown_variant.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
    let bad_id = good.replace("\"action_id\":\"a1\"", "\"action_id\":\"a 1\"");
    assert!(matches!(
        decode_envelope(bad_id.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
    let advance = encode_envelope(&envelope(
        "a2",
        Request::AdvanceSession {
            session: "s1".into(),
            max_steps: 1,
        },
    ))
    .unwrap();
    let unknown_inner = advance.replace("\"max_steps\":1", "\"max_steps\":1,\"force\":true");
    assert!(matches!(
        decode_envelope(unknown_inner.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
    let negative_steps = advance.replace("\"max_steps\":1", "\"max_steps\":-1");
    assert!(matches!(
        decode_envelope(negative_steps.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));

    // Responses: unknown stage and inconsistent partition are rejected.
    let response = Response {
        action_id: "a1".into(),
        status: Status::Complete,
        payload: Payload::Partition {
            pool: Vec::new(),
            partition: Partition {
                hypotheses: vec!["c1".into(), "c2".into()],
                kernel: vec![0, 1],
            },
            signatures: vec![vec![true], vec![false]],
            discriminants: vec![0],
            covers: Vec::new(),
        },
        steps_consumed: 0,
        budget: BudgetState {
            actions_remaining: 1,
            steps_remaining: 1,
            sessions_remaining: 1,
            candidates_remaining: 1,
        },
    };
    let text = encode_response(&response).unwrap();
    assert_eq!(decode_response(text.as_bytes(), &limits).unwrap(), response);
    let inconsistent = text.replace("\"kernel\":[0,1]", "\"kernel\":[0]");
    assert!(matches!(
        decode_response(inconsistent.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
    let bad_status = text.replace("\"status\":\"complete\"", "\"status\":\"perfect\"");
    assert!(matches!(
        decode_response(bad_status.as_bytes(), &limits),
        Err(WireError::Validation(_))
    ));
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
