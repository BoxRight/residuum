//! Freeze checks of the Pilot 004 contract (docs/pilot-004-contract.md):
//! vocabulary, dossiers, candidate catalogue and initial descriptions. Every
//! closure is computed by `close_program`. No valuation is read or produced,
//! and no prediction table is printed.
use residuum::typed::{
    Antecedent, Closure, Declaration, DeclarationOrigin, DerivationTime, Event, Module, Polarity,
    Rule, RuleBody, Term, TimeExpr, VerbKind,
};
use residuum::{DerivationClock, close_program, load_formal_module};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const FROZEN: &str = "docs/legal-geometry/pilot-004/frozen";
const ONTOLOGY: &str = "examples/experiments/legal_geometry/ontology_reference/legal_general.res";
const VOCABULARY: &str = "Pilot004Vocabulary";
const HEADS: [&str; 3] = ["prohibition", "permission", "obligation"];
const DOSSIERS: [&str; 4] = ["dossier1", "dossier2", "dossier3", "dossier4"];
const CONDITIONS: [&str; 2] = ["allergic", "visuallyImpaired"];
const KINDS: [&str; 2] = ["animal", "food"];

fn path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn frozen(file: &str) -> Module {
    load_formal_module(path(&format!("{FROZEN}/{file}"))).unwrap()
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(self.0 < 10_000, "finite pilot fixture unexpectedly expands");
        DerivationTime {
            name: format!("pilot-{}", self.0),
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

fn run(module: &Module, seeds: &[Event]) -> Closure {
    close_program(seeds.to_vec(), module, &mut Clock::default()).unwrap()
}

fn constant(term: &Term) -> String {
    match term {
        Term::Const(c) => c.clone(),
        other => panic!("constant expected, found {other:?}"),
    }
}

fn variable(term: &Term) -> Option<&str> {
    match term {
        Term::Var(v) => Some(v),
        _ => None,
    }
}

fn atom(event: &Event) -> (String, Vec<String>) {
    (
        event.verb.clone(),
        event.args.iter().map(constant).collect(),
    )
}

fn atoms(a: &Antecedent) -> Vec<&Event> {
    match a {
        Antecedent::Unit => vec![],
        Antecedent::Event(e) => vec![e],
        Antecedent::And(l, r) => [atoms(l), atoms(r)].concat(),
    }
}

fn vocabulary_verbs(module: &Module) -> BTreeSet<String> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v)
                if module.origin(&v.name).map(|o| o.module.as_str()) == Some(VOCABULARY) =>
            {
                Some(v.name.clone())
            }
            _ => None,
        })
        .collect()
}

fn catalogue() -> Vec<(String, Rule)> {
    let mut rules = Vec::new();
    for head in HEADS {
        let module = frozen(&format!("catalog_{head}.res"));
        for d in &module.declarations {
            if let Declaration::Rule(r) = d {
                if r.name.starts_with("p4c") {
                    rules.push((head.to_string(), r.clone()));
                }
            }
        }
    }
    rules.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    rules
}

/// Literal of a candidate body without its variable names: verb, polarity
/// and arguments as role letters or constants.
fn literal(e: &Event) -> (String, bool, Vec<String>) {
    let args = e
        .args
        .iter()
        .map(|a| match a {
            Term::Var(v) => format!("?{v}"),
            Term::Const(c) => c.clone(),
            other => panic!("unexpected term {other:?}"),
        })
        .collect();
    (e.verb.clone(), e.polarity == Polarity::EvidentialNot, args)
}

/// Canonical form: head kind and body literals after renaming variables by
/// first occurrence (anchor, body, head). The rule name is ignored.
fn canon(head: &str, rule: &Rule) -> String {
    let RuleBody::HornClause(c) = &rule.body else {
        panic!("{} is not Horn", rule.name)
    };
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    let mut events = atoms(&c.antecedent);
    events.push(&c.consequent);
    let mut out = vec![head.to_string()];
    for e in events {
        let args: Vec<_> = e
            .args
            .iter()
            .map(|a| match a {
                Term::Var(v) => {
                    let next = format!("v{}", names.len());
                    names.entry(v.clone()).or_insert(next).clone()
                }
                Term::Const(k) => k.clone(),
                other => panic!("unexpected term {other:?}"),
            })
            .collect();
        let neg = if e.polarity == Polarity::EvidentialNot {
            "~"
        } else {
            ""
        };
        out.push(format!("{neg}{}({})", e.verb, args.join(",")));
    }
    out.join(" ")
}

