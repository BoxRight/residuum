use super::*;
use crate::typed::{
    Arg, Declaration, DerivationTime, Event, Module, Polarity, PropositionKind, PropositionType,
    StateTime, StateTransitionType, Substitution, SubstitutionBinding, SubstitutionValue, Term,
    TimeExpr, Type,
};
use crate::{DerivationClock, close, instantiate_from_known, parse_formal, parse_formal_to_typed};

const HORN: &str = include_str!("../../examples/residual/horn_formal.res");
const CURRIED: &str = include_str!("../../examples/residual/curried_formal.res");
const UNITS: &str = include_str!("../../examples/residual/unit_formal.res");

fn rule<'a>(module: &'a Module, name: &str) -> &'a Rule {
    module
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Rule(rule) if rule.name == name => Some(rule),
            _ => None,
        })
        .unwrap()
}

fn seed<const N: usize>(verb: &str, args: [&str; N], time: &str) -> Event {
    Event {
        polarity: Polarity::Positive,
        verb: verb.into(),
        args: args.into_iter().map(|arg| Term::Var(arg.into())).collect(),
        proposition: PropositionType {
            kind: PropositionKind::Seeded,
            time: TimeExpr::At(time.into()),
        },
        transition: None,
    }
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}

fn goal() -> Event {
    let mut event = seed("eligible", ["alice", "car"], "tau");
    event.proposition.kind = PropositionKind::Derived;
    event.proposition.time = TimeExpr::After(Box::new(TimeExpr::At("tau".into())));
    event
}

#[test]
fn unit_elaborates_explicitly_in_horn_and_residual_without_time_or_binders() {
    let surface = parse_formal(UNITS).unwrap();
    assert_eq!(surface, crate::formal::parse_formal_manual(UNITS).unwrap());
    assert_eq!(surface, parse_formal(&UNITS.replace("-o", "⊸")).unwrap());
    let module = crate::elaborate_formal(surface).unwrap();
    let empty = rule(&module, "emptyContext");
    let RuleBody::ResidualClause(clause) = &empty.body else {
        panic!("residual");
    };
    assert_eq!(clause.antecedent, Antecedent::Unit);
    assert_eq!(
        empty.parameters,
        vec![
            Arg {
                name: "person".into(),
                ty: Type::Entity("Person".into())
            },
            Arg {
                name: "object".into(),
                ty: Type::Entity("Thing".into())
            },
            Arg {
                name: "t".into(),
                ty: Type::PropositionTime
            },
        ]
    );
    let RuleBody::ResidualClause(clause) = &rule(&module, "noRequirement").body else {
        panic!("residual");
    };
    assert_eq!(clause.consequent.required, Antecedent::Unit);
    for name in ["emptyContext", "leftUnit", "rightUnit", "noRequirement"] {
        let lambda = rule(&module, name);
        assert_eq!(residuate(&unresiduate(lambda)).unwrap(), *lambda);
    }
    let horn = rule(&module, "unitHorn");
    assert_eq!(unresiduate(&residuate(horn).unwrap()), *horn);
    let specialized = parse_formal_to_typed(&format!(
        "{}\nrule specialized = emptyContext(alice)\n",
        UNITS.replace("entity Person", "entity Person\nconst alice : Person")
    ))
    .unwrap();
    let lambda = rule(&specialized, "specialized");
    let RuleBody::ResidualClause(clause) = &lambda.body else {
        panic!("residual");
    };
    assert_eq!(clause.antecedent, Antecedent::Unit);
    assert_eq!(
        clause.consequent.consequent.args[0],
        Term::Const("alice".into())
    );
    assert_eq!(lambda.parameters.len(), 2);
}

#[test]
fn unit_match_is_neutral_independently_of_known_and_is_identity_on_both_sides() {
    use crate::derive::{match_antecedents, match_antecedents_with, substitution_record};
    let request = seed("requests", ["alice", "car"], "tau");
    for known in [
        vec![],
        vec![request.clone(), seed("available", ["car"], "tau")],
    ] {
        let matches = match_antecedents(&Antecedent::Unit, &known, &[]);
        assert_eq!(
            matches.len(),
            1,
            "Unit has one neutral witness for any Known"
        );
        assert!(matches[0].antecedents.is_empty());
        assert!(matches[0].substitutions.is_empty());
    }
    let unit_unit = Antecedent::And(Box::new(Antecedent::Unit), Box::new(Antecedent::Unit));
    assert_eq!(match_antecedents(&unit_unit, &[], &[]).len(), 1);
    let module = parse_formal_to_typed(CURRIED).unwrap();
    let RuleBody::ResidualClause(clause) = &rule(&module, "eligibility").body else {
        panic!("residual");
    };
    let atom = clause.antecedent.clone();
    let known = vec![request.clone(), seed("requests", ["bob", "house"], "later")];
    let parameters = &rule(&module, "eligibility").parameters;
    let expected: Vec<_> = match_antecedents(&atom, &known, parameters)
        .into_iter()
        .map(|matched| {
            (
                matched.antecedents,
                substitution_record(&matched.substitutions),
            )
        })
        .collect();
    for antecedent in [
        Antecedent::And(Box::new(Antecedent::Unit), Box::new(atom.clone())),
        Antecedent::And(Box::new(atom.clone()), Box::new(Antecedent::Unit)),
        Antecedent::And(Box::new(unit_unit), Box::new(atom)),
    ] {
        let actual: Vec<_> = match_antecedents(&antecedent, &known, parameters)
            .into_iter()
            .map(|matched| {
                (
                    matched.antecedents,
                    substitution_record(&matched.substitutions),
                )
            })
            .collect();
        assert_eq!(actual, expected);
    }
    let bindings = std::collections::BTreeMap::from([
        (
            "object".into(),
            SubstitutionValue::Term(Term::Var("car".into())),
        ),
        (
            "t".into(),
            SubstitutionValue::Time(TimeExpr::At("tau".into())),
        ),
    ]);
    let matches = match_antecedents_with(
        &Antecedent::Unit,
        &[],
        parameters,
        bindings.clone(),
        vec![request.clone()],
    );
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].substitutions, bindings);
    assert_eq!(matches[0].antecedents, vec![request]);
}

