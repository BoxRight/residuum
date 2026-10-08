use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{Closure, DerivationTime, Experiment, Module, Program, RuleBody, Type};
use residuum::{DerivationClock, close_program};
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = include_str!("../../../../examples/experiments/peano_after.res");

struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        // A failed termination invariant must not silently run forever.
        assert!(
            self.0 < 10_000,
            "unexpected proof growth in finite Peano fixture"
        );
        DerivationTime {
            name: format!("audit-{}", self.0),
        }
    }
}

fn experiment<'a>(module: &'a Module, name: &str) -> &'a Experiment {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}
fn program<'a>(module: &'a Module, name: &str) -> &'a Program {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Program(p) if p.identity == name => Some(p),
            _ => None,
        })
        .unwrap()
}
fn run(module: &Module, name: &str, offset: usize) -> Closure {
    close_program(
        experiment(module, name).input.clone(),
        module,
        &mut Clock(offset),
    )
    .unwrap()
}
fn peano(n: usize) -> TimeExpr {
    // Independent verifier only; no numeric result is supplied to the engine.
    (0..n).fold(TimeExpr::At("zeroTime".into()), |t, _| {
        TimeExpr::After(Box::new(t))
    })
}
fn results(closure: &Closure) -> BTreeMap<String, TimeExpr> {
    closure
        .known
        .iter()
        .filter(|e| e.verb == "sum")
        .map(|e| {
            let [Term::Const(request)] = e.args.as_slice() else {
                panic!("non-ground result")
            };
            (request.clone(), e.proposition.time.clone())
        })
        .collect()
}

#[test]
fn peano_after_computes_all_twenty_five_sums_with_recursive_programs() {
    let module = residuum::parse_formal_to_typed(SOURCE).unwrap();
    let original = module.clone();
    let closure = run(&module, "allPairs", 0);
    let actual = results(&closure);
    assert_eq!(actual.len(), 25);
    for left in 0..5 {
        for right in 0..5 {
            assert_eq!(actual[&format!("r{left}{right}")], peano(left + right));
            assert_eq!(
                actual[&format!("r{left}{right}")],
                actual[&format!("r{right}{left}")]
            );
            println!(
                "{left} + {right} -> {}",
                render_event(
                    closure
                        .known
                        .iter()
                        .find(|e| {
                            e.verb == "sum"
                                && e.args == vec![Term::Const(format!("r{left}{right}"))]
                        })
                        .unwrap()
                )
            );
        }
    }
    // Check Peano equations directly on independently computed results.
    for right in 0..5 {
        assert_eq!(actual[&format!("r0{right}")], peano(right));
        for left in 0..4 {
            assert_eq!(
                actual[&format!("r{}{right}", left + 1)],
                TimeExpr::After(Box::new(actual[&format!("r{left}{right}")].clone()))
            );
        }
    }
    // Eight successors are constructed although the input spines stop at four.
    assert_eq!(actual["r44"], peano(8));
    assert!(
        closure
            .known
            .contains(&experiment(&module, "allPairs").goal)
    );
    for input in &experiment(&module, "allPairs").input {
        assert!(closure.known.contains(input));
    }
    assert_eq!(module, original);
    assert_eq!(closure.known.len(), 245);
    assert_eq!(closure.derivations.len(), 200);

    let mut counts = BTreeMap::<String, usize>::new();
    for proof in &closure.derivations {
        assert!(proof.event.transition.is_none());
        assert!(
            proof
                .record
                .antecedents
                .iter()
                .all(|a| closure.known.contains(a))
        );
        if proof.record.rule == "observe" {
            *counts.entry("observe".into()).or_default() += 1;
            continue;
        }
        let origin = program_rule_origin(&module, &proof.record.rule).unwrap();
        assert_eq!(origin.defining_module, "PeanoAfter");
        assert_eq!(origin.rule_origin.module, "PeanoAfter");
        let rule = module
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Rule(r) if r.name == origin.rule => Some(r),
                _ => None,
            })
            .unwrap();
        // Replay source rules to validate substitutions, witnesses and times.
        let replay = residuum::instantiate_from_known(rule, &proof.record.antecedents);
        assert!(replay.iter().any(|p| p.event == proof.event
            && p.event.proposition == proof.event.proposition
            && p.record.antecedents == proof.record.antecedents));
        let trigger_verb = match origin.program.as_str() {
            "Addition" => "add",
            "LeftWalk" => "leftWalk",
            "RightWalk" => "rightWalk",
            name => panic!("unexpected activated program {name}"),
        };
        assert!(
            proof
                .record
                .antecedents
                .iter()
                .any(|a| a.verb == trigger_verb)
        );
        let trigger = proof
            .record
            .antecedents
            .iter()
            .find(|a| a.verb == trigger_verb)
            .unwrap();
        assert!(
            proof
                .record
                .substitution
                .bindings
                .iter()
                .any(|b| b.parameter == format!("#{}:shared:t", origin.program)
                    && b.value == SubstitutionValue::Time(trigger.proposition.time.clone()))
        );
        *counts.entry(origin.rule).or_default() += 1;
    }
    assert_eq!(
        counts,
        BTreeMap::from([
            ("initialize".into(), 25),
            ("leftSuccessor".into(), 50),
            ("changePhase".into(), 25),
            ("rightSuccessor".into(), 50),
            ("finish".into(), 25),
            ("observe".into(), 25),
        ])
    );
    // The output number depends on proposition structure, not stamping policy.
    let other_clock = run(&module, "allPairs", 1000);
    assert_eq!(closure.known, other_clock.known);
    assert_eq!(
        closure
            .derivations
            .iter()
            .map(|d| (&d.event, &d.record))
            .collect::<Vec<_>>(),
        other_clock
            .derivations
            .iter()
            .map(|d| (&d.event, &d.record))
            .collect::<Vec<_>>()
    );
    assert_ne!(
        closure.derivations[0].derived_at,
        other_clock.derivations[0].derived_at
    );
}

