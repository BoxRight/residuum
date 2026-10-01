use super::*;
use crate::typed::{
    DerivationTime, PropositionType, StateTime, StateTransitionType, Term, TimeExpr,
};
use crate::{DerivationClock, close, parse_formal_to_typed};

const ALTERNATIVES: &str = include_str!("../../../examples/residual/alternatives_formal.res");
const LEGAL: &str = include_str!("../../../examples/legal/sales_delivery_formal.res");

fn event(verb: &str, kind: PropositionKind) -> Event {
    Event {
        verb: verb.into(),
        args: vec![],
        polarity: Polarity::Positive,
        proposition: PropositionType {
            kind,
            time: TimeExpr::At("tau".into()),
        },
        transition: None,
    }
}

fn seed(verb: &str) -> Event {
    event(verb, PropositionKind::Seeded)
}
fn derived(verb: &str) -> Event {
    event(verb, PropositionKind::Derived)
}
fn allowed(names: &[&str]) -> Vec<Abducible> {
    names
        .iter()
        .map(|name| Abducible {
            verb: (*name).into(),
            polarity: Polarity::Positive,
        })
        .collect()
}
fn query(
    module: &Module,
    evidence: &[Event],
    goal: &Event,
    names: &[&str],
) -> ProgramResidualQuery {
    query_program_residual_with_abducibles(
        module,
        evidence,
        goal,
        &allowed(names),
        SearchLimits::default(),
    )
    .unwrap()
}
fn atoms(required: &Antecedent) -> Vec<Event> {
    match required {
        Antecedent::Unit => vec![],
        Antecedent::Event(e) => vec![e.clone()],
        Antecedent::And(a, b) => {
            let mut events = atoms(a);
            events.extend(atoms(b));
            events
        }
    }
}
fn origin(proof: &ResidualProof) -> &str {
    match proof {
        ResidualProof::Rule { rule, .. } => &rule.name,
        _ => panic!("rule witness expected"),
    }
}
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("hypothetical-{}", self.0),
        }
    }
}
fn reaches(module: &Module, seeds: &[Event], goal: &Event) -> bool {
    let rules: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    close(seeds.to_vec(), &rules, &mut Clock::default())
        .known
        .contains(goal)
}

#[test]
fn legal_program_returns_forward_supported_delivery_in_three_scenarios() {
    let module = parse_formal_to_typed(LEGAL).unwrap();
    let original = module.clone();
    let mut price = seed("agreesPrice");
    price.args = ["alice", "car", "bob"]
        .map(|s| Term::Var(s.into()))
        .to_vec();
    let mut object = seed("agreesObject");
    object.args = ["bob", "car", "alice"]
        .map(|s| Term::Var(s.into()))
        .to_vec();
    let input = TimeExpr::After(Box::new(TimeExpr::At("tau".into())));
    let output = TimeExpr::After(Box::new(input.clone()));
    let mut goal = event("do", PropositionKind::Effect);
    goal.args = vec![
        Term::Var("alice".into()),
        Term::Const("deliver".into()),
        Term::Var("car".into()),
        Term::Var("bob".into()),
    ];
    goal.proposition.time = output.clone();
    goal.transition = Some(StateTransitionType {
        input: StateTime::At(input),
        output: StateTime::At(output),
    });
    for (evidence, expected) in [
        (
            vec![],
            Antecedent::And(
                Box::new(Antecedent::Event(price.clone())),
                Box::new(Antecedent::Event(object.clone())),
            ),
        ),
        (vec![price.clone()], Antecedent::Event(object.clone())),
        (vec![price.clone(), object.clone()], Antecedent::Unit),
    ] {
        let unchanged = evidence.clone();
        let result =
            query_program_residual(&module, &evidence, &goal, SearchLimits::default()).unwrap();
        assert_eq!(
            result.explanations.len(),
            1,
            "only specialized delivery can be instantiated forward"
        );
        assert_eq!(result.required, Some(RequirementFormula::from(&expected)));
        assert_eq!(
            result
                .explanations
                .iter()
                .map(|e| origin(&e.proof))
                .collect::<Vec<_>>(),
            ["delivery"]
        );
        assert_eq!(result.failure, None);
        assert_eq!(result.metrics.rejected_not_forward_supported, 1);
        for explanation in result.explanations {
            assert_eq!(explanation.required, expected);
            let ResidualProof::Rule {
                goal: witness_goal,
                antecedent,
                substitution,
                ..
            } = &explanation.proof
            else {
                unreachable!()
            };
            assert_eq!(witness_goal.transition, goal.transition);
            assert!(!substitution.bindings.is_empty());
            let ResidualProof::Rule {
                rule,
                goal: intermediate,
                ..
            } = antecedent.as_ref()
            else {
                panic!("sales witness expected")
            };
            assert_eq!(rule.name, "sales");
            assert_eq!(intermediate.verb, "give");
            assert_eq!(
                intermediate.proposition.time,
                TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
            );
            assert_eq!(
                intermediate.transition.as_ref().unwrap().input,
                StateTime::Unresolved
            );
            let mut hypothetical = evidence.clone();
            hypothetical.extend(atoms(&explanation.required));
            assert!(reaches(&module, &hypothetical, &goal));
        }
        assert_eq!(evidence, unchanged);
        assert_eq!(module, original);
    }
}

