use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{
    Antecedent, Closure, DerivationTime, Experiment, Module, Program, Rule, RuleBody, Type,
};
use residuum::{DerivationClock, close, close_program, instantiate_from_known, query_residual};
use std::path::Path;

fn module() -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/temporal_constructor_scenario.res"),
    )
    .unwrap()
}
fn rule<'a>(m: &'a Module, n: &str) -> &'a Rule {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == n => Some(r),
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
fn experiment<'a>(m: &'a Module, n: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == n => Some(e),
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
            self.0 < 1000,
            "finite constructor fixture expands unexpectedly"
        );
        DerivationTime {
            name: format!("constructors-{}", self.0),
        }
    }
}
fn run(m: &Module, n: &str) -> Closure {
    close_program(experiment(m, n).input.clone(), m, &mut Clock::default()).unwrap()
}
fn peano(n: usize) -> TimeExpr {
    // Independent verifier only. The source rules compute their own answers.
    (0..n).fold(TimeExpr::At("zeroTime".into()), |t, _| {
        TimeExpr::After(Box::new(t))
    })
}
fn event<'a>(c: &'a Closure, v: &str) -> &'a Event {
    c.known.iter().find(|e| e.verb == v).unwrap()
}

#[test]
fn computed_peano_result_crosses_arguments_and_indices_without_supplied_answers() {
    let m = module();
    let original = experiment(&m, "computed").input.clone();
    assert!(original.iter().all(|e| !matches!(
        e.verb.as_str(),
        "sum" | "coefficient" | "verified" | "packed" | "unpacked"
    )));
    let c = run(&m, "computed");
    assert_eq!(experiment(&m, "computed").input, original);
    let first = c
        .known
        .iter()
        .find(|e| e.verb == "sum" && e.args == vec![Term::Const("r23".into())])
        .unwrap();
    assert_eq!(first.proposition.time, peano(5));
    let coefficient = event(&c, "coefficient");
    assert_eq!(
        coefficient.args,
        vec![
            Term::Const("polynomial".into()),
            peano(2).to_term(),
            peano(5).to_term()
        ]
    );
    let restart = c
        .known
        .iter()
        .find(|e| e.verb == "add" && e.args[2] == Term::Const("r02".into()))
        .unwrap();
    assert_eq!(restart.proposition.time, peano(5));
    assert!(!original.contains(restart));
    let verified = event(&c, "verified");
    assert_eq!(verified.args[1], peano(7).to_term());
    assert!(c.known.contains(&experiment(&m, "computed").goal));
    assert_eq!(event(&c, "unpacked").args[0], peano(7).to_term());
    assert_eq!(event(&c, "unpacked").proposition.time, peano(7));
    assert_eq!(event(&c, "predecessor").args[0], peano(4).to_term());
    assert_eq!(event(&c, "predecessor").proposition.time, peano(4));
    assert!(c.known.iter().all(|e| e.args.iter().all(Term::is_ground)));
    println!(
        "computed: {}; reused: {}; result: {}",
        render_event(coefficient),
        render_event(restart),
        render_event(verified)
    );
}

#[test]
fn computed_constructor_provenance_preserves_imports_rules_and_programs() {
    let m = module();
    let c = run(&m, "computed");
    for name in ["publish", "resume", "verify", "pack", "unpack", "peel"] {
        let proof = c
            .derivations
            .iter()
            .find(|p| p.record.rule == name)
            .unwrap();
        let replay = instantiate_from_known(rule(&m, name), &proof.record.antecedents);
        let same = replay.iter().find(|p| p.event == proof.event).unwrap();
        assert_eq!(same.record, proof.record);
        assert_eq!(same.event.proposition, proof.event.proposition);
        assert_eq!(m.origin(name).unwrap().module, "TemporalConstructorRules");
        assert!(
            m.origin(name)
                .unwrap()
                .file
                .as_ref()
                .unwrap()
                .ends_with("temporal_constructor_rules.res")
        );
    }
    let resume = c
        .derivations
        .iter()
        .find(|p| p.record.rule == "resume")
        .unwrap();
    assert_eq!(
        resume.record.substitution.bindings[0].value,
        SubstitutionValue::Term(peano(5).to_term())
    );
    let products: Vec<_> = c
        .derivations
        .iter()
        .filter(|p| p.event.verb == "sum")
        .collect();
    assert_eq!(products.len(), 2);
    for proof in products {
        let origin = program_rule_origin(&m, &proof.record.rule).unwrap();
        assert_eq!(origin.program, "RightWalk");
        assert_eq!(origin.rule, "finish");
        assert_eq!(origin.defining_module, "PeanoAfter");
        assert!(
            proof
                .record
                .antecedents
                .iter()
                .any(|e| e.verb == "rightWalk")
        );
    }
}

