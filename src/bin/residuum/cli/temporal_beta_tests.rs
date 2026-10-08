use super::*;
use residuum::typed::{Antecedent, Module, Rule, RuleBody, StateTime, Type};

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

fn input(module: &Module, name: &str) -> Event {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e.input[0].clone()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn temporal_constants_substitute_in_arguments_indices_and_transitions() {
    let module = residuum::parse_formal_to_typed(include_str!(
        "../../../../examples/experiments/temporal_beta_arguments.res"
    ))
    .unwrap();
    for (partial, fixed, direct, inline) in [
        (
            "partiallyRemember",
            "fixedRemember",
            "directRemember",
            "inlineRemember",
        ),
        (
            "partialRegister",
            "fixedRegister",
            "directRegister",
            "inlineRegister",
        ),
    ] {
        let fixed = rule(&module, fixed);
        assert!(fixed.parameters.is_empty());
        assert_eq!(fixed.body, rule(&module, direct).body);
        assert_eq!(fixed.body, rule(&module, inline).body);
        let clause = match &rule(&module, partial).body {
            RuleBody::HornClause(c) => c.clone(),
            RuleBody::ResidualClause(c) => c.unresiduate(),
        };
        let first = match &clause.antecedent {
            Antecedent::Event(e) => e,
            Antecedent::And(a, _) => match &**a {
                Antecedent::Event(e) => e,
                _ => panic!("event"),
            },
            _ => panic!("event"),
        };
        assert!(first.args.contains(&Term::Const("oldDate".into())));
        assert!(
            clause
                .consequent
                .args
                .contains(&Term::Const("oldDate".into()))
        );
        assert!(!first.args.contains(&Term::Var("effective".into())));
    }
    let partial = rule(&module, "partiallyRemember");
    assert_eq!(partial.parameters[0].name, "t");
    assert_eq!(partial.parameters[0].ty, Type::PropositionTime);
    let RuleBody::HornClause(c) = &partial.body else {
        panic!("Horn")
    };
    assert_eq!(
        c.consequent.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::Var("t".into())))
    );
    let remembered =
        residuum::instantiate(rule(&module, "fixedRemember"), &input(&module, "right")).unwrap();
    assert_eq!(remembered.event.verb, "recorded");
    assert_eq!(
        remembered.event.args,
        vec![Term::Const("juan".into()), Term::Const("oldDate".into())]
    );
    assert_eq!(
        remembered.event.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
    );
    assert!(remembered.record.substitution.bindings.is_empty());
    for wrong in ["wrongArgument", "wrongTime"] {
        assert!(
            residuum::instantiate(rule(&module, "fixedRemember"), &input(&module, wrong)).is_none()
        );
    }
    let registered = residuum::instantiate(
        rule(&module, "fixedRegister"),
        &input(&module, "sameBinderBothPositions"),
    )
    .unwrap();
    assert!(
        registered
            .event
            .args
            .contains(&Term::Const("oldDate".into()))
    );
    let after = TimeExpr::After(Box::new(TimeExpr::At("oldDate".into())));
    assert_eq!(registered.event.proposition.time, after);
    let transition = registered.event.transition.unwrap();
    assert_eq!(
        transition.input,
        StateTime::At(TimeExpr::At("oldDate".into()))
    );
    assert_eq!(transition.output, StateTime::At(after));
}
