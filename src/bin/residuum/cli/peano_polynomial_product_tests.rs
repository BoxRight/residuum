use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{
    Antecedent, Closure, DerivationTime, Experiment, Module, Program, PropositionKind, Rule,
    RuleBody,
};
use residuum::{DerivationClock, close_program, instantiate_from_known};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn module() -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/peano_polynomial_product_scenario.res"),
    )
    .unwrap()
}
fn experiment<'a>(m: &'a Module, n: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == n => Some(e),
            _ => None,
        })
        .unwrap()
}
fn program<'a>(m: &'a Module, n: &str) -> &'a Program {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Program(p) if p.identity == n => Some(p),
            _ => None,
        })
        .unwrap()
}
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(
            self.0 < 4000,
            "unexpected growth in finite polynomial product"
        );
        DerivationTime {
            name: format!("polynomial-product-{}", self.0),
        }
    }
}
fn close_input(m: &Module, input: Vec<Event>) -> Closure {
    assert!(input.iter().all(|e| matches!(
        e.verb.as_str(),
        "inputCoefficient"
            | "exponent"
            | "firstFactorExponent"
            | "nextFactorExponent"
            | "lastFactorExponent"
            | "multiplyPolynomial"
            | "multiplyPolynomialFromP"
            | "addPolynomial"
    )));
    let c = close_program(input.clone(), m, &mut Clock::default()).unwrap();
    assert!(input.iter().all(|e| c.known.contains(e)));
    assert_eq!(c.known.iter().collect::<BTreeSet<_>>().len(), c.known.len());
    assert!(c.known.iter().all(|e| e.args.iter().all(Term::is_ground)));
    c
}
fn run(m: &Module, n: &str) -> Closure {
    let e = experiment(m, n);
    let before = e.input.clone();
    let c = close_input(m, before.clone());
    assert_eq!(e.input, before);
    println!(
        "{n}: {} facts, {} justifications",
        c.known.len(),
        c.derivations.len()
    );
    c
}
fn natural(t: &Term) -> usize {
    // Independent decoder only; it never adds evidence or computes for the engine.
    match t {
        Term::Const(z) if z == "zeroTime" => 0,
        Term::Constructor { argument, .. } => 1 + natural(argument),
        _ => panic!("not a ground Peano value: {t:?}"),
    }
}
fn outputs(c: &Closure, polynomial: &str) -> BTreeMap<usize, usize> {
    let mut result = BTreeMap::new();
    for e in c
        .known
        .iter()
        .filter(|e| e.verb == "coefficient" && e.args[0] == Term::Const(polynomial.into()))
    {
        assert_eq!(e.proposition.kind, PropositionKind::Derived);
        assert_eq!(e.proposition.time, TimeExpr::At("zeroTime".into()));
        assert!(e.transition.is_none());
        assert!(
            result
                .insert(natural(&e.args[1]), natural(&e.args[2]))
                .is_none(),
            "multiple coefficients per exponent"
        );
        println!("{}", render_event(e));
    }
    result
}
fn expected(values: &[usize]) -> BTreeMap<usize, usize> {
    values.iter().copied().enumerate().collect()
}

#[test]
fn polynomial_product_small_recursive_smoke_computes_all_three_coefficients() {
    let m = module();
    let c = run(&m, "partialProduct");
    assert_eq!(outputs(&c, "r"), expected(&[2, 5, 2]));
    assert!(c.known.contains(&experiment(&m, "partialProduct").goal));
    assert_eq!(c.known.iter().filter(|e| e.verb == "pairValue").count(), 4);
    assert_eq!(
        c.known
            .iter()
            .filter(|e| e.verb == "productFinished")
            .count(),
        3
    );
}

