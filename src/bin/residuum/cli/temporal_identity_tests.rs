use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{
    Antecedent, DerivationTime, Experiment, Module, Program, Rule, RuleBody, Type,
};
use residuum::{DerivationClock, close_program, instantiate_from_known, query_residual};

const SOURCE: &str = include_str!("../../../../examples/experiments/temporal_identity_formal.res");
fn rule<'a>(m: &'a Module, name: &str) -> &'a Rule {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == name => Some(r),
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
fn experiment<'a>(m: &'a Module, name: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}
fn clause(r: &Rule) -> &residuum::typed::HornClause {
    let RuleBody::HornClause(c) = &r.body else {
        panic!("Horn")
    };
    c
}
fn factors(a: &Antecedent) -> Vec<&Event> {
    match a {
        Antecedent::Unit => vec![],
        Antecedent::Event(e) => vec![e],
        Antecedent::And(a, b) => [factors(a), factors(b)].concat(),
    }
}
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        assert!(self.0 < 100, "finite identity fixture unexpectedly expands");
        DerivationTime {
            name: format!("identity-{}", self.0),
        }
    }
}

#[test]
fn temporal_partial_application_retains_homonymous_constant_and_binder() {
    let m = residuum::parse_formal_to_typed(SOURCE).unwrap();
    let partial = rule(&m, "partial");
    assert_eq!(
        partial
            .parameters
            .iter()
            .map(|p| (&*p.name, &p.ty))
            .collect::<Vec<_>>(),
        vec![
            ("object", &Type::Entity("Item".into())),
            ("t", &Type::PropositionTime)
        ]
    );
    let c = clause(partial);
    let events = factors(&c.antecedent);
    assert_eq!(events[0].proposition.time, TimeExpr::At("t".into()));
    assert_eq!(events[1].proposition.time, TimeExpr::Var("t".into()));
    assert_eq!(
        c.consequent.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::Var("t".into())))
    );
    assert_ne!(events[0].proposition.time, events[1].proposition.time);
    assert_eq!(rule(&m, "fixed").body, rule(&m, "direct").body);
    assert!(rule(&m, "fixed").parameters.is_empty());
    let ground = factors(&clause(rule(&m, "fixed")).antecedent);
    assert_eq!(ground[0].proposition.time, TimeExpr::At("t".into()));
    assert_eq!(ground[1].proposition.time, TimeExpr::At("tau".into()));

    let known = &experiment(&m, "correct").input[..2];
    let matched = instantiate_from_known(partial, known);
    assert_eq!(matched.len(), 1);
    assert_eq!(
        matched[0]
            .record
            .substitution
            .bindings
            .iter()
            .find(|b| b.parameter == "t")
            .unwrap()
            .value,
        SubstitutionValue::Time(TimeExpr::At("tau".into()))
    );
    assert_eq!(matched[0].record.antecedents, known);
    assert_eq!(
        matched[0].event.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
    );
    assert!(
        instantiate_from_known(partial, &experiment(&m, "wrongConstant").input[..2]).is_empty()
    );
    let wrong = &experiment(&m, "wrongParameter").input[..2];
    // The unactivated residual lambda can bind its free t to constant t.
    assert_eq!(
        instantiate_from_known(partial, wrong)[0]
            .event
            .proposition
            .time,
        TimeExpr::After(Box::new(TimeExpr::At("t".into())))
    );
    assert!(instantiate_from_known(rule(&m, "fixed"), wrong).is_empty());
}

