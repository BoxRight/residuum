use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{Arg, Closure, DerivationTime, Module, Program, Rule, RuleBody, Type};
use residuum::{DerivationClock, close};
use std::collections::BTreeSet;
use std::path::Path;

const CHAIN: &str =
    include_str!("../../../../examples/experiments/associated_multi_rule_program.res");
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
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("program-{}", self.0),
        }
    }
}
fn known(closure: &Closure) -> BTreeSet<Event> {
    closure.known.iter().cloned().collect()
}
fn body(rule: &Rule) -> residuum::typed::HornClause {
    match &rule.body {
        RuleBody::HornClause(c) => c.clone(),
        RuleBody::ResidualClause(c) => c.unresiduate(),
    }
}

#[test]
fn normal_sale_program_beta_and_joint_fixpoint_match_sales() {
    let module = residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/legal/modules/programs/normal_sale_scenario.res"),
    )
    .unwrap();
    let base = program(&module, "NormalSale");
    assert_eq!(base.rule_uses.len(), 3);
    assert_eq!(base.defining_module, "Sales");
    let partial = program(&module, "NormalSaleForJuan");
    assert_eq!(partial.parameters.len(), 8);
    for member in &partial.rule_uses {
        assert!(
            member
                .bindings
                .iter()
                .any(|b| b.value == Term::Const("juan".into()))
        );
    }
    let ground = program(&module, "NormalSaleGround");
    assert!(ground.parameters.is_empty());
    assert_eq!(
        ground.rule_uses,
        program(&module, "NormalSaleDirect").rule_uses
    );
    assert_eq!(ground.defining_module, base.defining_module);
    let seeds = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == "plain" => Some(e.input.clone()),
            _ => None,
        })
        .unwrap();
    let original: Vec<_> = [
        "normalSale",
        "reciprocalTransferPermission",
        "plainTransferObligation",
        "obligationCreatesRelation",
    ]
    .iter()
    .map(|n| rule(&module, n).clone())
    .collect();
    let inline = close(seeds.clone(), &original, &mut Clock::default());
    for name in [
        "NormalSale",
        "NormalSaleForJuan",
        "NormalSaleGround",
        "NormalSaleDirect",
    ] {
        let mut compiled = compile_program(program(&module, name), &module).unwrap();
        compiled.push(rule(&module, "obligationCreatesRelation").clone());
        let actual = close(seeds.clone(), &compiled, &mut Clock::default());
        assert_eq!(known(&actual), known(&inline));
        assert_eq!(actual.derivations.len(), inline.derivations.len());
        for proof in &actual.derivations {
            let original_name =
                if let Some(origin) = program_rule_origin(&module, &proof.record.rule) {
                    assert_eq!(origin.program, name);
                    assert_eq!(origin.defining_module, "Sales");
                    assert_eq!(origin.rule_origin.module, "Sales");
                    origin.rule
                } else {
                    assert_eq!(proof.record.rule, "obligationCreatesRelation");
                    assert_eq!(
                        module.origin(&proof.record.rule).unwrap().module,
                        "LegalMonotonicCase"
                    );
                    proof.record.rule.clone()
                };
            // Beta removes applied binders; replay in the original clause restores
            // those bindings and compares the semantic proof, not binder spelling.
            let replay = residuum::instantiate_from_known(
                rule(&module, &original_name),
                &proof.record.antecedents,
            )
            .into_iter()
            .find(|p| p.event == proof.event)
            .unwrap();
            let expected = inline
                .derivations
                .iter()
                .find(|p| p.event == proof.event && p.record.rule == original_name)
                .unwrap();
            assert_eq!(replay.record, expected.record);
            assert_eq!(proof.record.antecedents, expected.record.antecedents);
            assert_eq!(proof.event.proposition, expected.event.proposition);
            assert_eq!(proof.event.transition, expected.event.transition);
        }
        // The first member cannot independently produce the final do effect.
        let first_only = close(seeds.clone(), &compiled[..1], &mut Clock::default());
        assert!(!first_only.known.iter().any(|e| e.verb == "do"));
        assert!(actual.known.iter().any(|e| e.verb == "do"));
    }
}

#[test]
fn program_beta_preserves_local_binders_times_and_nested_records() {
    let module = residuum::parse_formal_to_typed(CHAIN).unwrap();
    let partial = program(&module, "Partial");
    assert_eq!(
        partial.parameters,
        vec![Arg {
            name: "t".into(),
            ty: Type::PropositionTime
        }]
    );
    for member in &partial.rule_uses {
        assert!(
            member
                .bindings
                .iter()
                .any(|b| b.value == Term::Const("item".into()))
        );
        assert!(
            member
                .bindings
                .iter()
                .any(|b| b.value == Term::Var("t".into()))
        );
    }
    let ground = program(&module, "Ground");
    assert_eq!(ground.rule_uses, program(&module, "Direct").rule_uses);
    let compiled = compile_program(ground, &module).unwrap();
    let closure = close(vec![], &compiled, &mut Clock::default());
    assert_eq!(closure.known.len(), 2);
    assert!(closure.known.iter().any(|e| e.verb == "completed"));
    let local = compile_program(program(&module, "WithLocals"), &module).unwrap();
    assert!(
        local[0]
            .parameters
            .iter()
            .any(|p| p.name.contains(":use0:when"))
    );
    assert!(!local[0].parameters.iter().any(|p| p.name == "when"));
    let temporal = compile_program(program(&module, "TemporalGround"), &module).unwrap();
    let remembered = body(&temporal[0]).consequent;
    assert_eq!(
        remembered.args,
        vec![Term::Const("item".into()), Term::Const("later".into())]
    );
    assert_eq!(
        remembered.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
    );
    let sealed = body(&temporal[1]).consequent;
    assert!(format!("{:?}", sealed.args).contains("Const(\"item\")"));
    assert!(!format!("{:?}", sealed.args).contains("Var("));
    assert_eq!(
        close(vec![], &temporal, &mut Clock::default()).known.len(),
        2
    );
}

