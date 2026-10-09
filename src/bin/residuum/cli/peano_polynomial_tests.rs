use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{
    Antecedent, Closure, DerivationTime, Experiment, Module, Program, PropositionKind, Rule,
    RuleBody, Type,
};
use residuum::{DerivationClock, close_program, instantiate_from_known};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn module() -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/peano_polynomial_scenario.res"),
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

fn program<'a>(m: &'a Module, name: &str) -> &'a Program {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Program(p) if p.identity == name => Some(p),
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
            self.0 < 3000,
            "unexpected proof growth in finite polynomial addition"
        );
        DerivationTime {
            name: format!("polynomial-{}", self.0),
        }
    }
}

fn run(m: &Module, name: &str) -> Closure {
    let e = experiment(m, name);
    let before = e.input.clone();
    assert!(e.input.iter().all(|f| matches!(
        f.verb.as_str(),
        "exponent" | "inputCoefficient" | "addPolynomial" | "addPolynomialFromP"
    )));
    let c = close_program(e.input.clone(), m, &mut Clock::default()).unwrap();
    assert_eq!(e.input, before);
    assert!(before.iter().all(|f| c.known.contains(f)));
    assert_eq!(c.known.iter().collect::<BTreeSet<_>>().len(), c.known.len());
    assert!(c.known.iter().all(|f| f.args.iter().all(Term::is_ground)));
    println!(
        "{name}: {} facts, {} justifications",
        c.known.len(),
        c.derivations.len()
    );
    c
}

fn natural(t: &Term) -> usize {
    // Decoder/oracle only: this never supplies evidence to the source program.
    match t {
        Term::Const(z) if z == "zeroTime" => 0,
        Term::Constructor { argument, .. } => 1 + natural(argument),
        _ => panic!("not a ground Peano value: {t:?}"),
    }
}

fn coefficients(c: &Closure, polynomial: &str) -> BTreeMap<usize, usize> {
    let mut values = BTreeMap::new();
    for e in c
        .known
        .iter()
        .filter(|e| e.verb == "coefficient" && e.args[0] == Term::Const(polynomial.into()))
    {
        assert_eq!(e.proposition.kind, PropositionKind::Derived);
        assert_eq!(e.proposition.time, TimeExpr::At("zeroTime".into()));
        assert!(e.transition.is_none());
        assert!(
            values
                .insert(natural(&e.args[1]), natural(&e.args[2]))
                .is_none(),
            "more than one result for exponent of {polynomial}"
        );
        println!("{}", render_event(e));
    }
    values
}

fn expected(values: &[usize]) -> BTreeMap<usize, usize> {
    values.iter().copied().enumerate().collect()
}

fn field<'a>(term: &'a Term, name: &str) -> &'a Term {
    let Term::RecordLiteral { fields, .. } = term else {
        panic!("record")
    };
    &fields.iter().find(|f| f.field == name).unwrap().value
}

#[test]
fn polynomial_principal_computes_four_coefficients_from_requests_and_input_only() {
    let m = module();
    let c = run(&m, "principal");
    assert_eq!(coefficients(&c, "r"), expected(&[6, 2, 6, 4]));
    assert!(c.known.contains(&experiment(&m, "principal").goal));
    assert_eq!(c.known.iter().filter(|e| e.verb == "add").count(), 4);
    assert_eq!(c.known.iter().filter(|e| e.verb == "sum").count(), 4);
    // This uses the imported Peano implementation, including its recursive walk.
    assert!(
        c.derivations
            .iter()
            .any(|d| d.record.rule == "RightWalk::0::rightSuccessor")
    );
    assert!(
        c.derivations
            .iter()
            .any(|d| d.record.rule == "OperandSuccessor::1::continueOperand")
    );
}

#[test]
fn polynomial_explicit_zeros_preserve_identity_and_intermediate_zero_terms() {
    let m = module();
    for name in ["zeroRight", "zeroLeft"] {
        let c = run(&m, name);
        assert_eq!(coefficients(&c, "preserved"), expected(&[1, 2, 3, 4]));
        assert!(c.known.contains(&experiment(&m, name).goal));
    }
    let c = run(&m, "bothZero");
    assert_eq!(coefficients(&c, "preserved"), expected(&[0, 0, 0, 0]));
    let c = run(&m, "intermediateZeros");
    assert_eq!(coefficients(&c, "gapsResult"), expected(&[2, 0, 2, 1]));
    assert!(c.known.contains(&experiment(&m, "intermediateZeros").goal));
}

