//! Pilot 004-R circuit checks (docs/pilot-004r-generative-contract.md):
//! installation of rules with existing heads, the conservative and the
//! attributed certificates, the independent closure checker, the admissible
//! language, and complete trajectories driven by scripted learners. No model
//! provider is called. Synthetic dossiers are test fixtures of the circuit,
//! never part of the documentary corpus.
use residuum::experiments::exploration::{certify_addition, install, theory_preserved};
use residuum::experiments::generative::{
    Condition, DossierInput, FeedbackView, HypothesisStatus, Label, Learner, LearnerAction,
    LearnerView, Phase, Position, ReservedValuations, RevisionEffect, TrajectoryBudgets,
    TrajectoryManifest, TrajectoryReport, Usage, Valuation, certify_attributed, check_closure,
    profile_004r, revision_effect, run_trajectory,
};
use residuum::typed::{
    Closure, Declaration, DerivationTime, Event, Module, Rule, SubstitutionValue, Term, VerbKind,
};
use residuum::{DerivationClock, close_program, load_formal_module};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

const VOCABULARY: &str = "docs/legal-geometry/pilot-004r/vocabulary.res";
const DOSSIERS: &str = "docs/legal-geometry/pilot-004r/dossiers.res";
const RESERVED: &str = "docs/legal-geometry/pilot-004r/reserved/valuations.tsv";

const CORR: &str = "rule corr(id, s, c, o, b, au, e, t) =
    obligation(id, s, c, o, b, au, e) @ t
    <= right(id, s, c, o, b, au, e) @ t
";

const CORR_AGREED: &str = "rule corrAgreed(id, s, c, o, b, au, e, t, d) =
    obligation(id, s, c, o, b, au, e) @ t & Act(id, s, agree, d, b, au, e) @ t
    <= right(id, s, c, o, b, au, e) @ t
";

fn path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(self.0 < 10_000, "finite 004-R fixture unexpectedly expands");
        DerivationTime {
            name: format!("r-{}", self.0),
        }
    }
}

fn close(module: &Module, seeds: &[Event]) -> Closure {
    close_program(seeds.to_vec(), module, &mut Clock::default()).unwrap()
}

