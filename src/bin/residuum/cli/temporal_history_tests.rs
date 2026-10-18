//! Certification of the temporal history of formalizations
//! (docs/legal-temporal-history.md). Every closure is computed by
//! `close_program`; this file only loads modules, projects closures onto the
//! orbit of a dossier moment, compares them and checks the locality
//! conditions on the typed modules. It implements no inference.
use residuum::experiments::render_event;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{
    Antecedent, BehaviorKind, Closure, Declaration, DeclarationOrigin, DerivationTime, Event,
    Module, Rule, RuleBody, TimeExpr,
};
use residuum::{DerivationClock, close_program, load_formal_module};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const FIXTURES: &str = "docs/legal-geometry/temporal-history/fixtures";
const ONTOLOGY: &str = "LegalGeneral";

fn path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(relative)
}

fn load(relative: &str) -> Module {
    load_formal_module(path(relative)).unwrap()
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(
            self.0 < 10_000,
            "finite history fixture unexpectedly expands"
        );
        DerivationTime {
            name: format!("history-{}", self.0),
        }
    }
}

fn seeds(module: &Module, experiment: &str) -> Vec<Event> {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == experiment => Some(e.input.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("experiment {experiment}"))
}

fn run(module: &Module, experiment: &str) -> Closure {
    close_program(seeds(module, experiment), module, &mut Clock::default()).unwrap()
}

fn at(name: &str) -> TimeExpr {
    TimeExpr::At(name.into())
}

fn after(time: TimeExpr) -> TimeExpr {
    TimeExpr::After(Box::new(time))
}

/// orb(base) = {base, after(base), after(after(base)), ...}.
fn in_orbit(time: &TimeExpr, base: &TimeExpr) -> bool {
    time == base || matches!(time, TimeExpr::After(inner) if in_orbit(inner, base))
}

/// The constant at the bottom of a ground time term.
fn root(time: &TimeExpr) -> Option<&str> {
    match time {
        TimeExpr::At(name) => Some(name),
        TimeExpr::After(inner) => root(inner),
        TimeExpr::Var(_) => None,
    }
}

fn origin(o: &DeclarationOrigin) -> String {
    let file = o
        .file
        .as_ref()
        .map(|f| f.canonicalize().unwrap_or(f.clone()));
    format!("{} {:?}", o.module, file)
}

/// Identity of a producer, the contract shared by isolated and accumulated
/// runs: the rule name (unique in a module graph: the resolver rejects
/// duplicate symbols) together with its declaration origin, and for a
/// compiled Program member the Program, its defining module, the member index
/// and the origin of the member rule.
fn producer(module: &Module, rule: &str) -> String {
    match program_rule_origin(module, rule) {
        Some(p) => format!(
            "{rule} [program {} of {} member {} rule {} of {}]",
            p.program,
            p.defining_module,
            p.rule_use,
            p.rule,
            origin(&p.rule_origin)
        ),
        None => format!(
            "{rule} [{}]",
            origin(
                module
                    .origin(rule)
                    .unwrap_or_else(|| panic!("origin of {rule}"))
            )
        ),
    }
}

fn event_key(event: &Event) -> String {
    render_event(event)
}

/// A justification without its derivation-time label: event, producer,
/// antecedents (in order) and the substitution (sorted by binder).
fn justification(module: &Module, derived: &residuum::typed::DerivedEvent) -> String {
    let record = &derived.record;
    let mut bindings: Vec<_> = record
        .substitution
        .bindings
        .iter()
        .map(|b| format!("{}={:?}", b.parameter, b.value))
        .collect();
    bindings.sort();
    let antecedents: Vec<_> = record.antecedents.iter().map(event_key).collect();
    format!(
        "{} <= {} | {} | {{{}}}",
        event_key(&derived.event),
        producer(module, &record.rule),
        antecedents.join(" & "),
        bindings.join(", ")
    )
}

#[derive(Debug, PartialEq, Eq)]
struct View {
    known: BTreeSet<String>,
    /// Multiset: justification -> multiplicity.
    justifications: BTreeMap<String, usize>,
}

/// pi_base: the events of the closure in orb(base) and their justifications.
fn project(module: &Module, closure: &Closure, base: &TimeExpr) -> View {
    let known = closure
        .known
        .iter()
        .filter(|e| in_orbit(&e.proposition.time, base))
        .map(event_key)
        .collect();
    let mut justifications = BTreeMap::new();
    for derived in &closure.derivations {
        if in_orbit(&derived.event.proposition.time, base) {
            *justifications
                .entry(justification(module, derived))
                .or_insert(0) += 1;
        }
    }
    View {
        known,
        justifications,
    }
}

