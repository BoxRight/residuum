use super::{CHAIN, Chain, GROUP_SUBSETS, GroupSubset};
use crate::algebra::{
    CommutativeResiduatedLattice, Lattice, Monoid, OrderComparison, Preorder, interpret_requirement,
};
use crate::parse_formal_to_typed;
use crate::residual::{
    ProgramResidualQuery, RequirementFormula, SearchLimits, query_program_residual,
};
use crate::typed::{
    Declaration, Event, Module, Polarity, PropositionKind, PropositionType, RuleBody, TimeExpr,
};

const SOURCE: &str = include_str!("../../../examples/residual/order_formal.res");

fn experiment() -> (Module, ProgramResidualQuery) {
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let goal = Event {
        verb: "goal".into(),
        args: vec![],
        polarity: Polarity::Positive,
        proposition: PropositionType {
            kind: PropositionKind::Derived,
            time: TimeExpr::At("tau".into()),
        },
        transition: None,
    };
    let query = query_program_residual(&module, &[], &goal, SearchLimits::default()).unwrap();
    assert_eq!(query.explanations.len(), 2);
    assert!(matches!(
        query.required,
        Some(RequirementFormula::Join(_, _))
    ));
    (module, query)
}

// These example valuations assign the same value at every time, so they satisfy
// the nullary rule schemas for all t when the two inequalities hold. This does
// not identify different proposition times in the AST or in general valuations.
fn valuation<L: Clone>(a: L, b: L, goal: L) -> impl Fn(&Event) -> Result<L, &'static str> {
    move |event| {
        if event.polarity != Polarity::Positive {
            return Err("unassigned polarity");
        }
        match event.verb.as_str() {
            "a" => Ok(a.clone()),
            "b" => Ok(b.clone()),
            "goal" => Ok(goal.clone()),
            _ => Err("unassigned atom"),
        }
    }
}

fn satisfies_program<L: Lattice + Monoid>(
    module: &Module,
    atom: &impl Fn(&Event) -> Result<L, &'static str>,
) -> bool {
    module
        .declarations
        .iter()
        .all(|declaration| match declaration {
            Declaration::Rule(rule) => {
                let clause = match &rule.body {
                    RuleBody::HornClause(clause) => clause.clone(),
                    RuleBody::ResidualClause(clause) => clause.unresiduate(),
                };
                interpret_requirement(&RequirementFormula::from(&clause.antecedent), atom)
                    .unwrap()
                    .le(&atom(&clause.consequent).unwrap())
            }
            _ => true,
        })
}

fn branch_values<L: Lattice + Monoid>(
    query: &ProgramResidualQuery,
    atom: &impl Fn(&Event) -> Result<L, &'static str>,
) -> (L, L) {
    let first = interpret_requirement(
        &RequirementFormula::from(&query.explanations[0].required),
        atom,
    )
    .unwrap();
    let second = interpret_requirement(
        &RequirementFormula::from(&query.explanations[1].required),
        atom,
    )
    .unwrap();
    (first, second)
}

fn check_query_bounds<L: CommutativeResiduatedLattice + Clone>(carrier: &[L]) {
    let (module, query) = experiment();
    let mut admissible = 0;
    for a in carrier {
        for b in carrier {
            for g in carrier {
                let atom = valuation(a.clone(), b.clone(), g.clone());
                if !satisfies_program(&module, &atom) {
                    continue;
                }
                admissible += 1;
                let (h1, h2) = branch_values(&query, &atom);
                assert!(h1.equivalent(a));
                assert!(h2.equivalent(&a.tensor(b)));
                assert!(h1.le(g) && h2.le(g));
                let join = interpret_requirement(query.required.as_ref().unwrap(), &atom).unwrap();
                assert!(join.le(g), "the alternatives jointly remain below the goal");
                assert!(h1.le(&join) && h2.le(&join));
                for upper in carrier {
                    assert_eq!(
                        join.le(upper),
                        h1.le(upper) && h2.le(upper),
                        "least upper bound of real query branches"
                    );
                }
                // Connection to residuation, for the second producer rule.
                assert_eq!(h2.le(g), a.le(&b.residual(g)));
            }
        }
    }
    assert!(admissible > 0);
}

#[test]
fn query_join_is_least_upper_bound_below_goal_in_both_models() {
    check_query_bounds(&GROUP_SUBSETS);
    check_query_bounds(&CHAIN);
}

#[test]
fn integral_example_orders_more_requirements_below_fewer() {
    let (module, query) = experiment();
    let atom = valuation(Chain(1), Chain(1), Chain(1));
    assert!(satisfies_program(&module, &atom));
    let (fewer, more) = branch_values(&query, &atom);
    assert_eq!(fewer, Chain(1));
    assert_eq!(more, Chain(0));
    assert_eq!(fewer.compare(&more), OrderComparison::Above);
    assert!(
        !Chain(1)
            .tensor(&Chain(1))
            .equivalent(&Chain(1).meet(&Chain(1)))
    );
    // This is conditional on the value of B being below the monoidal unit.
    for a in CHAIN {
        for b in CHAIN {
            assert!(b.le(&Chain::unit()));
            assert!(a.tensor(&b).le(&a));
        }
    }
}