fn experiment_seeds(module: &Module, name: &str) -> Vec<Event> {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e.input.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("experiment {name}"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = path(&format!("target/pilot004r/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A module importing the 004-R vocabulary, written under target/.
fn module_with(name: &str, body: &str) -> Module {
    let dir = scratch(&format!("modules/{name}"));
    let file = dir.join(format!("{name}.res"));
    // Imports are relative: target/pilot004r/modules/<name> is four levels deep.
    let source = format!("module {name}\n\nimport \"../../../../{VOCABULARY}\"\n\n{body}");
    std::fs::write(&file, source).unwrap();
    load_formal_module(&file).unwrap()
}

fn rule_from(name: &str, text: &str) -> Rule {
    let module = module_with(name, text);
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r)
                if module.origin(&r.name).map(|o| o.module.as_str()) == Some(name) =>
            {
                Some(r.clone())
            }
            _ => None,
        })
        .unwrap()
}

fn baseline() -> Module {
    load_formal_module(path(VOCABULARY)).unwrap()
}

fn brookside() -> Vec<Event> {
    experiment_seeds(&load_formal_module(path(DOSSIERS)).unwrap(), "brookside")
}

fn casa_fiesta() -> Vec<Event> {
    experiment_seeds(&load_formal_module(path(DOSSIERS)).unwrap(), "casa_fiesta")
}

fn verbs(closure: &Closure, verb: &str) -> usize {
    closure.known.iter().filter(|e| e.verb == verb).count()
}

fn learned(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Installation and certificates.

#[test]
fn pilot004r_install_admits_rule_with_existing_seeded_head_right() {
    let base = baseline();
    let corr = rule_from("InstallCorr", CORR);
    let installed = install(&base, &[], &[corr.clone()], "learned").unwrap();
    assert!(
        installed
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Rule(r) if *r == corr))
    );
    assert_eq!(installed.origin("corr").unwrap().module, "learned");
    theory_preserved(&base, &installed).unwrap();
    let right = base
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == "right" => Some(v.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(right.kind, VerbKind::Seeded);
    // install cannot redeclare an existing signature, only add rules.
    let mut derived = right.clone();
    derived.kind = VerbKind::Derived;
    let err = install(&base, &[derived], &[corr], "learned").unwrap_err();
    assert!(err.to_string().contains("S_INSTALL"), "{err}");
}

#[test]
fn pilot004r_conservative_certificate_covers_right_on_brookside() {
    let base = baseline();
    let seeds = brookside();
    let extended = install(&base, &[], &[rule_from("ConsCorr", CORR)], "learned").unwrap();
    let reference = close(&base, &seeds);
    assert_eq!(verbs(&reference, "right"), 0);
    assert_eq!(verbs(&reference, "obligation"), 5);
    let ext = close(&extended, &seeds);
    assert_eq!(verbs(&ext, "right"), 5);
    let conservation = certify_addition(&reference, &ext, &learned(&["right"])).unwrap();
    assert!(conservation.holds(), "{conservation:?}");
    let (certificate, _) =
        certify_attributed(&base, &extended, &learned(&["corr"]), &seeds).unwrap();
    assert!(certificate.holds(), "{certificate:?}");
    assert_eq!(certificate.new_events.len(), 5);
    assert!(
        certificate
            .new_events
            .iter()
            .all(|n| n.direct == learned(&["corr"]) && n.chained.is_empty())
    );
    assert!(certificate.activated_base_rules.is_empty());
}

/// Synthetic engine check, not a corpus fact: Brookside plus an explicit
/// non-performance, so that a derived `right` activates `rightEnforceability`.
fn brookside_with_non_performance() -> Vec<Event> {
    let module = module_with(
        "SyntheticNonPerformance",
        "experiment x {
    seeds {
        ~Act(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB,
    }
    query residual
    Act(saB_j, jant, agree, saDocB, usa, private, d20210216) @ tB
}
",
    );
    let mut seeds = brookside();
    seeds.extend(experiment_seeds(&module, "x"));
    seeds
}

#[test]
fn pilot004r_conservative_certificate_rejects_chained_consequences_attributed_accepts() {
    let base = baseline();
    let seeds = brookside_with_non_performance();
    let extended = install(&base, &[], &[rule_from("ChainCorr", CORR)], "learned").unwrap();
    let reference = close(&base, &seeds);
    let ext = close(&extended, &seeds);
    assert_eq!(verbs(&reference, "enforceability"), 0);
    assert_eq!(verbs(&ext, "enforceability"), 1);
    let conservation = certify_addition(&reference, &ext, &learned(&["right"])).unwrap();
    assert!(!conservation.holds(), "{conservation:?}");
    let (certificate, _) =
        certify_attributed(&base, &extended, &learned(&["corr"]), &seeds).unwrap();
    assert!(certificate.holds(), "{certificate:?}");
    assert!(
        certificate
            .activated_base_rules
            .contains("rightEnforceability")
    );
    let chained: Vec<_> = certificate
        .new_events
        .iter()
        .filter(|n| n.event.verb == "enforceability")
        .collect();
    assert_eq!(chained.len(), 1);
    assert_eq!(chained[0].chained, learned(&["rightEnforceability"]));
    assert!(chained[0].direct.is_empty());
}

#[test]
fn pilot004r_checker_detects_tampered_closures() {
    let base = baseline();
    let seeds = brookside_with_non_performance();
    let extended = install(&base, &[], &[rule_from("TamperCorr", CORR)], "learned").unwrap();
    let clean = close(&extended, &seeds);
    let check = check_closure(&extended, &seeds, &clean).unwrap();
    assert!(check.holds(), "{check:?}");

    // A derived event removed from Known: justifications mention it and the
    // instance yields an event outside Known.
    let mut missing = clean.clone();
    let victim = missing
        .known
        .iter()
        .position(|e| e.verb == "enforceability")
        .unwrap();
    missing.known.remove(victim);
    let c = check_closure(&extended, &seeds, &missing).unwrap();
    assert!(!c.sound && !c.event_complete, "{c:?}");

    // A substitution value altered: the record is no longer an instance.
    let mut altered = clean.clone();
    let d = altered
        .derivations
        .iter_mut()
        .find(|d| d.record.rule == "corr")
        .unwrap();
    for b in d.record.substitution.bindings.iter_mut() {
        if b.parameter == "o" {
            b.value = SubstitutionValue::Term(Term::Const("signB".into()));
        }
    }
    let c = check_closure(&extended, &seeds, &altered).unwrap();
    assert!(!c.sound, "{c:?}");

    // An unsupported event in Known.
    let mut fabricated = clean.clone();
    let mut fake = fabricated
        .known
        .iter()
        .find(|e| e.verb == "right")
        .unwrap()
        .clone();
    fake.args[2] = Term::Const("sue".into());
    fabricated.known.push(fake);
    let c = check_closure(&extended, &seeds, &fabricated).unwrap();
    assert!(!c.supported, "{c:?}");

    // A justification dropped: the instance is no longer recorded.
    let mut dropped = clean.clone();
    let i = dropped
        .derivations
        .iter()
        .position(|d| d.record.rule == "rightEnforceability")
        .unwrap();
    dropped.derivations.remove(i);
    let c = check_closure(&extended, &seeds, &dropped).unwrap();
    assert!(!c.justification_complete && !c.supported, "{c:?}");
}

#[test]
fn pilot004r_attributed_certificate_rejects_unattributed_or_lossy_extensions() {
    let base = baseline();
    let seeds = brookside();
    let corr = rule_from("LossyCorr", CORR);
    let extended = install(&base, &[], &[corr], "learned").unwrap();
    // The addition is not declared as learned.
    let (c, _) = certify_attributed(&base, &extended, &BTreeSet::new(), &seeds).unwrap();
    assert!(!c.theory_preserved && !c.holds(), "{c:?}");
    // The "extension" drops a rule the reference uses.
    let (c, _) = certify_attributed(&extended, &base, &BTreeSet::new(), &seeds).unwrap();
    assert!(
        !c.theory_preserved && !c.monotone_events && !c.holds(),
        "{c:?}"
    );
}

// ---------------------------------------------------------------------------
// Trajectories with scripted learners.

type Key = (String, bool);

/// Fixed actions per (dossier, explore?) plus an optional reactive rule;
/// records every view it receives.
struct Scripted {
    queue: BTreeMap<Key, VecDeque<LearnerAction>>,
    react: Option<Box<dyn FnMut(&LearnerView) -> Option<Vec<LearnerAction>>>>,
    usage: Usage,
    views: Vec<LearnerView>,
}

impl Scripted {
    fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            react: None,
            usage: Usage {
                calls: 1,
                ..Usage::default()
            },
            views: Vec::new(),
        }
    }

    fn at(mut self, dossier: &str, phase: Phase, actions: Vec<LearnerAction>) -> Self {
        self.queue
            .entry((dossier.into(), phase == Phase::Explore))
            .or_default()
            .extend(actions);
        self
    }
}