/// The grammar of contract §4, enumerated independently of the generator:
/// canonical bodies as sets of literals over roles s, b, x and the thing o.
fn grammar_bodies() -> BTreeSet<Vec<(String, bool, Vec<String>)>> {
    let mut lits = Vec::new();
    for v in ["s", "b", "x"] {
        for k in CONDITIONS {
            for neg in [false, true] {
                lits.push((
                    "condition".to_string(),
                    neg,
                    vec![format!("?{v}"), k.to_string()],
                ));
            }
        }
    }
    for k in KINDS {
        for neg in [false, true] {
            lits.push(("kind".to_string(), neg, vec!["?o".into(), k.to_string()]));
        }
    }
    for (p, q) in [("s", "b"), ("s", "x"), ("b", "x")] {
        for neg in [false, true] {
            lits.push((
                "copresent".to_string(),
                neg,
                vec![format!("?{p}"), format!("?{q}")],
            ));
        }
    }
    let admissible = |body: &[(String, bool, Vec<String>)]| {
        let atoms: BTreeSet<_> = body
            .iter()
            .map(|(v, _, a)| (v.clone(), a.clone()))
            .collect();
        let uses_x = body.iter().any(|(_, _, a)| a.contains(&"?x".to_string()));
        let linked = body
            .iter()
            .any(|(v, _, a)| v == "copresent" && a.contains(&"?x".to_string()));
        atoms.len() == body.len() && (linked || !uses_x)
    };
    let mut bodies = BTreeSet::new();
    for i in 0..lits.len() {
        if admissible(&lits[i..=i]) {
            bodies.insert(vec![lits[i].clone()]);
        }
        for j in i + 1..lits.len() {
            let body = vec![lits[i].clone(), lits[j].clone()];
            if admissible(&body) {
                bodies.insert(body);
            }
        }
    }
    bodies
}

fn gloss(constant: &str) -> &'static str {
    match constant {
        "allergic" => "an allergy",
        "visuallyImpaired" => "a visual impairment",
        "animal" => "an animal",
        "food" => "food or drink",
        other => panic!("no gloss for {other}"),
    }
}

/// d0 rendered from the typed rule (independent of the generator).
fn render(head: &str, rule: &Rule) -> String {
    let RuleBody::HornClause(c) = &rule.body else {
        unreachable!()
    };
    let body = atoms(&c.antecedent);
    let mut seen_x = false;
    let mut role = |t: &Term| -> String {
        match variable(t) {
            Some("s") => "the subject".into(),
            Some("b") => "the beneficiary".into(),
            Some("x") if seen_x => "X".into(),
            Some("x") => {
                seen_x = true;
                "a person X".into()
            }
            other => panic!("unexpected role {other:?}"),
        }
    };
    let mut clauses = Vec::new();
    for e in &body[1..] {
        let neg = e.polarity == Polarity::EvidentialNot;
        let clause = match e.verb.as_str() {
            "condition" => {
                let who = role(&e.args[0]);
                let what = gloss(&constant(&e.args[1]));
                if neg {
                    format!(
                        "there is explicit evidence that {who} does not have the condition {what}"
                    )
                } else {
                    format!("{who} has the condition {what}")
                }
            }
            "kind" => {
                let what = gloss(&constant(&e.args[1]));
                if neg {
                    format!("there is explicit evidence that the thing is not {what}")
                } else {
                    format!("the thing is {what}")
                }
            }
            "copresent" => {
                let (p, q) = (role(&e.args[0]), role(&e.args[1]));
                if neg {
                    format!("there is explicit evidence that {p} and {q} are not in the same place")
                } else {
                    format!("{p} and {q} are in the same place")
                }
            }
            other => panic!("unexpected verb {other}"),
        };
        clauses.push(clause);
    }
    let then = match head {
        "prohibition" => {
            "a prohibition holds on the subject performing that conduct with that thing towards the beneficiary"
        }
        "permission" => {
            "a permission holds for the subject performing that conduct with that thing towards the beneficiary"
        }
        "obligation" => {
            "an obligation holds on the subject to perform that conduct with that thing towards the beneficiary"
        }
        other => panic!("unexpected head {other}"),
    };
    format!(
        "If an act is recorded in which the subject performs a conduct with a thing for a beneficiary, and {}, then {then}.",
        clauses.join(", and ")
    )
}

/// T ⊕ h: the dossier module with one candidate rule appended, under its own
/// origin. Nothing in the module is removed or rewritten.
fn install(base: &Module, rule: &Rule) -> Module {
    let mut m = base.clone();
    m.declarations.push(Declaration::Rule(rule.clone()));
    m.origins.insert(
        rule.name.clone(),
        DeclarationOrigin {
            module: "Pilot004Candidate".into(),
            file: None,
        },
    );
    m
}

