use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{Closure, DerivationTime, Experiment, Module, Program, Rule, RuleBody, Type};
use residuum::{DerivationClock, close_program};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(
            self.0 < 5000,
            "unexpected proof growth in finite multiplication"
        );
        DerivationTime {
            name: format!("multiplication-audit-{}", self.0),
        }
    }
}
fn load(file: &str) -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments")
            .join(file),
    )
    .unwrap()
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
fn rule<'a>(module: &'a Module, name: &str) -> &'a Rule {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == name => Some(r),
            _ => None,
        })
        .unwrap()
}
fn run(module: &Module, name: &str) -> Closure {
    close_program(
        experiment(module, name).input.clone(),
        module,
        &mut Clock(0),
    )
    .unwrap()
}
fn peano(n: usize) -> TimeExpr {
    // This independent oracle supplies no arithmetic facts or output to Residuum.
    (0..n).fold(TimeExpr::At("zeroTime".into()), |t, _| {
        TimeExpr::After(Box::new(t))
    })
}
fn product_event<'a>(closure: &'a Closure, request: &str) -> &'a Event {
    let events: Vec<_> = closure
        .known
        .iter()
        .filter(|e| e.verb == "product" && e.args == vec![Term::Const(request.into())])
        .collect();
    assert_eq!(events.len(), 1, "exactly one product for {request}");
    events[0]
}
fn ground(term: &Term) -> bool {
    match term {
        Term::Var(_) => false,
        Term::Constructor { argument, .. } => ground(argument),
        Term::RecordLiteral { fields, .. } => fields.iter().all(|f| ground(&f.value)),
        _ => true,
    }
}
fn check_provenance(module: &Module, closure: &Closure) {
    let mut compiled = BTreeMap::new();
    for d in &module.declarations {
        if let Declaration::Program(p) = d {
            for r in compile_program(p, module).unwrap() {
                compiled.insert(r.name.clone(), r);
            }
        }
    }
    for proof in &closure.derivations {
        assert!(proof.event.transition.is_none());
        assert!(proof.event.args.iter().all(ground));
        assert!(
            proof
                .record
                .antecedents
                .iter()
                .all(|a| closure.known.contains(a))
        );
        let producer = if let Some(origin) = program_rule_origin(module, &proof.record.rule) {
            let expected_module = match origin.program.as_str() {
                "Multiply" | "MultiplyStep" | "AdditionContinuation" => "PeanoMultiplication",
                "Addition" | "LeftWalk" | "RightWalk" => "PeanoAfter",
                other => panic!("unexpected activated program {other}"),
            };
            assert_eq!(origin.defining_module, expected_module);
            assert_eq!(origin.rule_origin.module, expected_module);
            compiled.get(&proof.record.rule).unwrap()
        } else {
            assert!(["observe", "observeProduct"].contains(&proof.record.rule.as_str()));
            rule(module, &proof.record.rule)
        };
        let replay = residuum::instantiate_from_known(producer, &proof.record.antecedents);
        assert!(
            replay.iter().any(|p| p.event == proof.event
                && p.event.proposition == proof.event.proposition
                && p.record == proof.record),
            "replay of {}",
            proof.record.rule
        );
    }
}
fn check_case(module: &Module, a: usize, b: usize) -> (Closure, TimeExpr) {
    let name = format!("product{a}{b}");
    let input = experiment(module, &name);
    // Only requests and unary operand spines are evidence.
    assert!(
        input
            .input
            .iter()
            .all(|e| ["start", "successor", "end", "multiply"].contains(&e.verb.as_str()))
    );
    let closure = run(module, &name);
    assert!(input.input.iter().all(|e| closure.known.contains(e)));
    assert!(closure.known.contains(&input.goal));
    let output = product_event(&closure, &format!("r{a}{b}"));
    assert_eq!(output.proposition.time, peano(a * b));
    println!("{a} * {b} -> {}", render_event(output));
    let time = output.proposition.time.clone();
    let calls: Vec<_> = closure.known.iter().filter(|e| e.verb == "add").collect();
    assert_eq!(calls.len(), b);
    assert_eq!(closure.known.iter().filter(|e| e.verb == "sum").count(), b);
    let mut positions = BTreeSet::new();
    for call in calls {
        assert_eq!(call.args[0], Term::Const("nZero".into()));
        let Term::RecordLiteral { entity, fields } = &call.args[2] else {
            panic!("expected invocation record")
        };
        assert_eq!(entity, "AdditionInvocation");
        assert_eq!(
            fields
                .iter()
                .find(|f| f.field == "operation")
                .unwrap()
                .value,
            Term::Const(format!("r{a}{b}"))
        );
        positions.insert(
            fields
                .iter()
                .find(|f| f.field == "position")
                .unwrap()
                .value
                .clone(),
        );
        assert!(closure.derivations.iter().any(
            |d| d.event == *call && d.record.rule == "AdditionContinuation::0::requestAddition"
        ));
    }
    assert_eq!(positions.len(), b);
    let steps: Vec<_> = closure
        .derivations
        .iter()
        .filter(|d| d.record.rule == "AdditionContinuation::1::acceptAddition")
        .collect();
    assert_eq!(steps.len(), b);
    for step in steps {
        let before = step
            .record
            .antecedents
            .iter()
            .find(|e| e.verb == "awaitingSum")
            .unwrap();
        let sum = step
            .record
            .antecedents
            .iter()
            .find(|e| e.verb == "sum")
            .unwrap();
        assert_eq!(before.args[4], sum.args[0]);
        assert_eq!(step.event.proposition.time, sum.proposition.time);
        assert_eq!(
            sum.proposition.time,
            (0..a).fold(before.proposition.time.clone(), |t, _| TimeExpr::After(
                Box::new(t)
            ))
        );
    }
    check_provenance(module, &closure);
    (closure, time)
}