#[test]
fn temporal_program_beta_preserves_capture_across_all_rule_uses() {
    let m = residuum::parse_formal_to_typed(SOURCE).unwrap();
    let partial = program(&m, "CapturedTime");
    assert_eq!(
        partial
            .parameters
            .iter()
            .map(|p| &*p.name)
            .collect::<Vec<_>>(),
        vec!["object", "t"]
    );
    let compiled = compile_program(partial, &m).unwrap();
    let events = factors(&clause(&compiled[0]).antecedent);
    assert_eq!(events[0].proposition.time, TimeExpr::At("t".into()));
    assert_eq!(
        events[1].proposition.time,
        TimeExpr::Var("#CapturedTime:shared:t".into())
    );
    assert_eq!(
        program(&m, "ProgramGround").rule_uses,
        program(&m, "ProgramDirect").rule_uses
    );
    for name in ["ProgramGround", "ProgramDirect"] {
        let rules = compile_program(program(&m, name), &m).unwrap();
        assert_eq!(rules[0].body, rule(&m, "fixed").body);
        let actual = instantiate_from_known(&rules[0], &experiment(&m, "correct").input[..2]);
        assert_eq!(
            actual[0].event,
            instantiate_from_known(rule(&m, "fixed"), &experiment(&m, "correct").input[..2])[0]
                .event
        );
        let origin = program_rule_origin(&m, &rules[0].name).unwrap();
        assert_eq!(origin.program, name);
        assert_eq!(origin.rule, "template");
        assert_eq!(origin.defining_module, "TemporalIdentity");
    }
}

#[test]
fn temporal_activation_and_provenance_do_not_reinterpret_captured_constants() {
    let m = residuum::parse_formal_to_typed(SOURCE).unwrap();
    for name in [
        "correct",
        "wrongConstant",
        "wrongParameter",
        "absentTriggers",
    ] {
        let original = experiment(&m, name).input.clone();
        let closure = close_program(original.clone(), &m, &mut Clock::default()).unwrap();
        assert_eq!(experiment(&m, name).input, original);
        let results: Vec<_> = closure.known.iter().filter(|e| e.verb == "C").collect();
        if name != "correct" {
            assert!(results.is_empty(), "{name}");
            continue;
        }
        assert_eq!(results.len(), 1);
        assert_eq!(closure.derivations.len(), 2);
        for proof in &closure.derivations {
            assert_eq!(
                proof.event.proposition.time,
                TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
            );
            let witnesses = &proof.record.antecedents;
            assert!(
                witnesses
                    .iter()
                    .any(|e| e.verb == "A" && e.proposition.time == TimeExpr::At("t".into()))
            );
            assert!(
                witnesses
                    .iter()
                    .any(|e| e.verb == "B" && e.proposition.time == TimeExpr::At("tau".into()))
            );
            assert!(
                proof
                    .record
                    .substitution
                    .bindings
                    .iter()
                    .any(|b| b.parameter.ends_with("t")
                        && b.value == SubstitutionValue::Time(TimeExpr::At("tau".into())))
            );
            if proof.record.rule == "partial" {
                assert!(witnesses.iter().any(|e| e.verb == "activateRule"));
            } else {
                assert!(witnesses.iter().any(|e| e.verb == "activateProgram"));
                let origin = program_rule_origin(&m, &proof.record.rule).unwrap();
                assert_eq!(origin.program, "CapturedTime");
                assert_eq!(origin.rule, "template");
                assert_eq!(origin.defining_module, "TemporalIdentity");
            }
        }
    }
}

#[test]
fn temporal_residual_goal_substitution_keeps_homonymous_constant_fixed() {
    let m = residuum::parse_formal_to_typed(SOURCE).unwrap();
    let goal = clause(rule(&m, "fixed")).consequent.clone();
    let partial = rule(&m, "partial");
    let correct = &experiment(&m, "correct").input;
    let requirements = query_residual(partial, &goal, &correct[1..2]).unwrap();
    assert_eq!(requirements.len(), 1);
    let events = factors(&requirements[0].required);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].verb, "A");
    assert_eq!(events[0].proposition.time, TimeExpr::At("t".into()));
    assert!(requirements[0].parameters.is_empty());
    assert_eq!(
        requirements[0]
            .goal_substitution
            .bindings
            .iter()
            .find(|b| b.parameter == "t")
            .unwrap()
            .value,
        SubstitutionValue::Time(TimeExpr::At("tau".into()))
    );
    assert!(
        !query_residual(partial, &goal, &experiment(&m, "wrongConstant").input[..2])
            .unwrap()
            .is_empty()
    );
    assert!(
        query_residual(partial, &goal, &correct[..2])
            .unwrap()
            .is_empty()
    );
}
