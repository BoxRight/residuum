//! A scripted episode against the in-process worker: statuses, budgets,
//! idempotency and reuse of the verified stages. Run individually through
//! `scripts/test_safe.py`.
use super::exploration::*;
use super::proposal::ProposalStatus;
use super::worker::*;
use crate::QueryBinding;
use crate::typed::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

const HEAD: &str = "hypotheticalSuccessionImpediment";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(name: &str) -> Module {
    crate::load_formal_module(
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

const NOT_KILLING: &str = "rule innocentImpediment(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
";

const FIVE: &str = "rule r(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
";

fn manifest(workdir: &str, budgets: Budgets) -> (Manifest, Module) {
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
        episode_id: "riggs-worker-001".into(),
        manifest_hash: "h-001".into(),
        // The problem manifest declares the experimental head; the cases file
        // declares the problem constants that typed bindings resolve against.
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
        budgets,
    };
    (manifest, base)
}

fn budgets() -> Budgets {
    Budgets {
        max_actions: 60,
        max_steps_per_action: 8,
        max_steps_total: 200,
        max_sessions: 10,
        max_candidates: 6,
        max_clauses: 2,
        search_limits: crate::SearchLimits::default(),
    }
}

fn env(action: &str, request: Request) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.into(),
        episode_id: "riggs-worker-001".into(),
        action_id: action.into(),
        manifest_hash: "h-001".into(),
        request,
    }
}

fn message(r: &Response) -> &str {
    match &r.payload {
        Payload::Error { message } => message,
        other => panic!("expected an error payload, got {other:?}"),
    }
}

