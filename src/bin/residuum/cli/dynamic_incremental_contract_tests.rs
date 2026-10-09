//! Guarded-clause model and production scheduler compared with the previous
//! full-match evaluator. No normalization of bindings/proof multiplicity here.
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::*;
use residuum::{
    DerivationClock, IncrementalClosure, IncrementalProgramClosure, close, close_program,
    close_program_reference, instantiate_from_known,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("dynamic-contract-{}", self.0),
        }
    }
}
fn module(file: &str) -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments")
            .join(file),
    )
    .unwrap()
}
fn experiment<'a>(m: &'a Module, name: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}

struct Adapter {
    rules: Vec<Rule>,
    // Each original compiled member and the verb guarding this reference.
    guards: BTreeSet<(String, String)>,
    ordinary: Vec<Rule>,
}
impl Adapter {
    fn new(m: &Module) -> Self {
        assert!(
            !m.declarations
                .iter()
                .any(|d| matches!(d, Declaration::Default(_)))
        );
        let mut definitions = BTreeSet::new();
        let mut rules = Vec::new();
        let mut guards = BTreeSet::new();
        for d in &m.declarations {
            let Declaration::Verb(v) = d else {
                continue;
            };
            let Some(reference) = &v.program else {
                continue;
            };
            let (parameters, members) = match reference.kind {
                BehaviorKind::Program => {
                    let p = m
                        .declarations
                        .iter()
                        .find_map(|d| match d {
                            Declaration::Program(p) if p.identity == reference.name => Some(p),
                            _ => None,
                        })
                        .unwrap();
                    definitions.extend(p.rule_uses.iter().map(|u| u.rule.name.clone()));
                    let parameters = p
                        .parameters
                        .iter()
                        .map(|a| Arg {
                            name: format!("#{}:shared:{}", p.identity, a.name),
                            ty: a.ty.clone(),
                        })
                        .collect::<Vec<_>>();
                    (parameters, compile_program(p, m).unwrap())
                }
                BehaviorKind::Rule => {
                    let r = m
                        .declarations
                        .iter()
                        .find_map(|d| match d {
                            Declaration::Rule(r) if r.name == reference.name => Some(r.clone()),
                            _ => None,
                        })
                        .unwrap();
                    definitions.insert(r.name.clone());
                    (r.parameters.clone(), vec![r])
                }
            };
            if !matches!(v.kind, VerbKind::Seeded | VerbKind::Derived) {
                continue;
            }
            assert_eq!(
                parameters.iter().map(|a| a.ty.clone()).collect::<Vec<_>>(),
                v.args
                    .iter()
                    .map(|a| a.ty.clone())
                    .chain([Type::PropositionTime])
                    .collect::<Vec<_>>()
            );
            let trigger = Event {
                polarity: Polarity::Positive,
                verb: v.name.clone(),
                args: parameters[..parameters.len() - 1]
                    .iter()
                    .map(|a| Term::Var(a.name.clone()))
                    .collect(),
                proposition: PropositionType {
                    kind: if v.kind == VerbKind::Seeded {
                        PropositionKind::Seeded
                    } else {
                        PropositionKind::Derived
                    },
                    time: TimeExpr::Var(parameters.last().unwrap().name.clone()),
                },
                transition: None,
            };
            for mut r in members {
                let mut clause = match r.body {
                    RuleBody::HornClause(c) => c,
                    RuleBody::ResidualClause(c) => c.unresiduate(),
                };
                clause.antecedent = Antecedent::And(
                    Box::new(Antecedent::Event(trigger.clone())),
                    Box::new(clause.antecedent),
                );
                r.body = RuleBody::HornClause(clause);
                guards.insert((r.name.clone(), v.name.clone()));
                rules.push(r);
            }
        }
        let ordinary = m
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) if !definitions.contains(&r.name) => Some(r.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        rules.extend(ordinary.clone());
        Self {
            rules,
            guards,
            ordinary,
        }
    }
    fn canonical_record(&self, rule: &str, antecedents: &mut Vec<Event>) {
        let Some(trigger) = antecedents.first() else {
            return;
        };
        if self
            .guards
            .contains(&(rule.to_string(), trigger.verb.clone()))
            && antecedents[1..].contains(trigger)
        {
            // Remove only the adapter's extra guard; retain all original body
            // occurrences, including a body with the same witness twice.
            antecedents.remove(0);
        }
    }
    fn normalize(&self, c: &Closure) -> Closure {
        let mut c = c.clone();
        for p in &mut c.derivations {
            self.canonical_record(&p.record.rule, &mut p.record.antecedents);
        }
        c
    }
}