#[test]
fn unit_identities_preserve_forward_provenance_and_residual_requirements() {
    let module = parse_formal_to_typed(UNITS).unwrap();
    let baseline = parse_formal_to_typed(CURRIED).unwrap();
    let request = seed("requests", ["alice", "car"], "tau");
    let available = seed("available", ["car"], "tau");
    let expected = query_residual(rule(&baseline, "eligibility"), &goal(), &[request.clone()])
        .unwrap()
        .remove(0);
    for name in ["leftUnit", "rightUnit"] {
        let lambda = rule(&module, name);
        let mut requirement = query_residual(lambda, &goal(), &[request.clone()])
            .unwrap()
            .remove(0);
        assert_eq!(requirement.rule.name, name);
        requirement.rule = expected.rule.clone();
        assert_eq!(requirement, expected);
        let closure = close(
            vec![request.clone(), available.clone()],
            &[lambda.clone()],
            &mut Clock::default(),
        );
        assert_eq!(
            closure.known,
            vec![request.clone(), available.clone(), goal()]
        );
        assert_eq!(closure.derivations.len(), 1);
        assert_eq!(
            closure.derivations[0].record.antecedents,
            vec![request.clone(), available.clone()]
        );
        assert_eq!(
            closure.derivations[0].record.substitution,
            expected.substitution
        );
        assert_eq!(closure.derivations[0].derived_at.name, "d1");
    }
    let lambda = rule(&module, "noRequirement");
    let known = vec![request.clone()];
    assert!(
        query_residual(lambda, &goal(), &known).unwrap().is_empty(),
        "I is already satisfied"
    );
    let closure = close(known.clone(), &[lambda.clone()], &mut Clock::default());
    assert_eq!(known, vec![request.clone()]);
    assert_eq!(closure.known, vec![request.clone(), goal()]);
    assert_eq!(closure.derivations[0].record.antecedents, vec![request]);
    let empty = rule(&module, "emptyContext");
    let pending = query_residual(empty, &goal(), &[]).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].required, Antecedent::Event(available));
    assert!(pending[0].evidence.is_empty());
    assert!(pending[0].parameters.is_empty());
    assert_eq!(pending[0].goal_substitution, expected.goal_substitution);
}

#[test]
fn unit_ground_rule_starts_fixpoint_from_empty_seeds_and_stamps_once() {
    use crate::typed::HornClause;
    // Ground runtime identities/time have no remaining lambda parameters.
    let axiom = Rule {
        name: "unitAxiom".into(),
        parameters: vec![],
        body: RuleBody::HornClause(HornClause {
            antecedent: Antecedent::Unit,
            consequent: goal(),
        }),
    };
    let mut clock = Clock::default();
    let closure = close(vec![], &[axiom.clone()], &mut clock);
    assert_eq!(closure.known, vec![goal()]);
    assert_eq!(closure.derivations.len(), 1);
    assert!(closure.derivations[0].record.antecedents.is_empty());
    assert!(
        closure.derivations[0]
            .record
            .substitution
            .bindings
            .is_empty()
    );
    assert_eq!(closure.derivations[0].record.rule, "unitAxiom");
    assert_eq!(closure.derivations[0].derived_at.name, "d1");
    assert_eq!(
        clock.0, 1,
        "repeating an empty match does not restamp the same justification"
    );
    let source = format!(
        "{CURRIED}\nrule corroborate(person, object, t) = eligible(person, object) @ t <= eligible(person, object) @ t\n"
    );
    let module = parse_formal_to_typed(&source).unwrap();
    let closure = close(
        vec![],
        &[rule(&module, "corroborate").clone(), axiom],
        &mut Clock::default(),
    );
    assert_eq!(closure.known, vec![goal()]);
    assert_eq!(
        closure.derivations.len(),
        2,
        "ordinary Horn sees the proposition justified by I"
    );
    assert_eq!(closure.derivations[1].record.rule, "corroborate");
    assert_eq!(closure.derivations[1].record.antecedents, vec![goal()]);
    assert_eq!(closure.derivations[1].derived_at.name, "d2");
    let mut clock = Clock::default();
    let ordinary = close(vec![], &[rule(&module, "eligibility").clone()], &mut clock);
    assert!(ordinary.known.is_empty());
    assert!(ordinary.derivations.is_empty());
    assert_eq!(clock.0, 0);
}

