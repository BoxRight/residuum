use std::collections::BTreeSet;

use crate::typed::{
    Antecedent, Declaration, DerivationTime, Event, HornClause, Polarity, PropositionKind,
    PropositionType, Rule, RuleBody, StateTime, StateTransitionType, Term, TimeExpr,
};
use crate::{DerivationClock, close, parse_formal_to_typed, query_residual};

fn give() -> Event {
    let input = TimeExpr::At("tau".into());
    let output = TimeExpr::After(Box::new(input.clone()));
    Event {
        polarity: Polarity::Positive,
        verb: "give".into(),
        args: ["alice", "car", "bob"]
            .map(|name| Term::Var(name.into()))
            .to_vec(),
        proposition: PropositionType {
            kind: PropositionKind::Effect,
            time: output.clone(),
        },
        transition: Some(StateTransitionType {
            input: StateTime::At(input),
            output: StateTime::At(output),
        }),
    }
}

#[test]
fn logical_event_equality_and_order_ignore_operational_metadata() {
    let resolved = give();
    let mut unresolved = resolved.clone();
    unresolved.transition.as_mut().unwrap().input = StateTime::Unresolved;
    let mut missing = resolved.clone();
    missing.transition = None;
    let mut different = resolved.clone();
    different.transition = Some(StateTransitionType {
        input: StateTime::At(TimeExpr::At("other".into())),
        output: StateTime::Unresolved,
    });
    let mut classified = resolved.clone();
    classified.proposition.kind = PropositionKind::Derived;
    let variants = [resolved.clone(), unresolved, missing, different, classified];
    for a in &variants {
        for b in &variants {
            assert_eq!(a, b);
            assert_eq!(a.cmp(b), std::cmp::Ordering::Equal);
            assert_eq!(a.partial_cmp(b), Some(std::cmp::Ordering::Equal));
        }
    }
    assert_eq!(variants.into_iter().collect::<BTreeSet<_>>().len(), 1);
    let mut differences = Vec::new();
    let mut changed = resolved.clone();
    changed.verb = "do".into();
    differences.push(changed);
    let mut changed = resolved.clone();
    changed.args[1] = Term::Var("house".into());
    differences.push(changed);
    let mut changed = resolved.clone();
    changed.args[0] = Term::Const("alice".into());
    differences.push(changed);
    let mut changed = resolved.clone();
    changed.polarity = Polarity::EvidentialNot;
    differences.push(changed);
    let mut changed = resolved.clone();
    changed.proposition.time = TimeExpr::After(Box::new(changed.proposition.time));
    differences.push(changed);
    for other in differences {
        assert_ne!(resolved, other);
        assert_ne!(resolved.cmp(&other), std::cmp::Ordering::Equal);
    }
}

#[derive(Default)]
struct Clock(usize);

impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}

#[test]
fn closure_deduplicates_transition_variants_and_retains_interpretations() {
    let resolved = give();
    let mut unresolved = resolved.clone();
    unresolved.transition.as_mut().unwrap().input = StateTime::Unresolved;
    let rules: Vec<_> = ["first", "second"]
        .into_iter()
        .map(|name| Rule {
            name: name.into(),
            parameters: vec![],
            body: RuleBody::HornClause(HornClause {
                antecedent: Antecedent::Unit,
                consequent: resolved.clone(),
            }),
        })
        .collect();
    let mut clock = Clock::default();
    let closure = close(
        vec![unresolved.clone(), resolved.clone()],
        &rules,
        &mut clock,
    );
    assert_eq!(closure.known.len(), 1);
    assert_eq!(closure.known[0].transition, unresolved.transition);
    assert_eq!(closure.derivations.len(), 2);
    assert_eq!(clock.0, 2);
    for derived in &closure.derivations {
        assert_eq!(derived.event, unresolved);
        assert_eq!(derived.event.transition, resolved.transition);
        assert!(derived.record.antecedents.is_empty());
    }
}

#[test]
fn residual_query_preserves_goal_metadata_without_using_it_as_identity() {
    let module =
        parse_formal_to_typed(include_str!("../examples/legal/sales_delivery_formal.res")).unwrap();
    let sales = module
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Rule(rule) if rule.name == "sales" => Some(rule),
            _ => None,
        })
        .unwrap();
    let resolved = give();
    let mut unresolved = resolved.clone();
    unresolved.transition.as_mut().unwrap().input = StateTime::Unresolved;
    let mut missing = resolved.clone();
    missing.transition = None;
    let baseline = query_residual(sales, &resolved, &[]).unwrap();
    assert_eq!(baseline.len(), 1);
    for goal in [unresolved, missing] {
        let pending = query_residual(sales, &goal, &[]).unwrap();
        assert_eq!(pending, baseline);
        assert_eq!(pending[0].goal.transition, goal.transition);
    }
}