impl Learner for Scripted {
    fn act(&mut self, view: &LearnerView) -> (LearnerAction, Usage) {
        self.views.push(view.clone());
        let key = (view.dossier.clone(), view.phase == Phase::Explore);
        if !self.queue.contains_key(&key) {
            let planned = self.react.as_mut().and_then(|f| f(view));
            self.queue
                .insert(key.clone(), planned.unwrap_or_default().into());
        }
        let action = self
            .queue
            .get_mut(&key)
            .and_then(|q| q.pop_front())
            .unwrap_or(LearnerAction::EndPhase);
        (action, self.usage)
    }
}

fn propose(text: &str, parent: Option<&str>) -> LearnerAction {
    LearnerAction::Propose {
        text: text.into(),
        parent: parent.map(str::to_string),
        rationale: "scripted".into(),
    }
}

fn evaluate(h: &str, d: &str) -> LearnerAction {
    LearnerAction::Evaluate {
        hypothesis: h.into(),
        dossier: d.into(),
    }
}

fn commit(hs: &[&str]) -> LearnerAction {
    LearnerAction::Commit {
        hypotheses: hs.iter().map(|s| s.to_string()).collect(),
    }
}

fn budgets() -> TrajectoryBudgets {
    TrajectoryBudgets {
        max_verified_evaluations: 8,
        max_proposals: 12,
        max_turns_per_phase: 12,
        max_clauses: 2,
    }
}