#[test]
fn pilot_vocabulary_is_a_general_descriptive_signature_with_glosses() {
    let vocabulary = frozen("vocabulary.res");
    let ontology = load_formal_module(path(ONTOLOGY)).unwrap();
    let mut own = Vec::new();
    for d in &vocabulary.declarations {
        let origin = vocabulary.origin(d.name()).unwrap().module.clone();
        if origin != VOCABULARY {
            assert_eq!(origin, "LegalGeneral");
            assert!(ontology.declarations.contains(d), "{} altered", d.name());
            continue;
        }
        match d {
            Declaration::Entity(_) | Declaration::Const(_) => {}
            Declaration::Verb(v) => {
                assert_eq!(v.kind, VerbKind::Seeded, "{}", v.name);
                assert!(v.effect.is_none() && v.program.is_none(), "{}", v.name);
            }
            other => panic!("vocabulary declares {}", other.name()),
        }
        own.push(d.name().to_string());
    }
    assert_eq!(own.len(), 2 + 7 + 3);
    let forbidden = [
        "allow", "permit", "forbid", "prohib", "oblig", "duty", "must", "danger", "risk", "harm",
        "legit", "reason", "necess", "except", "nuisance", "justif", "proper",
    ];
    for name in &own {
        let lower = name.to_lowercase();
        assert!(forbidden.iter().all(|f| !lower.contains(f)), "{name}");
    }
    let glosses = std::fs::read_to_string(path(&format!("{FROZEN}/glosses.tsv"))).unwrap();
    let glossed: BTreeSet<_> = glosses
        .lines()
        .skip(1)
        .map(|l| l.split('\t').next().unwrap().to_string())
        .collect();
    for name in own
        .iter()
        .filter(|n| !["PersonCondition", "ThingKind"].contains(&n.as_str()))
    {
        assert!(glossed.contains(name), "no gloss for {name}");
    }
    for name in [
        "Act",
        "prohibition",
        "permission",
        "obligation",
        "private",
        "administrative",
        "~",
    ] {
        assert!(glossed.contains(name), "no gloss for {name}");
    }
}

#[test]
fn pilot_dossiers_are_admissible_and_include_a_relational_contrast() {
    let module = frozen("dossiers.res");
    let vocabulary = vocabulary_verbs(&module);
    let mut states = BTreeMap::new();
    for name in DOSSIERS {
        let s = seeds(&module, name);
        let acts: Vec<_> = s.iter().filter(|e| e.verb == "Act").collect();
        assert_eq!(acts.len(), 1, "{name}: one act anchor");
        let index = format!("{:?}", acts[0].proposition.time);
        let date = constant(&acts[0].args[6]);
        assert!(
            !index.contains(&date),
            "{name}: the act date is not the index"
        );
        let mut state = BTreeMap::new();
        for e in &s {
            assert_eq!(
                format!("{:?}", e.proposition.time),
                index,
                "{name}: one index"
            );
            assert!(
                e.verb == "Act" || vocabulary.contains(&e.verb),
                "{name}: {}",
                e.verb
            );
            if e.verb == "Act" {
                continue;
            }
            let previous = state.insert(atom(e), e.polarity.clone());
            assert!(previous.is_none(), "{name}: {:?} recorded twice", atom(e));
            if e.verb == "copresent" {
                let a = atom(e);
                let mirror = (
                    "copresent".to_string(),
                    vec![a.1[1].clone(), a.1[0].clone()],
                );
                assert!(
                    s.iter()
                        .any(|f| atom(f) == mirror && f.polarity == e.polarity),
                    "{name}: copresence recorded in both orientations"
                );
            }
        }
        // The base theory derives no position from a dossier.
        let closure = run(&module, &s);
        assert!(
            closure
                .known
                .iter()
                .all(|e| !HEADS.contains(&e.verb.as_str())),
            "{name}"
        );
        states.insert(name, state);
    }
    // One-factor variations from dossier1.
    let differing = |other: &str| -> BTreeSet<String> {
        let (a, b) = (&states["dossier1"], &states[other]);
        a.keys()
            .chain(b.keys())
            .filter(|k| a.get(*k) != b.get(*k))
            .map(|k| k.0.clone())
            .collect()
    };
    assert_eq!(
        differing("dossier2"),
        BTreeSet::from(["copresent".to_string()])
    );
    assert_eq!(differing("dossier3"), BTreeSet::from(["kind".to_string()]));
    assert_eq!(
        differing("dossier4"),
        BTreeSet::from(["condition".to_string()])
    );
    // dossier1 and dossier2 have the same individual attributes; they differ
    // only in a relation between participants.
    let attributes = |n: &str| -> BTreeMap<_, _> {
        states[n]
            .iter()
            .filter(|(k, _)| k.0 != "copresent")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };
    assert_eq!(attributes("dossier1"), attributes("dossier2"));
}