fn opened(r: &Response, kind: SessionKind) -> String {
    assert_eq!(r.status, Status::Complete, "{r:?}");
    match &r.payload {
        Payload::SessionOpened { session, kind: k } if *k == kind => session.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn worker_runs_a_scripted_episode_with_statuses_budgets_and_idempotency() {
    let (manifest, base) = manifest("worker", budgets());
    let historical = experiment(&base, "historicalRiggs");
    let attr = historical.goal.clone();
    let imp = manifest.decision.components[1].clone();
    let mut w = Worker::new(manifest.clone()).unwrap();

    // Protocol and harness errors never reach a tool.
    let mut bad = env("x1", Request::InspectTheory);
    bad.protocol_version = "other/9".into();
    assert!(message(&w.handle(bad)).starts_with("protocol"));
    let mut bad = env("x2", Request::InspectTheory);
    bad.manifest_hash = "h-002".into();
    assert!(message(&w.handle(bad)).starts_with("harness"));
    let mut bad = env("x3", Request::InspectTheory);
    bad.episode_id = "other".into();
    assert!(message(&w.handle(bad)).starts_with("protocol"));
    assert_eq!(
        w.budget().actions_remaining,
        60,
        "rejected envelopes are not actions"
    );

    // Inventory of the baseline theory.
    let r = w.handle(env("a1", Request::InspectTheory));
    assert_eq!(r.status, Status::Complete);
    match &r.payload {
        Payload::Theory(t) => {
            assert!(t.rules.iter().any(|(n, o)| {
                n == "literalTestamentaryEntitlement" && o == "RiggsSuccessionReference"
            }));
            assert!(t.derived_verbs.iter().any(|v| v == HEAD));
            assert_eq!(t.dossiers.len(), 6);
        }
        other => panic!("{other:?}"),
    }

    // Forward session: step limit, completion, idempotent replay.
    let s1 = opened(
        &w.handle(env(
            "a2",
            Request::BeginForward {
                dossier: "historicalRiggs".into(),
                theory: None,
            },
        )),
        SessionKind::Forward,
    );
    let r = w.handle(env(
        "a3",
        Request::AdvanceSession {
            session: s1.clone(),
            max_steps: 2,
        },
    ));
    assert_eq!(r.status, Status::StepLimit);
    assert_eq!(r.steps_consumed, 2);
    let advance_all = env(
        "a4",
        Request::AdvanceSession {
            session: s1.clone(),
            max_steps: 10,
        },
    );
    let r = w.handle(advance_all.clone());
    assert_eq!(r.status, Status::Complete);
    assert!(r.steps_consumed <= 8, "per-action cap");
    let expected_known = crate::close_program(
        historical.input.clone(),
        &fixture("riggs_cases"),
        &mut CountingClock::default(),
    )
    .unwrap()
    .known
    .len();
    match &r.payload {
        Payload::Forward {
            known, complete, ..
        } => {
            assert!(*complete);
            assert_eq!(*known, expected_known);
        }
        other => panic!("{other:?}"),
    }
    let budget_after = w.budget();
    let replay = w.handle(advance_all);
    assert_eq!(replay, r, "same action id and content replays the response");
    assert_eq!(w.budget(), budget_after, "a replay charges nothing");
    let reused = w.handle(env(
        "a4",
        Request::AdvanceSession {
            session: s1.clone(),
            max_steps: 1,
        },
    ));
    assert!(message(&reused).contains("reused"));

    // Proposals: admitted, structurally rejected, admitted.
    let r = w.handle(env(
        "a5",
        Request::ProposeCandidate {
            text: PURPOSEFUL.into(),
        },
    ));
    assert_eq!(r.status, Status::Complete);
    let c1 = match &r.payload {
        Payload::Candidate {
            candidate: Some(c),
            report,
        } => {
            assert_eq!(
                report.stages,
                vec!["parsed", "elaborated", "extracted", "admitted"]
            );
            c.clone()
        }
        other => panic!("{other:?}"),
    };
    let r = w.handle(env("a6", Request::ProposeCandidate { text: FIVE.into() }));
    assert_eq!(r.status, Status::StructuralReject);
    match &r.payload {
        Payload::Candidate {
            candidate: None,
            report,
        } => match &report.status {
            ProposalStatus::StructuralReject(reasons) => assert!(reasons.contains("B_ATOMS")),
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
    let r = w.handle(env(
        "a7",
        Request::ProposeCandidate {
            text: NOT_KILLING.into(),
        },
    ));
    let c2 = match &r.payload {
        Payload::Candidate {
            candidate: Some(c), ..
        } => c.clone(),
        other => panic!("{other:?}"),
    };
    assert_ne!(c1, c2);

    // Candidate evaluation by session: compatible, then incompatible.
    let s2 = opened(
        &w.handle(env(
            "a8",
            Request::BeginCandidateEvaluation {
                candidate: c1.clone(),
                dossier: "historicalRiggs".into(),
            },
        )),
        SessionKind::Evaluation,
    );
    let r = w.handle(env(
        "a9",
        Request::AdvanceSession {
            session: s2.clone(),
            max_steps: 8,
        },
    ));
    assert_eq!(r.status, Status::Complete);
    let first_report = match &r.payload {
        Payload::Evaluation {
            report: Some(report),
            complete: true,
            ..
        } => {
            assert_eq!(report_label(report), "evaluated_compatible");
            report.clone()
        }
        other => panic!("{other:?}"),
    };
    let again = w.handle(env(
        "a10",
        Request::AdvanceSession {
            session: s2.clone(),
            max_steps: 8,
        },
    ));
    assert_eq!(again.status, Status::Complete);
    assert_eq!(again.steps_consumed, 0);
    match &again.payload {
        Payload::Evaluation {
            report: Some(report),
            ..
        } => assert_eq!(*report, first_report),
        other => panic!("{other:?}"),
    }
    let s3 = opened(
        &w.handle(env(
            "a11",
            Request::BeginCandidateEvaluation {
                candidate: c2.clone(),
                dossier: "historicalRiggs".into(),
            },
        )),
        SessionKind::Evaluation,
    );
    let r = w.handle(env(
        "a12",
        Request::AdvanceSession {
            session: s3,
            max_steps: 8,
        },
    ));
    match &r.payload {
        Payload::Evaluation {
            report: Some(report),
            ..
        } => assert_eq!(report_label(report), "evaluated_incompatible"),
        other => panic!("{other:?}"),
    }

    // Residual session on the baseline without bindings: the engine's
    // documented unresolved-parameter error, mapped and terminal.
    let s4 = opened(
        &w.handle(env(
            "a13",
            Request::BeginResidual {
                dossier: "historicalRiggs".into(),
                theory: None,
                goal: attr.clone(),
                scope: None,
                bindings: Vec::new(),
                allowed: None,
                limits: None,
            },
        )),
        SessionKind::Residual,
    );
    let r = w.handle(env(
        "a14",
        Request::AdvanceSession {
            session: s4.clone(),
            max_steps: 8,
        },
    ));
    assert_eq!(r.status, Status::UnresolvedParameters, "{r:?}");
    assert!(message(&r).contains("unresolved query parameters"));
    let r = w.handle(env(
        "a14b",
        Request::AdvanceSession {
            session: s4,
            max_steps: 8,
        },
    ));
    assert_eq!(r.status, Status::UnresolvedParameters);
    assert_eq!(r.steps_consumed, 0);
    assert!(message(&r).contains("terminal"));

    // The same query with an explicit scope and typed bindings completes.
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
    let s4b = opened(
        &w.handle(env(
            "a13b",
            Request::BeginResidual {
                dossier: "historicalRiggs".into(),
                theory: None,
                goal: attr.clone(),
                scope: Some(crate::HornQueryScope::Rules(vec![literal.clone()])),
                bindings: bindings.clone(),
                allowed: None,
                limits: None,
            },
        )),
        SessionKind::Residual,
    );
    let r = w.handle(env(
        "a14c",
        Request::AdvanceSession {
            session: s4b,
            max_steps: 8,
        },
    ));
    assert_eq!(r.status, Status::Complete, "{r:?}");
    match &r.payload {
        Payload::Residual {
            complete,
            requirement,
            explanations,
            kind,
            ..
        } => {
            assert!(*complete);
            assert_eq!(*kind, SessionKind::Residual);
            assert!(requirement.is_some(), "{r:?}");
            assert!(*explanations >= 1);
        }
        other => panic!("{other:?}"),
    }
    let r = w.handle(env(
        "a15",
        Request::BeginResidual {
            dossier: "historicalRiggs".into(),
            theory: None,
            goal: attr.clone(),
            scope: Some(crate::HornQueryScope::Rules(vec![RuleRef {
                name: "noSuchRule".into(),
            }])),
            bindings: Vec::new(),
            allowed: None,
            limits: None,
        },
    ));
    assert_eq!(r.status, Status::BindingError, "{r:?}");

    // Explicit search limits may tighten the episode's, never widen them,
    // in ordinary and in active queries alike.
    let cap = crate::SearchLimits::default();
    let wider = crate::SearchLimits {
        max_nodes: cap.max_nodes + 1,
        ..cap
    };
    let narrower = crate::SearchLimits {
        max_depth: 4,
        max_nodes: 64,
        max_branches: 64,
    };
    let scoped = |limits: Option<crate::SearchLimits>| Request::BeginResidual {
        dossier: "historicalRiggs".into(),
        theory: None,
        goal: attr.clone(),
        scope: Some(crate::HornQueryScope::Rules(vec![literal.clone()])),
        bindings: bindings.clone(),
        allowed: None,
        limits,
    };
    let r = w.handle(env("a15b", scoped(Some(wider))));
    assert_eq!(r.status, Status::Error, "{r:?}");
    assert!(message(&r).starts_with("budget: search limits"));
    let r = w.handle(env(
        "a15c",
        Request::BeginActiveResidual {
            dossier: "historicalRiggs".into(),
            theory: None,
            goal: attr.clone(),
            scope: Some(crate::HornQueryScope::Rules(vec![literal.clone()])),
            bindings: bindings.clone(),
            allowed: None,
            limits: Some(crate::SearchLimits {
                max_depth: cap.max_depth + 1,
                ..cap
            }),
        },
    ));
    assert!(message(&r).starts_with("budget: search limits"));
    let sessions_before = w.budget().sessions_remaining;
    opened(
        &w.handle(env("a15d", scoped(Some(narrower)))),
        SessionKind::Residual,
    );
    assert_eq!(w.budget().sessions_remaining, sessions_before - 1);

    // Active residual with the same scope on a dossier lacking the survivor
    // record: the certified requirement names the missing evidence.
    let s5 = opened(
        &w.handle(env(
            "a16",
            Request::BeginActiveResidual {
                dossier: "survivalUnestablished".into(),
                theory: None,
                goal: attr.clone(),
                scope: Some(crate::HornQueryScope::Rules(vec![literal.clone()])),
                bindings,
                allowed: None,
                limits: None,
            },
        )),
        SessionKind::ActiveResidual,
    );
    let r = w.handle(env(
        "a17",
        Request::AdvanceSession {
            session: s5,
            max_steps: 8,
        },
    ));
    assert_eq!(r.status, Status::Complete, "{r:?}");
    match &r.payload {
        Payload::Residual {
            kind,
            complete,
            requirement,
            ..
        } => {
            assert_eq!(*kind, SessionKind::ActiveResidual);
            assert!(*complete);
            let requirement = requirement.as_deref().unwrap_or("");
            assert!(requirement.contains("survivorRecorded"), "{requirement}");
        }
        other => panic!("{other:?}"),
    }

    // Geometry over the admitted candidates.
    let r = w.handle(env(
        "a18",
        Request::MinimalSupports {
            candidate: c1.clone(),
            goal: imp.clone(),
        },
    ));
    assert_eq!(r.status, Status::Complete, "{r:?}");
    match &r.payload {
        Payload::Supports {
            goal_in_universe,
            frontier,
            ..
        } => {
            assert!(*goal_in_universe);
            assert_eq!(frontier.len(), 1);
            assert_eq!(
                frontier[0].len(),
                6,
                "the purposeful candidate needs all six facts"
            );
        }
        other => panic!("{other:?}"),
    }
    let r = w.handle(env(
        "a19",
        Request::ComparePartition {
            candidates: vec![c1.clone(), c2.clone()],
            goal: imp.clone(),
        },
    ));
    assert_eq!(r.status, Status::Complete, "{r:?}");
    match &r.payload {
        Payload::Partition {
            partition,
            discriminants,
            covers,
            ..
        } => {
            assert_eq!(partition.hypotheses, vec![c1.clone(), c2.clone()]);
            assert_eq!(partition.classes(), 2);
            assert!(!discriminants.is_empty());
            assert!(
                covers.is_empty(),
                "neither candidate's region contains the other's"
            );
        }
        other => panic!("{other:?}"),
    }
    let r = w.handle(env(
        "a20",
        Request::ComparePartition {
            candidates: vec![c1.clone()],
            goal: imp.clone(),
        },
    ));
    assert_eq!(r.status, Status::Error);
    let r = w.handle(env(
        "a21",
        Request::MinimalSupports {
            candidate: "c9".into(),
            goal: imp.clone(),
        },
    ));
    assert_eq!(r.status, Status::Error);
    let r = w.handle(env(
        "a22",
        Request::AdvanceSession {
            session: "s99".into(),
            max_steps: 1,
        },
    ));
    assert!(message(&r).starts_with("reference"));
}

#[test]
fn worker_budgets_are_enforced_as_errors_not_verdicts() {
    let mut small = budgets();
    small.max_actions = 3;
    small.max_steps_total = 3;
    small.max_candidates = 1;
    let (manifest, _) = manifest("worker-budget", small);
    let mut w = Worker::new(manifest).unwrap();
    let s1 = opened(
        &w.handle(env(
            "b1",
            Request::BeginForward {
                dossier: "historicalRiggs".into(),
                theory: None,
            },
        )),
        SessionKind::Forward,
    );
    let r = w.handle(env(
        "b2",
        Request::AdvanceSession {
            session: s1.clone(),
            max_steps: 8,
        },
    ));
    assert_eq!(r.status, Status::StepLimit);
    assert_eq!(r.steps_consumed, 3, "total step budget caps the action");
    let r = w.handle(env(
        "b3",
        Request::AdvanceSession {
            session: s1,
            max_steps: 1,
        },
    ));
    assert!(message(&r).starts_with("budget: steps"));
    let r = w.handle(env("b4", Request::InspectTheory));
    assert!(message(&r).starts_with("budget: actions"));
    assert_eq!(w.budget().actions_remaining, 0);
}

#[derive(Default)]
struct CountingClock(usize);
impl crate::DerivationClock for CountingClock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}
