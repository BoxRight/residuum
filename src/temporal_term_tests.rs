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
