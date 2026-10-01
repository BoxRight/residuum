//! Programs, data, and goals are defined by .res fixtures. Rust compares the
//! module-aware forward engine against existing rule inference and controls.
use super::*;
use crate::typed::{PropositionKind, StateTime, SubstitutionValue};
use crate::{close_program, parse_formal_to_typed};

const ASSOCIATED: &str = include_str!("../../examples/experiments/sells_associated_program.res");
const BASELINE: &str = include_str!("../../examples/experiments/sells_program.res");
const DERIVED: &str =
    include_str!("../../examples/experiments/sells_derived_associated_program.res");
const UNIT: &str = include_str!("../../examples/experiments/sells_unit_associated_program.res");
const WAITS: &str = include_str!("../../examples/experiments/associated_program_waits.res");
const EFFECT: &str = include_str!("../../examples/experiments/effect_program_inactive.res");
const COMPOSED: &str = include_str!("../../examples/experiments/sells_composed_programs.res");
const COMPOSED_INLINE: &str = include_str!("../../examples/experiments/sells_composed_inline.res");

fn result<'a>(results: &'a [ExperimentResult], name: &str) -> &'a ExperimentResult {
    results.iter().find(|r| r.experiment.name == name).unwrap()
}

fn exact_event(a: &Event, b: &Event) {
    assert_eq!(a, b);
    assert_eq!(a.proposition, b.proposition);
    assert_eq!(a.transition, b.transition);
}

fn equivalent(a: &Closure, b: &Closure) {
    assert_eq!(
        a.known.iter().cloned().collect::<BTreeSet<_>>(),
        b.known.iter().cloned().collect()
    );
    for event in &a.known {
        exact_event(event, b.known.iter().find(|e| *e == event).unwrap());
    }
    assert_eq!(a.derivations.len(), b.derivations.len());
    for proof in &a.derivations {
        let other = b
            .derivations
            .iter()
            .find(|p| p.event == proof.event && p.record == proof.record)
            .unwrap();
        exact_event(&proof.event, &other.event);
        for (witness, compared) in proof
            .record
            .antecedents
            .iter()
            .zip(&other.record.antecedents)
        {
            exact_event(witness, compared);
        }
        // Clock stamps depend on scheduling; both must be present and unique.
        assert!(!proof.derived_at.name.is_empty());
    }
    assert_eq!(
        a.derivations
            .iter()
            .map(|p| &p.derived_at.name)
            .collect::<BTreeSet<_>>()
            .len(),
        a.derivations.len()
    );
}

#[test]
fn associated_seed_activates_program_with_baseline_events_bindings_times_and_provenance() {
    let module = parse_formal_to_typed(ASSOCIATED).unwrap();
    let unchanged = module.clone();
    let actual = run_forward_experiments(&module).unwrap();
    let expected = run_experiments(&parse_formal_to_typed(BASELINE).unwrap()).unwrap();
    for (a, b) in actual.iter().zip(&expected) {
        assert_eq!(a.horn_goal, b.horn_goal);
        equivalent(&a.closure, &b.closure);
        assert!(a.residual.is_none() && a.four.is_none());
    }
    assert!(result(&actual, "real").horn_goal);
    assert!(result(&actual, "reuse").horn_goal);
    assert!(result(&actual, "hypothetical").closure.known.is_empty());
    // The buyer, as in the baseline, owns the transferred Thing.
    let real = result(&actual, "real");
    let owns = real
        .closure
        .known
        .iter()
        .find(|e| e.verb == "owns")
        .unwrap();
    assert_eq!(owns.args[0], Term::Const("pedro".into()));
    let give = real
        .closure
        .known
        .iter()
        .find(|e| e.verb == "give")
        .unwrap();
    let transition = give.transition.as_ref().unwrap();
    assert_eq!(transition.input, StateTime::At(TimeExpr::At("tau".into())));
    assert_eq!(
        transition.output,
        StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("tau".into()))))
    );
    assert_eq!(module, unchanged);
}

