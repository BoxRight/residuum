//! Add-only, semi-naive maintenance of a closure under a fixed rule set.
//! The existing close initializes the state and remains the comparison oracle.

use super::{AntecedentMatch, DerivationClock, finish_instantiation, match_event, stamp};
use crate::typed::{Antecedent, Closure, DerivedEvent, Event, Rule, RuleBody, UnstampedDerivation};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IncrementalStats {
    pub rounds: usize,
    pub rules_examined: usize,
    pub delta_pivot_matches: usize,
    pub full_matches: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClosureDelta {
    /// New logical events, including newly supplied seeds. Never old events.
    pub known: Vec<Event>,
    /// New justifications can conclude an event that was already known.
    pub derivations: Vec<DerivedEvent>,
    pub stats: IncrementalStats,
}

/// Rules are copied once and cannot change through the update interface.
/// All stored state is private; the caller cannot supply a non-closed base.
#[derive(Debug, Clone)]
pub struct IncrementalClosure {
    rules: Vec<Rule>,
    closure: Closure,
    known_keys: BTreeSet<Event>,
    seen: Vec<UnstampedDerivation>,
}

impl IncrementalClosure {
    pub fn new<C: DerivationClock>(seeds: Vec<Event>, rules: &[Rule], clock: &mut C) -> Self {
        let closure = super::close(seeds, rules, clock);
        let known_keys = closure.known.iter().cloned().collect();
        let seen = closure
            .derivations
            .iter()
            .map(|derived| UnstampedDerivation {
                event: derived.event.clone(),
                record: derived.record.clone(),
            })
            .collect();
        Self {
            rules: rules.to_vec(),
            closure,
            known_keys,
            seen,
        }
    }

    pub fn closure(&self) -> &Closure {
        &self.closure
    }

    /// Caller supplies derivation time independently of rounds and PropTime.
    /// Empty/duplicate additions do no inference work and receive no stamps.
    /// Like close, this requires a terminating positive Horn instance.
    pub fn update<C: DerivationClock>(
        &mut self,
        additions: Vec<Event>,
        clock: &mut C,
    ) -> ClosureDelta {
        let mut delta = ClosureDelta::default();
        let mut frontier = Vec::new();
        for event in additions {
            if self.known_keys.insert(event.clone()) {
                self.closure.known.push(event.clone());
                delta.known.push(event.clone());
                frontier.push(event);
            }
        }
        while !frontier.is_empty() {
            delta.stats.rounds += 1;
            let mut next = Vec::new();
            for rule in &self.rules {
                delta.stats.rules_examined += 1;
                for unstamped in
                    instantiate_delta(rule, &self.closure.known, &frontier, &mut delta.stats)
                {
                    if self.seen.contains(&unstamped) {
                        continue;
                    }
                    self.seen.push(unstamped.clone());
                    let is_new = self.known_keys.insert(unstamped.event.clone());
                    let derived = stamp(unstamped, clock.next_derivation_time());
                    if is_new {
                        self.closure.known.push(derived.event.clone());
                        delta.known.push(derived.event.clone());
                        next.push(derived.event.clone());
                    }
                    self.closure.derivations.push(derived.clone());
                    delta.derivations.push(derived);
                }
            }
            frontier = next;
        }
        delta
    }
}

fn leaves<'a>(antecedent: &'a Antecedent, atoms: &mut Vec<&'a Event>) {
    match antecedent {
        Antecedent::Unit => {}
        Antecedent::Event(event) => atoms.push(event),
        Antecedent::And(a, b) => {
            leaves(a, atoms);
            leaves(b, atoms);
        }
    }
}

fn instantiate_delta(
    rule: &Rule,
    known: &[Event],
    delta: &[Event],
    stats: &mut IncrementalStats,
) -> Vec<UnstampedDerivation> {
    let body = match &rule.body {
        RuleBody::HornClause(clause) => std::borrow::Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => std::borrow::Cow::Owned(clause.unresiduate()),
    };
    let mut atoms = Vec::new();
    leaves(&body.antecedent, &mut atoms);
    let mut result = Vec::new();
    // For each conjunct position, anchor it in the new frontier first. Other
    // conjuncts use Known with the same bindings. Unit has no event position.
    // Restoring the original witness order preserves forward provenance.
    for (pivot, pattern) in atoms.iter().enumerate() {
        for event in delta {
            let mut bindings = std::collections::BTreeMap::new();
            if match_event(pattern, event, &rule.parameters, &mut bindings).is_none() {
                continue;
            }
            stats.delta_pivot_matches += 1;
            let mut witnesses = vec![None; atoms.len()];
            witnesses[pivot] = Some(event.clone());
            let mut matches = vec![(bindings, witnesses)];
            for (position, atom) in atoms.iter().enumerate() {
                if position == pivot {
                    continue;
                }
                let mut joined = Vec::new();
                for (bindings, witnesses) in matches {
                    for candidate in known {
                        let mut bound = bindings.clone();
                        if match_event(atom, candidate, &rule.parameters, &mut bound).is_none() {
                            continue;
                        }
                        let mut witness = witnesses.clone();
                        witness[position] = Some(candidate.clone());
                        joined.push((bound, witness));
                    }
                }
                matches = joined;
            }
            for (bindings, witnesses) in matches {
                if let Some(derivation) = finish_instantiation(
                    rule,
                    &body.consequent,
                    AntecedentMatch {
                        substitutions: bindings,
                        antecedents: witnesses
                            .into_iter()
                            .map(|event| event.expect("complete witness"))
                            .collect(),
                    },
                ) {
                    stats.full_matches += 1;
                    result.push(derivation);
                }
            }
        }
    }
    result
}
