use super::*;
use crate::surface::{FormalDeclaration, SurfaceRuleBody};
use crate::typed::{DerivationRecord, ExperimentInputKind, Type};
use crate::{IncrementalClosure, elaborate_formal, instantiate, parse_formal};

const PROPERTY: &str = include_str!("../../examples/experiments/property.res");
const PROTECTION: &str = include_str!("../../examples/experiments/protection.res");
const INLINE: &str = include_str!("../../examples/experiments/protection_inline.res");

fn composed(consumer: &str) -> Module {
    elaborate_composed_fixture(
        parse_formal(PROPERTY).unwrap(),
        parse_formal(consumer).unwrap(),
    )
    .unwrap()
}

fn rules(module: &Module) -> Vec<Rule> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(rule) => Some(rule.clone()),
            _ => None,
        })
        .collect()
}

fn keys(closure: &Closure) -> BTreeSet<Event> {
    closure.known.iter().cloned().collect()
}

fn proofs(closure: &Closure) -> Vec<(Event, DerivationRecord)> {
    closure
        .derivations
        .iter()
        .map(|d| {
            let mut record = d.record.clone();
            if record.rule == "inlineProtection" {
                record.rule = "protection".into();
            }
            (d.event.clone(), record)
        })
        .collect()
}

fn equivalent(a: &Closure, b: &Closure) {
    assert_eq!(keys(a), keys(b));
    let (a, b) = (proofs(a), proofs(b));
    assert_eq!(a.len(), b.len());
    assert!(
        a.iter().all(|proof| b.contains(proof)),
        "only the inline/applied rule name is renamed; witnesses and substitutions stay exact"
    );
}

#[test]
fn property_lambda_is_referenced_from_consumer_and_beta_reduces_to_inline() {
    for source in [PROPERTY, PROTECTION, INLINE] {
        assert_eq!(
            parse_formal(source).unwrap(),
            crate::formal::parse_formal_manual(source).unwrap()
        );
    }
    let consumer = parse_formal(PROTECTION).unwrap();
    assert!(consumer.declarations.iter().any(|d| matches!(d, FormalDeclaration::Rule(r) if r.name == "protection" && matches!(&r.body, SurfaceRuleBody::Application { rule, args } if rule == "ownsProgram" && args == &["charlie", "molest"]))));
    assert!(
        consumer
            .declarations
            .iter()
            .all(|d| !matches!(d, FormalDeclaration::Rule(r) if r.name == "ownsProgram"))
    );
    assert!(
        elaborate_formal(consumer).is_err(),
        "composition supplies shared declarations; there are no automatic imports"
    );
    let a = composed(PROTECTION);
    let b = composed(INLINE);
    let applied = rules(&a)
        .into_iter()
        .find(|r| r.name == "protection")
        .unwrap();
    let mut inline = rules(&b)
        .into_iter()
        .find(|r| r.name == "inlineProtection")
        .unwrap();
    inline.name = applied.name.clone();
    assert_eq!(applied, inline);
    assert_eq!(
        applied
            .parameters
            .iter()
            .map(|p| p.ty.clone())
            .collect::<Vec<_>>(),
        [
            Type::Entity("Person".into()),
            Type::Entity("Thing".into()),
            Type::PropositionTime
        ]
    );
    let changed = PROPERTY.replace("@ after t", "@ t");
    let changed_module = elaborate_composed_fixture(
        parse_formal(&changed).unwrap(),
        parse_formal(PROTECTION).unwrap(),
    )
    .unwrap();
    let changed_rule = rules(&changed_module)
        .into_iter()
        .find(|r| r.name == "protection")
        .unwrap();
    assert_ne!(
        changed_rule.body, applied.body,
        "the consumer follows the library definition, not a cached inline copy"
    );
}