fn observations(c: &Closure) -> (BTreeSet<String>, BTreeMap<String, usize>) {
    // Include event types, transitions, witness order, substitutions and
    // multiplicity; exclude only discovery order and concrete derivedAt values.
    let known = c.known.iter().map(|e| format!("{e:?}")).collect();
    let mut proofs = BTreeMap::new();
    for p in &c.derivations {
        *proofs
            .entry(format!("{:?} {:?}", p.event, p.record))
            .or_default() += 1;
    }
    (known, proofs)
}
fn equivalent(a: &Closure, b: &Closure) {
    assert_eq!(observations(a), observations(b));
    assert_eq!(a.known.len(), a.known.iter().collect::<BTreeSet<_>>().len());
    assert_eq!(b.known.len(), b.known.iter().collect::<BTreeSet<_>>().len());
    for c in [a, b] {
        assert_eq!(
            c.derivations
                .iter()
                .map(|d| &d.derived_at.name)
                .collect::<BTreeSet<_>>()
                .len(),
            c.derivations.len()
        );
        assert!(observations(c).1.values().all(|n| *n == 1));
    }
}
fn check(m: &Module, adapter: &Adapter, inc: &IncrementalClosure, seeds: &[Event]) -> Closure {
    let actual = adapter.normalize(inc.closure());
    let expected = close_program_reference(seeds.to_vec(), m, &mut Clock::default()).unwrap();
    equivalent(&actual, &expected);
    // Producer identities are unchanged: Program + Rule + imported module.
    for proof in &actual.derivations {
        if let Some(origin) = program_rule_origin(m, &proof.record.rule) {
            assert!(origin.rule_origin.file.as_ref().unwrap().is_file());
            assert!(!origin.program.is_empty() && !origin.defining_module.is_empty());
        }
    }
    actual
}