#[test]
fn associated_program_members_activate_together_only_for_positive_known_instances() {
    let module = residuum::parse_formal_to_typed(CHAIN).unwrap();
    let results = run_forward_experiments(&module).unwrap();
    let case = |n: &str| results.iter().find(|r| r.experiment.name == n).unwrap();
    for name in ["real", "derived"] {
        let result = case(name);
        assert!(result.horn_goal);
        for proof in result
            .closure
            .derivations
            .iter()
            .filter(|p| p.event.verb == "middle" || p.event.verb == "completed")
        {
            let origin = program_rule_origin(&module, &proof.record.rule).unwrap();
            assert_eq!(origin.program, "Chain");
            assert_eq!(origin.rule_origin.module, "AssociatedMultiRuleProgram");
            assert!(proof.record.antecedents.iter().any(|a| a.verb
                == if name == "real" {
                    "starts"
                } else {
                    "activated"
                }));
        }
    }
    let observed = |name: &str| {
        case(name)
            .closure
            .derivations
            .iter()
            .filter(|p| p.event.verb == "middle" || p.event.verb == "completed")
            .map(|p| (p.event.clone(), format!("{:?}", p.record.substitution)))
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(observed("real"), observed("derived"));
    for name in ["absent", "negative"] {
        assert!(!case(name).horn_goal);
        assert!(
            !case(name)
                .closure
                .known
                .iter()
                .any(|e| e.verb == "middle" || e.verb == "completed")
        );
    }
    let output = execute(Command::Explain, CHAIN).unwrap();
    assert!(output.contains("program Chain | module AssociatedMultiRuleProgram"));
    assert!(output.contains("rule first | rule module AssociatedMultiRuleProgram"));
    assert!(output.contains("rule second | rule module AssociatedMultiRuleProgram"));
}

#[test]
fn program_elaboration_rejects_invalid_references_bindings_and_applications() {
    for (source, message) in [
        (
            CHAIN.replace("use first {", "use missing {"),
            "unknown program member rule",
        ),
        (
            CHAIN.replace("local = object, when = t", "absent = object, when = t"),
            "unknown parameter",
        ),
        (
            CHAIN.replace("local = object, when = t", "local = t, when = t"),
            "binding",
        ),
        (
            CHAIN.replace(
                "local = object, when = t",
                "local = object, local = object, when = t",
            ),
            "duplicate binding",
        ),
        (
            CHAIN.replace(
                "program Partial = Chain(item)",
                "program Partial = Chain(tau)",
            ),
            "not assignable",
        ),
        (
            CHAIN.replace(
                "program Partial = Chain(item)",
                "program Partial = Missing(item)",
            ),
            "unknown program",
        ),
        (
            CHAIN.replace(
                "program Partial = Chain(item)",
                "program Chain = Chain(item)",
            ),
            "duplicate symbol",
        ),
        (
            CHAIN.replace(
                "program Partial = Chain(item)",
                "program Partial = Chain(missing)",
            ),
            "unknown constant",
        ),
        (
            CHAIN.replace(
                "program Chain(object : Thing, t : PropositionTime)",
                "program Chain(object : Missing, t : PropositionTime)",
            ),
            "unknown program parameter type",
        ),
    ] {
        let error = residuum::parse_formal_to_typed(&source)
            .unwrap_err()
            .to_string();
        assert!(error.contains(message), "{message}: {error}");
    }
}

#[test]
fn imported_program_associations_preserve_definition_scope_and_member_origins() {
    let module = residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/multi_rule_program_consumer.res"),
    )
    .unwrap();
    let results = run_forward_experiments(&module).unwrap();
    assert_eq!(results.len(), 1); // Imported experiments are never executed.
    assert!(results[0].horn_goal);
    assert_eq!(
        program(&module, "ImportedGround").defining_module,
        "AssociatedMultiRuleProgram"
    );
    assert_eq!(
        module.origin("ImportedGround").unwrap().module,
        "MultiRuleProgramConsumer"
    );
    for proof in &results[0].closure.derivations {
        let origin = program_rule_origin(&module, &proof.record.rule).unwrap();
        assert_eq!(origin.program, "Chain");
        assert_eq!(origin.defining_module, "AssociatedMultiRuleProgram");
        assert_eq!(origin.rule_origin.module, "AssociatedMultiRuleProgram");
        assert!(
            proof
                .record
                .antecedents
                .iter()
                .any(|e| e.verb == "consumerStarts")
        );
    }
    assert_eq!(
        compile_program(program(&module, "ImportedGround"), &module).unwrap()[0].body,
        compile_program(program(&module, "Ground"), &module).unwrap()[0].body
    );
}
