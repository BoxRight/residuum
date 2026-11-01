//! Synchronous immediate consequences. A round reads one frozen snapshot;
//! existing full-closure evaluators and their scheduling remain unchanged.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardStep {
    pub round: usize,
    /// T(X), including conclusions already in X, without duplicate events.
    pub consequences: Vec<Event>,
    /// All immediate rule instances, including alternative justifications.
    pub witnesses: Vec<UnstampedDerivation>,
    /// T(X) \ X, independent of rule order as a logical set.
    pub added: Vec<Event>,
    pub derivations: Vec<DerivedEvent>,
    pub known: Vec<Event>,
    pub complete: bool,
}

#[derive(Debug, Clone)]
pub struct ForwardSession {
    rules: Vec<Rule>,
    programs: Vec<ProgramSource>,
    closure: Closure,
    seen: BTreeSet<UnstampedDerivation>,
    round: usize,
    complete: bool,
}

impl ForwardSession {
    pub fn new(seeds: Vec<Event>, rules: &[Rule]) -> Self {
        Self::from_parts(seeds, rules.to_vec(), Vec::new())
    }
    pub fn for_module(seeds: Vec<Event>, module: &Module) -> Result<Self, crate::elab::Error> {
        let (rules, programs) = prepare_programs(module)?;
        Ok(Self::from_parts(seeds, rules, programs))
    }
    fn from_parts(seeds: Vec<Event>, rules: Vec<Rule>, programs: Vec<ProgramSource>) -> Self {
        let mut keys = BTreeSet::new();
        let known = seeds
            .into_iter()
            .filter(|e| keys.insert(e.clone()))
            .collect();
        Self {
            rules,
            programs,
            closure: Closure {
                known,
                derivations: Vec::new(),
            },
            seen: BTreeSet::new(),
            round: 0,
            complete: false,
        }
    }
    pub fn closure(&self) -> &Closure {
        &self.closure
    }
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn step(&mut self, clock: &mut dyn DerivationClock) -> ForwardStep {
        let index = KnownIndex::new(&self.closure.known);
        let view = KnownView {
            events: &self.closure.known,
            index: Some(&index),
        };
        let mut witnesses = Vec::new();
        for rule in &self.rules {
            witnesses.extend(instantiate_from_view(rule, view));
        }
        for source in &self.programs {
            for trigger in &self.closure.known {
                let Some(bindings) = activation_bindings(source, trigger) else {
                    continue;
                };
                for rule in &source.rules {
                    witnesses.extend(instantiate_program(
                        &ActivatedProgram {
                            rule,
                            trigger: trigger.clone(),
                            bindings: bindings.clone(),
                        },
                        view,
                    ));
                }
            }
        }
        // Retain distinct witnesses; event identity alone does not identify proofs.
        let mut distinct = Vec::new();
        for witness in witnesses {
            if !distinct.contains(&witness) {
                distinct.push(witness);
            }
        }
        let mut consequences = Vec::new();
        let mut consequence_keys = BTreeSet::new();
        let mut known_keys: BTreeSet<_> = self.closure.known.iter().cloned().collect();
        let mut added = Vec::new();
        let mut derivations = Vec::new();
        for witness in &distinct {
            if consequence_keys.insert(witness.event.clone()) {
                consequences.push(witness.event.clone());
            }
            if let Some((derived, is_new)) =
                retain_derivation(witness.clone(), &mut self.seen, &mut known_keys, clock)
            {
                if is_new {
                    added.push(derived.event.clone());
                }
                derivations.push(derived);
            }
        }
        self.closure.known.extend(added.iter().cloned());
        self.closure.derivations.extend(derivations.iter().cloned());
        self.round += 1;
        self.complete = added.is_empty();
        ForwardStep {
            round: self.round,
            consequences,
            witnesses: distinct,
            added,
            derivations,
            known: self.closure.known.clone(),
            complete: self.complete,
        }
    }

    /// Budget is rounds, never a claim that an unfinished prefix is a closure.
    pub fn trace(&mut self, max_steps: usize, clock: &mut dyn DerivationClock) -> Vec<ForwardStep> {
        let mut steps = Vec::new();
        for _ in 0..max_steps {
            let step = self.step(clock);
            let done = step.complete;
            steps.push(step);
            if done {
                break;
            }
        }
        steps
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivativeStep {
    pub round: usize,
    /// Cumulative canonical D, always disjoint from the closed base.
    pub delta: Vec<Event>,
    pub added: Vec<Event>,
    pub derivations: Vec<DerivedEvent>,
    pub complete: bool,
}

#[derive(Debug, Clone)]
pub struct DerivativeSession {
    forward: ForwardSession,
    base: BTreeSet<Event>,
    additions: Vec<Event>,
    initial: bool,
    round: usize,
}
impl DerivativeSession {
    pub(super) fn from_closed(closure: &Closure, rules: &[Rule], additions: Vec<Event>) -> Self {
        let mut forward = ForwardSession::new(closure.known.clone(), rules);
        forward.closure = closure.clone();
        forward.seen = closure
            .derivations
            .iter()
            .map(|d| UnstampedDerivation {
                event: d.event.clone(),
                record: d.record.clone(),
            })
            .collect();
        Self {
            forward,
            base: closure.known.iter().cloned().collect(),
            additions,
            initial: true,
            round: 0,
        }
    }
    pub fn closure(&self) -> &Closure {
        self.forward.closure()
    }
    pub fn step(&mut self, clock: &mut dyn DerivationClock) -> DerivativeStep {
        self.round += 1;
        let (added, derivations, complete) = if self.initial {
            self.initial = false;
            let mut keys = self.base.clone();
            let added: Vec<_> = self
                .additions
                .iter()
                .filter(|e| keys.insert((*e).clone()))
                .cloned()
                .collect();
            self.forward.closure.known.extend(added.iter().cloned());
            let complete = added.is_empty();
            (added, Vec::new(), complete)
        } else {
            let step = self.forward.step(clock);
            (step.added, step.derivations, step.complete)
        };
        let delta = self
            .forward
            .closure
            .known
            .iter()
            .filter(|e| !self.base.contains(*e))
            .cloned()
            .collect();
        DerivativeStep {
            round: self.round,
            delta,
            added,
            derivations,
            complete,
        }
    }
    pub fn trace(
        &mut self,
        max_steps: usize,
        clock: &mut dyn DerivationClock,
    ) -> Vec<DerivativeStep> {
        let mut steps = Vec::new();
        for _ in 0..max_steps {
            let step = self.step(clock);
            let done = step.complete;
            steps.push(step);
            if done {
                break;
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests;
