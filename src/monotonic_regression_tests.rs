use crate::typed::{
    Declaration, DerivationTime, Event, Polarity, PropositionKind, PropositionType, Rule, RuleBody,
    StateTime, Term, TimeExpr,
};
use crate::{
    DerivationClock, IncrementalClosure, instantiate, parse_formal_to_typed, query_residual,
};

#[derive(Default)]
struct Clock;
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        DerivationTime {
            name: "regression".into(),
        }
    }
}

fn rule(module: &crate::Module, name: &str) -> Rule {
    module
        .declarations
        .iter()
        .find_map(|decl| match decl {
            Declaration::Rule(rule) if rule.name == name => Some(rule.clone()),
            _ => None,
        })
        .unwrap()
}

fn atom(verb: &str, kind: PropositionKind, time: &str) -> Event {
    Event {
        verb: verb.into(),
        args: vec![],
        polarity: Polarity::Positive,
        proposition: PropositionType {
            kind,
            time: TimeExpr::At(time.into()),
        },
        transition: None,
    }
}

#[test]
fn constant_time_is_literal_in_forward_replay_and_incremental_matching() {
    let module = parse_formal_to_typed(
        "module TimeScope
const tau : PropositionTime
const sigma : PropositionTime
seeded verb A()
derived verb B()
rule fixed() = A() @ tau <= B() @ tau
rule variable(t) = A() @ t <= B() @ after t",
    )
    .unwrap();
    let fixed = rule(&module, "fixed");
    let right = atom("A", PropositionKind::Seeded, "tau");
    let wrong = atom("A", PropositionKind::Seeded, "sigma");
    assert!(instantiate(&fixed, &wrong).is_none());
    let derived = instantiate(&fixed, &right).unwrap();
    assert_eq!(derived.event.proposition.time, TimeExpr::At("tau".into()));
    assert!(derived.record.substitution.bindings.is_empty());
    assert!(crate::derive::instantiate_from_witness(&fixed, &[wrong.clone()]).is_none());
    assert!(crate::derive::instantiate_from_witness(&fixed, &[right.clone()]).is_some());
    let goal = atom("B", PropositionKind::Derived, "tau");
    assert_eq!(
        query_residual(&fixed, &goal, &[wrong.clone()])
            .unwrap()
            .len(),
        1
    );
    assert!(
        query_residual(&fixed, &goal, &[right.clone()])
            .unwrap()
            .is_empty()
    );
    let mut state = IncrementalClosure::new(vec![], &[fixed], &mut Clock);
    assert!(
        state
            .update(vec![wrong.clone()], &mut Clock)
            .derivations
            .is_empty()
    );
    assert_eq!(state.update(vec![right], &mut Clock).derivations.len(), 1);
    let variable = instantiate(&rule(&module, "variable"), &wrong).unwrap();
    assert_eq!(
        variable.event.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("sigma".into())))
    );
}

#[test]
fn beta_reduction_substitutes_time_in_horn_residual_and_state_indices() {
    for separator in ["<=", "<= I -o"] {
        let source = format!(
            "module TemporalBeta
entity Person
const juan : Person
const tau : PropositionTime
seeded verb ready(subject : Person)
effect verb moved(subject : Person)
rule r(x, t) = ready(x) @ t {separator} moved(x) @ after t
rule fixed = r(juan, tau)
rule inline() = ready(juan) @ tau {separator} moved(juan) @ after tau"
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let fixed = rule(&module, "fixed");
        assert!(fixed.parameters.is_empty());
        assert_eq!(fixed.body, rule(&module, "inline").body);
        let mut seed = atom("ready", PropositionKind::Seeded, "tau");
        seed.args = vec![Term::Const("juan".into())];
        let event = instantiate(&fixed, &seed).unwrap().event;
        let transition = event.transition.unwrap();
        assert_eq!(transition.input, StateTime::At(TimeExpr::At("tau".into())));
        assert_eq!(transition.output, StateTime::At(event.proposition.time));
        assert!(
            instantiate(
                &fixed,
                &Event {
                    proposition: PropositionType {
                        kind: PropositionKind::Seeded,
                        time: TimeExpr::At("sigma".into())
                    },
                    ..seed
                }
            )
            .is_none()
        );
        assert!(matches!(
            fixed.body,
            RuleBody::HornClause(_) | RuleBody::ResidualClause(_)
        ));
    }
}

#[test]
fn rule_constants_keep_declared_types_in_arguments_and_time() {
    let declarations = "module ConstantTypes
entity Person
entity Conduct
entity Thing
entity Movable <: Thing
const deliver : Conduct
const car : Movable
const tau : PropositionTime
seeded verb trigger()
seeded verb sees(object : Thing)
derived verb marked(subject : Person)
derived verb performs(conduct : Conduct, object : Thing?)";
    for body in [
        "rule bad(t) = trigger() @ t <= marked(deliver) @ t",
        "rule bad(t) = sees(deliver) @ t <= performs(deliver, car) @ t",
        "rule bad() = I <= performs(deliver, car) @ deliver",
    ] {
        let error = parse_formal_to_typed(&format!("{declarations}\n{body}")).unwrap_err();
        assert!(error.to_string().contains("deliver"), "{error}");
    }
    let valid = parse_formal_to_typed(&format!(
        "{declarations}\nrule good() = I <= performs(deliver, car) @ tau"
    ))
    .unwrap();
    let good = rule(&valid, "good");
    assert!(
        good.parameters.is_empty(),
        "constants are not inferred binders"
    );
    let RuleBody::HornClause(body) = good.body else {
        panic!("Horn")
    };
    assert_eq!(
        body.consequent.args,
        [Term::Const("deliver".into()), Term::Const("car".into())]
    );
}
