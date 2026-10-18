//! Checks of the semantic-revision domain contract
//! (docs/legal-semantic-domain-contract.md). Every closure is computed by
//! `close_program`; this file loads the fixtures, checks the admissibility
//! conditions of the vocabulary and of the witness situations, projects
//! normative positions and compares the results. It implements no inference
//! and attaches no valuation.
use residuum::derive::instantiate_from_known;
use residuum::typed::{Closure, Declaration, DerivationTime, Event, Module, Rule, Term, VerbKind};
use residuum::{DerivationClock, close_program, load_formal_module};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const FIXTURES: &str = "docs/legal-geometry/semantic-domain/fixtures";
const ONTOLOGY: &str = "examples/experiments/legal_geometry/ontology_reference/legal_general.res";
const VOCABULARY: &str = "SharedSpacesVocabulary";
const POSITIONS: [&str; 3] = ["prohibition", "permission", "obligation"];
const HYPOTHESES: [&str; 7] = [
    "hThing",
    "hThingNoCert",
    "hThirdParty",
    "hSharedTrait",
    "hBeneficiaryTrait",
    "hCustodyEnter",
    "hCustody",
];
const SITUATIONS: [&str; 9] = [
    "s0",
    "sCert",
    "sNoSens",
    "sNotAnimal",
    "sRoleSubject",
    "sRoleBeneficiary",
    "sConduct",
    "sMinor",
    "sNeutral",
];

fn path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn load(relative: &str) -> Module {
    load_formal_module(path(&format!("{FIXTURES}/{relative}"))).unwrap()
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(
            self.0 < 10_000,
            "finite domain fixture unexpectedly expands"
        );
        DerivationTime {
            name: format!("domain-{}", self.0),
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

fn rule<'a>(module: &'a Module, name: &str) -> &'a Rule {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == name => Some(r),
            _ => None,
        })
        .unwrap_or_else(|| panic!("rule {name}"))
}

fn constant(term: &Term) -> &str {
    match term {
        Term::Const(name) => name,
        other => panic!("ground constant expected, found {other:?}"),
    }
}

fn time(event: &Event) -> String {
    format!("{:?}", event.proposition.time)
}

/// The atom of a seed without its polarity: verb and ground arguments.
fn atom(event: &Event) -> (String, Vec<String>) {
    (
        event.verb.clone(),
        event.args.iter().map(|a| constant(a).to_string()).collect(),
    )
}

/// (subject, conduct, object, beneficiary) of the unique anchor.
fn anchor(seeds: &[Event]) -> [String; 4] {
    let anchors: Vec<_> = seeds.iter().filter(|e| e.verb == "considered").collect();
    assert_eq!(anchors.len(), 1, "exactly one anchor");
    let a = &anchors[0].args;
    [0, 1, 2, 3].map(|i| constant(&a[i]).to_string())
}

/// Admissibility K1-K5 of a situation. K1 (typing) holds because the
/// experiment elaborated; `vocabulary` lists the Seeded verbs of the
/// descriptive vocabulary.
fn admissible(seeds: &[Event], vocabulary: &BTreeSet<String>) -> Result<(), String> {
    let [s, _, _, b] = anchor(seeds);
    if s == b {
        return Err("K2: subject and beneficiary coincide".into());
    }
    let mut states = BTreeMap::new();
    for e in seeds {
        if !vocabulary.contains(&e.verb) {
            return Err(format!("K4: `{}` is outside the vocabulary", e.verb));
        }
        if time(e) != time(&seeds[0]) {
            return Err("K5: more than one proposition index".into());
        }
        if let Some(previous) = states.insert(atom(e), e.polarity.clone()) {
            if previous != e.polarity {
                return Err(format!("K3: {:?} with both polarities", atom(e)));
            }
        }
    }
    Ok(())
}

/// One admissible move between two situations: the anchor changes in exactly
/// one field (conduct, subject or beneficiary), or exactly one atom changes
/// state among positive, evidential negative and unknown.
fn one_move(from: &[Event], to: &[Event]) -> bool {
    let state = |seeds: &[Event]| -> BTreeMap<_, _> {
        seeds
            .iter()
            .filter(|e| e.verb != "considered")
            .map(|e| (atom(e), e.polarity.clone()))
            .collect()
    };
    let (a, b) = (anchor(from), anchor(to));
    let anchor_changes = (0..4).filter(|&i| a[i] != b[i]).count();
    let (x, y) = (state(from), state(to));
    let keys: BTreeSet<_> = x.keys().chain(y.keys()).collect();
    let atom_changes = keys.iter().filter(|k| x.get(*k) != y.get(*k)).count();
    anchor_changes + atom_changes == 1
}