#[test]
fn polynomial_concurrent_requests_have_disjoint_jobs_and_independent_results() {
    let m = module();
    let first = run(&m, "principal");
    let second = run(&m, "intermediateZeros");
    let together = run(&m, "concurrent");
    assert_eq!(coefficients(&together, "r"), coefficients(&first, "r"));
    assert_eq!(
        coefficients(&together, "gapsResult"),
        coefficients(&second, "gapsResult")
    );
    assert_eq!(
        together.known.iter().collect::<BTreeSet<_>>(),
        first
            .known
            .iter()
            .chain(&second.known)
            .collect::<BTreeSet<_>>()
    );
    let separate: Vec<_> = first
        .derivations
        .iter()
        .chain(&second.derivations)
        .collect();
    assert!(together.derivations.iter().all(|d| {
        separate
            .iter()
            .any(|s| s.event == d.event && s.record == d.record)
    }));
    assert!(separate.iter().all(|s| {
        together
            .derivations
            .iter()
            .any(|d| s.event == d.event && s.record == d.record)
    }));
    let jobs: BTreeSet<_> = together
        .known
        .iter()
        .filter(|e| e.verb == "add")
        .map(|e| e.args[2].clone())
        .collect();
    assert_eq!(jobs.len(), 8);
    for job in jobs {
        assert!(matches!(field(&job, "result"), Term::Const(r) if r == "r" || r == "gapsResult"));
        assert_eq!(field(&job, "requestedAt"), &Term::Const("zeroTime".into()));
    }
    let timed = run(&m, "distinctRequestTimes");
    assert_eq!(coefficients(&timed, "r"), expected(&[6, 2, 6, 4]));
    assert_eq!(
        coefficients(&timed, "secondResult"),
        expected(&[6, 2, 6, 4])
    );
    for call in timed.known.iter().filter(|e| e.verb == "add") {
        let job = &call.args[2];
        let wanted = if field(job, "result") == &Term::Const("r".into()) {
            TimeExpr::At("zeroTime".into())
        } else {
            TimeExpr::After(Box::new(TimeExpr::At("zeroTime".into())))
        };
        assert_eq!(TimeExpr::from_term(field(job, "requestedAt")), Some(wanted));
    }
}