fn manifest(id: &str, condition: Condition, order: Vec<DossierInput>) -> TrajectoryManifest {
    TrajectoryManifest {
        trajectory_id: id.into(),
        condition,
        imports: vec![path(VOCABULARY)],
        workdir: scratch(&format!("work/{id}")),
        profile: profile_004r(),
        order,
        budgets: budgets(),
        base_module: "LegalGeneral".into(),
    }
}

fn corpus_order() -> Vec<DossierInput> {
    vec![
        DossierInput {
            id: "casaFiesta".into(),
            seeds: casa_fiesta(),
        },
        DossierInput {
            id: "flowers".into(),
            seeds: Vec::new(),
        },
        DossierInput {
            id: "brookside".into(),
            seeds: brookside(),
        },
    ]
}

fn reserved() -> ReservedValuations {
    ReservedValuations::load_tsv(&path(RESERVED)).unwrap()
}

fn statuses<'a>(report: &'a TrajectoryReport, h: &str) -> &'a Vec<HypothesisStatus> {
    report.statuses.get(h).unwrap()
}

#[test]
fn pilot004r_corpus_baseline_without_hypotheses() {
    let m = manifest("baseline", Condition::Feedback, corpus_order());
    let mut learner = Scripted::new();
    let report = run_trajectory(&m, &reserved(), &mut learner).unwrap();
    let losses: Vec<usize> = report.prequential.iter().map(|s| s.loss).collect();
    assert_eq!(losses, vec![0, 0, 5]);
    let missed: Vec<&str> = report.prequential[2]
        .missed
        .iter()
        .map(|v| v.id.as_str())
        .collect();
    assert_eq!(missed, vec!["B7", "B8", "B9", "B10", "B11"]);
    // The controls are derived by O itself; Flowers' exclusions are unreachable.
    assert_eq!(report.prequential[2].valued, 11);
    assert_eq!(report.prequential[1].valued, 10);
    assert_eq!(report.prequential[0].valued, 0);
}

#[test]
fn pilot004r_prospective_trajectory_casa_fiesta_flowers_brookside() {
    let m = manifest("prospective", Condition::Feedback, corpus_order());
    let mut learner = Scripted::new().at(
        "casaFiesta",
        Phase::Explore,
        vec![
            propose(CORR, None),
            evaluate("h1", "casaFiesta"),
            commit(&["h1"]),
        ],
    );
    let report = run_trajectory(&m, &reserved(), &mut learner).unwrap();
    assert_eq!(report.hypotheses[0].admission, "admitted");
    assert!(!report.hypotheses[0].case_specific);
    let losses: Vec<usize> = report.prequential.iter().map(|s| s.loss).collect();
    assert_eq!(losses, vec![0, 0, 0]);
    let st = statuses(&report, "h1");
    assert!(st.contains(&HypothesisStatus::Valid), "{st:?}");
    assert!(
        st.contains(&HypothesisStatus::Prospective {
            dossiers: 1,
            exposures: 5
        }),
        "{st:?}"
    );
    assert!(
        st.contains(&HypothesisStatus::Compatible {
            dossiers: 1,
            exposures: 5
        }),
        "{st:?}"
    );
    // Generated before Brookside, it cannot be refuted by any of the three.
    assert!(
        !st.iter()
            .any(|s| matches!(s, HypothesisStatus::Refuted { .. }))
    );
    assert_eq!(report.counters.attempts, 1);
    assert_eq!(report.counters.verified_evaluations, 1);
    assert_eq!(report.counters.commits, 1);
}