#[test]
fn polynomial_product_computes_complete_dense_convolution_on_the_verified_domain() {
    let m = module();
    let c = run(&m, "partialProduct");
    assert_eq!(outputs(&c, "r"), expected(&[2, 5, 2]));
    assert_eq!(c.known.iter().filter(|e| e.verb == "pairValue").count(), 4);
    assert_eq!(
        c.known
            .iter()
            .filter(|e| e.verb == "productFinished")
            .count(),
        3
    );
    assert_eq!(
        c.known.iter().filter(|e| e.verb == "productFold").count(),
        6
    );
    assert_eq!(
        c.known
            .iter()
            .filter(|e| e.verb == "rowContribution")
            .count(),
        6
    );
    assert!(
        c.derivations
            .iter()
            .any(|d| d.record.rule == "AdditionContinuation::1::acceptAddition")
    );
    assert!(
        c.derivations
            .iter()
            .any(|d| d.record.rule.ends_with("::advanceRow"))
    );
    assert!(
        c.derivations
            .iter()
            .any(|d| d.record.rule == "FoldAddition::3::acceptRowSum")
    );
    let max_pair = c
        .known
        .iter()
        .filter(|e| e.verb == "pairValue")
        .map(|e| natural(&e.args[4]))
        .max()
        .unwrap();
    assert_eq!(max_pair, 4); // Products use dynamically constructed operands.
}

#[test]
fn polynomial_product_zero_is_complete_without_row_additions() {
    let m = module();
    let c = run(&m, "zeroProduct");
    assert_eq!(outputs(&c, "r"), expected(&[0, 0, 0]));
    assert!(!c.known.iter().any(|e| e.verb == "foldAddition"));
}

#[test]
fn polynomial_product_unit_preserves_every_coefficient() {
    let m = module();
    let c = run(&m, "unitProduct");
    assert_eq!(outputs(&c, "r"), expected(&[1, 2, 0]));
}

#[test]
fn polynomial_product_intermediate_zeros_remain_explicit() {
    let m = module();
    let c = run(&m, "gapsProduct");
    assert_eq!(outputs(&c, "r"), expected(&[1, 0, 2, 0, 1]));
}

#[test]
fn polynomial_product_independent_requests_keep_results_and_job_times_separate() {
    let m = module();
    let c = run(&m, "independentProducts");
    assert_eq!(outputs(&c, "r"), expected(&[1, 1, 0]));
    assert_eq!(outputs(&c, "gapsResult"), expected(&[1, 2, 1]));
    for proof in c.derivations.iter().filter(|d| {
        d.event.verb == "coefficient"
            && [Term::Const("r".into()), Term::Const("gapsResult".into())]
                .contains(&d.event.args[0])
    }) {
        assert_eq!(proof.record.rule, "publishProduct");
        let finished = &proof.record.antecedents[0];
        assert_eq!(finished.verb, "productFinished");
        let Term::RecordLiteral { fields, .. } = &finished.args[0] else {
            panic!("job")
        };
        assert_eq!(
            fields.iter().find(|f| f.field == "result").unwrap().value,
            proof.event.args[0]
        );
        let time = if proof.event.args[0] == Term::Const("r".into()) {
            TimeExpr::At("zeroTime".into())
        } else {
            TimeExpr::After(Box::new(TimeExpr::At("zeroTime".into())))
        };
        assert_eq!(
            TimeExpr::from_term(
                &fields
                    .iter()
                    .find(|f| f.field == "requestedAt")
                    .unwrap()
                    .value
            ),
            Some(time)
        );
        assert_eq!(finished.args[1], proof.event.args[1]);
        assert_eq!(finished.args[2], proof.event.args[2]);
    }
}