#[test]
fn unit_parser_requires_a_token_and_preserves_i_verb_applications() {
    let antecedent = "requests(person, object) @ t";
    for invalid in ["", "I &", "& I", "I @ t", "~I"] {
        let source = CURRIED.replace(antecedent, invalid);
        assert!(parse_formal(&source).is_err(), "{invalid}");
        assert!(
            crate::formal::parse_formal_manual(&source).is_err(),
            "{invalid}"
        );
    }
    let source = CURRIED.replace("requests(", "I(");
    assert_eq!(
        parse_formal(&source).unwrap(),
        crate::formal::parse_formal_manual(&source).unwrap()
    );
    let module = parse_formal_to_typed(&source).unwrap();
    let RuleBody::ResidualClause(clause) = &rule(&module, "eligibility").body else {
        panic!("residual");
    };
    let Antecedent::Event(event) = &clause.antecedent else {
        panic!("I(...) is a timed event");
    };
    assert_eq!(event.verb, "I");
    assert_eq!(event.proposition.time, TimeExpr::At("t".into()));
    assert_eq!(
        query_residual(
            rule(&module, "eligibility"),
            &goal(),
            &[seed("I", ["alice", "car"], "tau")]
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn query_returns_requirement_without_asserting_or_stamping_evidence() {
    let module = parse_formal_to_typed(CURRIED).unwrap();
    let lambda = rule(&module, "eligibility");
    let request = seed("requests", ["alice", "car"], "tau");
    let available = seed("available", ["car"], "tau");
    let target = goal();
    let mut clock = Clock::default();
    let context = close(vec![request.clone()], &[lambda.clone()], &mut clock);
    let original = context.clone();
    let original_rule = lambda.clone();
    let requirements = query_residual(lambda, &target, &context.known).unwrap();
    assert_eq!(requirements.len(), 1);
    let requirement = &requirements[0];
    assert_eq!(requirement.rule.name, "eligibility");
    assert_eq!(requirement.evidence, vec![request.clone()]);
    assert_eq!(requirement.goal, target);
    assert_eq!(requirement.required, Antecedent::Event(available.clone()));
    assert!(requirement.parameters.is_empty());
    assert_eq!(requirement.goal_substitution, requirement.substitution);
    assert_eq!(
        requirement.substitution,
        Substitution {
            bindings: vec![
                SubstitutionBinding {
                    parameter: "object".into(),
                    value: SubstitutionValue::Term(Term::Var("car".into()))
                },
                SubstitutionBinding {
                    parameter: "person".into(),
                    value: SubstitutionValue::Term(Term::Var("alice".into()))
                },
                SubstitutionBinding {
                    parameter: "t".into(),
                    value: SubstitutionValue::Time(TimeExpr::At("tau".into()))
                },
            ]
        }
    );
    assert_eq!(context, original);
    assert_eq!(*lambda, original_rule);
    assert_eq!(clock.0, 0);
    assert!(!context.known.contains(&available));
    assert!(!context.known.contains(&target));
    assert!(context.derivations.is_empty());
    assert_eq!(
        query_residual(lambda, &target, &context.known).unwrap(),
        requirements
    );
    let horn = parse_formal_to_typed(HORN).unwrap();
    assert_eq!(
        query_residual(rule(&horn, "eligibility"), &target, &context.known).unwrap(),
        requirements
    );
    assert_eq!(
        query_residual(
            &residuate(rule(&horn, "eligibility")).unwrap(),
            &target,
            &context.known
        )
        .unwrap(),
        requirements
    );
    // B is supplied independently by the caller as evidence; only forward Horn derives C.
    let facts = vec![request.clone(), available.clone()];
    assert!(query_residual(lambda, &target, &facts).unwrap().is_empty());
    let completed = close(facts.clone(), &[lambda.clone()], &mut clock);
    assert_eq!(completed.derivations.len(), 1);
    assert_eq!(completed.derivations[0].event, target);
    assert_eq!(completed.derivations[0].record.antecedents, facts);
    assert_eq!(completed.derivations[0].derived_at.name, "d1");
    assert!(
        completed
            .derivations
            .iter()
            .all(|derived| derived.event != available)
    );
}

#[test]
fn query_filters_goal_and_checks_b_using_a_bindings() {
    let module = parse_formal_to_typed(CURRIED).unwrap();
    let lambda = rule(&module, "eligibility");
    let request = seed("requests", ["alice", "car"], "tau");
    let available = seed("available", ["car"], "tau");
    let target = goal();
    let pending = query_residual(lambda, &target, &[]).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].required,
        Antecedent::And(
            Box::new(Antecedent::Event(request.clone())),
            Box::new(Antecedent::Event(available.clone())),
        )
    );
    assert!(pending[0].evidence.is_empty());
    assert!(pending[0].parameters.is_empty());
    let pending = query_residual(lambda, &target, &[available.clone()]).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].required, Antecedent::Event(request.clone()));
    assert_eq!(pending[0].evidence, vec![available.clone()]);
    let mut other_person = target.clone();
    other_person.args[0] = Term::Var("bob".into());
    let mut other_object = target.clone();
    other_object.args[1] = Term::Var("house".into());
    let mut other_time = target.clone();
    other_time.proposition.time = TimeExpr::After(Box::new(TimeExpr::At("later".into())));
    let mut other_verb = target.clone();
    other_verb.verb = "other".into();
    let mut other_kind = target.clone();
    other_kind.proposition.kind = PropositionKind::Seeded;
    let mut other_sign = target.clone();
    other_sign.polarity = Polarity::EvidentialNot;
    for (other, request, available) in [
        (
            other_person,
            seed("requests", ["bob", "car"], "tau"),
            seed("available", ["car"], "tau"),
        ),
        (
            other_object,
            seed("requests", ["alice", "house"], "tau"),
            seed("available", ["house"], "tau"),
        ),
        (
            other_time,
            seed("requests", ["alice", "car"], "later"),
            seed("available", ["car"], "later"),
        ),
    ] {
        let pending =
            query_residual(lambda, &other, &[seed("requests", ["alice", "car"], "tau")]).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].goal, other);
        assert_eq!(
            pending[0].required,
            Antecedent::And(
                Box::new(Antecedent::Event(request)),
                Box::new(Antecedent::Event(available)),
            )
        );
        assert!(
            pending[0].evidence.is_empty(),
            "unrelated evidence cannot override the goal bindings"
        );
    }
    for other in [other_verb, other_kind, other_sign] {
        assert!(
            query_residual(lambda, &other, &[request.clone()])
                .unwrap()
                .is_empty()
        );
    }
    let mut wrong_object = available.clone();
    wrong_object.args[0] = Term::Var("house".into());
    let mut wrong_time = available.clone();
    wrong_time.proposition.time = TimeExpr::At("later".into());
    let mut wrong_sign = available.clone();
    wrong_sign.polarity = Polarity::EvidentialNot;
    for other in [wrong_object, wrong_time, wrong_sign] {
        let requirements = query_residual(lambda, &target, &[request.clone(), other]).unwrap();
        assert_eq!(
            requirements.len(),
            1,
            "unrelated B must not rebind car or tau and suppress the requirement"
        );
        assert_eq!(
            requirements[0].required,
            Antecedent::Event(available.clone())
        );
    }
    let another_request = seed("requests", ["alice", "house"], "tau");
    let requirements =
        query_residual(lambda, &target, &[another_request, request.clone()]).unwrap();
    assert_eq!(requirements.len(), 1);
    assert_eq!(requirements[0].evidence, vec![request]);
}