#[test]
fn pilot004r_feedback_and_control_share_budget_and_isolate_valuations() {
    let react = || -> Box<dyn FnMut(&LearnerView) -> Option<Vec<LearnerAction>>> {
        Box::new(|view: &LearnerView| {
            let missed_right = view.feedback.iter().any(|f| match f {
                FeedbackView::Discrepancies { missed, .. } => {
                    missed.iter().any(|p| p.head == "right")
                }
                FeedbackView::Evaluated { .. } => false,
            });
            (view.phase == Phase::Revise && missed_right && view.committed.is_empty()).then(|| {
                vec![
                    propose(CORR, None),
                    evaluate("h1", &view.dossier),
                    commit(&["h1"]),
                ]
            })
        })
    };
    let mut with_feedback = Scripted::new();
    with_feedback.react = Some(react());
    let mut control = Scripted::new();
    control.react = Some(react());
    let fm = manifest("feedback", Condition::Feedback, corpus_order());
    let cm = manifest("control", Condition::NoFeedback, corpus_order());
    assert_eq!(fm.budgets, cm.budgets);
    let f = run_trajectory(&fm, &reserved(), &mut with_feedback).unwrap();
    let c = run_trajectory(&cm, &reserved(), &mut control).unwrap();

    assert_eq!(f.prequential_loss(), 5);
    assert_eq!(c.prequential_loss(), 5);
    assert_eq!(f.compatibility.last().unwrap()[2].loss, 0);
    assert_eq!(c.compatibility.last().unwrap()[2].loss, 5);
    let st = statuses(&f, "h1");
    assert!(
        st.contains(&HypothesisStatus::Compatible {
            dossiers: 1,
            exposures: 5
        }),
        "{st:?}"
    );
    assert!(
        !st.iter()
            .any(|s| matches!(s, HypothesisStatus::Prospective { .. }))
    );
    assert!(c.hypotheses.is_empty());
    assert!(matches!(
        revision_effect(&f, &c),
        RevisionEffect::NotMeasurable(_)
    ));

    // Reserved material never reaches a view; discrepancies only after
    // commitment and only in the feedback condition.
    for view in with_feedback.views.iter().chain(&control.views) {
        let text = format!("{view:?}");
        for forbidden in [
            "Required",
            "Excluded",
            "non-waiver",
            "Opinion",
            "B7",
            "F1",
            "control",
        ] {
            assert!(!text.contains(forbidden), "{forbidden} leaked: {text}");
        }
        if view.phase == Phase::Explore {
            assert!(!view.feedback.iter().any(|fb| match fb {
                FeedbackView::Discrepancies { dossier, .. }
                | FeedbackView::Evaluated { dossier } => *dossier == view.dossier,
            }));
        }
    }
    assert!(control.views.iter().all(|v| {
        v.feedback
            .iter()
            .all(|fb| matches!(fb, FeedbackView::Evaluated { .. }))
    }));
}

#[test]
fn pilot004r_budget_counts_verified_evaluations_and_usage_separately() {
    let m = manifest("budget", Condition::NoFeedback, corpus_order());
    let mut actions = vec![propose(CORR, None)];
    actions.extend((0..9).map(|_| evaluate("h1", "casaFiesta")));
    actions.push(evaluate("h1", "brookside"));
    let mut learner = Scripted::new().at("casaFiesta", Phase::Explore, actions);
    learner.usage = Usage {
        calls: 1,
        input_tokens: 100,
        output_tokens: 10,
        cost_microusd: 5,
    };
    let report = run_trajectory(&m, &reserved(), &mut learner).unwrap();
    let k = &report.counters;
    assert_eq!(k.attempts, 1);
    assert_eq!(k.verified_evaluations, 8);
    // The ninth evaluation exceeds the budget; Brookside is not yet presented.
    assert_eq!(k.refused_actions, 2);
    assert_eq!(k.usage.calls, k.turns as u64);
    assert_eq!(k.usage.input_tokens, 100 * k.turns as u64);
    assert_eq!(k.usage.output_tokens, 10 * k.turns as u64);
    assert_eq!(k.usage.cost_microusd, 5 * k.turns as u64);
}