fn positions(events: impl IntoIterator<Item = Event>) -> Vec<Event> {
    events
        .into_iter()
        .filter(|e| POSITIONS.contains(&e.verb.as_str()))
        .collect()
}

/// o_Q: a position without id, authority and effective, which the ablation
/// keeps constant by construction.
fn project(e: &Event) -> (String, Vec<String>, String) {
    (
        e.verb.clone(),
        e.args[1..5]
            .iter()
            .map(|a| constant(a).to_string())
            .collect(),
        time(e),
    )
}

/// Pairs of distinct positions that the projection would identify.
fn conflations(events: &[Event]) -> Vec<(Event, Event)> {
    let mut seen: BTreeMap<_, &Event> = BTreeMap::new();
    let mut conflated = Vec::new();
    for e in events {
        match seen.get(&project(e)) {
            Some(other) if *other != e => conflated.push(((*other).clone(), e.clone())),
            _ => {
                seen.insert(project(e), e);
            }
        }
    }
    conflated
}

/// The kinds of position derived about the conduct under examination.
fn observation(closure: &Closure, seeds: &[Event]) -> BTreeSet<String> {
    let anchor = anchor(seeds);
    positions(closure.known.iter().cloned())
        .iter()
        .filter(|e| (1..5).all(|i| constant(&e.args[i]) == anchor[i - 1]))
        .map(|e| e.verb.clone())
        .collect()
}