#[test]
fn query_preserves_conjunctive_a_and_each_witness_substitution() {
    let source = CURRIED
        .replace("seeded verb available(object : Thing)", "seeded verb available(object : Thing)\nseeded verb confirms(subject : Person, object : Thing)")
        .replace("person, object, t)", "person, object, t, witness)")
        .replace("requests(person, object) @ t", "requests(person, object) @ t & confirms(witness, object) @ t");
    let module = parse_formal_to_typed(&source).unwrap();
    let lambda = rule(&module, "eligibility");
    let request = seed("requests", ["alice", "car"], "tau");
    let bob = seed("confirms", ["bob", "car"], "tau");
    let charlie = seed("confirms", ["charlie", "car"], "tau");
    for wrong in [
        seed("confirms", ["bob", "house"], "tau"),
        seed("confirms", ["bob", "car"], "later"),
    ] {
        let pending = query_residual(lambda, &goal(), &[request.clone(), wrong]).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].required,
            Antecedent::And(
                Box::new(Antecedent::Event(seed(
                    "confirms",
                    ["witness", "car"],
                    "tau"
                ))),
                Box::new(Antecedent::Event(seed("available", ["car"], "tau"))),
            )
        );
        assert_eq!(pending[0].evidence, vec![request.clone()]);
        assert_eq!(
            pending[0].parameters,
            vec![Arg {
                name: "witness".into(),
                ty: Type::Entity("Person".into()),
            }]
        );
    }
    let known = vec![request.clone(), bob.clone(), charlie.clone()];
    let original = known.clone();
    let requirements = query_residual(lambda, &goal(), &known).unwrap();
    assert_eq!(
        requirements.len(),
        2,
        "distinct A justifications are retained"
    );
    for (requirement, evidence) in requirements.iter().zip([bob, charlie]) {
        assert_eq!(
            requirement.evidence,
            vec![request.clone(), evidence.clone()]
        );
        assert_eq!(
            requirement.required,
            Antecedent::Event(seed("available", ["car"], "tau"))
        );
        assert_eq!(requirement.substitution.bindings.len(), 4);
        assert_eq!(requirement.goal_substitution.bindings.len(), 3);
        assert!(
            requirement
                .substitution
                .bindings
                .contains(&SubstitutionBinding {
                    parameter: "witness".into(),
                    value: SubstitutionValue::Term(evidence.args[0].clone())
                })
        );
        assert!(requirement.parameters.is_empty());
    }
    assert_eq!(known, original);
}

