//! Fixtures provide all semantics and inputs. Rust only observes and compares.
use super::*;
use std::path::Path;

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("profile-{}", self.0),
        }
    }
}
fn load(file: &str) -> Module {
    crate::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments")
            .join(file),
    )
    .unwrap()
}
fn input(module: &Module, name: &str) -> Vec<Event> {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e.input.clone()),
            _ => None,
        })
        .unwrap()
}
fn product(name: &str) -> (Module, Vec<Event>) {
    let module = load("peano_polynomial_product_scenario.res");
    let seeds = input(&module, name);
    (module, seeds)
}

#[test]
fn profiling_preserves_full_closure_and_stamps() {
    let (module, seeds) = product("unitProduct");
    let original = {
        let _scan = profile::linear_scan();
        close_program(seeds.clone(), &module, &mut Clock::default()).unwrap()
    };
    let session = profile::Session::new("unitProduct");
    let measured = close_program(seeds, &module, &mut Clock::default()).unwrap();
    let stats = session.finish();
    // Debug includes proposition types, transitions, witness order, substitutions,
    // clocks and multiplicity, which logical Event equality deliberately ignores.
    assert_eq!(format!("{original:?}"), format!("{measured:?}"));
    assert!(stats.probes > 0 && stats.duplicates > 0);
}

#[test]
fn profile_polynomial_product_fixture() {
    // Normal verification always uses the completed small fixture. The larger
    // diagnostic is explicitly opt-in and still uses the runner's same timeout.
    let name =
        std::env::var("RESIDUUM_PROFILE_PRODUCT").unwrap_or_else(|_| "partialProduct".into());
    assert!(["partialProduct", "principalProduct"].contains(&name.as_str()));
    let (module, seeds) = product(&name);
    let label = if name == "partialProduct" {
        "partialProduct"
    } else {
        "principalProduct"
    };
    let session = profile::Session::new(label);
    let closure = close_program(seeds, &module, &mut Clock::default()).unwrap();
    let stats = session.finish();
    if name == "partialProduct" {
        assert_eq!((closure.known.len(), closure.derivations.len()), (223, 244));
        assert!(stats.associated_evaluations > stats.activated);
    }
    let goal = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(&e.goal),
            _ => None,
        })
        .unwrap();
    assert!(closure.known.contains(goal));
    if name == "principalProduct" {
        fn natural(term: &Term) -> usize {
            fn count(time: TimeExpr) -> usize {
                match time {
                    TimeExpr::At(name) if name == "zeroTime" => 0,
                    TimeExpr::After(inner) => 1 + count(*inner),
                    _ => panic!("not a ground Peano natural"),
                }
            }
            count(TimeExpr::from_term(term).unwrap())
        }
        let output = closure
            .known
            .iter()
            .filter(|e| e.verb == "coefficient" && e.args[0] == Term::Const("r".into()))
            .map(|e| (natural(&e.args[1]), natural(&e.args[2])))
            .collect::<Vec<_>>();
        assert_eq!(output.len(), 7, "exactly one result per output exponent");
        assert_eq!(
            output.into_iter().collect::<BTreeMap<_, _>>(),
            [5, 10, 18, 26, 9, 12, 0].into_iter().enumerate().collect()
        );
        // The expected result is only an assertion; no coefficient of r was seeded.
        assert!(
            input(&module, "principalProduct")
                .iter()
                .all(|e| !(e.verb == "inputCoefficient" && e.args[0] == Term::Const("r".into())))
        );
    }
}

fn semantic_proofs(closure: &Closure) -> BTreeMap<Event, Vec<String>> {
    let mut proofs: BTreeMap<Event, Vec<String>> = BTreeMap::new();
    for d in &closure.derivations {
        proofs
            .entry(d.event.clone())
            .or_default()
            .push(format!("{:?}", d.record));
    }
    for p in proofs.values_mut() {
        p.sort();
    }
    proofs
}

#[test]
fn profile_fixed_rules_incremental_preserves_events_and_provenance() {
    let module = load("forward_incremental.res");
    let base = input(&module, "base");
    let delta = input(&module, "delta");
    let rules = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut incremental = IncrementalClosure::new(base.clone(), &rules, &mut Clock::default());
    let session = profile::Session::new("fixed-rules incremental update");
    let change = incremental.update(delta.clone(), &mut Clock::default());
    let update = session.finish();
    let session = profile::Session::new("fixed-rules full recomputation");
    let recomputed = close(
        base.into_iter().chain(delta.clone()).collect(),
        &rules,
        &mut Clock::default(),
    );
    let full = session.finish();
    assert_eq!(
        incremental.closure().known.iter().collect::<BTreeSet<_>>(),
        recomputed.known.iter().collect()
    );
    assert_eq!(
        semantic_proofs(incremental.closure()),
        semantic_proofs(&recomputed)
    );
    assert_eq!(change.derivations.len(), 2);
    println!(
        "INCREMENTAL COMPARISON update_probes={} recompute_probes={} update_evaluations={} recompute_evaluations={} update_candidates={} recompute_candidates={}",
        update.probes,
        full.probes,
        update.evaluations,
        full.evaluations,
        update.candidates,
        full.candidates
    );
    // Observation, not a general speedup law. A duplicate delta must do no work.
    let session = profile::Session::new("fixed-rules duplicate update");
    let duplicate = incremental.update(delta, &mut Clock::default());
    let no_work = session.finish();
    assert!(duplicate.known.is_empty() && duplicate.derivations.is_empty());
    assert_eq!((no_work.probes, no_work.evaluations), (0, 0));
}