fn events_only(view: &View) -> BTreeSet<String> {
    view.known
        .iter()
        .filter(|e| !e.starts_with("applies"))
        .cloned()
        .collect()
}

fn factors(antecedent: &Antecedent) -> Vec<&Event> {
    match antecedent {
        Antecedent::Unit => vec![],
        Antecedent::Event(e) => vec![e],
        Antecedent::And(a, b) => [factors(a), factors(b)].concat(),
    }
}

fn clause(rule: &Rule) -> residuum::typed::HornClause {
    match &rule.body {
        RuleBody::HornClause(c) => c.clone(),
        RuleBody::ResidualClause(c) => c.unresiduate(),
    }
}

fn rules(module: &Module) -> Vec<&Rule> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r),
            _ => None,
        })
        .collect()
}

/// Rules that close_program treats as definitions: the rules referenced by a
/// verb association, directly or as members of an associated Program.
fn definitions(module: &Module) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for declaration in &module.declarations {
        let Declaration::Verb(verb) = declaration else {
            continue;
        };
        let Some(reference) = &verb.program else {
            continue;
        };
        match reference.kind {
            BehaviorKind::Rule => {
                out.insert(reference.name.clone());
            }
            BehaviorKind::Program => {
                for d in &module.declarations {
                    if let Declaration::Program(p) = d {
                        if p.identity == reference.name {
                            out.extend(p.rule_uses.iter().map(|u| u.rule.name.clone()));
                        }
                    }
                }
            }
        }
    }
    out
}

/// Anchors: axioms (antecedent I) whose consequent verb activates a Program.
fn anchors(module: &Module) -> Vec<(String, String, TimeExpr)> {
    let activating: BTreeSet<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) if v.program.is_some() => Some(v.name.clone()),
            _ => None,
        })
        .collect();
    rules(module)
        .into_iter()
        .filter_map(|r| {
            let c = clause(r);
            (matches!(c.antecedent, Antecedent::Unit) && activating.contains(&c.consequent.verb))
                .then(|| {
                    (
                        r.name.clone(),
                        c.consequent.verb.clone(),
                        c.consequent.proposition.time.clone(),
                    )
                })
        })
        .collect()
}

/// Condition 1, 2 and the one-formalization-per-moment condition: every
/// anchor is a bare constant, and no two anchors share a moment or a root.
fn moment_violations(module: &Module) -> Vec<String> {
    let mut out = Vec::new();
    let mut roots = BTreeMap::<String, String>::new();
    for (name, _, time) in anchors(module) {
        if !matches!(time, TimeExpr::At(_)) {
            out.push(format!("{name}: moment {time:?} is not a bare constant"));
        }
        let r = root(&time).unwrap_or("?").to_string();
        if let Some(previous) = roots.insert(r.clone(), name.clone()) {
            out.push(format!(
                "{name} and {previous}: orbits of the same root {r}"
            ));
        }
    }
    out
}

/// Activation instances come only from anchors: no other rule concludes a
/// verb associated with a Program, and no seed is such an instance.
fn activation_violations(module: &Module, seeds: &[Event]) -> Vec<String> {
    let activating: BTreeSet<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) if v.program.is_some() => Some(v.name.clone()),
            _ => None,
        })
        .collect();
    let anchor_names: BTreeSet<_> = anchors(module).into_iter().map(|a| a.0).collect();
    let definitions = definitions(module);
    let mut out: Vec<_> = rules(module)
        .into_iter()
        .filter(|r| !anchor_names.contains(&r.name) && !definitions.contains(&r.name))
        .filter(|r| activating.contains(&clause(r).consequent.verb))
        .map(|r| r.name.clone())
        .collect();
    out.extend(
        seeds
            .iter()
            .filter(|e| activating.contains(&e.verb))
            .map(event_key),
    );
    out
}

/// Condition 8: every seed of a dossier is indexed by its bare moment.
fn seed_violations(seeds: &[Event], moment: &TimeExpr) -> Vec<String> {
    seeds
        .iter()
        .filter(|e| &e.proposition.time != moment)
        .map(event_key)
        .collect()
}

