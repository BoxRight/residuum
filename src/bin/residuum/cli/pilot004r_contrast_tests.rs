//! Pilot 004-R contrast between run-003's hypotheses h1 and h2
//! (docs/legal-geometry/pilot-004r/contrast/README.md). Uses the existing
//! worker operations `minimal_supports` and `compare_partition` and the
//! engine's closure; neither the worker nor the explorer tools are changed.
//! h1 was generated after the Brookside feedback and reproduces the
//! formalization convention of B7–B11; h2 was proposed in Casa Fiesta explore
//! without any prior discrepancy. The counterfactual pool is a test fixture:
//! it carries no valuation and is not a documented fact. No model is called.
use residuum::SearchLimits;
use residuum::experiments::exploration::{
    Criterion, Decision, frontiers, goal_index, install, joint_universe, seeded_indices,
};
use residuum::experiments::generative::{
    Condition, DossierInput, Learner, LearnerAction, LearnerView, PROJECTED_HEADS, Phase,
    ReservedValuations, Scoring, TrajectoryBudgets, TrajectoryManifest, Usage, profile_004r,
    run_trajectory,
};
use residuum::experiments::worker::{
    Budgets, Envelope, Manifest, PROTOCOL_VERSION, Payload, Request, Status, Worker,
};
use residuum::typed::{
    Closure, Declaration, DerivationTime, Event, Module, Polarity, Rule, SubstitutionValue, Term,
    TimeExpr,
};
use residuum::{DerivationClock, close_program, load_formal_module};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const VOCABULARY: &str = "docs/legal-geometry/pilot-004r/vocabulary.res";
const DOSSIERS: &str = "docs/legal-geometry/pilot-004r/dossiers.res";
const POOL: &str = "docs/legal-geometry/pilot-004r/contrast/counterfactual_pool.res";
const RESERVED: &str = "docs/legal-geometry/pilot-004r/reserved/valuations.tsv";

/// Verbatim from run-003 (docs/legal-geometry/pilot-004r/runs/run-003/run/).
const H1: &str = "rule obligationYieldsRight(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= right(id, subject, conduct, object, beneficiary, authority, effective) @ t";
const H2: &str = "rule obligationYieldsRightCreditor(id, subject, conduct, object, beneficiary, authority, effective, t) =
    obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= right(id, beneficiary, conduct, object, subject, authority, effective) @ t";

fn path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(
            self.0 < 10_000,
            "finite contrast fixture unexpectedly expands"
        );
        DerivationTime {
            name: format!("c-{}", self.0),
        }
    }
}

fn close(module: &Module, seeds: &[Event]) -> Closure {
    close_program(seeds.to_vec(), module, &mut Clock::default()).unwrap()
}