#[test]
fn polynomial_partial_beta_specializes_all_members_and_preserves_local_parameters() {
    let m = module();
    let p = program(&m, "AddFromP");
    assert_eq!(
        p.parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["right", "result", "t"]
    );
    assert_eq!(p.parameters[2].ty, Type::PropositionTime);
    assert_eq!(program(&m, "AddPAndQ").parameters.len(), 2);
    assert_eq!(p.rule_uses.len(), 5);
    for member in &p.rule_uses {
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
                .find(|b| b.parameter == "right")
                .unwrap()
                .value,
            Term::Var("right".into())
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
    assert_eq!(
        program(&m, "AddPQGround").rule_uses,
        program(&m, "AddPQDirect").rule_uses
    );
    let partial = compile_program(p, &m).unwrap();
    let begin = partial
        .iter()
        .find(|r| r.name.ends_with("::beginCoefficient"))
        .unwrap();
    assert!(
        begin
            .parameters
            .iter()
            .any(|p| p.name == "#AddFromP:shared:t")
    );
    assert!(
        begin
            .parameters
            .iter()
            .any(|p| p.name == "#AddFromP:use3:e")
    );
    let ground = compile_program(program(&m, "AddPQGround"), &m).unwrap();
    let direct = compile_program(program(&m, "AddPQDirect"), &m).unwrap();
    // The three ground members have no local binders, so compare exact bodies.
    for (a, b) in ground.iter().zip(&direct).take(3) {
        assert_eq!(a.body, b.body);
    }
    assert!(
        ground
            .iter()
            .all(|r| !r.parameters.iter().any(|p| p.name.contains(":shared:")))
    );
    let c = run(&m, "partial");
    assert_eq!(
        coefficients(&c, "r"),
        coefficients(&run(&m, "principal"), "r")
    );
    assert!(
        c.derivations
            .iter()
            .any(|d| d.record.rule.starts_with("AddFromP::"))
    );
}

#[test]
fn polynomial_missing_input_is_not_zero_and_absent_request_does_not_activate() {
    let m = module();
    let empty = run(&m, "absent");
    assert!(coefficients(&empty, "r").is_empty());
    assert!(!empty.known.iter().any(|e| matches!(
        e.verb.as_str(),
        "polynomialRequest" | "coefficientAddition" | "add" | "sum" | "unfoldOperand"
    )));
    let c = run(&m, "missingCoefficient");
    assert_eq!(
        coefficients(&c, "r"),
        [(0, 6), (2, 6), (3, 4)].into_iter().collect()
    );
    assert!(!c.known.contains(&experiment(&m, "missingCoefficient").goal));
}

#[test]
fn polynomial_computed_coefficients_feed_a_second_sum_in_the_same_fixpoint() {
    let m = module();
    let c = run(&m, "chained");
    assert_eq!(coefficients(&c, "r"), expected(&[6, 2, 6, 4]));
    assert_eq!(coefficients(&c, "secondResult"), expected(&[6, 2, 6, 4]));
    assert!(
        experiment(&m, "chained")
            .input
            .iter()
            .filter(|e| e.verb == "inputCoefficient")
            .all(|e| e.args[0] != Term::Const("r".into()))
    );
    for proof in c.derivations.iter().filter(|d| {
        d.event.verb == "coefficientAddition" && d.event.args[0] == Term::Const("r".into())
    }) {
        let operand = proof
            .record
            .antecedents
            .iter()
            .find(|e| e.verb == "coefficient" && e.args[0] == Term::Const("r".into()))
            .unwrap();
        assert!(c.derivations.iter().any(|d| d.event == *operand));
    }
}

#[test]
fn polynomial_coefficients_beyond_the_original_zero_to_four_domain_are_computed() {
    let m = module();
    let c = run(&m, "largerCoefficients");
    assert_eq!(coefficients(&c, "r"), expected(&[20, 0, 0, 0]));
    assert!(c.known.contains(&experiment(&m, "largerCoefficients").goal));
}

fn verify_provenance(m: &Module, c: &Closure) {
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
            let expected_module =
                if ["Addition", "LeftWalk", "RightWalk"].contains(&origin.program.as_str()) {
                    "PeanoAfter"
                } else {
                    "PeanoPolynomialAddition"
                };
            assert_eq!(origin.defining_module, expected_module);
            assert_eq!(origin.rule_origin.module, expected_module);
            assert!(origin.rule_origin.file.as_ref().unwrap().ends_with(
                if expected_module == "PeanoAfter" {
                    "peano_after.res"
                } else {
                    "peano_polynomial_addition.res"
                }
            ));
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
            // Replay the implicit activation guard with the existing matcher.
            // The verifier adds no arithmetic and makes no changes to the engine.
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
        let replay = instantiate_from_known(&rule, &proof.record.antecedents);
        assert!(
            replay.into_iter().any(|mut d| {
                // close_program cites a trigger once when it is also a body witness.
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
    // Every result has a support path back to its request and coefficient inputs.
    for output in c
        .known
        .iter()
        .filter(|e| e.verb == "coefficient" && e.args[0] == Term::Const("r".into()))
    {
        let mut pending = vec![output.clone()];
        let mut seen = BTreeSet::new();
        let mut has_request = false;
        let mut input_polynomials = BTreeSet::new();
        let mut programs = BTreeSet::new();
        while let Some(event) = pending.pop() {
            if !seen.insert(event.clone()) {
                continue;
            }
            has_request |= event.verb == "addPolynomial";
            if event.verb == "inputCoefficient" {
                input_polynomials.insert(event.args[0].clone());
            }
            for proof in c.derivations.iter().filter(|d| d.event == event) {
                if let Some(o) = program_rule_origin(m, &proof.record.rule) {
                    programs.insert(o.program);
                }
                pending.extend(proof.record.antecedents.clone());
            }
        }
        assert!(has_request);
        assert!(input_polynomials.contains(&Term::Const("p".into())));
        assert!(input_polynomials.contains(&Term::Const("q".into())));
        for p in [
            "PolynomialAddition",
            "CoefficientAddition",
            "Addition",
            "LeftWalk",
            "RightWalk",
        ] {
            assert!(programs.contains(p), "missing provenance for {p}");
        }
    }
}

#[test]
fn polynomial_provenance_replays_every_step_and_connects_each_result_to_its_request() {
    let m = module();
    verify_provenance(&m, &run(&m, "principal"));
    verify_provenance(&m, &run(&m, "concurrent"));
}