#[test]
fn polynomial_product_partial_beta_substitutes_all_members_and_preserves_times() {
    let m = module();
    let partial = program(&m, "ProductFromP");
    assert_eq!(
        partial
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["right", "result", "t"]
    );
    assert_eq!(partial.rule_uses.len(), 14);
    for member in partial.rule_uses.iter().take(6) {
        assert_eq!(
            member
                .bindings
                .iter()
                .find(|b| b.parameter == "left")
                .unwrap()
                .value,
            Term::Const("p".into())
        );
        assert_eq!(
            member
                .bindings
                .iter()
                .find(|b| b.parameter == "t")
                .unwrap()
                .value,
            Term::Var("t".into())
        );
    }
    for member in partial.rule_uses.iter().skip(6) {
        let capture = &member
            .bindings
            .iter()
            .find(|b| b.parameter == "job")
            .unwrap()
            .value;
        let Term::RecordLiteral { fields, .. } = capture else {
            panic!("captured job")
        };
        assert_eq!(
            fields.iter().find(|f| f.field == "left").unwrap().value,
            Term::Const("p".into())
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.field == "requestedAt")
                .unwrap()
                .value,
            Term::Var("t".into())
        );
    }
    assert_eq!(
        program(&m, "ProductGround").rule_uses,
        program(&m, "ProductDirect").rule_uses
    );
    let compiled = compile_program(partial, &m).unwrap();
    assert!(
        compiled[3]
            .parameters
            .iter()
            .any(|p| p.name == "#ProductFromP:use3:i")
    );
    let input = experiment(&m, "partialProduct").input.clone();
    let mut ordinary = input.clone();
    let request = ordinary
        .iter_mut()
        .find(|e| e.verb == "multiplyPolynomialFromP")
        .unwrap();
    request.verb = "multiplyPolynomial".into();
    request.args.insert(0, Term::Const("p".into()));
    let direct = close_input(&m, ordinary);
    let partial = close_input(&m, input);
    assert_eq!(outputs(&partial, "r"), outputs(&direct, "r"));
    assert_eq!(
        partial
            .known
            .iter()
            .filter(|e| e.verb != "multiplyPolynomialFromP")
            .collect::<BTreeSet<_>>(),
        direct
            .known
            .iter()
            .filter(|e| e.verb != "multiplyPolynomial")
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn polynomial_product_output_feeds_existing_polynomial_addition_in_one_close() {
    let m = module();
    let c = run(&m, "computedProductThenSum");
    assert_eq!(outputs(&c, "r"), expected(&[2, 5, 2]));
    assert_eq!(outputs(&c, "secondResult"), expected(&[2, 5, 2]));
    assert!(
        experiment(&m, "computedProductThenSum")
            .input
            .iter()
            .filter(|e| e.verb == "inputCoefficient")
            .all(|e| e.args[0] != Term::Const("r".into()))
    );
    assert!(
        c.known
            .contains(&experiment(&m, "computedProductThenSum").goal)
    );
}

#[test]
fn polynomial_product_absence_and_missing_coefficients_never_supply_zero_evidence() {
    let m = module();
    let c = run(&m, "absentProduct");
    assert!(outputs(&c, "r").is_empty());
    assert!(!c.known.iter().any(|e| matches!(
        e.verb.as_str(),
        "pairWork" | "productFold" | "multiply" | "add"
    )));
    let input = experiment(&m, "partialProduct")
        .input
        .iter()
        .filter(|e| {
            !(e.verb == "inputCoefficient"
                && e.args[0] == Term::Const("p".into())
                && natural(&e.args[1]) == 1)
        })
        .cloned()
        .collect();
    let c = close_input(&m, input);
    assert_eq!(outputs(&c, "r"), [(0, 2)].into_iter().collect());
    assert!(!c.known.contains(&experiment(&m, "partialProduct").goal));
}

fn replay_provenance(m: &Module, c: &Closure) {
    let mut rules: BTreeMap<String, Rule> = BTreeMap::new();
    for d in &m.declarations {
        match d {
            Declaration::Rule(r) => {
                rules.insert(r.name.clone(), r.clone());
            }
            Declaration::Program(p) => {
                for r in compile_program(p, m).unwrap() {
                    rules.insert(r.name.clone(), r);
                }
            }
            _ => (),
        }
    }
    for (i, proof) in c.derivations.iter().enumerate() {
        assert!(
            !c.derivations[..i]
                .iter()
                .any(|d| d.event == proof.event && d.record == proof.record)
        );
        assert!(!proof.record.antecedents.is_empty());
        assert!(proof.record.antecedents.iter().all(|e| c.known.contains(e)));
        let mut rule = rules[&proof.record.rule].clone();
        if let Some(origin) = program_rule_origin(m, &proof.record.rule) {
            assert!(
                [
                    "PeanoAfter",
                    "PeanoMultiplication",
                    "PeanoPolynomialAddition",
                    "PeanoPolynomialProduct"
                ]
                .contains(&origin.defining_module.as_str())
            );
            assert_eq!(origin.rule_origin.module, origin.defining_module);
            assert!(origin.rule_origin.file.as_ref().unwrap().is_file());
            let p = program(m, &origin.program);
            let verb = m
                .declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Verb(v)
                        if v.program.as_ref().is_some_and(|r| r.name == origin.program) =>
                    {
                        Some(v)
                    }
                    _ => None,
                })
                .unwrap();
            let trigger = proof
                .record
                .antecedents
                .iter()
                .find(|e| e.verb == verb.name)
                .unwrap();
            let mut guard = trigger.clone();
            guard.args = p.parameters[..p.parameters.len() - 1]
                .iter()
                .map(|a| Term::Var(format!("#{}:shared:{}", p.identity, a.name)))
                .collect();
            guard.proposition.time = TimeExpr::Var(format!(
                "#{}:shared:{}",
                p.identity,
                p.parameters.last().unwrap().name
            ));
            let RuleBody::HornClause(clause) = &mut rule.body else {
                panic!("Horn")
            };
            clause.antecedent = Antecedent::And(
                Box::new(Antecedent::Event(guard)),
                Box::new(clause.antecedent.clone()),
            );
        }
        assert!(
            instantiate_from_known(&rule, &proof.record.antecedents)
                .into_iter()
                .any(|mut d| {
                    if d.record.antecedents.len() > 1
                        && d.record.antecedents[1..].contains(&d.record.antecedents[0])
                    {
                        d.record.antecedents.remove(0);
                    }
                    d.event == proof.event
                        && d.event.proposition == proof.event.proposition
                        && d.record == proof.record
                }),
            "cannot replay {}",
            proof.record.rule
        );
    }
}

