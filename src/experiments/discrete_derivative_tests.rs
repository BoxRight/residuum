//! Characterization of the Horn derivative; no production update API changes.
use super::*;
use crate::typed::UnstampedDerivation;
use crate::{IncrementalClosure, instantiate_from_known, parse_formal_to_typed};

type Known = BTreeSet<Event>;
const SOURCE: &str = include_str!("../../examples/experiments/discrete_derivative.res");

fn inputs() -> (Vec<Rule>, Vec<Event>) {
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let rules = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let events = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) => Some(e.input.clone()),
            _ => None,
        })
        .unwrap();
    (rules, events)
}

fn subset<T: Clone>(items: &[T], mask: usize) -> Vec<T> {
    items
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, item)| item.clone())
        .collect()
}

fn keys(closure: &Closure) -> Known {
    closure.known.iter().cloned().collect()
}

fn ground_clause(rule: &Rule) -> &crate::typed::HornClause {
    assert!(rule.parameters.is_empty());
    let RuleBody::HornClause(clause) = &rule.body else {
        panic!("this finite experiment uses ground Horn only")
    };
    clause
}

// Independent ground support evaluation: no matcher, close, FOUR valuation,
// or rule instantiation. EvidentialNot is simply a signed literal here.
fn supported(body: &Antecedent, known: &Known) -> bool {
    match body {
        Antecedent::Unit => true,
        Antecedent::Event(e) => known.contains(e),
        Antecedent::And(a, b) => supported(a, known) && supported(b, known),
    }
}

fn touches(body: &Antecedent, frontier: &Known) -> bool {
    match body {
        Antecedent::Unit => false,
        Antecedent::Event(e) => frontier.contains(e),
        Antecedent::And(a, b) => touches(a, frontier) || touches(b, frontier),
    }
}

fn consequences(rules: &[Rule], known: &Known) -> Known {
    rules
        .iter()
        .filter_map(|r| {
            let clause = ground_clause(r);
            supported(&clause.antecedent, known).then(|| clause.consequent.clone())
        })
        .collect()
}

// The derivative is a least fixed point over events OUTSIDE the closed base.
// This is not defined/computed by subtracting two calls to close.
fn derivative_fixpoint(base: &Known, additions: &Known, rules: &[Rule]) -> Known {
    let initial: Known = additions.difference(base).cloned().collect();
    let mut delta = Known::new();
    for _ in 0..=4 {
        let all = base.union(&delta).cloned().collect();
        let next = initial
            .union(&consequences(rules, &all))
            .filter(|e| !base.contains(*e))
            .cloned()
            .collect();
        if delta == next {
            return delta;
        }
        assert!(delta.is_subset(&next));
        delta = next;
    }
    panic!("a four-literal finite universe must stabilize")
}

// Differential evaluation: old-rule witnesses must touch the frontier;
// new rules are additionally evaluated on the whole base on the first round.
fn frontier_derivative(
    base: &Known,
    additions: &Known,
    old_rules: &[Rule],
    added_rules: &[Rule],
) -> Known {
    assert!(consequences(old_rules, base).is_subset(base));
    let mut delta: Known = additions.difference(base).cloned().collect();
    let mut frontier = delta.clone();
    for round in 0..=4 {
        let all: Known = base.union(&delta).cloned().collect();
        let mut next = Known::new();
        for (rule, is_new_rule) in old_rules
            .iter()
            .map(|r| (r, false))
            .chain(added_rules.iter().map(|r| (r, true)))
        {
            let clause = ground_clause(rule);
            if supported(&clause.antecedent, &all)
                && ((round == 0 && is_new_rule) || touches(&clause.antecedent, &frontier))
                && !all.contains(&clause.consequent)
            {
                next.insert(clause.consequent.clone());
            }
        }
        if next.is_empty() {
            return delta;
        }
        delta.extend(next.iter().cloned());
        frontier = next;
    }
    panic!("a four-literal finite universe must stabilize")
}

fn proofs(closure: &Closure) -> Vec<UnstampedDerivation> {
    closure
        .derivations
        .iter()
        .map(|d| UnstampedDerivation {
            event: d.event.clone(),
            record: d.record.clone(),
        })
        .collect()
}