#[test]
fn constructor_beta_partial_application_reaches_nested_records_and_all_rule_uses() {
    let m = module();
    assert_eq!(rule(&m, "groundEmit").body, rule(&m, "directEmit").body);
    assert_eq!(rule(&m, "partialEmit").parameters[0].name, "t");
    assert_eq!(
        rule(&m, "partialEmit").parameters[0].ty,
        Type::PropositionTime
    );
    let partial = program(&m, "PartialEcho");
    assert_eq!(partial.parameters.len(), 1);
    assert_eq!(partial.parameters[0].name, "t");
    assert_eq!(partial.rule_uses.len(), 2);
    assert_eq!(
        program(&m, "GroundEcho").rule_uses,
        program(&m, "DirectEcho").rule_uses
    );
    let compiled = compile_program(partial, &m).unwrap();
    assert_eq!(compiled.len(), 2);
    for r in &compiled {
        let RuleBody::HornClause(c) = &r.body else {
            panic!("Horn")
        };
        assert_eq!(
            c.consequent.proposition.time,
            TimeExpr::Var("#PartialEcho:shared:t".into())
        );
    }
    let ground = compile_program(program(&m, "GroundEcho"), &m).unwrap();
    let c = close(vec![], &ground, &mut Clock::default());
    assert_eq!(event(&c, "observed").args, vec![peano(2).to_term()]);
    let Term::RecordLiteral { fields, .. } = &event(&c, "packed").args[0] else {
        panic!("Envelope")
    };
    let Term::RecordLiteral { fields, .. } = &fields[0].value else {
        panic!("Box")
    };
    assert_eq!(fields[0].value, peano(2).to_term());
    let activated = run(&m, "constructedInput");
    assert_eq!(event(&activated, "observed").args, vec![peano(4).to_term()]);
    assert_eq!(event(&activated, "indexed").proposition.time, peano(4));
    let proof = activated
        .derivations
        .iter()
        .find(|p| p.event.verb == "observed")
        .unwrap();
    assert_eq!(
        program_rule_origin(&m, &proof.record.rule).unwrap().program,
        "Echo"
    );
    assert!(
        proof
            .record
            .antecedents
            .iter()
            .any(|e| e.verb == "inspect" && e.args[0] == peano(2).to_term())
    );
    assert!(run(&m, "empty").derivations.is_empty());
}

#[test]
fn constructor_matching_is_structural_ground_and_residual_compatible() {
    let m = module();
    let c = run(&m, "computed");
    let predecessor = event(&c, "predecessor").clone();
    let queries = query_residual(rule(&m, "peel"), &predecessor, &[]).unwrap();
    assert_eq!(queries.len(), 1);
    let Antecedent::Event(needed) = &queries[0].required else {
        panic!("requirement")
    };
    assert_eq!(needed, event(&c, "coefficient"));
    assert!(
        query_residual(rule(&m, "peel"), &predecessor, std::slice::from_ref(needed))
            .unwrap()
            .is_empty()
    );
    let mut wrong = needed.clone();
    wrong.args[2] = peano(3).to_term();
    assert!(
        !query_residual(rule(&m, "peel"), &predecessor, &[wrong])
            .unwrap()
            .is_empty()
    );
    let mut unresolved = needed.clone();
    unresolved.args[2] = TimeExpr::After(Box::new(TimeExpr::Var("pending".into()))).to_term();
    assert!(instantiate_from_known(rule(&m, "peel"), &[unresolved]).is_empty());
}