#[test]
fn guarded_derivative_handles_old_evidence_recursive_activation_and_proof_only_changes() {
    let m = module("dynamic_incremental_programs.res");
    let adapter = Adapter::new(&m);
    let mut clock = Clock::default();
    let mut seeds = experiment(&m, "base").input.clone();
    let mut inc = IncrementalClosure::new(seeds.clone(), &adapter.rules, &mut clock);
    check(&m, &adapter, &inc, &seeds);
    let activate = experiment(&m, "activate").input.clone();
    let change = inc.update(activate.clone(), &mut clock);
    seeds.extend(activate);
    let before = check(&m, &adapter, &inc, &seeds);
    for name in [
        "receipt",
        "activated",
        "shared",
        "finished",
        "echo",
        "branchReady",
    ] {
        assert!(
            change.known.iter().any(|e| e.verb == name),
            "missing {name}"
        );
    }
    assert!(!before.known.iter().any(|e| e.verb == "complemented"));
    let late = experiment(&m, "late").input.clone();
    let change = inc.update(late.clone(), &mut clock);
    seeds.extend(late);
    assert!(change.known.iter().any(|e| e.verb == "complemented"));
    let before = check(&m, &adapter, &inc, &seeds);
    let alternate = experiment(&m, "alternative").input.clone();
    let change = inc.update(alternate.clone(), &mut clock);
    seeds.extend(alternate);
    assert_eq!(change.known.len(), 1); // Only the new alternativeSupport seed.
    assert_eq!(change.derivations.len(), 1);
    assert_eq!(change.derivations[0].event.verb, "shared");
    assert!(
        !change
            .derivations
            .iter()
            .any(|d| d.event.verb == "finished")
    );
    let after = check(&m, &adapter, &inc, &seeds);
    for old in &before.derivations {
        assert!(
            after.derivations.contains(old),
            "old clock/proof must remain intact"
        );
    }
    assert_eq!(
        after
            .derivations
            .iter()
            .filter(|d| d.event.verb == "shared")
            .count(),
        2
    );
    assert_eq!(
        after
            .derivations
            .iter()
            .filter(|d| d.event.verb == "finished")
            .count(),
        2
    );
    let second = experiment(&m, "secondActivation").input.clone();
    inc.update(second.clone(), &mut clock);
    seeds.extend(second);
    check(&m, &adapter, &inc, &seeds);
    let count = clock.0;
    let duplicate = inc.update(seeds.clone(), &mut clock);
    assert!(duplicate.known.is_empty() && duplicate.derivations.is_empty());
    assert_eq!(clock.0, count);
    println!(
        "sequential activation, old evidence, late evidence, DeltaDerivations and duplicate updates agree"
    );
}

#[test]
fn new_bound_rule_initialization_preserves_proofs_when_event_delta_is_empty() {
    let m = module("dynamic_incremental_programs.res");
    let adapter = Adapter::new(&m);
    let snapshot = experiment(&m, "existingSnapshot").input.clone();
    let mut clock = Clock::default();
    let base = close(snapshot, &adapter.ordinary, &mut clock);
    let new_rule = adapter
        .rules
        .iter()
        .find(|r| r.name == "Root::1::useOld")
        .unwrap();
    // No new event/trigger is supplied: only a newly pending rule activation.
    assert_eq!(
        base.derivations
            .iter()
            .filter(|d| d.event.verb == "shared")
            .count(),
        1
    );
    let candidates = instantiate_from_known(new_rule, &base.known);
    assert_eq!(candidates.len(), 1);
    assert!(base.known.contains(&candidates[0].event));
    assert!(
        !base
            .derivations
            .iter()
            .any(|p| p.record.rule == new_rule.name)
    );
    let changed = close(
        base.known.clone(),
        std::slice::from_ref(new_rule),
        &mut clock,
    );
    assert_eq!(
        base.known.iter().collect::<BTreeSet<_>>(),
        changed.known.iter().collect()
    );
    assert_eq!(changed.derivations.len(), 1);
    let proof = &adapter.normalize(&changed).derivations[0];
    assert_eq!(proof.record.antecedents.len(), 2); // request + oldSupport.
    assert_eq!(proof.record.rule, new_rule.name);
    assert_eq!(proof.event.verb, "shared");
    println!("DeltaEvents=empty, pending activation emits one new justification for an old event");
}

#[test]
fn guarded_derivative_exhaustively_preserves_finite_input_splits_and_negative_triggers() {
    let m = module("dynamic_incremental_programs.res");
    let adapter = Adapter::new(&m);
    let universe = ["base", "activate", "late", "alternative", "negativeTrigger"]
        .into_iter()
        .flat_map(|n| experiment(&m, n).input.clone())
        .collect::<Vec<_>>();
    // Six facts, all subsets and all choices of the singleton added last.
    let mut comparisons = 0;
    for mask in 0..(1 << universe.len()) {
        let seeds = universe
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect::<Vec<_>>();
        let mut clock = Clock::default();
        let inc = IncrementalClosure::new(seeds.clone(), &adapter.rules, &mut clock);
        check(&m, &adapter, &inc, &seeds);
        comparisons += 1;
        for added in &universe {
            let mut updated = inc.clone();
            let mut update_clock = Clock(clock.0);
            updated.update(vec![added.clone()], &mut update_clock);
            let mut all = seeds.clone();
            all.push(added.clone());
            check(&m, &adapter, &updated, &all);
            comparisons += 1;
        }
    }
    let mut clock = Clock::default();
    let negative = experiment(&m, "negativeTrigger").input.clone();
    let inc = IncrementalClosure::new(negative, &adapter.rules, &mut clock);
    assert!(inc.closure().derivations.is_empty());
    println!("{comparisons} finite comparisons preserve events and full canonical provenance");
}