#[test]
fn query_reduces_partial_b_and_retains_typed_unbound_parameters() {
    let source = CURRIED
        .replace("seeded verb available(object : Thing)", "seeded verb available(object : Thing)\nseeded verb approved(subject : Person, object : Thing)")
        .replace("person, object, t)", "person, object, t, observer)")
        .replace("available(object) @ t", "available(object) @ t & approved(observer, object) @ t");
    let module = parse_formal_to_typed(&source).unwrap();
    let lambda = rule(&module, "eligibility");
    let request = seed("requests", ["alice", "car"], "tau");
    let available = seed("available", ["car"], "tau");
    let known = vec![request.clone(), available.clone()];
    let requirement = query_residual(lambda, &goal(), &known).unwrap().remove(0);
    assert_eq!(
        requirement.parameters,
        vec![Arg {
            name: "observer".into(),
            ty: Type::Entity("Person".into())
        }]
    );
    assert_eq!(
        requirement.required,
        Antecedent::Event(seed("approved", ["observer", "car"], "tau"))
    );
    assert_eq!(
        requirement.evidence,
        vec![request.clone(), available.clone()]
    );
    assert_eq!(
        requirement.substitution.bindings.len(),
        3,
        "observer remains a typed unbound parameter"
    );
    assert!(
        !requirement
            .substitution
            .bindings
            .iter()
            .any(|binding| binding.parameter == "observer")
    );
    let mut wrong = known.clone();
    wrong.push(seed("approved", ["bob", "house"], "tau"));
    assert_eq!(
        query_residual(lambda, &goal(), &wrong).unwrap(),
        vec![requirement]
    );
    let mut satisfied = known;
    satisfied.push(seed("approved", ["bob", "car"], "tau"));
    assert!(
        query_residual(lambda, &goal(), &satisfied)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn query_preserves_explicit_negative_requirements_and_nested_times() {
    let signed = CURRIED
        .replace("available(object) @ t", "~available(object) @ t")
        .replace("-o eligible", "-o ~eligible");
    let module = parse_formal_to_typed(&signed).unwrap();
    let lambda = rule(&module, "eligibility");
    let request = seed("requests", ["alice", "car"], "tau");
    let available = seed("available", ["car"], "tau");
    let mut negative = available.clone();
    negative.polarity = Polarity::EvidentialNot;
    let mut target = goal();
    target.polarity = Polarity::EvidentialNot;
    let known = vec![request.clone(), available.clone()];
    let original = known.clone();
    let requirements = query_residual(lambda, &target, &known).unwrap();
    assert_eq!(requirements.len(), 1);
    assert_eq!(
        requirements[0].required,
        Antecedent::Event(negative.clone())
    );
    assert_eq!(requirements[0].goal, target);
    assert_eq!(
        known, original,
        "absence produces a pending negative requirement, never negative evidence"
    );
    assert!(!known.contains(&negative));
    assert!(query_residual(lambda, &goal(), &known).unwrap().is_empty());
    let satisfied = vec![request.clone(), available, negative];
    assert!(
        query_residual(lambda, &target, &satisfied)
            .unwrap()
            .is_empty()
    );
    let module = parse_formal_to_typed(
        &CURRIED.replace("available(object) @ t", "available(object) @ after t"),
    )
    .unwrap();
    let mut lambda = rule(&module, "eligibility").clone();
    let RuleBody::ResidualClause(clause) = &mut lambda.body else {
        panic!("residual");
    };
    clause.consequent.consequent.proposition.time = TimeExpr::After(Box::new(
        clause.consequent.consequent.proposition.time.clone(),
    ));
    let mut target = goal();
    target.proposition.time = TimeExpr::After(Box::new(target.proposition.time.clone()));
    let mut required = seed("available", ["car"], "tau");
    required.proposition.time = TimeExpr::After(Box::new(required.proposition.time.clone()));
    let requirements = query_residual(
        &lambda,
        &target,
        &[request.clone(), seed("available", ["car"], "tau")],
    )
    .unwrap();
    assert_eq!(
        requirements[0].required,
        Antecedent::Event(required.clone())
    );
    assert_eq!(requirements[0].goal, target);
    assert!(
        query_residual(&lambda, &goal(), &[request.clone()])
            .unwrap()
            .is_empty()
    );
    assert!(
        query_residual(&lambda, &target, &[request, required])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn query_of_beta_reduced_lambda_preserves_constants_and_origin() {
    let source = format!(
        "{}\nrule aliceEligibility = eligibility(alice)\nrule carEligibility = aliceEligibility(car)\n",
        CURRIED.replace(
            "entity Thing",
            "entity Thing\nconst alice : Person\nconst car : Thing"
        )
    );
    let module = parse_formal_to_typed(&source).unwrap();
    let lambda = rule(&module, "carEligibility");
    let mut request = seed("requests", ["alice", "car"], "tau");
    request.args = vec![Term::Const("alice".into()), Term::Const("car".into())];
    let mut target = goal();
    target.args = request.args.clone();
    let mut available = seed("available", ["car"], "tau");
    available.args = vec![Term::Const("car".into())];
    let requirements = query_residual(lambda, &target, &[request.clone()]).unwrap();
    assert_eq!(requirements.len(), 1);
    assert_eq!(requirements[0].rule.name, "carEligibility");
    assert_eq!(requirements[0].goal, target);
    assert_eq!(
        requirements[0].required,
        Antecedent::Event(available.clone())
    );
    assert_eq!(
        requirements[0].substitution.bindings,
        vec![SubstitutionBinding {
            parameter: "t".into(),
            value: SubstitutionValue::Time(TimeExpr::At("tau".into()))
        }]
    );
    assert!(requirements[0].parameters.is_empty());
    assert_eq!(
        query_residual(
            lambda,
            &target,
            &[request.clone(), seed("available", ["car"], "tau")]
        )
        .unwrap(),
        requirements,
        "constants are not wildcard runtime identifiers"
    );
    assert!(
        query_residual(lambda, &target, &[request, available])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn query_accepts_derived_a_and_keeps_effect_goal_out_of_closure() {
    let source = format!("{}\nrule observation(person, object, t) = observed(person, object) @ t <= requests(person, object) @ after t\n", CURRIED
        .replace("entity Person", "entity Person { assets : Set Thing }")
        .replace("seeded verb requests(subject : Person, object : Thing)", "seeded verb observed(subject : Person, object : Thing)\nderived verb requests(subject : Person, object : Thing)")
        .replace("derived verb eligible(subject : Person, object : Thing)", "effect verb eligible(subject : Person, object : Thing) => subject.assets += object"));
    let module = parse_formal_to_typed(&source).unwrap();
    let lambda = rule(&module, "eligibility");
    let mut clock = Clock::default();
    let context = close(
        vec![seed("observed", ["alice", "car"], "tau")],
        &[rule(&module, "observation").clone()],
        &mut clock,
    );
    let original = context.clone();
    let mut target = goal();
    let input = target.proposition.time.clone();
    let output = TimeExpr::After(Box::new(input.clone()));
    target.proposition.kind = PropositionKind::Effect;
    target.proposition.time = output.clone();
    target.transition = Some(StateTransitionType {
        input: StateTime::At(input.clone()),
        output: StateTime::At(output),
    });
    let mut available = seed("available", ["car"], "tau");
    available.proposition.time = input;
    let requirements = query_residual(lambda, &target, &context.known).unwrap();
    assert_eq!(requirements.len(), 1);
    assert_eq!(
        requirements[0].evidence,
        vec![context.derivations[0].event.clone()]
    );
    assert_eq!(requirements[0].goal, target);
    assert_eq!(
        requirements[0].required,
        Antecedent::Event(available.clone())
    );
    assert_eq!(context, original);
    assert_eq!(context.derivations[0].derived_at.name, "d1");
    assert_eq!(clock.0, 1);
    assert!(!context.known.contains(&available));
    assert!(!context.known.contains(&target));
}

#[test]
fn query_binds_head_only_parameters_and_keeps_unbound_body_scope() {
    let source = CURRIED
        .replace("person, object, t)", "person, object, alternate, t)")
        .replace("available(object) @ t", "available(alternate) @ t")
        .replace(
            "-o eligible(person, object)",
            "-o eligible(person, alternate)",
        );
    let module = parse_formal_to_typed(&source).unwrap();
    let lambda = rule(&module, "eligibility");
    let known = vec![seed("requests", ["alice", "car"], "tau")];
    let empty = query_residual(lambda, &goal(), &[]).unwrap();
    assert_eq!(empty.len(), 1);
    assert_eq!(
        empty[0].required,
        Antecedent::And(
            Box::new(Antecedent::Event(seed(
                "requests",
                ["alice", "object"],
                "tau"
            ))),
            Box::new(Antecedent::Event(seed("available", ["car"], "tau"))),
        )
    );
    assert_eq!(
        empty[0].parameters,
        vec![Arg {
            name: "object".into(),
            ty: Type::Entity("Thing".into()),
        }]
    );
    assert_eq!(empty[0].goal_substitution.bindings.len(), 3);
    let pending = query_residual(lambda, &goal(), &known).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].required,
        Antecedent::Event(seed("available", ["car"], "tau"))
    );
    assert!(pending[0].parameters.is_empty());
    assert_eq!(pending[0].evidence, known);
    assert_eq!(pending[0].goal_substitution, empty[0].goal_substitution);
    assert_eq!(pending[0].substitution.bindings.len(), 4);
    let mut symbolic = goal();
    symbolic.args[1] = Term::Var("alternate".into());
    let pending = query_residual(lambda, &symbolic, &known).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].required,
        Antecedent::Event(seed("available", ["alternate"], "tau"))
    );
    assert_eq!(
        pending[0].goal, symbolic,
        "runtime identifiers may share a spelling with a rule binder"
    );
}

#[test]
fn formal_aliases_preserve_surface_ast_and_typed_adjunction() {
    let horn = parse_formal_to_typed(HORN).unwrap();
    let source = CURRIED.replace("-o", "⊸").replace("<=", "≤");
    let ascii = parse_formal(CURRIED).unwrap();
    assert_eq!(ascii, parse_formal(&source).unwrap());
    assert_eq!(ascii, crate::formal::parse_formal_manual(CURRIED).unwrap());
    assert_eq!(ascii, crate::formal::parse_formal_manual(&source).unwrap());
    let curried = crate::elaborate_formal(ascii).unwrap();
    assert_eq!(curried, parse_formal_to_typed(&source).unwrap());
    let forward = rule(&horn, "eligibility");
    let residual = rule(&curried, "eligibility");
    assert_eq!(
        residual.parameters,
        vec![
            Arg {
                name: "person".into(),
                ty: Type::Entity("Person".into())
            },
            Arg {
                name: "object".into(),
                ty: Type::Entity("Thing".into())
            },
            Arg {
                name: "t".into(),
                ty: Type::PropositionTime
            },
        ]
    );
    assert!(matches!(residual.body, RuleBody::ResidualClause(_)));
    assert_eq!(residuate(forward).unwrap(), *residual);
    assert_eq!(unresiduate(residual), *forward);
    assert_eq!(residuate(&unresiduate(residual)).unwrap(), *residual);
    assert_eq!(unresiduate(forward), *forward);
    assert_eq!(
        residuate(residual).unwrap_err().to_string(),
        "residuate requires a Horn clause"
    );
    let mut unit = forward.clone();
    let RuleBody::HornClause(clause) = &mut unit.body else {
        panic!("Horn");
    };
    let Antecedent::And(left, _) = &clause.antecedent else {
        panic!("conjunction");
    };
    clause.antecedent = (**left).clone();
    assert_eq!(
        residuate(&unit).unwrap_err().to_string(),
        "residuate requires a conjunctive antecedent"
    );
}

#[test]
fn round_trip_preserves_grouping_signs_constants_and_effect_times() {
    let source = CURRIED
        .replace("entity Person", "entity Person { assets : Set Thing }")
        .replace("entity Thing", "entity Thing\nconst car : Thing")
        .replace(
            "derived verb eligible(subject : Person, object : Thing)",
            "effect verb eligible(subject : Person, object : Thing) => subject.assets += object",
        )
        .replace("person, object, t)", "person, object, t, u)")
        .replace(
            "requests(person, object) @ t",
            "requests(person, object) @ t & requests(person, object) @ u",
        )
        .replace(
            "available(object) @ t",
            "~available(car) @ after u & available(object) @ u",
        )
        .replace("-o eligible", "-o ~eligible");
    let module = parse_formal_to_typed(&source).unwrap();
    let mut original = rule(&module, "eligibility").clone();
    let RuleBody::ResidualClause(clause) = &mut original.body else {
        panic!("residual");
    };
    let Antecedent::And(left, _) = &mut clause.consequent.required else {
        panic!("grouped B");
    };
    let Antecedent::Event(event) = &mut **left else {
        panic!("event");
    };
    event.proposition.time = TimeExpr::After(Box::new(event.proposition.time.clone()));
    let forward = unresiduate(&original);
    let restored = residuate(&forward).unwrap();
    assert_eq!(
        restored, original,
        "exact tree round-trip, without reassociation or simplification"
    );
    let RuleBody::ResidualClause(clause) = &original.body else {
        panic!("residual");
    };
    assert!(matches!(clause.antecedent, Antecedent::And(_, _)));
    assert_eq!(
        clause.consequent.consequent.proposition.kind,
        PropositionKind::Effect
    );
    assert_eq!(
        clause.consequent.consequent.polarity,
        Polarity::EvidentialNot
    );
    let transition = clause.consequent.consequent.transition.as_ref().unwrap();
    let RuleBody::HornClause(forward_clause) = &forward.body else {
        panic!("forward Horn view");
    };
    let RuleBody::ResidualClause(restored_clause) = &restored.body else {
        panic!("restored residual");
    };
    assert_eq!(
        forward_clause.consequent.transition.as_ref(),
        Some(transition)
    );
    assert_eq!(
        restored_clause.consequent.consequent.transition.as_ref(),
        Some(transition),
        "logical Event equality does not check transition preservation"
    );
    assert_eq!(transition.input, StateTime::At(TimeExpr::At("t".into())));
    assert_eq!(
        transition.output,
        StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("t".into()))))
    );
}