#[test]
fn peano_after_partial_application_preserves_both_rules_and_temporal_beta() {
    let module = residuum::parse_formal_to_typed(SOURCE).unwrap();
    let partial = program(&module, "LeftTwo");
    assert_eq!(
        partial
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["right", "request", "cursor", "t"]
    );
    assert_eq!(partial.rule_uses.len(), 2);
    for member in &partial.rule_uses {
        assert!(
            member
                .bindings
                .iter()
                .any(|b| b.parameter == "left" && b.value == Term::Const("nTwo".into()))
        );
        assert!(
            member
                .bindings
                .iter()
                .any(|b| b.parameter == "t" && b.value == Term::Var("t".into()))
        );
    }
    let ground = program(&module, "LeftGround");
    assert!(ground.parameters.is_empty());
    assert_eq!(ground.rule_uses, program(&module, "LeftDirect").rule_uses);
    let rules = compile_program(ground, &module).unwrap();
    assert_eq!(rules.len(), 2);
    for rule in &rules {
        assert_eq!(rule.parameters.len(), 1);
        assert_eq!(rule.parameters[0].ty, Type::Entity("Cursor".into()));
    }
    let RuleBody::HornClause(first) = &rules[0].body else {
        panic!("expected Horn")
    };
    assert_eq!(first.consequent.proposition.time, peano(1));
    let RuleBody::HornClause(second) = &rules[1].body else {
        panic!("expected Horn")
    };
    assert_eq!(second.consequent.proposition.time, peano(0));
    // No construction of a Program, including its specializations, is a trigger.
    for name in ["absent", "negative"] {
        let closure = run(&module, name, 0);
        assert!(closure.derivations.is_empty());
        assert!(results(&closure).is_empty());
        assert_eq!(
            closure.known.iter().cloned().collect::<BTreeSet<_>>(),
            experiment(&module, name).input.iter().cloned().collect()
        );
    }
}

#[test]
fn peano_after_documents_the_cli_acyclicity_boundary() {
    let module = residuum::parse_formal_to_typed(SOURCE).unwrap();
    let error = run_forward_experiments(&module).unwrap_err();
    assert!(
        error.to_string().contains("acyclic rule dependencies only"),
        "{error}"
    );
}