/// A rule is time-local when all its antecedents share one time term base
/// and every time it mentions lies in the orbit of that base.
fn time_local(rule: &Rule, base: Option<&TimeExpr>) -> bool {
    let c = clause(rule);
    let events = factors(&c.antecedent);
    let base = match base {
        Some(b) => b.clone(),
        None => match events.first() {
            Some(e) => e.proposition.time.clone(),
            None => return true,
        },
    };
    events
        .iter()
        .chain(std::iter::once(&&c.consequent))
        .all(|e| in_orbit(&e.proposition.time, &base))
}

/// Condition 4: every compiled member of an associated Program only mentions
/// times in the orbit of the Program's time parameter.
fn member_violations(module: &Module) -> Vec<String> {
    let mut out = Vec::new();
    for declaration in &module.declarations {
        let Declaration::Program(program) = declaration else {
            continue;
        };
        let time = TimeExpr::Var(format!(
            "#{}:shared:{}",
            program.identity,
            program.parameters.last().unwrap().name
        ));
        for member in compile_program(program, module).unwrap() {
            if !time_local(&member, Some(&time)) {
                out.push(member.name.clone());
            }
        }
    }
    out
}

/// Condition 7: an ordinary rule outside the ontology whose times are not
/// constants is a template active at every moment.
fn active_template_violations(module: &Module) -> Vec<String> {
    let definitions = definitions(module);
    rules(module)
        .into_iter()
        .filter(|r| !definitions.contains(&r.name))
        .filter(|r| module.origin(&r.name).map(|o| o.module.as_str()) != Some(ONTOLOGY))
        .filter(|r| {
            let c = clause(r);
            factors(&c.antecedent)
                .into_iter()
                .chain(std::iter::once(&c.consequent))
                .any(|e| root(&e.proposition.time).is_none())
        })
        .map(|r| r.name.clone())
        .collect()
}

/// Condition 3 on the history itself: every derivation's antecedents lie in
/// the orbit-root of its event (no derivation joins two dossiers).
fn cross_moment_derivations(closure: &Closure) -> Vec<String> {
    closure
        .derivations
        .iter()
        .filter(|d| {
            let r = root(&d.event.proposition.time);
            d.record
                .antecedents
                .iter()
                .any(|a| root(&a.proposition.time) != r)
        })
        .map(|d| format!("{} by {}", event_key(&d.event), d.record.rule))
        .collect()
}

fn has(closure: &Closure, rendered: &str) -> bool {
    closure.known.iter().any(|e| event_key(e) == rendered)
}

const STAGES: [(&str, &str, &str); 3] = [
    ("t0", "stage0.res", "dossier0"),
    ("t1", "stage1.res", "dossier1"),
    ("t2", "stage2.res", "dossier2"),
];

#[test]
fn ontology_rules_are_time_local_and_effects_are_terminal() {
    let history = load("history2.res");
    let ontology: Vec<_> = rules(&history)
        .into_iter()
        .filter(|r| history.origin(&r.name).unwrap().module == ONTOLOGY)
        .collect();
    assert_eq!(ontology.len(), 27);
    for rule in &ontology {
        assert!(time_local(rule, None), "{} is not time-local", rule.name);
        let c = clause(rule);
        let base = factors(&c.antecedent)[0].proposition.time.clone();
        assert!(matches!(base, TimeExpr::Var(_)), "{}", rule.name);
        for e in factors(&c.antecedent) {
            assert_eq!(e.proposition.time, base, "{}", rule.name);
        }
    }
    let after_heads: BTreeSet<_> = ontology
        .iter()
        .filter(|r| matches!(clause(r).consequent.proposition.time, TimeExpr::After(_)))
        .map(|r| clause(r).consequent.verb)
        .collect();
    assert_eq!(
        after_heads,
        ["do", "give", "notDo"].map(String::from).into()
    );
    // No rule of the history, ontological or not, consumes an effect.
    for rule in rules(&history) {
        for e in factors(&clause(rule).antecedent) {
            assert!(
                !after_heads.contains(&e.verb),
                "{} consumes {}",
                rule.name,
                e.verb
            );
        }
    }
}