#[test]
fn conjunctions_share_lambda_scope_without_equating_times() {
    let source = CURRIED
        .replace("person, object, t)", "person, object, ta, tb)")
        .replace(
            "requests(person, object) @ t",
            "requests(person, object) @ ta",
        )
        .replace(
            "available(object) @ t",
            "available(object) @ tb & available(object) @ after tb",
        )
        .replace(
            "eligible(person, object) @ after t",
            "eligible(person, object) @ after ta",
        );
    assert_eq!(
        parse_formal(&source).unwrap(),
        crate::formal::parse_formal_manual(&source).unwrap()
    );
    let module = parse_formal_to_typed(&source).unwrap();
    let residual = rule(&module, "eligibility");
    assert_eq!(
        residual.parameters[2..],
        [
            Arg {
                name: "ta".into(),
                ty: Type::PropositionTime
            },
            Arg {
                name: "tb".into(),
                ty: Type::PropositionTime
            },
        ]
    );
    let RuleBody::ResidualClause(clause) = &residual.body else {
        panic!("residual");
    };
    assert!(matches!(clause.consequent.required, Antecedent::And(_, _)));
    let normalized = unresiduate(residual);
    assert_eq!(residuate(&normalized).unwrap(), *residual);
    let mut after_available = seed("available", ["car"], "later");
    after_available.proposition.time = TimeExpr::After(Box::new(TimeExpr::At("later".into())));
    let facts = vec![
        seed("requests", ["alice", "car"], "tau"),
        seed("available", ["car"], "later"),
        after_available,
    ];
    let derived = instantiate_from_known(residual, &facts);
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].record.antecedents, facts);
    assert_eq!(
        derived[0].event.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
    );
    assert!(
        derived[0]
            .record
            .substitution
            .bindings
            .contains(&SubstitutionBinding {
                parameter: "tb".into(),
                value: SubstitutionValue::Time(TimeExpr::At("later".into())),
            })
    );
}