#[test]
fn alternatives_form_join_and_conjunction_enumerates_cartesian_product() {
    let module = parse_formal_to_typed(ALTERNATIVES).unwrap();
    let result = query(&module, &[], &derived("goal"), &["a", "b", "c", "d"]);
    let branches: Vec<_> = result
        .explanations
        .iter()
        .map(|e| {
            atoms(&e.required)
                .into_iter()
                .map(|e| e.verb)
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        branches,
        [
            vec!["a", "c"],
            vec!["a", "d"],
            vec!["b", "c"],
            vec!["b", "d"]
        ]
    );
    let formulas: Vec<_> = result
        .explanations
        .iter()
        .map(|e| RequirementFormula::from(&e.required))
        .collect();
    assert_eq!(
        result.required,
        formulas
            .into_iter()
            .reduce(|a, b| RequirementFormula::Join(Box::new(a), Box::new(b)))
    );
    assert_eq!(result.metrics.head_matches, 5);
    assert_eq!(result.metrics.rules_examined, 15);
    for branch in &result.explanations {
        let ResidualProof::Rule { antecedent, .. } = &branch.proof else {
            unreachable!()
        };
        let ResidualProof::Tensor(left, right) = antecedent.as_ref() else {
            panic!("tensor witness expected")
        };
        assert!(["fromA", "fromB"].contains(&origin(left)));
        assert!(["fromC", "fromD"].contains(&origin(right)));
    }
}

#[test]
fn finite_forward_oracle_checks_sufficiency_and_search_coverage() {
    let module = parse_formal_to_typed(ALTERNATIVES).unwrap();
    let goal = derived("goal");
    let seeds = [seed("a"), seed("b"), seed("c"), seed("d")];
    for mask in 0..16 {
        let evidence: Vec<_> = seeds
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect();
        let result =
            query_program_residual(&module, &evidence, &goal, SearchLimits::default()).unwrap();
        for branch in &result.explanations {
            let mut hypothetical = evidence.clone();
            hypothetical.extend(atoms(&branch.required));
            assert!(reaches(&module, &hypothetical, &goal));
        }
        for extension_mask in 0..16 {
            let mut extended = evidence.clone();
            extended.extend(
                seeds
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| extension_mask & (1 << i) != 0)
                    .map(|(_, e)| e.clone()),
            );
            let covered = result
                .explanations
                .iter()
                .any(|branch| atoms(&branch.required).iter().all(|e| extended.contains(e)));
            assert_eq!(
                reaches(&module, &extended, &goal),
                covered,
                "evidence {mask}, extension {extension_mask}"
            );
        }
    }
}