#[test]
fn catalogue_is_the_complete_admissible_grammar_with_reproducible_descriptions() {
    let rules = catalogue();
    assert_eq!(rules.len(), 534);
    let expected = grammar_bodies();
    assert_eq!(expected.len(), 178);
    let descriptions =
        std::fs::read_to_string(path(&format!("{FROZEN}/descriptions.tsv"))).unwrap();
    let d0: BTreeMap<_, _> = descriptions
        .lines()
        .skip(1)
        .map(|l| {
            let cols: Vec<_> = l.split('\t').collect();
            (
                cols[0].to_string(),
                (cols[1].to_string(), cols[3].to_string()),
            )
        })
        .collect();
    assert_eq!(d0.len(), 534);
    let mut canons = BTreeSet::new();
    let mut per_head: BTreeMap<String, BTreeSet<Vec<(String, bool, Vec<String>)>>> =
        BTreeMap::new();
    for (head, rule) in &rules {
        let RuleBody::HornClause(c) = &rule.body else {
            panic!("{} is not Horn", rule.name)
        };
        let body = atoms(&c.antecedent);
        let all: Vec<&Event> = body.iter().copied().chain([&c.consequent]).collect();
        // One variable dossier index for every atom.
        let indices: BTreeSet<_> = all
            .iter()
            .map(|e| format!("{:?}", e.proposition.time))
            .collect();
        assert_eq!(indices.len(), 1, "{}", rule.name);
        assert!(
            matches!(c.consequent.proposition.time, TimeExpr::Var(_)),
            "{}",
            rule.name
        );
        // Anchor, then one or two descriptive antecedents.
        assert_eq!(body[0].verb, "Act", "{}", rule.name);
        assert!((2..=3).contains(&body.len()), "{}", rule.name);
        let anchor: Vec<_> = body[0]
            .args
            .iter()
            .map(|a| variable(a).unwrap_or(""))
            .collect();
        assert_eq!(anchor[1..5], ["s", "c", "o", "b"], "{}", rule.name);
        // Head: an existing position with the constants of the hypothetical
        // norm and the participants of the act (head safety).
        assert_eq!(&c.consequent.verb, head);
        let h: Vec<_> = c.consequent.args.iter().collect();
        assert_eq!(constant(h[0]), "hypNorm");
        assert_eq!(constant(h[5]), "administrative");
        assert_eq!(constant(h[6]), "hypEnacted");
        let participants: Vec<_> = h[1..5].iter().map(|a| variable(a).unwrap()).collect();
        assert_eq!(participants, ["s", "c", "o", "b"], "{}", rule.name);
        // No record or optional-field binding anywhere.
        assert!(
            all.iter().all(|e| e
                .args
                .iter()
                .all(|a| !matches!(a, Term::RecordLiteral { .. }))),
            "{}",
            rule.name
        );
        let lits: Vec<_> = body[1..].iter().map(|e| literal(e)).collect();
        per_head.entry(head.clone()).or_default().insert(lits);
        assert!(
            canons.insert(canon(head, rule)),
            "duplicate candidate {}",
            rule.name
        );
        let (tsv_head, text) = &d0[&rule.name];
        assert_eq!(tsv_head, head);
        assert_eq!(text, &render(head, rule), "{}", rule.name);
    }
    for head in HEADS {
        assert_eq!(
            per_head[head], expected,
            "{head}: catalogue equals the grammar"
        );
    }
}

#[test]
fn every_candidate_runs_in_isolation_with_constant_projected_arguments() {
    let dossiers = frozen("dossiers.res");
    let mut evaluated = 0;
    let mut signatures = BTreeSet::new();
    let mut inert = 0;
    for (_, rule) in catalogue() {
        let mut signature = Vec::new();
        let module = install(&dossiers, &rule);
        for name in DOSSIERS {
            let s = seeds(&dossiers, name);
            let closure = run(&module, &s);
            let again = run(&module, &s);
            assert_eq!(closure.known, again.known, "{} on {name}", rule.name);
            let act = s.iter().find(|e| e.verb == "Act").unwrap();
            for e in closure
                .known
                .iter()
                .filter(|e| HEADS.contains(&e.verb.as_str()))
            {
                assert_eq!(constant(&e.args[0]), "hypNorm");
                assert_eq!(constant(&e.args[5]), "administrative");
                assert_eq!(constant(&e.args[6]), "hypEnacted");
                // Positions concern exactly the participants of the act.
                assert_eq!(e.args[1..5], act.args[1..5], "{} on {name}", rule.name);
            }
            let kinds: BTreeSet<_> = closure
                .known
                .iter()
                .filter(|e| HEADS.contains(&e.verb.as_str()))
                .map(|e| e.verb.clone())
                .collect();
            signature.push(kinds);
            evaluated += 1;
        }
        if signature.iter().all(|k| k.is_empty()) {
            inert += 1;
        }
        signatures.insert(signature);
    }
    assert_eq!(evaluated, 534 * 4);
    // Aggregate structure only; no per-candidate prediction is printed.
    println!(
        "prediction classes: {}; candidates deriving no position on any dossier: {inert}",
        signatures.len()
    );
}