#[test]
fn delta_fixpoint_and_frontier_characterize_existing_update_exhaustively() {
    let (candidates, atoms) = inputs();
    let mut cases = 0;
    for rule_mask in 0..1 << candidates.len() {
        let rules = subset(&candidates, rule_mask);
        // Independent leastness oracle: ALL support models of these rules.
        let closed_worlds: Vec<Known> = (0..16)
            .map(|mask| subset(&atoms, mask).into_iter().collect())
            .filter(|world| consequences(&rules, world).is_subset(world))
            .collect();
        for base_mask in 0..16 {
            let seeds = subset(&atoms, base_mask);
            let mut clock = Clock::default();
            let state = IncrementalClosure::new(seeds.clone(), &rules, &mut clock);
            let base = keys(state.closure());
            let base_proofs = proofs(state.closure());
            for addition_mask in 0..16 {
                let additions = subset(&atoms, addition_mask);
                let events: Known = additions.iter().cloned().collect();
                let expected = derivative_fixpoint(&base, &events, &rules);
                assert_eq!(expected, frontier_derivative(&base, &events, &rules, &[]));
                let mut updated = state.clone();
                let delta = updated.update(additions.clone(), &mut clock);
                assert_eq!(delta.known.iter().cloned().collect::<Known>(), expected);
                let union: Known = base.union(&expected).cloned().collect();
                assert_eq!(keys(updated.closure()), union);
                assert!(consequences(&rules, &union).is_subset(&union));
                for world in &closed_worlds {
                    if base.is_subset(world) && events.is_subset(world) {
                        assert!(
                            union.is_subset(world),
                            "least closed extension, not just a fixpoint"
                        );
                    }
                }
                // Ordinary close is only a separate validation oracle.
                let full = close(
                    [seeds.clone(), additions].concat(),
                    &rules,
                    &mut Clock::default(),
                );
                assert_eq!(keys(&full), union);
                let all_proofs = proofs(&full);
                let added_proofs: Vec<_> = delta
                    .derivations
                    .iter()
                    .map(|d| UnstampedDerivation {
                        event: d.event.clone(),
                        record: d.record.clone(),
                    })
                    .collect();
                let expected_proofs: Vec<_> = all_proofs
                    .iter()
                    .filter(|d| !base_proofs.contains(d))
                    .cloned()
                    .collect();
                assert_eq!(added_proofs.len(), expected_proofs.len());
                assert!(expected_proofs.iter().all(|d| added_proofs.contains(d)));
                assert!(
                    state
                        .closure()
                        .derivations
                        .iter()
                        .all(|d| { updated.closure().derivations.contains(d) })
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 8192);
    println!(
        "fixed R: {cases} updates; delta fixpoint, frontier, update, forward oracle and justification differences agree"
    );
}

#[test]
fn additive_rule_and_fact_changes_have_a_joint_delta_fixpoint() {
    let (candidates, atoms) = inputs();
    let mut cases = 0;
    // Every rule among the first four is absent, old, or newly added.
    for assignment in 0usize..81 {
        let mut code = assignment;
        let mut old = Vec::new();
        let mut added = Vec::new();
        for rule in &candidates[..4] {
            match code % 3 {
                1 => old.push(rule.clone()),
                2 => added.push(rule.clone()),
                _ => {}
            }
            code /= 3;
        }
        let all_rules = [old.clone(), added.clone()].concat();
        for base_mask in 0..16 {
            let seeds = subset(&atoms, base_mask);
            let base = keys(&close(seeds.clone(), &old, &mut Clock::default()));
            for addition_mask in 0..16 {
                let additions: Known = subset(&atoms, addition_mask).into_iter().collect();
                let expected = derivative_fixpoint(&base, &additions, &all_rules);
                assert_eq!(
                    expected,
                    frontier_derivative(&base, &additions, &old, &added)
                );
                let all_seeds: Known = seeds.iter().cloned().chain(additions).collect();
                let full = keys(&close(
                    all_seeds.into_iter().collect(),
                    &all_rules,
                    &mut Clock::default(),
                ));
                assert_eq!(base.union(&expected).cloned().collect::<Known>(), full);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 20736);
    println!(
        "additive (S,R): {cases} updates; new-rule activation on old facts and Unit agrees with the joint delta fixpoint; harness only"
    );
}

#[test]
fn derivative_composes_but_is_not_additive_or_a_deletion_operator() {
    let (rules, atoms) = inputs();
    let conjunction = vec![rules[3].clone()];
    let empty = Known::new();
    let p = Known::from([atoms[0].clone()]);
    let not_p = Known::from([atoms[1].clone()]);
    let both: Known = p.union(&not_p).cloned().collect();
    let d1 = derivative_fixpoint(&empty, &p, &conjunction);
    let d2 = derivative_fixpoint(&empty, &not_p, &conjunction);
    let combined = derivative_fixpoint(&empty, &both, &conjunction);
    assert_ne!(combined, d1.union(&d2).cloned().collect());
    assert!(combined.contains(&atoms[3]));
    let already_known: Known = [atoms[0].clone(), atoms[2].clone()].into_iter().collect();
    let forward = vec![rules[1].clone()];
    assert!(derivative_fixpoint(&already_known, &p, &forward).is_empty());
    assert!(!derivative_fixpoint(&empty, &p, &forward).is_empty());
    // Remove the sole seed P from a theory whose closure is {P,Q}.
    let after_deletion = keys(&close(vec![], &forward, &mut Clock::default()));
    assert!(after_deletion.is_empty());
    assert!(!already_known.is_subset(&after_deletion));

    let mut cases = 0;
    for selected in [&rules[..], &conjunction[..]] {
        for base_mask in 0..16 {
            let base = keys(&close(
                subset(&atoms, base_mask),
                selected,
                &mut Clock::default(),
            ));
            for first_mask in 0..16 {
                let first: Known = subset(&atoms, first_mask).into_iter().collect();
                let d_first = derivative_fixpoint(&base, &first, selected);
                let updated: Known = base.union(&d_first).cloned().collect();
                for second_mask in 0..16 {
                    let second: Known = subset(&atoms, second_mask).into_iter().collect();
                    let d_second = derivative_fixpoint(&updated, &second, selected);
                    let additions = first.union(&second).cloned().collect();
                    let together = derivative_fixpoint(&base, &additions, selected);
                    assert!(d_first.is_disjoint(&d_second));
                    assert_eq!(together, d_first.union(&d_second).cloned().collect());
                    if first.is_subset(&second) {
                        assert!(d_first.is_subset(&derivative_fixpoint(&base, &second, selected)));
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 8192);
    // Independent confirmation that ground forward proof records have no
    // pending substitutions, while preserving the witness event order.
    let matches = instantiate_from_known(&rules[3], &both.into_iter().collect::<Vec<_>>());
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].record.antecedents, atoms[..2]);
    println!(
        "{cases} two-step paths satisfy the cocycle law; conjunction disproves additivity, and deletion cannot be represented by positive union"
    );
}
