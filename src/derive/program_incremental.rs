//! Add-only module closure with dynamic Verb.program activation.
//! Events drive subsequent matches; proofs are retained but are not event deltas.
use super::incremental::match_delta_antecedents;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramActivation {
    pub behavior: crate::typed::BehaviorRef,
    pub rule: String,
    pub trigger: Event,
    pub substitution: Substitution,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProgramClosureDelta {
    pub known: Vec<Event>,
    pub derivations: Vec<DerivedEvent>,
    pub activations: Vec<ProgramActivation>,
    pub stats: IncrementalStats,
    /// Full evaluations for initial ordinary rules and newly activated RuleUses.
    pub full_initializations: usize,
}

#[derive(Debug, Clone)]
struct Activation {
    source: usize,
    trigger: Event,
    bindings: BTreeMap<String, SubstitutionValue>,
    initialized: bool,
}

/// The resolved module and its rules remain fixed. Only positive known triggers
/// create activations. No default selection or State execution occurs here.
#[derive(Debug, Clone)]
pub struct IncrementalProgramClosure {
    ordinary: Vec<Rule>,
    programs: Vec<ProgramSource>,
    active: Vec<Activation>,
    closure: Closure,
    known_keys: BTreeSet<Event>,
    index: KnownIndex,
    seen: BTreeSet<UnstampedDerivation>,
}

impl IncrementalProgramClosure {
    pub fn new(
        seeds: Vec<Event>,
        module: &Module,
        clock: &mut dyn DerivationClock,
    ) -> Result<Self, crate::elab::Error> {
        let (ordinary, programs) = prepare_programs(module)?;
        let mut state = Self {
            ordinary,
            programs,
            active: Vec::new(),
            closure: Closure {
                known: Vec::new(),
                derivations: Vec::new(),
            },
            known_keys: BTreeSet::new(),
            index: KnownIndex::default(),
            seen: BTreeSet::new(),
        };
        let mut delta = ProgramClosureDelta::default();
        let frontier = state.add_seeds(seeds, &mut delta);
        state.drain(frontier, true, &mut delta, clock);
        Ok(state)
    }

    pub fn closure(&self) -> &Closure {
        &self.closure
    }
    pub fn into_closure(self) -> Closure {
        self.closure
    }

    /// Returns only after a fixed point. A runner timeout is not a completed
    /// closure or an equivalence result. The caller must supply terminating input.
    pub fn update(
        &mut self,
        additions: Vec<Event>,
        clock: &mut dyn DerivationClock,
    ) -> ProgramClosureDelta {
        let mut delta = ProgramClosureDelta::default();
        let frontier = self.add_seeds(additions, &mut delta);
        self.drain(frontier, false, &mut delta, clock);
        delta
    }

    fn add_seeds(&mut self, seeds: Vec<Event>, delta: &mut ProgramClosureDelta) -> Vec<Event> {
        seeds
            .into_iter()
            .filter_map(|event| {
                if !self.known_keys.insert(event.clone()) {
                    return None;
                }
                self.index.insert(self.closure.known.len(), &event);
                self.closure.known.push(event.clone());
                delta.known.push(event.clone());
                Some(event)
            })
            .collect()
    }

    fn discover(&mut self, frontier: &[Event], delta: &mut ProgramClosureDelta) {
        #[cfg(test)]
        let timer = profile::start();
        for trigger in frontier {
            for (source_index, source) in self.programs.iter().enumerate() {
                // A trigger is normally discovered once from DeltaEvents.
                // Keep this check explicit for pending activation initialization.
                if self
                    .active
                    .iter()
                    .any(|a| a.source == source_index && a.trigger == *trigger)
                {
                    continue;
                }
                let Some(bindings) = activation_bindings(source, trigger) else {
                    continue;
                };
                for rule in &source.rules {
                    delta.activations.push(ProgramActivation {
                        behavior: source.verb.program.clone().expect("resolved behavior"),
                        rule: rule.name.clone(),
                        trigger: trigger.clone(),
                        substitution: substitution_record(&bindings),
                    });
                }
                self.active.push(Activation {
                    source: source_index,
                    trigger: trigger.clone(),
                    bindings,
                    initialized: false,
                });
            }
        }
        #[cfg(test)]
        profile::activation(
            timer,
            frontier.len(),
            self.active
                .iter()
                .map(|a| self.programs[a.source].rules.len())
                .sum(),
        );
    }

    fn evidence(&self) -> KnownView<'_> {
        let view = KnownView {
            events: &self.closure.known,
            index: Some(&self.index),
        };
        #[cfg(test)]
        if profile::scan_known() {
            return KnownView::linear(&self.closure.known);
        }
        view
    }

    fn retain(
        &mut self,
        candidates: Vec<UnstampedDerivation>,
        delta: &mut ProgramClosureDelta,
        next: &mut Vec<Event>,
        clock: &mut dyn DerivationClock,
    ) {
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
            // A new proof for an old Event is recorded without scheduling it.
            self.closure.derivations.push(derived.clone());
            delta.derivations.push(derived);
        }
    }

    fn drain(
        &mut self,
        mut frontier: Vec<Event>,
        mut initial: bool,
        delta: &mut ProgramClosureDelta,
        clock: &mut dyn DerivationClock,
    ) {
        while initial || !frontier.is_empty() || self.active.iter().any(|a| !a.initialized) {
            delta.stats.rounds += 1;
            self.discover(&frontier, delta);
            let frontier_index = KnownIndex::new(&frontier);
            let delta_view = KnownView {
                events: &frontier,
                index: Some(&frontier_index),
            };
            #[cfg(test)]
            let delta_view = if profile::scan_known() {
                KnownView::linear(&frontier)
            } else {
                delta_view
            };
            let mut next = Vec::new();
            for index in 0..self.ordinary.len() {
                let rule = &self.ordinary[index];
                delta.stats.rules_examined += 1;
                #[cfg(test)]
                let timer = profile::start();
                let candidates = if initial {
                    delta.full_initializations += 1;
                    instantiate_from_view(rule, self.evidence())
                } else {
                    instantiate_delta_rule(
                        rule,
                        self.evidence(),
                        delta_view,
                        &BTreeMap::new(),
                        None,
                        &mut delta.stats,
                    )
                };
                #[cfg(test)]
                profile::rule(&rule.name, false, timer, candidates.len());
                self.retain(candidates, delta, &mut next, clock);
            }
            for index in 0..self.active.len() {
                let initialization = !self.active[index].initialized;
                let source = self.active[index].source;
                for member in 0..self.programs[source].rules.len() {
                    let activation = &self.active[index];
                    let rule = &self.programs[source].rules[member];
                    // Old activations have no work when only proofs changed.
                    if !initialization && frontier.is_empty() {
                        continue;
                    }
                    delta.stats.rules_examined += 1;
                    #[cfg(test)]
                    let timer = profile::start();
                    let candidates = if initialization {
                        delta.full_initializations += 1;
                        instantiate_program(
                            &ActivatedProgram {
                                rule,
                                trigger: activation.trigger.clone(),
                                bindings: activation.bindings.clone(),
                            },
                            self.evidence(),
                        )
                    } else {
                        instantiate_delta_rule(
                            rule,
                            self.evidence(),
                            delta_view,
                            &activation.bindings,
                            Some(&activation.trigger),
                            &mut delta.stats,
                        )
                    };
                    #[cfg(test)]
                    profile::rule(&rule.name, true, timer, candidates.len());
                    self.retain(candidates, delta, &mut next, clock);
                }
                self.active[index].initialized = true;
            }
            #[cfg(test)]
            profile::round(
                self.closure.known.len(),
                self.closure.derivations.len(),
                next.len(),
            );
            frontier = next;
            initial = false;
        }
    }
}