#[test]
fn history_satisfies_the_locality_conditions() {
    let history = load("history2.res");
    assert!(moment_violations(&history).is_empty());
    assert!(member_violations(&history).is_empty());
    assert!(active_template_violations(&history).is_empty());
    assert!(activation_violations(&history, &seeds(&history, "history")).is_empty());
    let anchored: Vec<_> = anchors(&history)
        .into_iter()
        .map(|(name, verb, time)| (name, verb, root(&time).unwrap().to_string()))
        .collect();
    assert_eq!(
        anchored,
        [
            ("anchorR0AtT0", "appliesR0", "t0"),
            ("anchorR1AtT1", "appliesR1", "t1"),
            ("anchorR1AtT2", "appliesR1", "t2"),
        ]
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
    );
    // The history seeds are exactly the disjoint union of the dossiers, and
    // every dossier is indexed by its own bare moment.
    let mut union = Vec::new();
    for (moment, stage, dossier) in STAGES {
        let s = seeds(&load(stage), dossier);
        assert!(seed_violations(&s, &at(moment)).is_empty(), "{dossier}");
        union.extend(s);
    }
    assert_eq!(seeds(&history, "history"), union);
    // Shared participants do not join dossiers: every derivation is local.
    assert!(cross_moment_derivations(&run(&history, "history")).is_empty());
    // The time-free templates are definitions in every stage and history.
    for module in [
        "stage0.res",
        "stage1.res",
        "stage2.res",
        "history1.res",
        "history2.res",
    ] {
        let m = load(module);
        let d = definitions(&m);
        for template in ["ordinaryBan", "refinedBan", "registrationDuty"] {
            assert!(d.contains(template), "{template} active in {module}");
        }
    }
}

#[test]
fn projection_of_the_history_equals_each_isolated_run() {
    let history = load("history2.res");
    let closure = run(&history, "history");
    let mut covered = BTreeSet::new();
    for (moment, stage, dossier) in STAGES {
        let module = load(stage);
        let isolated = run(&module, dossier);
        // The isolated closure lives entirely in the orbit of its moment.
        assert!(
            isolated
                .known
                .iter()
                .all(|e| in_orbit(&e.proposition.time, &at(moment)))
        );
        let expected = project(&module, &isolated, &at(moment));
        let projected = project(&history, &closure, &at(moment));
        assert_eq!(
            projected, expected,
            "pi_{moment} H2 differs from Run(T at {moment})"
        );
        assert_eq!(
            expected.justifications.values().sum::<usize>(),
            isolated.derivations.len()
        );
        covered.extend(projected.known);
    }
    // The orbits cover the history: nothing lies outside a dossier moment.
    let all: BTreeSet<_> = closure.known.iter().map(event_key).collect();
    assert_eq!(covered, all);
}

#[test]
fn adding_formalizations_preserves_the_historical_projections() {
    let stage0 = load("stage0.res");
    let h1 = load("history1.res");
    let h2 = load("history2.res");
    let c0 = run(&stage0, "dossier0");
    let c1 = run(&h1, "history");
    let c2 = run(&h2, "history");
    assert_eq!(
        project(&stage0, &c0, &at("t0")),
        project(&h1, &c1, &at("t0"))
    );
    for moment in ["t0", "t1"] {
        assert_eq!(
            project(&h1, &c1, &at(moment)),
            project(&h2, &c2, &at(moment)),
            "{moment}"
        );
    }
    // No capture: a rule ordinary in an earlier history stays ordinary (F7
    // replaced rules; here every template is a definition from its declaration).
    for (earlier, later) in [(&stage0, &h1), (&h1, &h2)] {
        let ordinary: BTreeSet<_> = rules(earlier)
            .into_iter()
            .map(|r| r.name.clone())
            .filter(|n| !definitions(earlier).contains(n))
            .collect();
        assert!(ordinary.is_disjoint(&definitions(later)));
        let declared: BTreeSet<_> = rules(later).into_iter().map(|r| r.name.clone()).collect();
        assert!(rules(earlier).iter().all(|r| declared.contains(&r.name)));
    }
    // H1 ⊆ H2 as closures: the history only grows.
    let k1: BTreeSet<_> = c1.known.iter().map(event_key).collect();
    let k2: BTreeSet<_> = c2.known.iter().map(event_key).collect();
    assert!(k1.is_subset(&k2));
}