#[test]
fn peano_multiplication_four_by_four_constructs_sixteen_successors() {
    let module = load("peano_multiplication.res");
    let before = module.clone();
    let (closure, output) = check_case(&module, 4, 4);
    assert_eq!(output, peano(16));
    assert_eq!(closure.derivations.len(), 47);
    assert_eq!(module, before);
}

#[test]
fn peano_multiplication_all_twenty_five_pairs_satisfy_recursive_equations() {
    let module = load("peano_multiplication.res");
    let mut actual = BTreeMap::new();
    for a in 0..5 {
        for b in 0..5 {
            actual.insert((a, b), check_case(&module, a, b).1);
        }
    }
    for a in 0..5 {
        assert_eq!(actual[&(a, 0)], peano(0));
        assert_eq!(actual[&(a, 1)], peano(a));
        for b in 0..4 {
            // prod(a,S(b)) = sum(prod(a,b),a), checked on computed indices.
            assert_eq!(
                actual[&(a, b + 1)],
                (0..a).fold(actual[&(a, b)].clone(), |t, _| TimeExpr::After(Box::new(t)))
            );
        }
        for b in 0..5 {
            assert_eq!(actual[&(a, b)], actual[&(b, a)]);
        }
    }
}

#[test]
fn peano_multiplication_beta_captures_and_concurrent_requests_keep_calls_separate() {
    let module = load("peano_multiplication.res");
    let partial = program(&module, "ProductStepPartial");
    assert_eq!(
        partial
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["cursor", "t"]
    );
    assert_eq!(partial.rule_uses.len(), 2);
    for member in &partial.rule_uses {
        assert!(
            member
                .bindings
                .iter()
                .any(|b| b.parameter == "a" && b.value == Term::Const("nFour".into()))
        );
        assert!(
            member
                .bindings
                .iter()
                .any(|b| b.parameter == "t" && b.value == Term::Var("t".into()))
        );
    }
    let capture = partial.rule_uses[0]
        .bindings
        .iter()
        .find(|b| b.parameter == "call")
        .unwrap();
    let Term::RecordLiteral { fields, .. } = &capture.value else {
        panic!("record capture")
    };
    assert_eq!(fields[0].value, Term::Const("r44".into()));
    assert_eq!(fields[1].value, Term::Var("cursor".into()));
    let ground_program = program(&module, "ProductStepGround");
    assert!(ground_program.parameters.is_empty());
    assert_eq!(
        ground_program.rule_uses,
        program(&module, "ProductStepDirect").rule_uses
    );
    let ground_rules = compile_program(ground_program, &module).unwrap();
    let RuleBody::HornClause(clause) = &ground_rules[0].body else {
        panic!("Horn")
    };
    assert_eq!(clause.consequent.proposition.time, peano(0));
    assert!(ground(&clause.consequent.args[4]));
    let root = compile_program(program(&module, "MultiplyByFour"), &module).unwrap();
    for b in 0..5 {
        let witness = &experiment(&module, &format!("product4{b}")).input;
        assert_eq!(residuum::instantiate_from_known(&root[0], witness).len(), 1);
    }
    let closure = run(&module, "concurrentProducts");
    assert_eq!(product_event(&closure, "r44").proposition.time, peano(16));
    assert_eq!(product_event(&closure, "r23").proposition.time, peano(6));
    assert_eq!(closure.known.iter().filter(|e| e.verb == "add").count(), 7);
    check_provenance(&module, &closure);
}