fn instantiate_delta_rule(
    rule: &Rule,
    known: KnownView<'_>,
    delta: KnownView<'_>,
    bindings: &BTreeMap<String, SubstitutionValue>,
    trigger: Option<&Event>,
    stats: &mut IncrementalStats,
) -> Vec<UnstampedDerivation> {
    let clause = match &rule.body {
        RuleBody::HornClause(c) => Cow::Borrowed(c),
        RuleBody::ResidualClause(c) => Cow::Owned(c.unresiduate()),
    };
    let candidates: Vec<_> = match_delta_antecedents(
        &clause.antecedent,
        &rule.parameters,
        known,
        delta,
        bindings,
        stats,
    )
    .into_iter()
    .filter_map(|matched| match trigger {
        Some(t) => finish_program_match(rule, &clause.consequent, t, matched),
        None => finish_instantiation(rule, &clause.consequent, matched),
    })
    .collect();
    stats.full_matches += candidates.len();
    candidates
}
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Clock(usize);
    impl DerivationClock for Clock {
        fn next_derivation_time(&mut self) -> DerivationTime {
            self.0 += 1;
            DerivationTime {
                name: format!("pending-{}", self.0),
            }
        }
    }

    #[test]
    fn pending_activations_initialize_against_old_known_with_empty_event_delta() {
        let m = crate::load_formal_module(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples/experiments/dynamic_incremental_programs.res"),
        )
        .unwrap();
        let input = m
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Experiment(e) if e.name == "existingSnapshot" => Some(e.input.clone()),
                _ => None,
            })
            .unwrap();
        let mut clock = Clock::default();
        let expected = close_program_reference(input, &m, &mut clock).unwrap();
        let mut stored = expected.clone();
        // A test-only scheduling checkpoint: all Events are already present,
        // one RuleUse justification remains pending. No public mutation API.
        stored
            .derivations
            .retain(|d| d.record.rule != "Root::1::useOld");
        assert_eq!(stored.derivations.len() + 1, expected.derivations.len());
        let (ordinary, programs) = prepare_programs(&m).unwrap();
        let mut state = IncrementalProgramClosure {
            ordinary,
            programs,
            active: vec![],
            known_keys: stored.known.iter().cloned().collect(),
            index: KnownIndex::new(&stored.known),
            seen: stored
                .derivations
                .iter()
                .map(|d| UnstampedDerivation {
                    event: d.event.clone(),
                    record: d.record.clone(),
                })
                .collect(),
            closure: stored,
        };
        let mut delta = ProgramClosureDelta::default();
        state.discover(&state.closure.known.clone(), &mut delta);
        assert!(!delta.activations.is_empty());
        state.drain(vec![], false, &mut delta, &mut clock);
        assert!(delta.known.is_empty());
        assert_eq!(delta.derivations.len(), 1);
        assert_eq!(delta.derivations[0].record.rule, "Root::1::useOld");
        assert!(delta.full_initializations > 0);
        let proofs = |c: &Closure| {
            let mut p = c
                .derivations
                .iter()
                .map(|d| format!("{:?} {:?}", d.event, d.record))
                .collect::<Vec<_>>();
            p.sort();
            p
        };
        assert_eq!(proofs(state.closure()), proofs(&expected));
        let stamps = clock.0;
        let empty = state.update(vec![], &mut clock);
        assert_eq!(empty, ProgramClosureDelta::default());
        assert_eq!(stamps, clock.0);
        println!(
            "DeltaEvents=empty; pending activations scan full Known and retain exactly one missing justification without reactivating consumers"
        );
    }
}
