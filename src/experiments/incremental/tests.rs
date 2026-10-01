use super::*;
use crate::typed::{DerivationTime, Rule};
use crate::{DerivationClock, parse_formal, parse_formal_to_typed};

const FORWARD: &str = include_str!("../../../examples/experiments/forward_incremental.res");
const PROVENANCE: &str = include_str!("../../../examples/experiments/forward_provenance.res");

fn inputs(source: &str) -> (Vec<Rule>, Vec<Event>) {
    let module = parse_formal_to_typed(source).unwrap();
    let rules = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(rule) => Some(rule.clone()),
            _ => None,
        })
        .collect();
    let events = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) => Some(e.input.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    (rules, events)
}

fn subset(events: &[Event], mask: usize) -> Vec<Event> {
    events
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, e)| e.clone())
        .collect()
}

fn assert_oracle(state: &IncrementalClosure, seeds: &[Event], rules: &[Rule]) {
    let full = close(seeds.to_vec(), rules, &mut Clock::default());
    assert_eq!(keys(state.closure()), keys(&full));
    assert!(same_justifications(state.closure(), &full));
    let distinct: Vec<_> = state
        .closure()
        .derivations
        .iter()
        .map(|d| (&d.event, &d.record))
        .collect();
    for (i, justification) in distinct.iter().enumerate() {
        assert!(
            !distinct[..i].contains(justification),
            "one stamp per justification"
        );
    }
}

#[test]
fn source_forward_incremental_has_equal_logical_and_provenance_results() {
    for source in [FORWARD, PROVENANCE] {
        assert_eq!(
            parse_formal(source).unwrap(),
            crate::formal::parse_formal_manual(source).unwrap()
        );
    }
    let module = parse_formal_to_typed(FORWARD).unwrap();
    let before = module.clone();
    let result = run_incremental_experiment(&module).unwrap();
    assert_eq!(module, before);
    assert!(result.logical_equal && result.provenance_equal);
    assert_eq!(result.base.known.len(), 1);
    assert_eq!(result.incremental.known.len(), 4);
    assert_eq!(
        result
            .delta
            .known
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        ["agreesObject", "saleEstablished", "deliverable"]
    );
    assert_eq!(result.delta.derivations.len(), 2);
    assert!(!result.base.known.contains(&result.goal));
    assert!(result.incremental.known.contains(&result.goal));
    assert!(render_incremental(&result).contains("logical equality: true"));
}

#[test]
fn all_seed_subsets_and_update_sequences_equal_full_close() {
    let (rules, events) = inputs(FORWARD);
    assert_eq!(events.len(), 2);
    // All 16 base/addition pairs, then every possible second update (64 paths).
    for base_mask in 0..4 {
        for first_mask in 0..4 {
            for second_mask in 0..4 {
                let mut seeds = subset(&events, base_mask);
                let mut clock = Clock::default();
                let mut state = IncrementalClosure::new(seeds.clone(), &rules, &mut clock);
                assert_oracle(&state, &seeds, &rules);
                for mask in [first_mask, second_mask] {
                    let before = state.closure().clone();
                    let additions = subset(&events, mask);
                    seeds.extend(additions.clone());
                    let delta = state.update(additions, &mut clock);
                    let expected = keys(state.closure())
                        .difference(&keys(&before))
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    assert_eq!(
                        delta.known.iter().cloned().collect::<BTreeSet<_>>(),
                        expected
                    );
                    assert_eq!(delta.known.len(), expected.len());
                    assert!(
                        before
                            .derivations
                            .iter()
                            .all(|d| state.closure().derivations.contains(d)),
                        "old provenance and stamps remain intact"
                    );
                    assert_oracle(&state, &seeds, &rules);
                }
            }
        }
    }
}

#[test]
fn empty_and_duplicate_updates_do_no_work_or_stamping() {
    let (rules, events) = inputs(FORWARD);
    let mut clock = Clock::default();
    let mut state = IncrementalClosure::new(events.clone(), &rules, &mut clock);
    let before = state.closure().clone();
    let stamps = clock.0;
    for additions in [
        Vec::new(),
        events.clone(),
        [events.clone(), events].concat(),
    ] {
        assert_eq!(state.update(additions, &mut clock), ClosureDelta::default());
        assert_eq!(state.closure(), &before);
        assert_eq!(clock.0, stamps);
    }
}

#[test]
fn new_justification_for_old_event_and_unit_axiom_are_preserved() {
    let module = parse_formal_to_typed(PROVENANCE).unwrap();
    let result = run_incremental_experiment(&module).unwrap();
    assert_eq!(result.delta.known.len(), 1);
    assert_eq!(result.delta.known[0].verb, "B");
    assert_eq!(result.delta.derivations.len(), 1);
    assert_eq!(result.delta.derivations[0].record.rule, "fromB");
    assert_eq!(result.delta.derivations[0].event.verb, "shared");
    assert_eq!(result.base.derivations.len(), 3);
    assert_eq!(result.incremental.derivations.len(), 4);
    assert!(
        result
            .base
            .derivations
            .iter()
            .all(|d| result.incremental.derivations.contains(d))
    );
    let axiom: Vec<_> = result
        .incremental
        .derivations
        .iter()
        .filter(|d| d.record.rule == "axiom")
        .collect();
    assert_eq!(axiom.len(), 1);
    assert!(axiom[0].record.antecedents.is_empty());
    assert!(result.logical_equal && result.provenance_equal);
}