fn run(module: &Module, seeds: &[Event]) -> Closure {
    close_program(seeds.to_vec(), module, &mut Clock::default()).unwrap()
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

#[test]
fn vocabulary_is_a_descriptive_signature_over_unchanged_legal_general() {
    let vocabulary = load("vocabulary_natural.res");
    let ontology = load_formal_module(path(ONTOLOGY)).unwrap();
    let (mut entities, mut constants, mut verbs) = (0, 0, 0);
    for d in &vocabulary.declarations {
        let origin = vocabulary.origin(d.name()).unwrap().module.clone();
        if origin != VOCABULARY {
            assert_eq!(origin, "LegalGeneral", "{}", d.name());
            assert!(ontology.declarations.contains(d), "{} altered", d.name());
            continue;
        }
        match d {
            Declaration::Entity(_) => entities += 1,
            Declaration::Const(_) => constants += 1,
            Declaration::Verb(v) => {
                assert_eq!(v.kind, VerbKind::Seeded, "{} must be Seeded", v.name);
                assert!(v.effect.is_none() && v.program.is_none(), "{}", v.name);
                verbs += 1;
            }
            other => panic!(
                "vocabulary declares {}: only a signature is admitted",
                other.name()
            ),
        }
    }
    assert_eq!((entities, constants, verbs), (3, 23, 6));
    assert_eq!(
        vocabulary.declarations.len() - entities - constants - verbs,
        ontology.declarations.len()
    );
    // Mechanical part of the lexical review; the glosses need a human review.
    let forbidden = [
        "allow", "permit", "forbid", "prohib", "oblig", "duty", "must", "danger", "risk", "harm",
        "legit", "reason", "necess", "except", "nuisance", "justif", "proper",
    ];
    for d in &vocabulary.declarations {
        if vocabulary.origin(d.name()).unwrap().module == VOCABULARY {
            let name = d.name().to_lowercase();
            assert!(
                forbidden.iter().all(|f| !name.contains(f)),
                "`{}` carries a normative or evaluative word",
                d.name()
            );
        }
    }
}

#[test]
fn projection_drops_only_arguments_constant_by_construction() {
    let witnesses = load("witnesses.res");
    let expected = ["hypNorm", "administrative", "hypEnacted"];
    let mut checked = 0;
    for h in HYPOTHESES {
        let module = load(&format!("hypotheses/{h}.res"));
        for s in SITUATIONS {
            let closure = run(&module, &seeds(&witnesses, s));
            let found = positions(closure.known.iter().cloned());
            for e in &found {
                let dropped = [0, 5, 6].map(|i| constant(&e.args[i]).to_string());
                assert_eq!(dropped, expected, "{h} on {s}");
                checked += 1;
            }
            assert!(conflations(&found).is_empty(), "{h} on {s}");
        }
    }
    assert!(checked > 0);
    // Counterexample: positions that differ only in authority are distinct;
    // the projection would identify them, and the check detects it.
    let conflation = load("conflation.res");
    let found = positions(run(&conflation, &seeds(&witnesses, "s0")).known);
    assert_eq!(found.len(), 2);
    let pairs = conflations(&found);
    assert_eq!(pairs.len(), 1);
    assert_ne!(constant(&pairs[0].0.args[5]), constant(&pairs[0].1.args[5]));
}

#[test]
fn initial_cardinalities_support_relational_contrasts_and_distinct_coverages() {
    let witnesses = load("witnesses.res");
    let vocabulary = vocabulary_verbs(&witnesses);
    let situations: BTreeMap<_, _> = SITUATIONS
        .iter()
        .map(|s| (*s, seeds(&witnesses, s)))
        .collect();
    for (name, s) in &situations {
        admissible(s, &vocabulary).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(*name == "s0" || one_move(&situations["s0"], s), "{name}");
    }
    // The base theory derives no position on the domain (K4).
    let cast = load("cast.res");
    for s in situations.values() {
        assert!(positions(run(&cast, s).known).is_empty());
    }

    let mut observed: BTreeMap<&str, BTreeMap<&str, BTreeSet<String>>> = BTreeMap::new();
    for h in HYPOTHESES {
        let module = load(&format!("hypotheses/{h}.res"));
        for (name, s) in &situations {
            let closure = run(&module, s);
            // Stratified evaluation (contract §8) on the witnesses: the
            // projected positions of the closure are the projected heads of
            // the hypothesis instantiated on the seeds.
            let closed: BTreeSet<_> = positions(closure.known.iter().cloned())
                .iter()
                .map(project)
                .collect();
            let direct: BTreeSet<_> = positions(
                instantiate_from_known(rule(&module, h), s)
                    .into_iter()
                    .map(|d| d.event),
            )
            .iter()
            .map(project)
            .collect();
            assert_eq!(closed, direct, "{h} on {name}");
            observed
                .entry(h)
                .or_default()
                .insert(name, observation(&closure, s));
        }
    }
    let fires = |h: &str, s: &str| !observed[h][s].is_empty();
    println!("hypothesis \\ situation: {}", SITUATIONS.join(" "));
    for h in HYPOTHESES {
        let row: Vec<_> = SITUATIONS
            .iter()
            .map(|s| if fires(h, s) { "1" } else { "0" })
            .collect();
        println!("{h}: {}", row.join(" "));
    }
    let expected: BTreeMap<&str, [u8; 9]> = BTreeMap::from([
        ("hThing", [1, 1, 1, 0, 1, 1, 1, 1, 1]),
        ("hThingNoCert", [1, 0, 1, 0, 1, 1, 1, 1, 1]),
        ("hThirdParty", [1, 1, 0, 1, 1, 0, 1, 1, 1]),
        ("hSharedTrait", [0, 0, 0, 0, 0, 1, 0, 0, 0]),
        ("hBeneficiaryTrait", [0, 0, 0, 0, 0, 0, 0, 1, 0]),
        ("hCustodyEnter", [1, 1, 1, 1, 0, 1, 0, 1, 1]),
        ("hCustody", [1, 1, 1, 1, 0, 1, 1, 1, 1]),
    ]);
    for h in HYPOTHESES {
        for (i, s) in SITUATIONS.iter().enumerate() {
            assert_eq!(fires(h, s), expected[h][i] == 1, "{h} on {s}");
        }
    }

    // Coverage over the witnesses: (situation, kind) pairs.
    let cov = |h: &str| -> BTreeSet<(String, String)> {
        observed[h]
            .iter()
            .flat_map(|(s, kinds)| kinds.iter().map(move |k| (s.to_string(), k.clone())))
            .collect()
    };
    let strict = |a: &str, b: &str| cov(a).is_subset(&cov(b)) && cov(a) != cov(b);
    let incomparable = |a: &str, b: &str| !cov(a).is_subset(&cov(b)) && !cov(b).is_subset(&cov(a));
    assert!(strict("hThingNoCert", "hThing"));
    assert!(strict("hCustodyEnter", "hCustody"));
    assert!(incomparable("hThing", "hThirdParty"));
    assert!(incomparable("hThirdParty", "hSharedTrait"));
    assert!(incomparable("hThing", "hBeneficiaryTrait"));
    let signatures: BTreeSet<_> = HYPOTHESES
        .iter()
        .map(|h| SITUATIONS.map(|s| observed[h][s].clone()))
        .collect();
    assert_eq!(
        signatures.len(),
        HYPOTHESES.len(),
        "the witnesses separate every hypothesis"
    );

    // Contrasts from s0: the hypotheses whose treatment of the conduct changes.
    let changed = |s: &str| -> BTreeSet<&str> {
        HYPOTHESES
            .iter()
            .copied()
            .filter(|h| observed[h][s] != observed[h]["s0"])
            .collect()
    };
    let expected_changes: [(&str, &[&str]); 8] = [
        ("sCert", &["hThingNoCert"]),
        ("sNoSens", &["hThirdParty"]),
        ("sNotAnimal", &["hThing", "hThingNoCert"]),
        ("sRoleSubject", &["hCustodyEnter", "hCustody"]),
        ("sRoleBeneficiary", &["hThirdParty", "hSharedTrait"]),
        ("sConduct", &["hCustodyEnter"]),
        ("sMinor", &["hBeneficiaryTrait"]),
        ("sNeutral", &[]),
    ];
    for (s, hs) in expected_changes {
        assert_eq!(changed(s), hs.iter().copied().collect(), "{s}");
    }
    // Discriminating: hThing and hThingNoCert agree on s0 and differ on sCert.
    assert_eq!(observed["hThing"]["s0"], observed["hThingNoCert"]["s0"]);
    assert_ne!(
        observed["hThing"]["sCert"],
        observed["hThingNoCert"]["sCert"]
    );
    // Non-discriminating for that pair (sNoSens) and for every witness (sNeutral).
    assert!(!changed("sNoSens").contains("hThing") && !changed("sNoSens").contains("hThingNoCert"));
    assert!(changed("sNeutral").is_empty());
    // Role changes: a third party becomes beneficiary while another person
    // remains a third party, and the subject changes while the former subject
    // stays in the situation.
    let roles = |s: &str| anchor(&situations[s]);
    assert_eq!(
        (roles("s0")[0].as_str(), roles("s0")[3].as_str()),
        ("ana", "bea")
    );
    assert_eq!(roles("sRoleBeneficiary")[3], "cal");
    assert_eq!(roles("sRoleSubject")[0], "dan");
    // Changes driven by participants other than the subject, through moves
    // that leave every atom about the subject untouched.
    let subject = roles("s0")[0].clone();
    let about_subject = |s: &str| -> BTreeSet<_> {
        situations[s]
            .iter()
            .filter(|e| e.verb != "considered" && e.args.iter().any(|a| constant(a) == subject))
            .map(|e| (atom(e), e.polarity.clone()))
            .collect()
    };
    for (s, h) in [
        ("sNoSens", "hThirdParty"),
        ("sMinor", "hBeneficiaryTrait"),
        ("sRoleBeneficiary", "hSharedTrait"),
    ] {
        assert_eq!(roles(s)[0], subject, "{s}");
        assert_eq!(about_subject(s), about_subject("s0"), "{s}");
        assert!(changed(s).contains(h), "{s}");
    }
}

#[test]
fn act_content_is_recorded_without_normative_effects_and_kept_apart_from_the_act() {
    const EXPRESSIVITY: &str = "docs/legal-geometry/semantic-domain/expressivity";
    let content = load_formal_module(path(&format!("{EXPRESSIVITY}/content.res"))).unwrap();
    let readers = load_formal_module(path(&format!("{EXPRESSIVITY}/readers.res"))).unwrap();
    let experiments = [
        "relationalContent",
        "actWithContent",
        "actWithoutContent",
        "contentWithoutAct",
    ];
    for name in experiments {
        let s = seeds(&content, name);
        // LegalGeneral reads neither constitutes nor certifies, and the Act of
        // certifying alone derives nothing: the closure equals the seeds.
        let closure = run(&content, &s);
        let known: BTreeSet<_> = closure
            .known
            .iter()
            .map(residuum::experiments::render_event)
            .collect();
        let seeded: BTreeSet<_> = s.iter().map(residuum::experiments::render_event).collect();
        assert_eq!(known, seeded, "{name}");
        assert!(closure.derivations.is_empty(), "{name}");
    }
    let derived = |name: &str, verb: &str| -> Vec<String> {
        run(&readers, &seeds(&content, name))
            .known
            .iter()
            .filter(|e| e.verb == verb)
            .map(residuum::experiments::render_event)
            .collect()
    };
    // Act and content are distinguishable: only their conjunction is read.
    assert_eq!(derived("actWithContent", "certifiedContent").len(), 1);
    assert!(derived("actWithoutContent", "certifiedContent").is_empty());
    assert!(derived("contentWithoutAct", "certifiedContent").is_empty());
    // Variables bind inside the Relation record, including the optional thing.
    let relational = derived("relationalContent", "relationalDebtor");
    assert!(
        relational
            .iter()
            .any(|e| e.contains("service, ana, obj, bea"))
    );
    // Observed typing gap, documented and not relied upon: the variable bound
    // to the optional field carries `none` into the head argument declared
    // `Thing` (not optional). Elaboration accepts the rule; matching does not
    // check the declared type of the head position.
    assert!(
        relational
            .iter()
            .any(|e| e.contains("appointment, cal, none, bea"))
    );
}