#[test]
fn guarded_derivative_preserves_canonical_product_bindings_and_all_justifications() {
    let m = module("peano_polynomial_product_scenario.res");
    let adapter = Adapter::new(&m);
    let input = experiment(&m, "partialProduct").input.clone();
    let mut clock = Clock::default();
    let base = input
        .iter()
        .filter(|e| e.verb != "multiplyPolynomialFromP")
        .cloned()
        .collect::<Vec<_>>();
    let triggers = input
        .iter()
        .filter(|e| e.verb == "multiplyPolynomialFromP")
        .cloned()
        .collect::<Vec<_>>();
    let mut inc = IncrementalClosure::new(base, &adapter.rules, &mut clock);
    let change = inc.update(triggers, &mut clock);
    let actual = adapter.normalize(inc.closure());
    let expected = close_program_reference(input, &m, &mut Clock::default()).unwrap();
    equivalent(&actual, &expected);
    assert_eq!((actual.known.len(), actual.derivations.len()), (223, 244));
    assert_eq!(
        (expected.known.len(), expected.derivations.len()),
        (223, 244)
    );
    check(&m, &adapter, &inc, &experiment(&m, "partialProduct").input);
    println!(
        "imported product: {} events, {} proofs; delta rounds={}, examined rules={}, full matches={}",
        actual.known.len(),
        actual.derivations.len(),
        change.stats.rounds,
        change.stats.rules_examined,
        change.stats.full_matches
    );
}

#[test]
fn guarded_derivative_temporal_bindings_are_canonical_for_every_pivot_and_update_order() {
    let m = module("dynamic_incremental_temporal_boundary.res");
    let adapter = Adapter::new(&m);
    for depth in 0..=5 {
        let value = (0..depth).fold(TimeExpr::At("tau".into()), |t, _| {
            TimeExpr::After(Box::new(t))
        });
        let mut base = experiment(&m, "base").input.clone();
        base[0].proposition.time = value.clone();
        let mut delta = experiment(&m, "delta").input.clone();
        let Term::RecordLiteral { fields, .. } = &mut delta[0].args[0] else {
            panic!("TimeBox")
        };
        fields[0].value = value.to_term();
        let all = base.iter().chain(&delta).cloned().collect::<Vec<_>>();
        let expected = close_program_reference(all.clone(), &m, &mut Clock::default()).unwrap();
        assert_eq!(expected.derivations.len(), 1);
        let binding = &expected.derivations[0]
            .record
            .substitution
            .bindings
            .iter()
            .find(|b| b.parameter == "#Timed:shared:t")
            .unwrap()
            .value;
        assert_eq!(*binding, SubstitutionValue::Time(value));
        let full_guarded = close(all.clone(), &adapter.rules, &mut Clock::default());
        equivalent(&adapter.normalize(&full_guarded), &expected);
        // Pivot at either leaf, either insertion order, or both leaves together.
        for (initial, added) in [
            (base.clone(), delta.clone()),
            (delta.clone(), base.clone()),
            (vec![], all.clone()),
            (vec![], all.iter().rev().cloned().collect()),
        ] {
            let mut clock = Clock::default();
            let mut inc = IncrementalClosure::new(initial, &adapter.rules, &mut clock);
            inc.update(added, &mut clock);
            equivalent(&adapter.normalize(inc.closure()), &expected);
            assert_eq!(inc.closure().derivations.len(), 1);
        }
        for (initial, added) in [
            (base.clone(), delta.clone()),
            (delta.clone(), base.clone()),
            (vec![], all.clone()),
        ] {
            let mut clock = Clock::default();
            let mut state = IncrementalProgramClosure::new(initial, &m, &mut clock).unwrap();
            state.update(added, &mut clock);
            equivalent(state.closure(), &expected);
        }
    }
    println!(
        "Canonical Time bindings: six constructor depths, both pivots and four update orders; exactly one justification"
    );
}