#[test]
fn associated_derived_instance_activates_the_same_downstream_program() {
    let actual = run_forward_experiments(&parse_formal_to_typed(DERIVED).unwrap()).unwrap();
    let seed = run_forward_experiments(&parse_formal_to_typed(ASSOCIATED).unwrap()).unwrap();
    let a = result(&actual, "real");
    let b = result(&seed, "real");
    assert!(a.horn_goal);
    let sells = a
        .closure
        .derivations
        .iter()
        .find(|p| p.event.verb == "sells")
        .unwrap();
    assert_eq!(sells.event.proposition.kind, PropositionKind::Derived);
    assert_eq!(sells.record.rule, "confirm");
    assert_eq!(sells.record.antecedents[0].verb, "confirmedSale");
    let downstream = |closure: &Closure| Closure {
        known: closure
            .known
            .iter()
            .filter(|e| e.verb != "sells" && e.verb != "confirmedSale")
            .cloned()
            .collect(),
        derivations: closure
            .derivations
            .iter()
            .filter(|p| p.event.verb != "sells")
            .cloned()
            .collect(),
    };
    let mut derived_downstream = downstream(&a.closure);
    // The only provenance difference is the incoming sells category. Preserve
    // and assert it, then compare logical proof identity against the real seed.
    for proof in &mut derived_downstream.derivations {
        for witness in &mut proof.record.antecedents {
            if witness.verb == "sells" {
                assert_eq!(witness.proposition.kind, PropositionKind::Derived);
                witness.proposition.kind = PropositionKind::Seeded;
            }
        }
    }
    equivalent(&derived_downstream, &downstream(&b.closure));
    assert!(result(&actual, "absent").closure.known.is_empty());
}

#[test]
fn unit_program_requires_positive_known_instance_and_specializes_each_call() {
    let module = parse_formal_to_typed(UNIT).unwrap();
    let actual = run_forward_experiments(&module).unwrap();
    let real = result(&actual, "real");
    assert!(real.horn_goal);
    assert_eq!(real.closure.known.len(), 4);
    let proof = real
        .closure
        .derivations
        .iter()
        .find(|p| p.record.rule == "sellsProgram")
        .unwrap();
    assert_eq!(proof.record.antecedents, real.experiment.input);
    assert_eq!(proof.record.substitution.bindings.len(), 4);
    assert!(
        proof
            .record
            .substitution
            .bindings
            .iter()
            .any(|b| b.parameter == "t"
                && b.value == SubstitutionValue::Time(TimeExpr::At("tau".into())))
    );
    assert!(result(&actual, "absent").closure.known.is_empty());
    let negative = result(&actual, "negativeOnly");
    assert!(!negative.horn_goal);
    assert!(negative.closure.derivations.is_empty());
    let two = result(&actual, "twoInstances");
    assert!(two.horn_goal);
    let calls: Vec<_> = two
        .closure
        .derivations
        .iter()
        .filter(|p| p.record.rule == "sellsProgram")
        .collect();
    assert_eq!(calls.len(), 2);
    for call in calls {
        let [trigger] = call.record.antecedents.as_slice() else {
            panic!("one trigger per call")
        };
        assert_eq!(call.event.args, trigger.args);
        assert_eq!(
            call.event.proposition.time,
            TimeExpr::After(Box::new(trigger.proposition.time.clone()))
        );
    }
    // No ordinary lambda can infer the unbound parameters of the I body.
    let rules: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let plain = close(real.experiment.input.clone(), &rules, &mut Clock::default());
    assert_eq!(plain.known, real.experiment.input);
    assert!(plain.derivations.is_empty());
}

#[test]
fn activated_program_waits_for_compatible_evidence_and_keeps_trigger_provenance() {
    let results = run_forward_experiments(&parse_formal_to_typed(WAITS).unwrap()).unwrap();
    let later = result(&results, "later");
    assert!(later.horn_goal);
    let proof = later
        .closure
        .derivations
        .iter()
        .find(|p| p.record.rule == "onStart")
        .unwrap();
    assert_eq!(
        proof
            .record
            .antecedents
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        ["starts", "ready"]
    );
    assert_eq!(later.closure.derivations.len(), 2);
    for name in ["wrongTime", "noInstance"] {
        let case = result(&results, name);
        assert!(!case.horn_goal);
        assert!(case.closure.known.iter().all(|e| e.verb != "done"));
    }
    let module = parse_formal_to_typed(EFFECT).unwrap();
    let effect = run_forward_experiments(&module).unwrap().pop().unwrap();
    assert!(!effect.horn_goal);
    assert_eq!(effect.closure.derivations.len(), 1);
    assert_eq!(
        effect.closure.derivations[0].event.proposition.kind,
        PropositionKind::Effect
    );
    assert!(effect.closure.known.iter().all(|e| e.verb != "observed"));
    // The rules-only engine is still explicitly available with its old policy.
    let direct = close_program(effect.experiment.input, &module, &mut Clock::default()).unwrap();
    equivalent(&direct, &effect.closure);
}