fn seeds_of(file: &str, name: &str) -> Vec<Event> {
    load_formal_module(path(file))
        .unwrap()
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e.input.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("experiment {name}"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = path(&format!("target/pilot004r/contrast/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rule(name: &str, text: &str) -> Rule {
    let dir = scratch(&format!("modules/{name}"));
    let file = dir.join(format!("{name}.res"));
    // target/pilot004r/contrast/modules/<name> is five levels deep.
    std::fs::write(
        &file,
        format!("module {name}\n\nimport \"../../../../../{VOCABULARY}\"\n\n{text}\n"),
    )
    .unwrap();
    let module = load_formal_module(&file).unwrap();
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

// ---------------------------------------------------------------------------
// Rendering (Formal-like text, for the map).

fn term(t: &Term) -> String {
    match t {
        Term::Const(c) | Term::Var(c) => c.clone(),
        Term::None => "none".into(),
        other => format!("{other:?}"),
    }
}

fn time(t: &TimeExpr) -> String {
    match t {
        TimeExpr::At(c) | TimeExpr::Var(c) => c.clone(),
        TimeExpr::After(inner) => format!("after {}", time(inner)),
    }
}

fn ev(e: &Event) -> String {
    let neg = if e.polarity == Polarity::EvidentialNot {
        "~"
    } else {
        ""
    };
    let args: Vec<String> = e.args.iter().map(term).collect();
    format!(
        "{neg}{}({}) @ {}",
        e.verb,
        args.join(", "),
        time(&e.proposition.time)
    )
}

fn set(events: &[Event]) -> String {
    format!(
        "{{{}}}",
        events.iter().map(ev).collect::<Vec<_>>().join(", ")
    )
}

// ---------------------------------------------------------------------------
// Worker plumbing (existing operations only).

struct Contrast {
    worker: Worker,
    episode: String,
    next: usize,
}

impl Contrast {
    fn new(episode: &str, dossiers: BTreeMap<String, Vec<Event>>) -> Self {
        let anchor = dossiers.values().flatten().next().cloned().unwrap();
        let worker = Worker::new(Manifest {
            episode_id: episode.into(),
            manifest_hash: "pilot004r-contrast".into(),
            imports: vec![path(VOCABULARY)],
            workdir: scratch(&format!("worker/{episode}")),
            profile: profile_004r(),
            // The analysis operations never read D; the worker validates one.
            decision: Decision {
                components: vec![anchor],
                criterion: Criterion::Component(0),
            },
            dossiers,
            budgets: Budgets {
                max_actions: 200,
                max_steps_per_action: 64,
                max_steps_total: 4096,
                max_sessions: 16,
                max_candidates: 4,
                max_clauses: 2,
                search_limits: SearchLimits::default(),
            },
        })
        .unwrap();
        Self {
            worker,
            episode: episode.into(),
            next: 0,
        }
    }

    fn send(&mut self, request: Request) -> Payload {
        self.next += 1;
        let response = self.worker.handle(Envelope {
            protocol_version: PROTOCOL_VERSION.into(),
            episode_id: self.episode.clone(),
            action_id: format!("a{}", self.next),
            manifest_hash: "pilot004r-contrast".into(),
            request,
        });
        assert_eq!(response.status, Status::Complete, "{:?}", response.payload);
        response.payload
    }

    fn propose(&mut self, text: &str) -> String {
        match self.send(Request::ProposeCandidate { text: text.into() }) {
            Payload::Candidate {
                candidate: Some(c), ..
            } => c,
            other => panic!("{other:?}"),
        }
    }

    fn supports(&mut self, candidate: &str, goal: &Event) -> BTreeSet<Vec<Event>> {
        match self.send(Request::MinimalSupports {
            candidate: candidate.into(),
            goal: goal.clone(),
        }) {
            Payload::Supports {
                goal_in_universe,
                frontier,
                ..
            } => {
                assert!(goal_in_universe, "{}", ev(goal));
                frontier.into_iter().collect()
            }
            other => panic!("{other:?}"),
        }
    }
}

struct PartitionView {
    pool: Vec<Vec<Event>>,
    signatures: Vec<Vec<bool>>,
    discriminants: Vec<usize>,
    covers: Vec<(String, String)>,
}

fn partition(c: &mut Contrast, candidates: &[String], goal: &Event) -> PartitionView {
    match c.send(Request::ComparePartition {
        candidates: candidates.to_vec(),
        goal: goal.clone(),
    }) {
        Payload::Partition {
            pool,
            signatures,
            discriminants,
            covers,
            ..
        } => PartitionView {
            pool,
            signatures,
            discriminants,
            covers,
        },
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The reference theory O, on the same joint universe as the worker.

fn baseline() -> Module {
    load_formal_module(path(VOCABULARY)).unwrap()
}

fn universe(rules: &[Rule], sigma: &[Event]) -> (Vec<Event>, BTreeSet<usize>) {
    let sigma: Vec<Event> = sigma
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let (joint, e) = joint_universe(&baseline(), &[], rules, &sigma).unwrap();
    let allowed = seeded_indices(&joint, &e);
    (e, allowed)
}

fn reference_supports(
    e: &[Event],
    allowed: &BTreeSet<usize>,
    goal: &Event,
) -> BTreeSet<Vec<Event>> {
    let f = frontiers(&baseline(), e, allowed).unwrap();
    let q = goal_index(e, goal).unwrap();
    f.supports[q]
        .iter()
        .map(|s| s.iter().map(|i| e[*i].clone()).collect())
        .collect()
}

fn find(e: &[Event], verb: &str, args: &[&str]) -> Event {
    e.iter()
        .find(|x| {
            x.verb == verb
                && x.polarity == Polarity::Positive
                && x.args.len() == args.len()
                && x.args.iter().zip(args).all(|(t, a)| term(t) == *a)
        })
        .cloned()
        .unwrap_or_else(|| panic!("{verb}{args:?} not in the joint universe"))
}

/// The derivation chain of `goal` when `module` is closed over
/// `configuration`: every derivation of the goal and, recursively, of its
/// derived antecedents, each with rule, owning module, antecedents and
/// substitution, from the leaves up.
fn provenance(module: &Module, configuration: &[Event], goal: &Event) -> Vec<String> {
    let closure = close(module, configuration);
    let mut chain = Vec::new();
    let mut seen = BTreeSet::new();
    let mut pending = vec![goal.clone()];
    while let Some(event) = pending.pop() {
        if !seen.insert(event.clone()) {
            continue;
        }
        for d in closure.derivations.iter().filter(|d| d.event == event) {
            pending.extend(d.record.antecedents.iter().cloned());
            chain.push(d);
        }
    }
    chain
        .iter()
        .rev()
        .map(|d| {
            let origin = module
                .origin(&d.record.rule)
                .map(|o| o.module.clone())
                .unwrap_or_default();
            let substitution: Vec<String> = d
                .record
                .substitution
                .bindings
                .iter()
                .map(|b| {
                    format!(
                        "{}={}",
                        b.parameter,
                        match &b.value {
                            SubstitutionValue::Term(t) => term(t),
                            SubstitutionValue::Time(t) => time(t),
                        }
                    )
                })
                .collect();
            format!(
                "{} => {} by {} [{}] substitution [{}]",
                set(&d.record.antecedents),
                ev(&d.event),
                d.record.rule,
                origin,
                substitution.join(", ")
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Scoring under the current loss (unchanged scorer, via run_trajectory).

struct Commit(VecDeque<LearnerAction>);
impl Learner for Commit {
    fn act(&mut self, view: &LearnerView) -> (LearnerAction, Usage) {
        let action = if view.phase == Phase::Explore {
            self.0.pop_front()
        } else {
            None
        };
        (
            action.unwrap_or(LearnerAction::EndPhase {
                abstention: Some("contrast scoring".into()),
            }),
            Usage::default(),
        )
    }
}

fn score_brookside(texts: &[&str]) -> Scoring {
    let ids: Vec<String> = (1..=texts.len()).map(|i| format!("h{i}")).collect();
    let mut actions: VecDeque<LearnerAction> = texts
        .iter()
        .map(|t| LearnerAction::Propose {
            text: t.to_string(),
            parent: None,
            rationale: "contrast scoring".into(),
        })
        .collect();
    actions.push_back(LearnerAction::Commit { hypotheses: ids });
    let manifest = TrajectoryManifest {
        trajectory_id: format!("contrast-{}", texts.len()),
        condition: Condition::NoFeedback,
        imports: vec![path(VOCABULARY)],
        workdir: scratch(&format!("score/{}", texts.len())),
        profile: profile_004r(),
        order: vec![DossierInput {
            id: "brookside".into(),
            seeds: seeds_of(DOSSIERS, "brookside"),
        }],
        budgets: TrajectoryBudgets {
            max_verified_evaluations: 0,
            max_proposals: 2,
            max_turns_per_phase: 8,
            max_clauses: 2,
            max_navigation_actions: 0,
            max_navigation_steps: 0,
        },
        base_module: "LegalGeneral".into(),
    };
    let reserved = ReservedValuations::load_tsv(&path(RESERVED)).unwrap();
    let report = run_trajectory(&manifest, &reserved, &mut Commit(actions)).unwrap();
    report.prequential[0].clone()
}

// ---------------------------------------------------------------------------

struct Goal {
    label: &'static str,
    event: Event,
}

#[test]
fn pilot004r_contrast_h1_h2_divergence_map() {
    let pool = seeds_of(POOL, "pool");
    let h1_rules = vec![rule("ContrastH1", H1)];
    let h2_rules = vec![rule("ContrastH2", H2)];
    let joint_rules: Vec<Rule> = h1_rules.iter().chain(&h2_rules).cloned().collect();
    let (e, allowed) = universe(&joint_rules, &pool);

    let mut c = Contrast::new("pool", BTreeMap::from([("pool".to_string(), pool.clone())]));
    let h1 = c.propose(H1);
    let h2 = c.propose(H2);

    let d = "d20210216";
    let goals = [
        Goal {
            label: "right, obligated party as subject",
            event: find(
                &e,
                "right",
                &[
                    "saB_j",
                    "jant",
                    "adoptPolicy",
                    "policyA",
                    "usa",
                    "private",
                    d,
                ],
            ),
        },
        Goal {
            label: "right, beneficiary as subject",
            event: find(
                &e,
                "right",
                &[
                    "saB_j",
                    "usa",
                    "adoptPolicy",
                    "policyA",
                    "jant",
                    "private",
                    d,
                ],
            ),
        },
        Goal {
            label: "compliance by the obligated party",
            event: find(
                &e,
                "compliance",
                &["saB_j", "actJ", "jant", "policyA", "usa", "private", d],
            ),
        },
        Goal {
            label: "compliance by the beneficiary",
            event: find(
                &e,
                "compliance",
                &["saB_j", "actU", "usa", "policyA", "jant", "private", d],
            ),
        },
        Goal {
            label: "enforceability against the obligated party",
            event: find(
                &e,
                "enforceability",
                &["saB_j", "actJ", "jant", "policyA", "usa", "private", d],
            ),
        },
        Goal {
            label: "enforceability against the beneficiary",
            event: find(
                &e,
                "enforceability",
                &["saB_j", "actU", "usa", "policyA", "jant", "private", d],
            ),
        },
    ];

    let base = baseline();
    let with_h1 = install(&base, &[], &h1_rules, "ContrastH1").unwrap();
    let with_h2 = install(&base, &[], &h2_rules, "ContrastH2").unwrap();

    let mut map = String::new();
    writeln!(
        map,
        "# h1/h2 divergence map (counterfactual pool; no valuations)\n"
    )
    .unwrap();
    writeln!(map, "h1 = {h1}: {}\nh2 = {h2}: {}\n", H1, H2).unwrap();
    let mut diverging = BTreeMap::new();
    for g in &goals {
        let o = reference_supports(&e, &allowed, &g.event);
        let s1 = c.supports(&h1, &g.event);
        let s2 = c.supports(&h2, &g.event);
        let p = partition(&mut c, &[h1.clone(), h2.clone()], &g.event);
        let projected = PROJECTED_HEADS.contains(&g.event.verb.as_str());
        diverging.insert(
            g.label,
            (o.clone(), s1.clone(), s2.clone(), p.discriminants.clone()),
        );

        writeln!(map, "## {} — `{}`", g.label, ev(&g.event)).unwrap();
        writeln!(
            map,
            "head `{}` {} by the current loss\n",
            g.event.verb,
            if projected {
                "is scored"
            } else {
                "is NOT scored"
            }
        )
        .unwrap();
        for (name, s) in [("O", &o), ("O+h1", &s1), ("O+h2", &s2)] {
            writeln!(map, "minimal supports {name}:").unwrap();
            for support in s {
                writeln!(map, "  - {}", set(support)).unwrap();
            }
        }
        writeln!(
            map,
            "compare_partition region pool and signatures (h1, h2):"
        )
        .unwrap();
        for (j, region) in p.pool.iter().enumerate() {
            writeln!(
                map,
                "  [{j}] {} h1={} h2={}{}",
                set(region),
                p.signatures[0][j],
                p.signatures[1][j],
                if p.discriminants.contains(&j) {
                    "  <- discriminant"
                } else {
                    ""
                }
            )
            .unwrap();
        }
        writeln!(map, "covers (general, specific): {:?}", p.covers).unwrap();
        // Provenance of the goal on each minimal configuration that any of
        // the three theories offers.
        let configurations: BTreeSet<&Vec<Event>> = o.iter().chain(&s1).chain(&s2).collect();
        writeln!(
            map,
            "derivations of the goal on each minimal configuration:"
        )
        .unwrap();
        for conf in configurations {
            writeln!(map, "  configuration {}", set(conf)).unwrap();
            for (name, m) in [("O", &base), ("O+h1", &with_h1), ("O+h2", &with_h2)] {
                if conf.contains(&g.event) {
                    writeln!(map, "    {name}: given in the configuration (seeded leaf)").unwrap();
                    continue;
                }
                let ds = provenance(m, conf, &g.event);
                if ds.is_empty() {
                    writeln!(map, "    {name}: not derived").unwrap();
                }
                for line in ds {
                    writeln!(map, "    {name}: {line}").unwrap();
                }
            }
        }
        writeln!(map).unwrap();
    }

    // Current loss on the real Brookside dossier (B7–B11 unchanged).
    let s1 = score_brookside(&[H1]);
    let s2 = score_brookside(&[H2]);
    let s12 = score_brookside(&[H1, H2]);
    writeln!(map, "## Brookside under the current loss (real dossier)").unwrap();
    for (name, s) in [("{h1}", &s1), ("{h2}", &s2), ("{h1, h2}", &s12)] {
        writeln!(
            map,
            "{name}: loss {} missed {:?} excess {} valued {} unvalued_derived {} exposures {:?}",
            s.loss,
            s.missed.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
            s.excess.len(),
            s.valued,
            s.unvalued_derived,
            s.exposures
                .iter()
                .map(|(h, x)| format!("{h}:{}:{}", x.valuation, x.correct))
                .collect::<Vec<_>>()
        )
        .unwrap();
    }

    let out = scratch("map");
    std::fs::write(out.join("divergence-map.md"), &map).unwrap();
    println!("{map}");

    // --- Checks ------------------------------------------------------------
    let only = |s: &BTreeSet<Vec<Event>>, o: &BTreeSet<Vec<Event>>| -> BTreeSet<Vec<Event>> {
        s.difference(o).cloned().collect()
    };
    let label = |i: usize| goals[i].label;
    let filed = &pool[0];
    let (o, a, b, disc) = &diverging[label(0)];
    assert_eq!(only(a, o), BTreeSet::from([vec![filed.clone()]]));
    assert!(only(b, o).is_empty());
    assert!(!disc.is_empty());
    let (o, a, b, disc) = &diverging[label(1)];
    assert!(only(a, o).is_empty());
    assert_eq!(only(b, o), BTreeSet::from([vec![filed.clone()]]));
    assert!(!disc.is_empty());
    // Compliance by the obligated party: O already derives it through
    // obligationCompliance; h1 adds a redundant justification only.
    let (o, a, b, disc) = &diverging[label(2)];
    assert_eq!(a, o);
    assert_eq!(b, o);
    assert!(disc.is_empty());
    for (i, extra_by_h1) in [(3, false), (4, true), (5, false)] {
        let (o, a, b, disc) = &diverging[label(i)];
        let (new, same) = if extra_by_h1 { (a, b) } else { (b, a) };
        assert_eq!(same, o, "{}", label(i));
        let added = only(new, o);
        assert_eq!(added.len(), 1, "{}", label(i));
        assert!(added.iter().next().unwrap().contains(filed));
        assert!(!disc.is_empty(), "{}", label(i));
    }
    assert_eq!(s1.loss, 0);
    assert_eq!(s2.loss, 5);
    assert_eq!(s12.loss, 0);
    assert!(s2.excess.is_empty() && s12.excess.is_empty());
}

// ---------------------------------------------------------------------------
// Role signatures: which body argument fills each head argument, read from
// the rule's syntax tree (no model, no closure). The operational gloss of
// `right` and `obligation` is read the same way from the LegalGeneral rules
// that consume them.

fn atoms(a: &residuum::typed::Antecedent, out: &mut Vec<Event>) {
    use residuum::typed::Antecedent;
    match a {
        Antecedent::Unit => {}
        Antecedent::Event(e) => out.push(e.clone()),
        Antecedent::And(l, r) => {
            atoms(l, out);
            atoms(r, out);
        }
    }
}

fn parameter_names(module: &Module, verb: &str) -> Vec<String> {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == verb => {
                Some(v.args.iter().map(|a| a.name.clone()).collect())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("verb {verb}"))
}

/// `head.param <- body.param` for every head argument, or `= constant`.
/// Several body positions sharing the variable are all listed.
fn role_signature(module: &Module, rule: &Rule) -> Vec<String> {
    let residuum::typed::RuleBody::HornClause(c) = &rule.body else {
        panic!("{} is not a Horn rule", rule.name)
    };
    let mut body = Vec::new();
    atoms(&c.antecedent, &mut body);
    let head = &c.consequent;
    let head_names = parameter_names(module, &head.verb);
    let mut out = Vec::new();
    for (i, t) in head.args.iter().enumerate() {
        let sources: Vec<String> = match t {
            Term::Var(x) => body
                .iter()
                .flat_map(|b| {
                    let names = parameter_names(module, &b.verb);
                    let neg = if b.polarity == Polarity::EvidentialNot {
                        "~"
                    } else {
                        ""
                    };
                    b.args
                        .iter()
                        .enumerate()
                        .filter(|(_, u)| matches!(u, Term::Var(y) if y == x))
                        .map(|(j, _)| format!("{neg}{}.{}", b.verb, names[j]))
                        .collect::<Vec<_>>()
                })
                .collect(),
            other => vec![format!("= {}", term(other))],
        };
        out.push(format!(
            "{}.{} <- {}",
            head.verb,
            head_names[i],
            sources.join(" = ")
        ));
    }
    out
}

/// For every LegalGeneral rule whose body reads `verb`, the positions of
/// other atoms (body or head) that share each of `verb`'s arguments.
fn operational_gloss(module: &Module, verb: &str) -> Vec<String> {
    let names = parameter_names(module, verb);
    let mut out = Vec::new();
    for d in &module.declarations {
        let Declaration::Rule(r) = d else { continue };
        if module.origin(&r.name).map(|o| o.module.as_str()) != Some("LegalGeneral") {
            continue;
        }
        let residuum::typed::RuleBody::HornClause(c) = &r.body else {
            continue;
        };
        let mut body = Vec::new();
        atoms(&c.antecedent, &mut body);
        let Some(reader) = body.iter().find(|b| b.verb == verb) else {
            continue;
        };
        let mut others: Vec<(&Event, &str)> = body
            .iter()
            .filter(|b| !std::ptr::eq(*b, reader))
            .map(|b| (b, "body"))
            .collect();
        others.push((&c.consequent, "head"));
        let mut links = Vec::new();
        for (i, t) in reader.args.iter().enumerate() {
            let Term::Var(x) = t else { continue };
            let shared: Vec<String> = others
                .iter()
                .flat_map(|(o, place)| {
                    let onames = parameter_names(module, &o.verb);
                    let neg = if o.polarity == Polarity::EvidentialNot {
                        "~"
                    } else {
                        ""
                    };
                    o.args
                        .iter()
                        .enumerate()
                        .filter(|(_, u)| matches!(u, Term::Var(y) if y == x))
                        .map(|(j, _)| format!("{place} {neg}{}.{}", o.verb, onames[j]))
                        .collect::<Vec<_>>()
                })
                .collect();
            if ["subject", "beneficiary"].contains(&names[i].as_str()) {
                links.push(format!("{verb}.{} = {}", names[i], shared.join(", ")));
            }
        }
        out.push(format!("{}: {}", r.name, links.join("; ")));
    }
    out
}

#[test]
fn pilot004r_role_signatures_h1_h2() {
    let base = baseline();
    let h1 = rule("RoleH1", H1);
    let h2 = rule("RoleH2", H2);
    let s1 = role_signature(&base, &h1);
    let s2 = role_signature(&base, &h2);
    let right_gloss = operational_gloss(&base, "right");
    let obligation_gloss = operational_gloss(&base, "obligation");

    let mut text = String::new();
    writeln!(text, "# Role signatures of run-003 h1 and h2\n").unwrap();
    for (name, rule, sig) in [("h1", &h1, &s1), ("h2", &h2, &s2)] {
        writeln!(text, "## {name} `{}`", rule.name).unwrap();
        for line in sig {
            writeln!(text, "- {line}").unwrap();
        }
        writeln!(text).unwrap();
    }
    writeln!(
        text,
        "## Operational gloss read from LegalGeneral (rules whose body reads the verb)\n"
    )
    .unwrap();
    for (verb, gloss) in [("right", &right_gloss), ("obligation", &obligation_gloss)] {
        writeln!(text, "### `{verb}`").unwrap();
        for line in gloss {
            writeln!(text, "- {line}").unwrap();
        }
        writeln!(text).unwrap();
    }
    let out = scratch("roles");
    std::fs::write(out.join("role-signatures.md"), &text).unwrap();
    println!("{text}");

    let has = |sig: &[String], line: &str| sig.iter().any(|l| l == line);
    assert!(has(&s1, "right.subject <- obligation.subject"), "{s1:?}");
    assert!(
        has(&s1, "right.beneficiary <- obligation.beneficiary"),
        "{s1:?}"
    );
    assert!(
        has(&s2, "right.subject <- obligation.beneficiary"),
        "{s2:?}"
    );
    assert!(
        has(&s2, "right.beneficiary <- obligation.subject"),
        "{s2:?}"
    );
    // The two signatures differ exactly in subject and beneficiary.
    let differing: Vec<_> = s1.iter().zip(&s2).filter(|(a, b)| a != b).collect();
    assert_eq!(differing.len(), 2, "{differing:?}");
    // In LegalGeneral, the subject of `right` is the one whose Act counts.
    assert!(
        right_gloss
            .iter()
            .any(|l| l.starts_with("rightCompliance:")
                && l.contains("right.subject = body Act.subject")),
        "{right_gloss:?}"
    );
    assert!(
        right_gloss
            .iter()
            .any(|l| l.starts_with("rightEnforceability:")
                && l.contains("right.subject = body ~Act.subject")),
        "{right_gloss:?}"
    );
}