#[test]
fn canonical_bindings_keep_distinct_witnesses_rules_and_program_origins() {
    let mut m = module("dynamic_incremental_temporal_boundary.res");
    // Two distinct Programs have identical member clauses: provenance must differ.
    let original = m
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Program(p) => Some(p.clone()),
            _ => None,
        })
        .unwrap();
    let mut second = original;
    second.identity = "OtherTimed".into();
    let mut trigger = m
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == "run" => Some(v.clone()),
            _ => None,
        })
        .unwrap();
    trigger.name = "otherRun".into();
    trigger.program.as_mut().unwrap().name = second.identity.clone();
    m.declarations.push(Declaration::Program(second));
    m.declarations.push(Declaration::Verb(trigger));
    let mut all = experiment(&m, "base").input.clone();
    let mut other = all[0].clone();
    other.verb = "otherRun".into();
    all.push(other);
    let evidence = experiment(&m, "delta").input[0].clone();
    let mut distinct = evidence.clone();
    distinct.proposition.time = TimeExpr::After(Box::new(distinct.proposition.time));
    all.extend([evidence, distinct]);
    let adapter = Adapter::new(&m);
    let expected = close_program_reference(all.clone(), &m, &mut Clock::default()).unwrap();
    assert_eq!(expected.derivations.len(), 4); // two Programs x two witnesses.
    equivalent(
        &close_program(all.clone(), &m, &mut Clock::default()).unwrap(),
        &expected,
    );
    let mut program_clock = Clock::default();
    let mut program_state = IncrementalProgramClosure::new(vec![], &m, &mut program_clock).unwrap();
    program_state.update(all.clone(), &mut program_clock);
    equivalent(program_state.closure(), &expected);
    let mut clock = Clock::default();
    let mut inc = IncrementalClosure::new(vec![], &adapter.rules, &mut clock);
    inc.update(all.clone(), &mut clock);
    check(&m, &adapter, &inc, &all);
    assert_eq!(inc.closure().derivations.len(), 4);
    for program in ["Timed", "OtherTimed"] {
        let proofs = expected
            .derivations
            .iter()
            .filter(|d| program_rule_origin(&m, &d.record.rule).unwrap().program == program)
            .collect::<Vec<_>>();
        assert_eq!(proofs.len(), 2);
        assert_ne!(proofs[0].record.antecedents, proofs[1].record.antecedents);
    }
}