#[test]
fn composed_verb_programs_activate_derived_owns_and_match_inline_beta_reduction() {
    let module = parse_formal_to_typed(COMPOSED).unwrap();
    let original = module.clone();
    let inline = parse_formal_to_typed(COMPOSED_INLINE).unwrap();
    let actual = run_forward_experiments(&module).unwrap();
    let expected = run_forward_experiments(&inline).unwrap();
    let rule = |m: &Module| {
        m.declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Rule(r) if r.name == "protectionFromCharlie" => Some(r.clone()),
                _ => None,
            })
            .unwrap()
    };
    // The associated target is a beta-reduced alias of the reusable lambda.
    // Its whole typed rule agrees with the independently written inline rule.
    assert_eq!(rule(&module), rule(&inline));
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(&expected) {
        assert_eq!(a.experiment, b.experiment);
        assert_eq!(a.horn_goal, b.horn_goal);
        equivalent(&a.closure, &b.closure);
    }
    let real = result(&actual, "real");
    assert!(real.horn_goal);
    assert_eq!(real.closure.known.len(), 3);
    assert_eq!(real.closure.derivations.len(), 2);
    let ownership = real
        .closure
        .derivations
        .iter()
        .find(|p| p.record.rule == "sellsProgram")
        .unwrap();
    assert_eq!(ownership.event.verb, "owns");
    assert_eq!(ownership.event.proposition.kind, PropositionKind::Derived);
    assert_eq!(ownership.record.antecedents, real.experiment.input);
    let protection = real
        .closure
        .derivations
        .iter()
        .find(|p| p.record.rule == "protectionFromCharlie")
        .unwrap();
    assert_eq!(protection.event.verb, "notDo");
    let [trigger] = protection.record.antecedents.as_slice() else {
        panic!("one derived owns instance activates protection")
    };
    exact_event(trigger, &ownership.event);
    assert_eq!(
        protection.event.args,
        [
            Term::Const("charlie".into()),
            Term::Const("molest".into()),
            ownership.event.args[1].clone(),
            ownership.event.args[0].clone(),
        ]
    );
    assert_eq!(
        protection.event.proposition.time,
        TimeExpr::After(Box::new(ownership.event.proposition.time.clone()))
    );
    assert!(
        protection
            .record
            .substitution
            .bindings
            .iter()
            .any(|binding| binding.parameter == "t"
                && binding.value
                    == SubstitutionValue::Time(ownership.event.proposition.time.clone()))
    );
    let ownership_stamp: usize = ownership
        .derived_at
        .name
        .strip_prefix('d')
        .unwrap()
        .parse()
        .unwrap();
    let protection_stamp: usize = protection
        .derived_at
        .name
        .strip_prefix('d')
        .unwrap()
        .parse()
        .unwrap();
    assert!(ownership_stamp < protection_stamp);
    // With program definitions excluded, ordinary rules cannot produce either
    // consequence. Both edges above therefore require association activation.
    let targets: BTreeSet<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) => v.program.as_ref().map(|r| r.name.as_str()),
            _ => None,
        })
        .collect();
    let ordinary: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) if !targets.contains(r.name.as_str()) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let without_activation = close(
        real.experiment.input.clone(),
        &ordinary,
        &mut Clock::default(),
    );
    assert_eq!(without_activation.known, real.experiment.input);
    assert!(without_activation.derivations.is_empty());
    assert_eq!(module, original);
}

#[test]
fn composed_programs_isolate_instances_and_do_not_reactivate_duplicate_seeds() {
    let module = parse_formal_to_typed(COMPOSED).unwrap();
    let results = run_forward_experiments(&module).unwrap();
    let absent = result(&results, "absent");
    assert!(absent.closure.known.is_empty());
    assert!(absent.closure.derivations.is_empty());
    let negative = result(&results, "negativeOnly");
    assert_eq!(negative.closure.known, negative.experiment.input);
    assert!(negative.closure.derivations.is_empty());
    let two = result(&results, "twoInstances");
    assert!(two.horn_goal);
    assert_eq!(two.closure.known.len(), 6);
    assert_eq!(two.closure.derivations.len(), 4);
    for seed in &two.experiment.input {
        let ownership = two
            .closure
            .derivations
            .iter()
            .find(|p| p.record.rule == "sellsProgram" && p.record.antecedents == [seed.clone()])
            .unwrap();
        assert_eq!(
            ownership.event.args,
            [seed.args[2].clone(), seed.args[1].clone()]
        );
        assert_eq!(
            ownership.event.proposition.time,
            TimeExpr::After(Box::new(seed.proposition.time.clone()))
        );
        let protection = two
            .closure
            .derivations
            .iter()
            .find(|p| {
                p.record.rule == "protectionFromCharlie"
                    && p.record.antecedents == [ownership.event.clone()]
            })
            .unwrap();
        assert_eq!(protection.event.args[2], seed.args[1]);
        assert_eq!(protection.event.args[3], seed.args[2]);
        assert_eq!(
            protection.event.proposition.time,
            TimeExpr::After(Box::new(TimeExpr::After(Box::new(
                seed.proposition.time.clone()
            ))))
        );
    }
    let mut duplicate_seeds = two.experiment.input.clone();
    duplicate_seeds.extend(two.experiment.input.clone());
    let repeated = close_program(duplicate_seeds, &module, &mut Clock::default()).unwrap();
    equivalent(&repeated, &two.closure);
}