#[test]
fn applied_inline_and_staged_composition_have_isomorphic_provenance() {
    let a = composed(PROTECTION);
    let b = composed(INLINE);
    let before = (a.clone(), b.clone());
    let applied = run_incremental_experiment(&a).unwrap();
    let inline = run_incremental_experiment(&b).unwrap();
    assert_eq!((a.clone(), b), before);
    equivalent(&applied.base, &inline.base);
    equivalent(&applied.incremental, &inline.incremental);
    assert_eq!(applied.delta.known, inline.delta.known);
    assert_eq!(applied.incremental.known.len(), 3);
    assert_eq!(applied.incremental.derivations.len(), 2);
    let rules = rules(&a);
    let transfer = rules
        .iter()
        .find(|r| r.name == "transferToOwnership")
        .unwrap();
    let protection = rules.iter().find(|r| r.name == "protection").unwrap();
    let ownership = applied
        .incremental
        .derivations
        .iter()
        .find(|d| d.event.verb == "owns")
        .unwrap();
    let norm = instantiate(protection, &ownership.event).unwrap();
    assert_eq!(norm.event.verb, "notDo");
    assert_eq!(
        norm.event.args,
        [
            Term::Const("charlie".into()),
            Term::Const("molest".into()),
            Term::Const("car".into()),
            Term::Const("bob".into())
        ]
    );
    assert_eq!(
        norm.event.proposition.time,
        TimeExpr::After(Box::new(ownership.event.proposition.time.clone()))
    );
    let mut clock = Clock::default();
    let first = close(
        applied.additions,
        std::slice::from_ref(transfer),
        &mut clock,
    );
    let mut second = close(first.known, std::slice::from_ref(protection), &mut clock);
    second.derivations.extend(first.derivations);
    equivalent(&applied.incremental, &second);
}

#[test]
fn reusable_lambda_matches_inline_monotonicity_and_incremental_deltas() {
    let a = composed(PROTECTION);
    let b = composed(INLINE);
    let ra = rules(&a);
    let rb = rules(&b);
    let event = run_incremental_experiment(&a).unwrap().additions[0].clone();
    let another = composed(&PROTECTION.replace(
        "transfer(alice, car, bob) @ tau,",
        "transfer(alice, car, alice) @ tau,",
    ));
    let other = run_incremental_experiment(&another).unwrap().additions[0].clone();
    let basis = [event, other];
    let inputs: Vec<Vec<Event>> = (0..4)
        .map(|mask| {
            basis
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, event)| event.clone())
                .collect()
        })
        .collect();
    let ca: Vec<_> = inputs
        .iter()
        .map(|input| close(input.clone(), &ra, &mut Clock::default()))
        .collect();
    let cb: Vec<_> = inputs
        .iter()
        .map(|input| close(input.clone(), &rb, &mut Clock::default()))
        .collect();
    for i in 0..4 {
        equivalent(&ca[i], &cb[i]);
        for j in 0..4 {
            if i & j == i {
                assert!(keys(&ca[i]).is_subset(&keys(&ca[j])));
                assert!(keys(&cb[i]).is_subset(&keys(&cb[j])));
            }
            let mut clock_a = Clock::default();
            let mut clock_b = Clock::default();
            let mut state_a = IncrementalClosure::new(inputs[i].clone(), &ra, &mut clock_a);
            let mut state_b = IncrementalClosure::new(inputs[i].clone(), &rb, &mut clock_b);
            let old_a = state_a.closure().clone();
            let old_b = state_b.closure().clone();
            let da = state_a.update(inputs[j].clone(), &mut clock_a);
            let db = state_b.update(inputs[j].clone(), &mut clock_b);
            let mut union = inputs[i].clone();
            union.extend(inputs[j].clone());
            let full_a = close(union.clone(), &ra, &mut Clock::default());
            let full_b = close(union, &rb, &mut Clock::default());
            equivalent(state_a.closure(), &full_a);
            equivalent(state_b.closure(), &full_b);
            equivalent(state_a.closure(), state_b.closure());
            assert_eq!(
                da.known.iter().cloned().collect::<BTreeSet<_>>(),
                db.known.iter().cloned().collect()
            );
            assert_eq!(
                da.known.iter().cloned().collect::<BTreeSet<_>>(),
                keys(&full_a).difference(&keys(&old_a)).cloned().collect()
            );
            equivalent(
                &Closure {
                    known: da.known,
                    derivations: da.derivations,
                },
                &Closure {
                    known: db.known,
                    derivations: db.derivations,
                },
            );
            assert!(
                old_a
                    .derivations
                    .iter()
                    .all(|d| state_a.closure().derivations.contains(d))
            );
            assert!(
                old_b
                    .derivations
                    .iter()
                    .all(|d| state_b.closure().derivations.contains(d))
            );
        }
    }
}

#[test]
fn fixture_composition_keeps_library_data_separate() {
    let library_with_data = format!(
        "{PROPERTY}\nconst tau : PropositionTime\nexperiment libraryInput {{ premises {{}} query fourFusion owns(owner, object) @ tau }}"
    );
    let error = elaborate_composed_fixture(
        parse_formal(&library_with_data).unwrap(),
        parse_formal(PROTECTION).unwrap(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("must not contain experiment inputs")
    );
    let consumer = parse_formal(PROTECTION).unwrap();
    assert!(consumer.declarations.iter().any(|d| matches!(d, FormalDeclaration::Experiment(e) if e.input_kind == ExperimentInputKind::Seeds)));
}