#[test]
fn consequences_effects_and_provenance_of_the_history() {
    let history = load("history2.res");
    let c = run(&history, "history");
    // R0 at t0: a prohibition, its abstention effect and the ontological
    // violation and liability, for rex although rex has A.
    for e in [
        "prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0",
        "notDo(ana, enterWith, rex, host) @ after(t0)",
        "violation(dogBan, visitAna0, ana, rex, host, administrative, enact0) @ t0",
        "patrimonialLiability(ana, rex, dogBan, administrative, enact0) @ t0",
        // R1 at t1: refined prohibition for fido (explicit not-A), duty for rex.
        "prohibition(dogBan, bea, enterWith, fido, host, administrative, enact0) @ t1",
        "obligation(regDuty, ana, register, rex, host, administrative, enact1) @ t1",
        "do(ana, register, rex, host) @ after(t1)",
        // Ontology only: a transfer and its give effect.
        "transferred(sale1, ana, rex, cal, private, saleDate1) @ t1",
        "give(ana, rex, cal) @ after(t1)",
        // R1 again at t2; annulment compares legal dates given as data.
        "prohibition(dogBan, bea, enterWith, fido, host, administrative, enact0) @ t2",
        "annulled(dogBan, permitFido, legislative, enact1) @ t2",
    ] {
        assert!(has(&c, e), "missing {e}");
    }
    // R0 is not active at t1 although rex is in both dossiers (no leak), and
    // the F4 notDo regression: rex's abstention exists only after t0.
    for e in [
        "prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t1",
        "notDo(ana, enterWith, rex, host) @ after(t1)",
        "notDo(ana, enterWith, rex, host) @ after(t2)",
    ] {
        assert!(!has(&c, e), "unexpected {e}");
    }
    let rex_abstentions: Vec<_> = c
        .known
        .iter()
        .filter(|e| e.verb == "notDo" && render_event(e).starts_with("notDo(ana"))
        .map(|e| e.proposition.time.clone())
        .collect();
    assert_eq!(rex_abstentions, vec![after(at("t0"))]);
    // No position uses a dossier moment as its legal date.
    for e in &c.known {
        if let Some(last) = e.args.last() {
            let rendered = residuum::experiments::render_term(last);
            assert!(
                !["t0", "t1", "t2"].contains(&rendered.as_str()),
                "{}",
                event_key(e)
            );
        }
    }
    // Provenance: the prohibition at t0 is produced by member 0 of R0, the
    // rule ordinaryBan of Generalizations, with t bound by the activation
    // appliesR0() @ t0, itself produced by the anchor of Stage0.
    let prohibition = c
        .derivations
        .iter()
        .find(|d| {
            event_key(&d.event)
                == "prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0"
        })
        .unwrap();
    assert_eq!(prohibition.record.rule, "R0::0::ordinaryBan");
    let origin = program_rule_origin(&history, "R0::0::ordinaryBan").unwrap();
    assert_eq!(
        (
            &*origin.program,
            &*origin.defining_module,
            &*origin.rule,
            &*origin.rule_origin.module
        ),
        ("R0", "FormalizationR0", "ordinaryBan", "Generalizations")
    );
    assert_eq!(
        event_key(&prohibition.record.antecedents[0]),
        "appliesR0() @ t0"
    );
    let bindings: BTreeMap<_, _> = prohibition
        .record
        .substitution
        .bindings
        .iter()
        .map(|b| (b.parameter.clone(), format!("{:?}", b.value)))
        .collect();
    assert_eq!(
        bindings["#R0:shared:t"],
        format!("{:?}", residuum::typed::SubstitutionValue::Time(at("t0")))
    );
    assert!(bindings.contains_key("#R0:use0:dog"));
    let anchor = c
        .derivations
        .iter()
        .find(|d| event_key(&d.event) == "appliesR0() @ t0")
        .unwrap();
    assert_eq!(anchor.record.rule, "anchorR0AtT0");
    assert!(anchor.record.antecedents.is_empty());
    assert_eq!(history.origin("anchorR0AtT0").unwrap().module, "Stage0");
    // The same formalization R1 anchored twice is told apart by the anchor.
    let r1_producers: BTreeSet<_> = c
        .derivations
        .iter()
        .filter(|d| d.record.rule.starts_with("R1::"))
        .map(|d| (d.record.rule.clone(), event_key(&d.record.antecedents[0])))
        .collect();
    assert!(r1_producers.contains(&("R1::0::refinedBan".into(), "appliesR1() @ t1".into())));
    assert!(r1_producers.contains(&("R1::0::refinedBan".into(), "appliesR1() @ t2".into())));
    // Alternative justifications are kept: the duty at t1 by R1 and by the
    // filed obligation of the ontology.
    let duty: BTreeSet<_> = c
        .derivations
        .iter()
        .filter(|d| {
            event_key(&d.event)
                == "obligation(regDuty, ana, register, rex, host, administrative, enact1) @ t1"
        })
        .map(|d| d.record.rule.clone())
        .collect();
    assert_eq!(
        duty,
        ["R1::1::registrationDuty", "filedObligation"]
            .map(String::from)
            .into()
    );
}