#[test]
fn universal_factor_discard_matches_integrality_in_reference_models() {
    fn check<L: CommutativeResiduatedLattice>(carrier: &[L], expected: bool) {
        let integral = carrier.iter().all(|b| b.le(&L::unit()));
        let discard = carrier
            .iter()
            .all(|a| carrier.iter().all(|b| a.tensor(b).le(a)));
        assert_eq!(discard, integral);
        assert_eq!(integral, expected);
    }
    check(&CHAIN, true);
    check(&GROUP_SUBSETS, false);
}

#[test]
fn nonintegral_countermodel_reverses_requirement_count_orientation() {
    let (module, query) = experiment();
    let atom = valuation(GroupSubset(2), GroupSubset(3), GroupSubset(3));
    assert!(satisfies_program(&module, &atom));
    let (fewer, more) = branch_values(&query, &atom);
    assert_eq!(fewer, GroupSubset(2)); // {1}
    assert_eq!(more, GroupSubset(3)); // {0,1}
    assert_eq!(fewer.compare(&more), OrderComparison::Below);
    assert!(!GroupSubset(3).le(&GroupSubset::unit()));
}

#[test]
fn same_goal_rules_admit_incomparable_explanations() {
    let (module, query) = experiment();
    let atom = valuation(GroupSubset(2), GroupSubset(2), GroupSubset(3));
    assert!(satisfies_program(&module, &atom));
    let (h1, h2) = branch_values(&query, &atom);
    assert_eq!(h1, GroupSubset(2)); // {1}
    assert_eq!(h2, GroupSubset(1)); // {0}
    assert_eq!(h1.compare(&h2), OrderComparison::Incomparable);
    // Both inequalities fail in a model satisfying all rule schemas. This is
    // an explicit refutation, not a failure of an incomplete proof procedure.
    assert!(!h1.le(&h2) && !h2.le(&h1));
    assert!(h1.le(&GroupSubset(3)) && h2.le(&GroupSubset(3)));
}

#[test]
fn semantic_equivalence_does_not_collapse_query_proofs() {
    let (module, query) = experiment();
    let atom = valuation(GroupSubset(2), GroupSubset::unit(), GroupSubset(2));
    assert!(satisfies_program(&module, &atom));
    let (h1, h2) = branch_values(&query, &atom);
    assert_eq!(h1.compare(&h2), OrderComparison::Equivalent);
    assert_ne!(
        query.explanations[0].required,
        query.explanations[1].required
    );
    assert_ne!(query.explanations[0].proof, query.explanations[1].proof);
    assert_eq!(query.explanations.len(), 2);
}

#[test]
fn interpretation_preserves_tensor_join_distribution_and_unit() {
    let (_, query) = experiment();
    let a = RequirementFormula::from(&query.explanations[0].required);
    let ab = RequirementFormula::from(&query.explanations[1].required);
    let join = RequirementFormula::Join(Box::new(a.clone()), Box::new(ab.clone()));
    let lhs = RequirementFormula::Tensor(Box::new(a.clone()), Box::new(join));
    let rhs = RequirementFormula::Join(
        Box::new(RequirementFormula::Tensor(
            Box::new(a.clone()),
            Box::new(a.clone()),
        )),
        Box::new(RequirementFormula::Tensor(
            Box::new(a.clone()),
            Box::new(ab),
        )),
    );
    for av in GROUP_SUBSETS {
        for bv in GROUP_SUBSETS {
            let atom = valuation(av, bv, GroupSubset(3));
            assert!(
                interpret_requirement(&lhs, &atom)
                    .unwrap()
                    .equivalent(&interpret_requirement(&rhs, &atom).unwrap())
            );
            let unit =
                RequirementFormula::Tensor(Box::new(RequirementFormula::Unit), Box::new(a.clone()));
            assert!(
                interpret_requirement(&unit, &atom)
                    .unwrap()
                    .equivalent(&interpret_requirement(&a, &atom).unwrap())
            );
        }
    }
}

#[test]
fn unassigned_atoms_and_polarities_remain_errors() {
    let (_, query) = experiment();
    let mut event = query.goal.clone();
    event.verb = "unknown".into();
    let atom = valuation(Chain(1), Chain(1), Chain(2));
    assert_eq!(
        interpret_requirement(&RequirementFormula::Event(event), &atom),
        Err("unassigned atom")
    );
    let mut event = query.goal.clone();
    event.polarity = Polarity::EvidentialNot;
    assert_eq!(
        interpret_requirement(&RequirementFormula::Event(event), &atom),
        Err("unassigned polarity")
    );
}