#[test]
fn delta_matches_share_objects_and_times_and_preserve_witness_order() {
    for replacement in [
        "agreesObject(bob, house, alice) @ tau,",
        "agreesObject(bob, car, alice) @ sigma,",
    ] {
        let source = FORWARD.replace("agreesObject(bob, car, alice) @ tau,", replacement);
        let source = format!("{source}\nconst house : Thing\nconst sigma : PropositionTime\n");
        let result = run_incremental_experiment(&parse_formal_to_typed(&source).unwrap()).unwrap();
        assert_eq!(result.delta.known.len(), 1);
        assert!(result.delta.derivations.is_empty());
        assert!(!result.incremental.known.contains(&result.goal));
    }
    for source in [
        FORWARD.to_owned(),
        FORWARD
            .replace("    agreesPrice", "    I & agreesPrice")
            .replace(
                "    & agreesObject(creditor, object, debtor) @ t",
                "    & agreesObject(creditor, object, debtor) @ t & I",
            ),
    ] {
        let result = run_incremental_experiment(&parse_formal_to_typed(&source).unwrap()).unwrap();
        let sale = &result.delta.derivations[0];
        assert_eq!(
            sale.record
                .antecedents
                .iter()
                .map(|e| e.verb.as_str())
                .collect::<Vec<_>>(),
            ["agreesPrice", "agreesObject"]
        );
    }
}

#[test]
fn finite_cycles_and_evidential_not_equal_full_close() {
    // Finite same-time cycles are tested in the library; the external mode
    // deliberately rejects all cycles as a conservative termination guard.
    let source = PROVENANCE.replace("rule fromB(t) = B() @ t <= shared() @ t", "rule fromB(t) = ~B() @ t <= shared() @ t\nrule cycle(t) = result() @ t <= shared() @ t")
        .replace("seeds { B() @ tau, }", "seeds { ~B() @ tau, }");
    let (rules, events) = inputs(&source);
    let mut clock = Clock::default();
    let mut state = IncrementalClosure::new(Vec::new(), &rules, &mut clock);
    let delta = state.update(events.clone(), &mut clock);
    assert_oracle(&state, &events, &rules);
    assert_eq!(delta.derivations.len(), 4);
    assert!(
        delta
            .known
            .iter()
            .any(|e| e.polarity == crate::typed::Polarity::EvidentialNot)
    );
    assert!(
        run_incremental_experiment(&parse_formal_to_typed(&source).unwrap())
            .unwrap_err()
            .to_string()
            .contains("acyclic")
    );
}

#[test]
fn incremental_mode_rejects_unsupported_inputs_before_closing() {
    for (source, message) in [
        (
            FORWARD.replace("experiment delta", "experiment other"),
            "missing delta",
        ),
        (
            FORWARD
                .replace(
                    "query residual deliverable(alice, car, bob)",
                    "query residual saleEstablished(alice, car, bob)",
                )
                .replacen(
                    "query residual saleEstablished",
                    "query residual deliverable",
                    1,
                ),
            "same goal",
        ),
        (
            format!("{FORWARD}\ndefault NormalSale = supernormal sales\n"),
            "defaults",
        ),
        (
            format!("{FORWARD}\neffect verb move(subject : Person, object : Thing)\n"),
            "effects",
        ),
        (
            FORWARD.replace(
                "    & agreesObject(creditor, object, debtor) @ t\n    <=",
                "    <= agreesObject(creditor, object, debtor) @ t -o",
            ),
            "Horn rules",
        ),
    ] {
        let error = run_incremental_experiment(&parse_formal_to_typed(&source).unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn derivation_clock_remains_external_to_incremental_rounds() {
    struct ExternalClock(usize);
    impl DerivationClock for ExternalClock {
        fn next_derivation_time(&mut self) -> DerivationTime {
            self.0 += 10;
            DerivationTime {
                name: format!("audit-{}", self.0),
            }
        }
    }
    let (rules, events) = inputs(FORWARD);
    let mut clock = ExternalClock(100);
    let mut state = IncrementalClosure::new(vec![events[0].clone()], &rules, &mut clock);
    let delta = state.update(vec![events[1].clone()], &mut clock);
    assert_eq!(
        delta
            .derivations
            .iter()
            .map(|d| d.derived_at.name.as_str())
            .collect::<Vec<_>>(),
        ["audit-110", "audit-120"]
    );
    assert!(
        delta
            .known
            .iter()
            .all(|e| e.proposition.time == events[0].proposition.time)
    );
}