#[test]
fn peano_multiplication_absence_and_negative_requests_do_not_activate() {
    let module = load("peano_multiplication.res");
    for name in ["absentProduct", "negativeProduct"] {
        let closure = run(&module, name);
        assert!(closure.derivations.is_empty());
        assert_eq!(
            closure.known.iter().cloned().collect::<BTreeSet<_>>(),
            experiment(&module, name).input.iter().cloned().collect()
        );
    }
    assert!(
        run_forward_experiments(&module)
            .unwrap_err()
            .to_string()
            .contains("acyclic rule dependencies only")
    );
}

#[test]
fn peano_computed_time_indices_export_as_coefficient_and_exponent_arguments() {
    let module = load("peano_polynomial_time_boundary.res");
    let closure = run(&module, "computedIndex");
    let sum = closure.known.iter().find(|e| e.verb == "sum").unwrap();
    assert_eq!(sum.proposition.time, peano(5));
    assert_eq!(
        closure
            .known
            .iter()
            .filter(|e| e.verb == "coefficient")
            .count(),
        2
    );
    for name in ["exportCoefficient", "exportExponent"] {
        let export = rule(&module, name);
        assert_eq!(
            export.parameters.iter().find(|p| p.name == "t").unwrap().ty,
            Type::PropositionTime
        );
        let exports = residuum::instantiate_from_known(export, std::slice::from_ref(sum));
        assert_eq!(exports.len(), 1);
        assert!(exports[0].event.args.contains(&peano(5).to_term()));
    }
    let copied = closure
        .known
        .iter()
        .find(|e| e.verb == "copiedTimeValue")
        .unwrap();
    assert_eq!(copied.args, vec![Term::Const("zeroTime".into())]);
    assert!(
        closure
            .derivations
            .iter()
            .any(|d| d.event == *copied && d.record.rule == "copyTimeValue")
    );
    assert!(
        !closure
            .known
            .contains(&experiment(&module, "computedIndex").goal)
    );
    println!(
        "computed index: {}; named argument copied from evidence: {}; computed coefficient/exponent exported structurally",
        render_event(sum),
        render_event(copied)
    );
}

#[test]
fn peano_time_is_not_a_nominal_natural_argument() {
    let error = residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/peano_polynomial_nominal_boundary.invalid.res"),
    )
    .unwrap_err();
    assert!(error.to_string().contains("not assignable"), "{error}");
    assert!(error.to_string().contains("PropositionTime"), "{error}");
    assert!(error.to_string().contains("Natural"), "{error}");
    println!("nominal boundary: {error}");
}

#[test]
fn peano_after_is_a_typed_argument_constructor_on_experimental_branch() {
    let source =
        include_str!("../../../../examples/experiments/peano_polynomial_after_argument.res");
    let module = residuum::parse_formal_to_typed(source).unwrap();
    let RuleBody::HornClause(c) = &rule(&module, "constructedArgument").body else {
        panic!("Horn")
    };
    assert_eq!(
        c.consequent.args[2],
        TimeExpr::After(Box::new(TimeExpr::Var("t".into()))).to_term()
    );
}