#[test]
fn program_derivative_exhaustively_preserves_448_input_splits_and_proof_multiplicity() {
    let m = module("dynamic_incremental_programs.res");
    let universe = ["base", "activate", "late", "alternative", "negativeTrigger"]
        .into_iter()
        .flat_map(|n| experiment(&m, n).input.clone())
        .collect::<Vec<_>>();
    let compare = |actual: &Closure, seeds: Vec<Event>| {
        let expected = close_program_reference(seeds, &m, &mut Clock::default()).unwrap();
        equivalent(actual, &expected);
    };
    let mut comparisons = 0;
    for mask in 0..(1 << universe.len()) {
        let seeds = universe
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect::<Vec<_>>();
        let mut clock = Clock::default();
        let initial = IncrementalProgramClosure::new(seeds.clone(), &m, &mut clock).unwrap();
        compare(initial.closure(), seeds.clone());
        comparisons += 1;
        for event in &universe {
            let mut inc = initial.clone();
            let mut clock = Clock(clock.0);
            let before = inc.closure().derivations.clone();
            let delta = inc.update(vec![event.clone()], &mut clock);
            assert!(before.iter().all(|d| inc.closure().derivations.contains(d)));
            let mut all = seeds.clone();
            all.push(event.clone());
            compare(inc.closure(), all);
            assert_eq!(
                inc.closure().derivations.len(),
                before.len() + delta.derivations.len()
            );
            let before_clock = clock.0;
            let duplicate = inc.update(vec![event.clone()], &mut clock);
            assert!(
                duplicate.known.is_empty()
                    && duplicate.derivations.is_empty()
                    && duplicate.activations.is_empty()
            );
            assert_eq!(duplicate.stats.rules_examined, 0);
            assert_eq!(clock.0, before_clock);
            comparisons += 1;
        }
    }
    assert_eq!(comparisons, 448);
    println!(
        "448 production Program comparisons: exact facts, types, times, witnesses, bindings and proof multiplicities; stable previous stamps"
    );
}

#[test]
fn program_derivative_small_product_reaches_the_same_fixed_point_and_244_proofs() {
    let m = module("peano_polynomial_product_scenario.res");
    let seeds = experiment(&m, "partialProduct").input.clone();
    let reference = close_program_reference(seeds.clone(), &m, &mut Clock::default()).unwrap();
    let cold = close_program(seeds.clone(), &m, &mut Clock::default()).unwrap();
    equivalent(&cold, &reference);
    let (trigger, base): (Vec<_>, Vec<_>) = seeds
        .into_iter()
        .partition(|e| e.verb == "multiplyPolynomialFromP");
    let mut clock = Clock::default();
    let mut inc = IncrementalProgramClosure::new(base, &m, &mut clock).unwrap();
    let change = inc.update(trigger, &mut clock);
    equivalent(inc.closure(), &reference);
    assert_eq!(
        (inc.closure().known.len(), inc.closure().derivations.len()),
        (223, 244)
    );
    assert!(!change.activations.is_empty());
    for proof in &inc.closure().derivations {
        if let Some(origin) = program_rule_origin(&m, &proof.record.rule) {
            assert!(m.origin(&origin.rule).is_some());
        }
    }
    println!(
        "FIXED POINT: small product 223 events / 244 identical justifications; {} new activations, {} full initializations, {} rules examined",
        change.activations.len(),
        change.full_initializations,
        change.stats.rules_examined
    );
}

#[test]
fn program_derivative_keeps_proof_only_changes_out_of_event_frontier() {
    let m = module("dynamic_incremental_programs.res");
    let mut clock = Clock::default();
    let mut inc =
        IncrementalProgramClosure::new(experiment(&m, "base").input.clone(), &m, &mut clock)
            .unwrap();
    let activation = inc.update(experiment(&m, "activate").input.clone(), &mut clock);
    assert!(
        activation
            .activations
            .iter()
            .any(|a| a.behavior.name == "Root")
    );
    assert!(
        activation
            .activations
            .iter()
            .any(|a| a.behavior.name == "Branch")
    );
    let copies = |c: &Closure, v: &str| c.derivations.iter().filter(|d| d.event.verb == v).count();
    let finished = copies(inc.closure(), "finished");
    let delta = inc.update(experiment(&m, "alternative").input.clone(), &mut clock);
    assert_eq!(copies(inc.closure(), "shared"), 2);
    assert_eq!(copies(inc.closure(), "finished"), finished);
    assert!(
        !delta
            .known
            .iter()
            .any(|e| e.verb == "shared" || e.verb == "finished")
    );
    assert!(delta.derivations.iter().any(|d| d.event.verb == "shared"));
    assert!(delta.activations.is_empty());
    println!("DeltaDerivations retained; an old shared Event is not rescheduled");
}