#[test]
fn beta_reduction_commutes_with_unresiduation() {
    let declarations = "entity Thing\nconst alice : Person\nconst car : Thing";
    let applications = "\nrule aliceEligibility = eligibility(alice)\nrule carEligibility = aliceEligibility(car)\n";
    let horn = parse_formal_to_typed(&format!(
        "{}{applications}",
        HORN.replace("entity Thing", declarations)
    ))
    .unwrap();
    let curried = parse_formal_to_typed(&format!(
        "{}{applications}",
        CURRIED.replace("entity Thing", declarations)
    ))
    .unwrap();
    for name in ["aliceEligibility", "carEligibility"] {
        let residual = rule(&curried, name);
        assert_eq!(unresiduate(residual), *rule(&horn, name));
        assert_eq!(residuate(rule(&horn, name)).unwrap(), *residual);
    }
    let specialized = rule(&curried, "carEligibility");
    assert_eq!(
        specialized.parameters,
        vec![Arg {
            name: "t".into(),
            ty: Type::PropositionTime
        }]
    );
    let RuleBody::ResidualClause(clause) = &specialized.body else {
        panic!("residual");
    };
    assert_eq!(
        clause.consequent.consequent.args,
        vec![Term::Const("alice".into()), Term::Const("car".into())]
    );
    for source in [HORN, CURRIED] {
        let wrong = format!(
            "{}\nrule wrong = eligibility(car)\n",
            source.replace("entity Thing", declarations)
        );
        assert!(
            parse_formal_to_typed(&wrong)
                .unwrap_err()
                .to_string()
                .contains("not assignable to rule parameter `person`")
        );
    }
}