#[test]
fn pilot004r_admissible_language_rejections_are_not_refutations() {
    let m = manifest("admission", Condition::Feedback, corpus_order());
    let texts = [
        // head Act: fabricates a fact
        "rule badAct(id, s, c, o, b, au, e, t) =
    obligation(id, s, c, o, b, au, e) @ t
    <= Act(id, s, c, o, b, au, e) @ t
",
        // head obligationFiled: fabricates a document
        "rule badFiled(id, s, c, o, b, au, e, t) =
    Act(id, s, c, o, b, au, e) @ t
    <= obligationFiled(id, s, c, o, b, au, e) @ t
",
        // a declaration: changes the signature
        "const extra : Thing
rule withConst(id, s, c, o, b, au, e, t) =
    obligation(id, s, c, o, b, au, e) @ t
    <= right(id, s, c, o, b, au, e) @ t
",
        // a constant of a dossier not yet presented
        "rule unseen(id, s, c, au, e, t) =
    obligation(id, s, c, policyA, usa, au, e) @ t
    <= right(id, s, c, policyA, usa, au, e) @ t
",
        // constants seen in a single dossier only
        "rule cfOnly(id, s, o, b, au, e, t) =
    Act(id, s, adoptPolicy, o, b, au, e) @ t
    <= obligation(id, s, adoptPolicy, o, b, au, e) @ t
",
        CORR,
        CORR,
    ];
    let mut learner = Scripted::new().at(
        "casaFiesta",
        Phase::Explore,
        texts.iter().map(|t| propose(t, None)).collect(),
    );
    let report = run_trajectory(&m, &reserved(), &mut learner).unwrap();
    let admission: Vec<&str> = report
        .hypotheses
        .iter()
        .map(|h| h.admission.as_str())
        .collect();
    assert!(admission[0].contains("P_HEAD_INTERFACE"), "{admission:?}");
    assert!(admission[1].contains("P_HEAD_INTERFACE"), "{admission:?}");
    assert!(admission[2].contains("S_DECLARATION"), "{admission:?}");
    assert!(
        admission[3].starts_with("A6_UNSEEN_CONSTANT"),
        "{admission:?}"
    );
    assert_eq!(admission[4], "admitted");
    assert!(report.hypotheses[4].case_specific);
    assert_eq!(admission[5], "admitted");
    assert!(!report.hypotheses[5].case_specific);
    assert_eq!(admission[6], "S_NAME_COLLISION");
    for h in ["h1", "h2", "h3", "h4", "h7"] {
        let st = statuses(&report, h);
        assert!(
            matches!(st[..], [HypothesisStatus::Pending(_)]),
            "{h}: {st:?}"
        );
    }
    assert!(statuses(&report, "h5").contains(&HypothesisStatus::CaseSpecific));
    assert_eq!(report.counters.attempts, 7);
    assert_eq!(report.counters.admitted, 2);
}

// ---------------------------------------------------------------------------
// Synthetic circuit: a refutation exists, is delivered, and is revised.

fn synthetic_order() -> Vec<DossierInput> {
    let module = module_with(
        "SyntheticCircuit",
        "experiment x0 {
    seeds {
        Act(saB_j, jant, agree, saDocB, usa, private, d20210216) @ tB,
        obligationFiled(saB_j, jant, post, signB, usa, private, d20210216) @ tB,
    }
    query residual
    Act(saB_j, jant, agree, saDocB, usa, private, d20210216) @ tB
}

experiment x1 {
    seeds {
        obligationFiled(saCF_c, casaFiesta, post, signCF, usa, private, dCF) @ tCF,
    }
    query residual
    obligationFiled(saCF_c, casaFiesta, post, signCF, usa, private, dCF) @ tCF
}

experiment x2 {
    seeds {
        Act(saCF_c, casaFiesta, agree, saDocCF, usa, private, dCF) @ tCF,
        obligationFiled(saCF_c, casaFiesta, adoptPolicy, policyCF, usa, private, dCF) @ tCF,
        obligationFiled(cfA1, casaFiesta, post, signCF, usa, private, dCF) @ tCF,
    }
    query residual
    Act(saCF_c, casaFiesta, agree, saDocCF, usa, private, dCF) @ tCF
}
",
    );
    ["x0", "x1", "x2"]
        .iter()
        .map(|id| DossierInput {
            id: id.to_string(),
            seeds: experiment_seeds(&module, id),
        })
        .collect()
}

fn synthetic_reserved() -> ReservedValuations {
    let v = |dossier: &str, id: &str, s: &str, c: &str, o: &str, label: Label| Valuation {
        dossier: dossier.into(),
        id: id.into(),
        position: Position {
            head: "right".into(),
            subject: s.into(),
            conduct: c.into(),
            object: o.into(),
            beneficiary: "usa".into(),
        },
        label,
        status: "synthetic".into(),
        source: "synthetic circuit fixture".into(),
    };
    ReservedValuations::new(vec![
        v("x0", "X0a", "jant", "post", "signB", Label::Required),
        v("x1", "X1a", "casaFiesta", "post", "signCF", Label::Excluded),
        v(
            "x2",
            "X2a",
            "casaFiesta",
            "adoptPolicy",
            "policyCF",
            Label::Required,
        ),
        v("x2", "X2b", "casaFiesta", "post", "signCF", Label::Excluded),
    ])
    .unwrap()
}

