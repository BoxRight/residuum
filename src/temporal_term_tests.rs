use crate::derive::{substitute_term, substitute_time};
use crate::typed::{Constructor, RecordFieldValue, SubstitutionValue, Term, TimeExpr};
use std::collections::BTreeMap;
fn after(t: TimeExpr) -> TimeExpr {
    TimeExpr::After(Box::new(t))
}
fn samples() -> Vec<TimeExpr> {
    [
        TimeExpr::At("t".into()),
        TimeExpr::Var("t".into()),
        TimeExpr::At("zeroTime".into()),
    ]
    .into_iter()
    .flat_map(|root| (0..9).map(move |depth| (0..depth).fold(root.clone(), |t, _| after(t))))
    .collect()
}
#[test]
fn conversions_preserve_structure_identity_and_typed_constructor_signature() {
    assert_eq!(
        Constructor::After.signature(),
        (
            crate::typed::Type::PropositionTime,
            crate::typed::Type::PropositionTime
        )
    );
    for t in samples() {
        let encoded = t.to_term();
        assert_eq!(TimeExpr::from_term(&encoded), Some(t.clone()));
        assert_eq!(TimeExpr::from_term(&encoded).unwrap().to_term(), encoded);
    }
    assert_ne!(
        TimeExpr::At("t".into()).to_term(),
        TimeExpr::Var("t".into()).to_term()
    );
    assert_eq!(TimeExpr::from_term(&Term::None), None);
    assert_eq!(
        TimeExpr::from_term(&Term::RecordLiteral {
            entity: "Box".into(),
            fields: vec![]
        }),
        None
    );
}
#[test]
fn substitution_commutes_with_encoding_for_term_and_time_bindings() {
    for value in samples() {
        for representation in [
            SubstitutionValue::Time(value.clone()),
            SubstitutionValue::Term(value.to_term()),
        ] {
            let sigma = BTreeMap::from([("t".into(), representation)]);
            for t in samples() {
                assert_eq!(
                    substitute_time(&t, &sigma).to_term(),
                    substitute_term(&t.to_term(), &sigma)
                );
            }
            assert_eq!(
                substitute_time(&TimeExpr::At("t".into()), &sigma),
                TimeExpr::At("t".into())
            );
            assert_eq!(
                substitute_term(&Term::Const("t".into()), &sigma),
                Term::Const("t".into())
            );
        }
    }
}
#[test]
fn substitution_recurses_into_records_without_capturing_homonymous_constants() {
    let t = after(after(TimeExpr::Var("t".into())));
    let record = Term::RecordLiteral {
        entity: "Envelope".into(),
        fields: vec![RecordFieldValue {
            field: "payload".into(),
            value: Term::RecordLiteral {
                entity: "Box".into(),
                fields: vec![
                    RecordFieldValue {
                        field: "variable".into(),
                        value: t.to_term(),
                    },
                    RecordFieldValue {
                        field: "constant".into(),
                        value: after(TimeExpr::At("t".into())).to_term(),
                    },
                ],
            },
        }],
    };
    let value = after(after(after(TimeExpr::At("zeroTime".into()))));
    let sigma_time = BTreeMap::from([("t".into(), SubstitutionValue::Time(value.clone()))]);
    let sigma_term = BTreeMap::from([("t".into(), SubstitutionValue::Term(value.to_term()))]);
    let actual = substitute_term(&record, &sigma_time);
    assert_eq!(actual, substitute_term(&record, &sigma_term));
    assert!(actual.is_ground());
    let Term::RecordLiteral { fields, .. } = actual else {
        panic!("Envelope")
    };
    let Term::RecordLiteral { fields, .. } = &fields[0].value else {
        panic!("Box")
    };
    assert_eq!(fields[0].value, after(after(value)).to_term());
    assert_eq!(fields[1].value, after(TimeExpr::At("t".into())).to_term());
}
#[test]
fn constructor_parser_preserves_surface_ast_agreement_with_manual_parser() {
    for source in [
        include_str!("../examples/experiments/temporal_constructor_rules.res"),
        include_str!("../examples/experiments/temporal_constructor_scenario.res"),
    ] {
        assert_eq!(
            crate::parse_formal(source).unwrap(),
            crate::formal::parse_formal_manual(source).unwrap()
        );
    }
}

#[test]
fn canonical_typed_bindings_preserve_identity_and_normalize_initial_constraints() {
    use crate::derive::{bind_typed, match_antecedents_with};
    use crate::typed::{Antecedent, Arg, Type};
    let time = Arg {
        name: "t".into(),
        ty: Type::PropositionTime,
    };
    for value in samples() {
        let expected = SubstitutionValue::Time(value.clone());
        for representation in [expected.clone(), SubstitutionValue::Term(value.to_term())] {
            assert_eq!(
                representation.canonical_for_type(&time.ty),
                Some(expected.clone())
            );
            assert_eq!(
                expected.canonical_for_type(&time.ty),
                Some(expected.clone())
            );
            let mut bindings = BTreeMap::new();
            bind_typed(&time, representation.clone(), &mut bindings).unwrap();
            bind_typed(&time, expected.clone(), &mut bindings).unwrap();
            assert_eq!(bindings["t"], expected);
            let matches = match_antecedents_with(
                &Antecedent::Unit,
                &[],
                std::slice::from_ref(&time),
                BTreeMap::from([("t".into(), representation)]),
                vec![],
            );
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].substitutions["t"], expected);
        }
    }
    let mut bindings = BTreeMap::new();
    bind_typed(
        &time,
        SubstitutionValue::Time(TimeExpr::At("t".into())),
        &mut bindings,
    )
    .unwrap();
    assert!(
        bind_typed(
            &time,
            SubstitutionValue::Time(TimeExpr::Var("t".into())),
            &mut bindings
        )
        .is_none()
    );
    assert!(bind_typed(&time, SubstitutionValue::Term(Term::None), &mut bindings).is_none());
    let entity = Arg {
        name: "entity".into(),
        ty: Type::Entity("Thing".into()),
    };
    bind_typed(
        &entity,
        SubstitutionValue::Term(Term::Const("t".into())),
        &mut bindings,
    )
    .unwrap();
    assert_eq!(
        bindings["entity"],
        SubstitutionValue::Term(Term::Const("t".into()))
    );
    assert!(
        bind_typed(
            &entity,
            SubstitutionValue::Time(TimeExpr::At("t".into())),
            &mut bindings
        )
        .is_none()
    );
    let optional = Type::Optional(Box::new(Type::PropositionTime));
    assert_eq!(
        SubstitutionValue::Term(Term::None).canonical_for_type(&optional),
        Some(SubstitutionValue::Term(Term::None))
    );
}