#[test]
fn indexed_known_matches_linear_product_with_identical_provenance() {
    let (module, seeds) = product("partialProduct");
    let (original, linear) = {
        let _scan = profile::linear_scan();
        let session = profile::Session::new("partialProduct linear");
        let closure = close_program(seeds.clone(), &module, &mut Clock::default()).unwrap();
        (closure, session.finish())
    };
    let session = profile::Session::new("partialProduct indexed");
    let indexed = close_program(seeds, &module, &mut Clock::default()).unwrap();
    let stats = session.finish();
    assert_eq!(format!("{original:?}"), format!("{indexed:?}"));
    assert_eq!(linear.probes, 2_117_296);
    assert_eq!(linear.header_rejects, 2_023_990);
    assert_eq!(stats.probes, linear.probes - linear.header_rejects);
    assert_eq!(stats.header_rejects, 0);
    assert_eq!(stats.structural_rejects, linear.structural_rejects);
    assert_eq!(
        (
            stats.rounds,
            stats.evaluations,
            stats.activated,
            stats.candidates,
            stats.duplicates,
            stats.comparisons
        ),
        (
            linear.rounds,
            linear.evaluations,
            linear.activated,
            linear.candidates,
            linear.duplicates,
            linear.comparisons
        )
    );
}

#[test]
fn indexed_known_preserves_negative_cycles_unit_and_new_justifications() {
    let module = load("discrete_derivative.res");
    let rules = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let events = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == "universe" => Some(e.input.clone()),
            _ => None,
        })
        .unwrap();
    // Every finite seed subset, including empty evidence, both polarities,
    // duplicate seeds, Unit, a cycle and alternative witnesses for old events.
    for mask in 0..(1 << events.len()) {
        let mut seeds = events
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect::<Vec<_>>();
        if let Some(first) = seeds.first().cloned() {
            seeds.push(first);
        }
        let original = {
            let _scan = profile::linear_scan();
            close(seeds.clone(), &rules, &mut Clock::default())
        };
        let indexed = close(seeds, &rules, &mut Clock::default());
        assert_eq!(
            format!("{original:?}"),
            format!("{indexed:?}"),
            "seed mask {mask}"
        );
    }
}

#[test]
fn known_header_index_preserves_order_identity_and_all_three_keys() {
    let (module, mut events) = product("unitProduct");
    // Index a mixed-arity typed fixture, both explicit polarities and a duplicate
    // representation. The index itself must preserve the supplied Vec exactly.
    let mut negative = events[0].clone();
    negative.polarity = Polarity::EvidentialNot;
    events.insert(1, negative);
    events.push(events[0].clone());
    let index = KnownIndex::new(&events);
    for pattern in &events {
        let expected = events
            .iter()
            .filter(|e| {
                e.verb == pattern.verb
                    && e.polarity == pattern.polarity
                    && e.args.len() == pattern.args.len()
            })
            .collect::<Vec<_>>();
        let actual = KnownView {
            events: &events,
            index: Some(&index),
        }
        .candidates(pattern)
        .collect::<Vec<_>>();
        assert_eq!(actual.len(), expected.len());
        assert!(
            actual
                .iter()
                .zip(expected)
                .all(|(a, b)| std::ptr::eq(*a, b))
        );
    }
    let mut missing = events[0].clone();
    missing.verb = "absentFromKnown".into();
    assert_eq!(
        KnownView {
            events: &events,
            index: Some(&index)
        }
        .candidates(&missing)
        .count(),
        0
    );
    // The same header is indexed immediately when a new fact is appended.
    let next = input(&module, "unitProduct").pop().unwrap();
    let mut updated = index;
    updated.insert(events.len(), &next);
    events.push(next.clone());
    assert!(std::ptr::eq(
        KnownView {
            events: &events,
            index: Some(&updated)
        }
        .candidates(&next)
        .last()
        .unwrap(),
        events.last().unwrap()
    ));
}