#[test]
fn satisfied_and_redundant_branches_are_retained_without_preference() {
    let module = parse_formal_to_typed(
        "module Choice
seeded verb a()
seeded verb b()
derived verb goal()
rule simple(t) = a() @ t <= goal() @ t
rule redundant(t) = a() @ t & b() @ t <= goal() @ t",
    )
    .unwrap();
    let result = query(&module, &[], &derived("goal"), &["a", "b"]);
    assert_eq!(result.explanations.len(), 2);
    assert_eq!(atoms(&result.explanations[0].required), [seed("a")]);
    assert_eq!(
        atoms(&result.explanations[1].required),
        [seed("a"), seed("b")]
    );
    let result = query(&module, &[seed("a")], &derived("goal"), &["a", "b"]);
    assert_eq!(
        result.required,
        Some(RequirementFormula::Join(
            Box::new(RequirementFormula::Unit),
            Box::new(RequirementFormula::Event(seed("b")))
        ))
    );
    let result = query(&module, &[derived("goal")], &derived("goal"), &[]);
    assert_eq!(result.required, Some(RequirementFormula::Unit));
    assert!(matches!(
        result.explanations[0].proof,
        ResidualProof::Evidence(_)
    ));
}

#[test]
fn identical_requirements_share_join_operand_but_keep_distinct_proofs() {
    let module = parse_formal_to_typed(
        "module Duplicate
seeded verb a()
derived verb goal()
rule first(t) = a() @ t <= goal() @ t
rule second(t) = a() @ t <= goal() @ t",
    )
    .unwrap();
    let result = query(&module, &[], &derived("goal"), &["a"]);
    assert_eq!(result.required, Some(RequirementFormula::Event(seed("a"))));
    assert_eq!(
        result
            .explanations
            .iter()
            .map(|e| origin(&e.proof))
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
}

#[test]
fn explicit_negative_hypotheses_require_their_own_permission() {
    let module = parse_formal_to_typed(
        "module Negative
seeded verb a()
derived verb goal()
rule negative(t) = ~a() @ t <= goal() @ t",
    )
    .unwrap();
    let goal = derived("goal");
    let result = query(&module, &[seed("a")], &goal, &["a"]);
    assert!(result.explanations.is_empty());
    assert_eq!(result.required, None);
    let allowed = [Abducible {
        verb: "a".into(),
        polarity: Polarity::EvidentialNot,
    }];
    let result = query_program_residual_with_abducibles(
        &module,
        &[seed("a")],
        &goal,
        &allowed,
        SearchLimits::default(),
    )
    .unwrap();
    let hypotheses = atoms(&result.explanations[0].required);
    assert_eq!(hypotheses[0].polarity, Polarity::EvidentialNot);
    let result = query_program_residual_with_abducibles(
        &module,
        &hypotheses,
        &goal,
        &[],
        SearchLimits::default(),
    )
    .unwrap();
    assert_eq!(result.required, Some(RequirementFormula::Unit));
}

#[test]
fn no_explanation_is_distinct_from_ground_unit_rule() {
    let mut module = parse_formal_to_typed(
        "module Empty
derived verb goal()
derived verb other()
rule ground() = I <= goal() @ tau",
    )
    .unwrap();
    // The surface elaborator infers tau as a lambda parameter. Supply the same
    // genuinely ground typed axiom used by the existing Unit runtime test.
    for declaration in &mut module.declarations {
        if let Declaration::Rule(rule) = declaration {
            rule.parameters.clear();
        }
    }
    let result = query(&module, &[], &derived("goal"), &[]);
    assert_eq!(result.required, Some(RequirementFormula::Unit));
    assert!(reaches(&module, &[], &derived("goal")));
    let result = query(&module, &[], &derived("other"), &[]);
    assert_eq!(result.required, None);
    assert!(result.explanations.is_empty());
}

#[test]
fn cycles_and_unbound_parameters_error_instead_of_returning_partial_search() {
    let module = parse_formal_to_typed(
        "module Cycle
seeded verb a()
derived verb goal()
rule valid(t) = a() @ t <= goal() @ t
rule cycle(t) = goal() @ t <= goal() @ t",
    )
    .unwrap();
    let error = query_program_residual_with_abducibles(
        &module,
        &[],
        &derived("goal"),
        &allowed(&["a"]),
        SearchLimits::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("cyclic"));
    let module = parse_formal_to_typed(
        "module Free
entity Thing
seeded verb a(object : Thing)
derived verb goal()
rule free(object, t) = a(object) @ t <= goal() @ t",
    )
    .unwrap();
    let error = query_program_residual_with_abducibles(
        &module,
        &[],
        &derived("goal"),
        &allowed(&["a"]),
        SearchLimits::default(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("parameter `object` not determined")
    );
}

#[test]
fn explicit_search_budgets_fail_without_truncated_results() {
    let module = parse_formal_to_typed(ALTERNATIVES).unwrap();
    let goal = derived("goal");
    for limits in [
        SearchLimits {
            max_nodes: 1,
            ..SearchLimits::default()
        },
        SearchLimits {
            max_depth: 1,
            ..SearchLimits::default()
        },
        SearchLimits {
            max_branches: 1,
            ..SearchLimits::default()
        },
    ] {
        let error = query_program_residual_with_abducibles(
            &module,
            &[],
            &goal,
            &allowed(&["a", "b", "c", "d"]),
            limits,
        )
        .unwrap_err();
        assert!(error.to_string().contains("limit exceeded"));
    }
    let error = query_program_residual_with_abducibles(
        &module,
        &[],
        &goal,
        &[],
        SearchLimits {
            max_depth: 65,
            ..SearchLimits::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("invalid"));
}

#[test]
fn expanding_temporal_goals_stop_at_depth_budget() {
    let module = parse_formal_to_typed(
        "module Temporal
derived verb goal()
rule expand(t) = goal() @ after t <= goal() @ t",
    )
    .unwrap();
    let error = query_program_residual_with_abducibles(
        &module,
        &[],
        &derived("goal"),
        &[],
        SearchLimits {
            max_depth: 8,
            ..SearchLimits::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("depth/node limit exceeded"));
    assert!(
        !error.to_string().contains("cyclic"),
        "after(t) is not collapsed to t"
    );
}

#[test]
fn tensor_multiplicity_is_preserved_and_only_unit_is_removed() {
    let module = parse_formal_to_typed(
        "module Tensor
seeded verb a()
derived verb goal()
rule twice(t) = I & a() @ t & a() @ t <= goal() @ t",
    )
    .unwrap();
    let result = query(&module, &[], &derived("goal"), &["a"]);
    assert_eq!(
        atoms(&result.explanations[0].required),
        [seed("a"), seed("a")]
    );
    assert_eq!(
        result.required,
        Some(RequirementFormula::Tensor(
            Box::new(RequirementFormula::Event(seed("a"))),
            Box::new(RequirementFormula::Event(seed("a"))),
        ))
    );
    let result = query(&module, &[seed("a")], &derived("goal"), &[]);
    assert_eq!(result.required, Some(RequirementFormula::Unit));
    // Known is currently reusable evidence. Keeping tensor multiplicity in the
    // result does not assert a resource-sensitive satisfaction model for L.
}

#[test]
fn configuration_and_default_policy_are_not_silently_assumed() {
    let module = parse_formal_to_typed(ALTERNATIVES).unwrap();
    let result = query(&module, &[], &derived("goal"), &["a"]);
    assert_eq!(result.required, None, "no admissible right branch");
    let error = query_program_residual_with_abducibles(
        &module,
        &[],
        &derived("goal"),
        &allowed(&["left"]),
        SearchLimits::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("must be a declared seeded"));
    let error = query_program_residual_with_abducibles(
        &module,
        &[],
        &derived("unknown"),
        &[],
        SearchLimits::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("unknown"));
    let error = query_program_residual_with_abducibles(
        &module,
        &[derived("a")],
        &derived("goal"),
        &[],
        SearchLimits::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("signature mismatch"));
    let with_default = format!("{ALTERNATIVES}\ndefault Normal = supernormal combine");
    let module = parse_formal_to_typed(&with_default).unwrap();
    let error = query_program_residual_with_abducibles(
        &module,
        &[],
        &derived("goal"),
        &[],
        SearchLimits::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("without defaults"));
}

#[test]
fn curried_and_horn_program_queries_agree() {
    let horn =
        parse_formal_to_typed(include_str!("../../../examples/residual/horn_formal.res")).unwrap();
    let curried = parse_formal_to_typed(include_str!(
        "../../../examples/residual/curried_formal.res"
    ))
    .unwrap();
    let mut goal = derived("eligible");
    goal.args = vec![Term::Var("alice".into()), Term::Var("car".into())];
    goal.proposition.time = TimeExpr::After(Box::new(TimeExpr::At("tau".into())));
    let a = query(&horn, &[], &goal, &["requests", "available"]);
    let b = query(&curried, &[], &goal, &["requests", "available"]);
    assert_eq!(a, b);
    assert_eq!(
        atoms(&a.explanations[0].required)[0].args,
        goal.args,
        "ground identifiers are preserved"
    );
}

#[test]
fn seeded_declarations_are_abducible_by_default_with_both_explicit_polarities() {
    let module = parse_formal_to_typed(ALTERNATIVES).unwrap();
    for declaration in &module.declarations {
        if let Declaration::Verb(verb) = declaration {
            assert_eq!(verb.is_abducible(), verb.kind == VerbKind::Seeded);
        }
    }
    let result =
        query_program_residual(&module, &[], &derived("goal"), SearchLimits::default()).unwrap();
    assert_eq!(result.explanations.len(), 4);
    assert_eq!(result.failure, None);
    for branch in &result.explanations {
        assert!(
            atoms(&branch.required)
                .iter()
                .all(|event| event.proposition.kind == PropositionKind::Seeded)
        );
        assert!(reaches(&module, &atoms(&branch.required), &derived("goal")));
    }
    let original = vec![seed("a")];
    let mut negative = seed("a");
    negative.polarity = Polarity::EvidentialNot;
    let result =
        query_program_residual(&module, &original, &negative, SearchLimits::default()).unwrap();
    assert_eq!(atoms(&result.explanations[0].required), [negative.clone()]);
    assert_eq!(
        original,
        [seed("a")],
        "negative hypothesis is not negative evidence"
    );
    let result = query_program_residual(
        &module,
        &[negative.clone()],
        &negative,
        SearchLimits::default(),
    )
    .unwrap();
    assert_eq!(result.required, Some(RequirementFormula::Unit));
}

#[test]
fn missing_derived_producer_is_not_abducible_and_explicit_restrictions_remain_possible() {
    let module = parse_formal_to_typed(
        "module Missing
seeded verb a()
derived verb goal()",
    )
    .unwrap();
    let result =
        query_program_residual(&module, &[], &derived("goal"), SearchLimits::default()).unwrap();
    assert_eq!(result.failure, Some(AbductionFailure::NotAbducible));
    assert_eq!(result.required, None);
    assert!(result.explanations.is_empty());
    let result = query_program_residual_with_abducibles(
        &module,
        &[],
        &seed("a"),
        &[],
        SearchLimits::default(),
    )
    .unwrap();
    assert_eq!(result.failure, Some(AbductionFailure::NotAbducible));
    let result = query_program_residual(&module, &[], &seed("a"), SearchLimits::default()).unwrap();
    assert_eq!(result.required, Some(RequirementFormula::Event(seed("a"))));
}

#[test]
fn head_only_bindings_cannot_certify_a_forward_explanation() {
    let module = parse_formal_to_typed(
        "module Unsafe
entity Thing
seeded verb a()
derived verb goal(object : Thing)
rule unsafe(object, t) = a() @ t <= goal(object) @ t",
    )
    .unwrap();
    let mut goal = derived("goal");
    goal.args = vec![Term::Var("car".into())];
    let result = query_program_residual(&module, &[], &goal, SearchLimits::default()).unwrap();
    assert_eq!(result.failure, Some(AbductionFailure::NotForwardSupported));
    assert!(result.explanations.is_empty());
    assert!(!reaches(&module, &[seed("a")], &goal));
}

#[test]
fn forward_certificate_preserves_runtime_identities_that_share_parameter_names() {
    let module =
        parse_formal_to_typed(include_str!("../../../examples/residual/horn_formal.res")).unwrap();
    let mut goal = derived("eligible");
    goal.args = vec![Term::Var("person".into()), Term::Var("object".into())];
    goal.proposition.time = TimeExpr::After(Box::new(TimeExpr::At("t".into())));
    let result = query_program_residual(&module, &[], &goal, SearchLimits::default()).unwrap();
    assert_eq!(result.failure, None);
    assert!(result.required.is_some());
    assert_eq!(result.explanations.len(), 1);
    let hypotheses = atoms(&result.explanations[0].required);
    assert_eq!(hypotheses.len(), 2);
    assert!(
        hypotheses
            .iter()
            .all(|event| event.proposition.time == TimeExpr::At("t".into()))
    );
    assert!(reaches(&module, &hypotheses, &goal));
    assert!(
        !reaches(&module, &[], &goal),
        "query did not insert hypotheses"
    );
}