/// Scripted revision: when a delivered discrepancy attributes an excluded
/// position to h1, propose a narrower rule with h1 as parent.
fn revise_on_refutation() -> Box<dyn FnMut(&LearnerView) -> Option<Vec<LearnerAction>>> {
    Box::new(|view: &LearnerView| {
        let refuted_h1 = view.feedback.iter().any(|f| {
            matches!(f, FeedbackView::Discrepancies { excess, .. }
                if excess.iter().any(|(_, owners)| owners.contains("h1")))
        });
        (view.phase == Phase::Revise && refuted_h1 && view.committed == ["h1"]).then(|| {
            vec![
                propose(CORR_AGREED, Some("h1")),
                evaluate("h2", &view.dossier),
                commit(&["h2"]),
            ]
        })
    })
}

#[test]
fn pilot004r_synthetic_refutation_is_delivered_and_revised_without_the_solution() {
    let first = || {
        Scripted::new().at(
            "x0",
            Phase::Explore,
            vec![propose(CORR, None), evaluate("h1", "x0"), commit(&["h1"])],
        )
    };
    let mut with_feedback = first();
    with_feedback.react = Some(revise_on_refutation());
    // Same reactive rule; it never fires without a discrepancy view.
    let mut control = first();
    control.react = Some(revise_on_refutation());
    let fm = manifest("synthetic-feedback", Condition::Feedback, synthetic_order());
    let cm = manifest(
        "synthetic-control",
        Condition::NoFeedback,
        synthetic_order(),
    );
    let f = run_trajectory(&fm, &synthetic_reserved(), &mut with_feedback).unwrap();
    let c = run_trajectory(&cm, &synthetic_reserved(), &mut control).unwrap();

    let losses = |r: &TrajectoryReport| r.prequential.iter().map(|s| s.loss).collect::<Vec<_>>();
    assert_eq!(losses(&f), vec![0, 1, 0]);
    assert_eq!(losses(&c), vec![0, 1, 1]);
    assert_eq!(
        revision_effect(&f, &c),
        RevisionEffect::Measured {
            dossiers: vec!["x2".into()],
            delta: 1
        }
    );
    let h1 = statuses(&f, "h1");
    assert!(
        h1.contains(&HypothesisStatus::Refuted {
            dossier: "x1".into(),
            valuations: vec!["X1a".into()]
        }),
        "{h1:?}"
    );
    assert!(
        h1.contains(&HypothesisStatus::SupersededBy("h2".into())),
        "{h1:?}"
    );
    let h2 = statuses(&f, "h2");
    assert!(
        h2.contains(&HypothesisStatus::Prospective {
            dossiers: 1,
            exposures: 1
        }),
        "{h2:?}"
    );
    assert!(
        !h2.iter()
            .any(|s| matches!(s, HypothesisStatus::Refuted { .. }))
    );
    // The delivered discrepancy names positions and owners, never a rule.
    let delivered = with_feedback
        .views
        .iter()
        .find_map(|v| {
            v.feedback.iter().find_map(|fb| match fb {
                FeedbackView::Discrepancies {
                    dossier, excess, ..
                } if dossier == "x1" => Some(excess.clone()),
                _ => None,
            })
        })
        .unwrap();
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].1, BTreeSet::from(["h1".to_string()]));
    let text = format!("{:?}", with_feedback.views.last().unwrap().feedback);
    assert!(
        !text.contains("agree") && !text.contains("corrAgreed"),
        "{text}"
    );
    // The control kept h1 and was refuted again on x2.
    let ch1 = statuses(&c, "h1");
    assert!(
        ch1.contains(&HypothesisStatus::Refuted {
            dossier: "x2".into(),
            valuations: vec!["X2b".into()]
        }),
        "{ch1:?}"
    );
}