#[test]
fn forward_view_matches_horn_fixpoint_and_provenance() {
    let horn = parse_formal_to_typed(HORN).unwrap();
    let curried = parse_formal_to_typed(CURRIED).unwrap();
    let request = seed("requests", ["alice", "car"], "tau");
    let available = seed("available", ["car"], "tau");
    let mut wrong_object = available.clone();
    wrong_object.args[0] = Term::Var("house".into());
    let mut wrong_time = available.clone();
    wrong_time.proposition.time = TimeExpr::At("later".into());
    let mut negative = available.clone();
    negative.polarity = Polarity::EvidentialNot;
    for facts in [
        vec![request.clone()],
        vec![available.clone()],
        vec![request.clone(), wrong_object],
        vec![request.clone(), wrong_time],
        vec![request.clone(), negative],
    ] {
        let closure = close(
            facts.clone(),
            &[rule(&curried, "eligibility").clone()],
            &mut Clock::default(),
        );
        assert!(
            closure.derivations.is_empty(),
            "A alone never supplies missing evidence B"
        );
        assert_eq!(closure.known, facts);
    }
    let facts = vec![request.clone(), available.clone()];
    let forward = close(
        facts.clone(),
        &[rule(&horn, "eligibility").clone()],
        &mut Clock::default(),
    );
    let residual = close(
        facts.clone(),
        &[rule(&curried, "eligibility").clone()],
        &mut Clock::default(),
    );
    assert_eq!(
        residual, forward,
        "same events, stamps and complete Horn provenance"
    );
    assert_eq!(residual.derivations.len(), 1);
    let derived = &residual.derivations[0];
    assert_eq!(derived.event.verb, "eligible");
    assert_eq!(
        derived.event.proposition,
        PropositionType {
            kind: PropositionKind::Derived,
            time: TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
        }
    );
    assert_eq!(derived.record.antecedents, facts);
    assert_eq!(derived.derived_at.name, "d1");
    for source in [HORN, CURRIED] {
        let signed = source
            .replace("available(object) @ t", "~available(object) @ t")
            .replace("<= eligible", "<= ~eligible")
            .replace("-o eligible", "-o ~eligible");
        let module = parse_formal_to_typed(&signed).unwrap();
        let mut negative = available.clone();
        negative.polarity = Polarity::EvidentialNot;
        assert!(
            instantiate_from_known(
                rule(&module, "eligibility"),
                &[request.clone(), available.clone()]
            )
            .is_empty()
        );
        let closure = close(
            vec![request.clone(), available.clone(), negative.clone()],
            &[rule(&module, "eligibility").clone()],
            &mut Clock::default(),
        );
        assert_eq!(closure.derivations.len(), 1);
        assert_eq!(
            closure.derivations[0].event.polarity,
            Polarity::EvidentialNot
        );
        assert_eq!(
            closure.derivations[0].record.antecedents,
            vec![request.clone(), negative]
        );
        assert!(
            closure.known.contains(&available),
            "explicit negative evidence does not retract positive evidence"
        );
    }
}

#[test]
fn operands_use_the_same_m2_checks_as_horn() {
    for (from, to, expected) in [
        (
            "available(object)",
            "available(person)",
            "conflicting inferred types",
        ),
        (
            "available(object)",
            "available(object, person)",
            "expects 1 args, got 2",
        ),
        ("available(object)", "unknown(object)", "unknown verb"),
        (
            "available(object) @ t",
            "available(object)",
            "unresolved PropositionTime",
        ),
        (
            "eligible(person, object) @ after t",
            "eligible(person, object)",
            "unresolved PropositionTime",
        ),
        (
            "eligible(person, object) @ after t",
            "~requests(person, object) @ after t",
            "seeded verbs cannot derive negative evidence",
        ),
    ] {
        let a = parse_formal_to_typed(&HORN.replace(from, to)).unwrap_err();
        let b = parse_formal_to_typed(&CURRIED.replace(from, to)).unwrap_err();
        assert_eq!(a, b);
        assert!(a.to_string().contains(expected), "{a}");
    }
}

#[test]
fn syntax_requires_both_operands_and_rejects_nested_fragment() {
    let rhs = "available(object) @ t\n    -o eligible(person, object) @ after t";
    for invalid in [
        "-o eligible(person, object) @ after t",
        "available(object) @ t -o",
        "available(object) @ t -o -o eligible(person, object) @ after t",
        "available(object) @ t & -o eligible(person, object) @ after t",
        "available(object) @ t -o eligible(person, object) @ after t -o eligible(person, object) @ t",
    ] {
        let source = CURRIED.replace(rhs, invalid);
        assert!(parse_formal(&source).is_err(), "{invalid}");
        assert!(
            crate::formal::parse_formal_manual(&source).is_err(),
            "{invalid}"
        );
    }
}