#[test]
fn native_alternatives_reproduce_the_events_of_the_program_history() {
    let history = load("history2.res");
    let reference = run(&history, "history");
    for alternative in [
        "alternatives/a_pinned.res",
        "alternatives/b_specialized.res",
    ] {
        let module = load(alternative);
        assert!(moment_violations(&module).is_empty());
        assert!(
            active_template_violations(&module).is_empty(),
            "{alternative}"
        );
        let closure = run(&module, "history");
        assert!(cross_moment_derivations(&closure).is_empty());
        for (moment, _, _) in STAGES {
            assert_eq!(
                events_only(&project(&module, &closure, &at(moment))),
                events_only(&project(&history, &reference, &at(moment))),
                "{alternative} at {moment}"
            );
        }
    }
    // B: the templates stay definitions, the specializations are ordinary
    // rules confined to their constant.
    let b = load("alternatives/b_specialized.res");
    let d = definitions(&b);
    assert!(d.contains("ordinaryBan") && !d.contains("b0OrdinaryBan"));
}

#[test]
fn counterexamples_are_detected_and_break_the_equivalence() {
    // A template that stays active (F6 premise).
    let loose = load("counterexamples/cx_active_template.res");
    assert_eq!(
        active_template_violations(&loose),
        vec!["looseBan".to_string()]
    );
    let c = run(&loose, "history");
    assert!(has(
        &c,
        "prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t1"
    ));

    // A member rule with an independent time variable.
    let mixed = load("counterexamples/cx_mixed_times.res");
    assert_eq!(
        member_violations(&mixed),
        vec!["RMixed::0::mixedBan".to_string()]
    );
    let isolated = run(&mixed, "isolated");
    let joined = run(&mixed, "history");
    let fido = "prohibition(dogBan, bea, enterWith, fido, host, administrative, enact0) @ t1";
    assert!(!has(&isolated, fido) && has(&joined, fido));
    assert_eq!(
        cross_moment_derivations(&joined),
        vec![format!("{fido} by RMixed::0::mixedBan")]
    );
    assert_ne!(
        project(&mixed, &isolated, &at("t1")),
        project(&mixed, &joined, &at("t1"))
    );

    // Intersecting orbits: a dossier at after(t0).
    let stage = load("counterexamples/cx_after_stage.res");
    let joined_module = load("counterexamples/cx_after_history.res");
    assert!(!moment_violations(&joined_module).is_empty());
    let moment = after(at("t0"));
    let alone = project(&stage, &run(&stage, "dossierAfter"), &moment);
    let joined = project(&joined_module, &run(&joined_module, "history"), &moment);
    let foreign: Vec<_> = joined.known.difference(&alone.known).cloned().collect();
    // Both abstentions produced by dossier 0 fall into the later orbit.
    assert_eq!(
        foreign,
        vec![
            "notDo(ana, enterWith, rex, host) @ after(t0)".to_string(),
            "notDo(bea, enterWith, fido, host) @ after(t0)".to_string(),
        ]
    );

    // Two formalizations at one moment (the F4 premise).
    let double = load("counterexamples/cx_double_anchor.res");
    assert_eq!(moment_violations(&double).len(), 1);
    let both = project(&double, &run(&double, "history"), &at("t0"));
    let hypothetical_module = load("hypothetical_r1_on_dossier0.res");
    let r1_alone = project(
        &hypothetical_module,
        &run(&hypothetical_module, "dossier0"),
        &at("t0"),
    );
    let stage0 = load("stage0.res");
    let r0_alone = project(&stage0, &run(&stage0, "dossier0"), &at("t0"));
    assert_ne!(both, r1_alone);
    assert_ne!(both, r0_alone);
    let rex = "notDo(ana, enterWith, rex, host) @ after(t0)";
    assert!(both.known.contains(rex) && !r1_alone.known.contains(rex));
}

#[test]
fn hypothetical_runs_stay_outside_the_history() {
    let module = load("hypothetical_r1_on_dossier0.res");
    let closure = run(&module, "dossier0");
    // R1 on dossier 0: without explicit not-A and without the duty's datum
    // nothing normative follows.
    assert!(
        closure
            .derivations
            .iter()
            .all(|d| d.record.rule == "anchorR1AtT0")
    );
    let history = load("history2.res");
    assert!(history.origin("anchorR1AtT0").is_none());
    let c = run(&history, "history");
    assert!(!has(&c, "appliesR1() @ t0"));
}