#[test]
fn polynomial_product_provenance_connects_every_coefficient_to_recursive_arithmetic() {
    let m = module();
    let c = run(&m, "partialProduct");
    replay_provenance(&m, &c);
    for output in c
        .known
        .iter()
        .filter(|e| e.verb == "coefficient" && e.args[0] == Term::Const("r".into()))
    {
        let mut seen = BTreeSet::new();
        let mut pending = vec![output.clone()];
        let mut programs = BTreeSet::new();
        let mut roots = BTreeSet::new();
        let mut requested = false;
        while let Some(event) = pending.pop() {
            if !seen.insert(event.clone()) {
                continue;
            }
            requested |= event.verb == "multiplyPolynomialFromP";
            if event.verb == "inputCoefficient" {
                roots.insert(event.args[0].clone());
            }
            for p in c.derivations.iter().filter(|p| p.event == event) {
                if let Some(o) = program_rule_origin(&m, &p.record.rule) {
                    programs.insert(o.program);
                }
                pending.extend(p.record.antecedents.clone());
            }
        }
        assert!(requested);
        assert!(
            roots.contains(&Term::Const("p".into())) && roots.contains(&Term::Const("q".into()))
        );
        for name in ["ProductFromP", "PairWork", "Multiply", "MultiplyStep"] {
            assert!(programs.contains(name), "missing program {name}");
        }
        if natural(&output.args[2]) > 0 {
            for name in [
                "AdditionContinuation",
                "Addition",
                "RightWalk",
                "FoldAddition",
            ] {
                assert!(programs.contains(name), "missing {name}");
            }
        }
    }
}
