//! Add-only, semi-naive maintenance of a closure under a fixed rule set.
//! The existing close initializes the state and remains the comparison oracle;
//! `new_by_derivative` builds the same base with the derivative itself.

use super::{
    AntecedentMatch, DerivationClock, KnownIndex, KnownView, finish_instantiation, match_event,
    retain_derivation,
};
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
    /// Header index over `closure.known`; positions follow its order.
    index: KnownIndex,
    seen: BTreeSet<UnstampedDerivation>,
}

impl IncrementalClosure {
    pub fn new<C: DerivationClock>(seeds: Vec<Event>, rules: &[Rule], clock: &mut C) -> Self {
        Self::new_with_clock(seeds, rules, clock)
    }

    /// Builds the same closed base through the additive derivative instead of
    /// running the naive oracle over the seeds:
    /// `Close_R(S) = Close_R(∅) ∪ ∂_R(Close_R(∅), S)` (docs/discrete-derivative.md §2).
    /// `close` only evaluates the empty-seed base, where just `I <= C` axioms fire.
    /// Events and justifications coincide with `new` as logical sets; Known order
    /// and DerivationTime assignment may differ. If a seed is logically equal to
    /// an axiom consequence, this keeps the axiom's representation, whereas
    /// `new` keeps the first supplied seed.
    pub fn new_by_derivative<C: DerivationClock>(
        seeds: Vec<Event>,
        rules: &[Rule],
        clock: &mut C,
    ) -> Self {
        let mut state = Self::new_with_clock(Vec::new(), rules, clock);
        state.update_with_clock(seeds, clock);
        state
    }

    fn new_with_clock(seeds: Vec<Event>, rules: &[Rule], clock: &mut dyn DerivationClock) -> Self {
        let closure = super::close_with_programs(seeds, rules, &[], clock);
        let known_keys = closure.known.iter().cloned().collect();
        let index = KnownIndex::new(&closure.known);
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
            index,
            seen,
        }
    }

    pub fn closure(&self) -> &Closure {
        &self.closure
    }

    fn evidence(&self) -> KnownView<'_> {
        #[cfg(test)]
        if super::profile::scan_known() {
            return KnownView::linear(&self.closure.known);
        }
        KnownView {
            events: &self.closure.known,
            index: Some(&self.index),
        }
    }

    /// Expose Kleene approximations of the additive derivative from this
    /// internally verified closed base, without mutating the base or calling update.
    pub fn begin_stepwise_update(&self, additions: Vec<Event>) -> super::DerivativeSession {
        super::steps::DerivativeSession::from_closed(&self.closure, &self.rules, additions)
    }

    /// Caller supplies derivation time independently of rounds and PropTime.
    /// Empty/duplicate additions do no inference work and receive no stamps.
    /// Like close, this requires a terminating positive Horn instance.
    pub fn update<C: DerivationClock>(
        &mut self,
        additions: Vec<Event>,
        clock: &mut C,
    ) -> ClosureDelta {
        self.update_with_clock(additions, clock)
    }

    fn update_with_clock(
        &mut self,
        additions: Vec<Event>,
        clock: &mut dyn DerivationClock,
    ) -> ClosureDelta {
        let mut delta = ClosureDelta::default();
        let mut frontier = Vec::new();
        for event in additions {
            if self.known_keys.insert(event.clone()) {
                self.index.insert(self.closure.known.len(), &event);
                self.closure.known.push(event.clone());
                delta.known.push(event.clone());
                frontier.push(event);
            }
        }
        while !frontier.is_empty() {
            delta.stats.rounds += 1;
            let frontier_index = KnownIndex::new(&frontier);
            let delta_view = KnownView {
                events: &frontier,
                index: Some(&frontier_index),
            };
            #[cfg(test)]
            let delta_view = if super::profile::scan_known() {
                KnownView::linear(&frontier)
            } else {
                delta_view
            };
            let mut next = Vec::new();
            for rule in &self.rules {
                delta.stats.rules_examined += 1;
                #[cfg(test)]
                let matching = super::profile::start();
                let candidates =
                    instantiate_delta(rule, self.evidence(), delta_view, &mut delta.stats);
                #[cfg(test)]
                super::profile::rule(&rule.name, false, matching, candidates.len());
                for unstamped in candidates {
                    let Some((derived, is_new)) =
                        retain_derivation(unstamped, &mut self.seen, &mut self.known_keys, clock)
                    else {
                        continue;
                    };
                    if is_new {
                        self.index.insert(self.closure.known.len(), &derived.event);
                        self.closure.known.push(derived.event.clone());
                        delta.known.push(derived.event.clone());
                        next.push(derived.event.clone());
                    }
                    self.closure.derivations.push(derived.clone());
                    delta.derivations.push(derived);
                }
            }
            #[cfg(test)]
            super::profile::round(
                self.closure.known.len(),
                self.closure.derivations.len(),
                next.len(),
            );
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
    known: KnownView<'_>,
    delta: KnownView<'_>,
    stats: &mut IncrementalStats,
) -> Vec<UnstampedDerivation> {
    let body = match &rule.body {
        RuleBody::HornClause(clause) => std::borrow::Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => std::borrow::Cow::Owned(clause.unresiduate()),
    };
    let candidates: Vec<_> = match_delta_antecedents(
        &body.antecedent,
        &rule.parameters,
        known,
        delta,
        &std::collections::BTreeMap::new(),
        stats,
    )
    .into_iter()
    .filter_map(|matched| finish_instantiation(rule, &body.consequent, matched))
    .collect();
    stats.full_matches += candidates.len();
    candidates
}

pub(super) fn match_delta_antecedents(
    antecedent: &Antecedent,
    parameters: &[crate::typed::Arg],
    known: super::KnownView<'_>,
    delta: super::KnownView<'_>,
    initial: &std::collections::BTreeMap<String, crate::typed::SubstitutionValue>,
    stats: &mut IncrementalStats,
) -> Vec<AntecedentMatch> {
    let mut atoms = Vec::new();
    leaves(antecedent, &mut atoms);
    let mut result = Vec::new();
    // Frontier membership, built only when some pivot has earlier positions.
    let mut fresh_keys: Option<BTreeSet<&Event>> = None;
    // For each conjunct position, anchor it in the new frontier first. Other
    // conjuncts use Known with the same bindings. Unit has no event position.
    // Restoring the original witness order preserves forward provenance.
    //
    // Positions before the pivot exclude frontier events: a match with several
    // frontier witnesses is produced once, at its first frontier position.
    // That is where the unrestricted scan first produced it within this call,
    // so the retained justifications, their order and their stamps are the
    // same; only repeated candidates disappear (the dA·B ∪ A_old·dB split).
    for (pivot, pattern) in atoms.iter().enumerate() {
        let fresh: Option<&BTreeSet<&Event>> = if pivot > 0 {
            Some(&*fresh_keys.get_or_insert_with(|| delta.events.iter().collect()))
        } else {
            None
        };
        for event in delta.candidates(pattern) {
            let mut bindings = initial.clone();
            if match_event(pattern, event, parameters, &mut bindings).is_none() {
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
                    for candidate in known.candidates(atom) {
                        if position < pivot && fresh.is_some_and(|f| f.contains(&candidate)) {
                            continue;
                        }
                        let mut bound = bindings.clone();
                        if match_event(atom, candidate, parameters, &mut bound).is_none() {
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
                result.push(AntecedentMatch {
                    substitutions: bindings,
                    antecedents: witnesses
                        .into_iter()
                        .map(|event| event.expect("complete witness"))
                        .collect(),
                });
            }
        }
    }
    result
}