#[test]
fn constructor_signature_rejects_wrong_types_arity_and_unknown_constructors() {
    let valid = "module C const z : PropositionTime entity Item const item : Item seeded verb v(value : PropositionTime) experiment e { seeds { v(after(after(z))) @ z } query residual v(z) @ z }";
    assert!(residuum::parse_formal_to_typed(valid).is_ok());
    let wrong = valid.replace("after(after(z))", "after(item)");
    let error = residuum::parse_formal_to_typed(&wrong).unwrap_err();
    assert!(
        error.to_string().contains("after requires PropositionTime"),
        "{error}"
    );
    for bad in ["after()", "after(z,z)", "successor(z)", "after(z) after(z)"] {
        assert!(
            residuum::parse_formal_to_typed(&valid.replace("after(after(z))", bad)).is_err(),
            "{bad}"
        );
    }
    let wrong_result = valid.replace("v(value : PropositionTime)", "v(value : Item)");
    assert!(
        residuum::parse_formal_to_typed(&wrong_result)
            .unwrap_err()
            .to_string()
            .contains("not assignable")
    );
}

#[test]
fn explicit_state_materialization_keeps_nested_constructor_values_symbolic() {
    use residuum::runtime::{FieldName, ObjectId, StateSnapshot, Store, Value, execute_effect};
    use residuum::typed::StateTime;
    let m = module();
    let c = run(&m, "computed");
    let before = c.clone();
    let retain = event(&c, "retain");
    assert_eq!(retain.args[1], peano(5).to_term());
    let verb = m
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == "retain" => Some(v),
            _ => None,
        })
        .unwrap();
    let input = StateSnapshot {
        time: StateTime::At(peano(0)),
        store: Store::new().with_set(
            ObjectId("shelf".into()),
            FieldName("entries".into()),
            vec![],
        ),
    };
    let original = input.clone();
    // Existing runtime adapter: the shelf constant names this concrete ObjectId.
    // Only the entity reference is adapted; the calculated temporal value is
    // passed unchanged. This does not alter the logical event in Known.
    let mut runtime_event = retain.clone();
    runtime_event.args[0] = Term::Var("shelf".into());
    let output = execute_effect(&runtime_event, verb, &input).unwrap();
    assert_eq!(input, original);
    assert_eq!(c, before);
    assert_eq!(output.time, StateTime::At(peano(1)));
    let constructor = (0..6).fold(Value::Const("zeroTime".into()), |argument, _| {
        Value::Constructor {
            constructor: residuum::typed::Constructor::After,
            argument: Box::new(argument),
        }
    });
    let record = Value::Record {
        entity: "TimeEnvelope".into(),
        fields: std::collections::BTreeMap::from([(
            FieldName("payload".into()),
            Value::Record {
                entity: "TimeBox".into(),
                fields: std::collections::BTreeMap::from([(
                    FieldName("value".into()),
                    constructor,
                )]),
            },
        )]),
    };
    assert_eq!(
        output
            .store
            .value(&ObjectId("shelf".into()), &FieldName("entries".into())),
        Some(&Value::Set(std::collections::BTreeSet::from([record])))
    );
    assert!(
        c.known
            .iter()
            .all(|e| !matches!(e.verb.as_str(), "Act" | "CounterAct"))
    );
}

#[test]
fn constructor_rule_use_bindings_and_cross_position_constraints_preserve_structure() {
    let m = module();
    let partial = program(&m, "PartialConstructed");
    assert_eq!(partial.parameters.len(), 1);
    assert_eq!(partial.parameters[0].name, "t");
    for r in compile_program(partial, &m).unwrap() {
        let RuleBody::HornClause(c) = r.body else {
            panic!("Horn")
        };
        assert_eq!(
            c.consequent.proposition.time,
            TimeExpr::After(Box::new(TimeExpr::Var(
                "#PartialConstructed:shared:t".into()
            )))
        );
    }
    assert_eq!(
        program(&m, "GroundConstructed").rule_uses,
        program(&m, "DirectConstructed").rule_uses
    );
    let ground = compile_program(program(&m, "GroundConstructed"), &m).unwrap();
    let c = close(vec![], &ground, &mut Clock::default());
    assert_eq!(event(&c, "observed").args, vec![peano(3).to_term()]);
    assert_eq!(event(&c, "observed").proposition.time, peano(1));
    let matched = run(&m, "alignedValue");
    assert_eq!(
        event(&matched, "alignedResult").args,
        vec![peano(1).to_term()]
    );
    assert_eq!(event(&matched, "alignedResult").proposition.time, peano(1));
    assert!(
        run(&m, "mismatchedValue")
            .known
            .iter()
            .all(|e| e.verb != "alignedResult")
    );
}
