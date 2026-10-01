pub mod algebra;
pub mod cnl;
pub mod defaults;
pub mod derive;
pub mod elab;
pub mod experiments;
pub mod formal;
pub mod residual;
pub mod runtime;
pub mod surface;
pub mod typed;

pub use cnl::parse_cnl;
pub use defaults::{
    BlockingScope, DefaultBlock, DefaultExtension, evaluate_defaults, select_active_rules,
};
pub use derive::{
    ClosureDelta, DerivationClock, IncrementalClosure, IncrementalStats, close, close_program,
    derive_once, instantiate, instantiate_from_known, stamp,
};
pub use elab::{Error, elaborate_cnl, elaborate_formal};
pub use formal::parse_formal;
pub use residual::{
    Abducible, AbductionFailure, ProgramResidualQuery, RequirementFormula, ResidualExplanation,
    ResidualProof, SearchLimits, SearchMetrics, query_program_residual,
    query_program_residual_with_abducibles, query_residual, residuate, unresiduate,
};
pub use runtime::execute_effect;
pub use typed::{Module, ResidualRequirement};

#[cfg(test)]
mod event_identity_tests;

#[cfg(test)]
mod legal_abduction_tests;

#[cfg(test)]
mod monotonic_regression_tests;

pub fn parse_formal_to_typed(source: &str) -> Result<Module, Error> {
    elaborate_formal(parse_formal(source)?)
}

pub fn parse_cnl_to_typed(source: &str) -> Result<Module, Error> {
    elaborate_cnl(parse_cnl(source)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{FieldName, ObjectId, StateSnapshot, Store, Value};
    use crate::typed::{
        Antecedent, Arg, Constant, Declaration, DefaultBody, DefaultRef, DerivationRecord,
        DerivationTime, Effect, Event, FieldAccess, HornClause, Polarity, PropositionKind,
        PropositionType, RecordFieldValue, Rule, RuleBody, RuleRef, StateOperation, StateTime,
        StateTransitionType, Substitution, SubstitutionBinding, SubstitutionValue, Term, TimeExpr,
        Type,
    };

    const FORMAL: &str = include_str!("../examples/transfer/formal.res");
    const CNL: &str = include_str!("../examples/transfer/cnl.res");
    const FORMAL_BAD_GRAMMATICAL_ROLES: &str =
        include_str!("../examples/transfer/formal_bad_roles.res");
    const CNL_BAD_GRAMMATICAL_ROLES: &str = include_str!("../examples/transfer/cnl_bad_roles.res");
    const FORMAL_UNRESOLVED_PROP_TIME: &str =
        include_str!("../examples/transfer/formal_unresolved_prop_time.res");
    const FORMAL_EFFECT_HIERARCHY: &str =
        include_str!("../examples/transfer/formal_effect_hierarchy.res");
    const FORMAL_BAD_EFFECT_BODY: &str =
        include_str!("../examples/transfer/formal_bad_effect_body.res");
    const FORMAL_EFFECT_AT_PLAIN_TIME: &str =
        include_str!("../examples/transfer/formal_effect_at_plain_time.res");
    const FORMAL_EVIDENTIAL_NOT: &str =
        include_str!("../examples/transfer/evidential_not_formal.res");
    const FORMAL_INCONSISTENT: &str = include_str!("../examples/transfer/inconsistent_formal.res");
    const LEGAL_FORMAL: &str = include_str!("../examples/legal/formal.res");
    const LEGAL_CNL: &str = include_str!("../examples/legal/cnl.res");
    const LEGAL_NESTED_PATHS_FORMAL: &str =
        include_str!("../examples/legal/nested_paths_formal.res");
    const LEGAL_NESTED_PATHS_CNL: &str = include_str!("../examples/legal/nested_paths_cnl.res");
    const LEGAL_RECORD_LITERAL_FORMAL: &str =
        include_str!("../examples/legal/record_literal_formal.res");
    const LEGAL_RECORD_LITERAL_CNL: &str = include_str!("../examples/legal/record_literal_cnl.res");
    const LEGAL_RECORD_LITERAL_BAD_THING: &str =
        include_str!("../examples/legal/record_literal_bad_thing_formal.res");
    const LEGAL_RECORD_LITERAL_BAD_CONDUCT: &str =
        include_str!("../examples/legal/record_literal_bad_conduct_formal.res");
    const LEGAL_RECORD_LITERAL_BAD_NONE: &str =
        include_str!("../examples/legal/record_literal_bad_none_formal.res");
    const LEGAL_GIVE_FORMAL: &str = include_str!("../examples/legal/give_formal.res");
    const LEGAL_GIVE_CNL: &str = include_str!("../examples/legal/give_cnl.res");
    const LEGAL_DO_NOTDO_FORMAL: &str = include_str!("../examples/legal/do_notdo_formal.res");
    const LEGAL_DO_NOTDO_CNL: &str = include_str!("../examples/legal/do_notdo_cnl.res");
    const LEGAL_SALES_FORMAL: &str = include_str!("../examples/legal/sales_formal.res");
    const LEGAL_SALES_CNL: &str = include_str!("../examples/legal/sales_cnl.res");
    const LEGAL_TRADITIO_DELIVERY_FORMAL: &str =
        include_str!("../examples/legal/traditio_delivery_formal.res");
    const LEGAL_TRADITIO_DELIVERY_CNL: &str =
        include_str!("../examples/legal/traditio_delivery_cnl.res");
    const LEGAL_SALES_DELIVERY_FORMAL: &str =
        include_str!("../examples/legal/sales_delivery_formal.res");
    const LEGAL_SALES_RETENTION_FORMAL: &str =
        include_str!("../examples/legal/sales_retention_formal.res");
    const LEGAL_SALES_RETENTION_WHEN_FORMAL: &str =
        include_str!("../examples/legal/sales_retention_when_formal.res");
    const LEGAL_SALES_BLOCKING_FORMAL: &str =
        include_str!("../examples/legal/sales_blocking_formal.res");
    const RESIDUAL_HORN_FORMAL: &str = include_str!("../examples/residual/horn_formal.res");
    const RESIDUAL_CURRIED_FORMAL: &str = include_str!("../examples/residual/curried_formal.res");
    const RESIDUAL_UNIT_FORMAL: &str = include_str!("../examples/residual/unit_formal.res");

    #[test]
    fn chumsky_formal_parser_matches_manual_surface_ast_for_existing_fixtures() {
        let fixtures = [
            ("transfer/formal.res", FORMAL),
            ("transfer/evidential_not_formal.res", FORMAL_EVIDENTIAL_NOT),
            ("transfer/inconsistent_formal.res", FORMAL_INCONSISTENT),
            (
                "transfer/formal_bad_roles.res",
                FORMAL_BAD_GRAMMATICAL_ROLES,
            ),
            (
                "transfer/formal_unresolved_prop_time.res",
                FORMAL_UNRESOLVED_PROP_TIME,
            ),
            (
                "transfer/formal_effect_hierarchy.res",
                FORMAL_EFFECT_HIERARCHY,
            ),
            (
                "transfer/formal_bad_effect_body.res",
                FORMAL_BAD_EFFECT_BODY,
            ),
            (
                "transfer/formal_effect_at_plain_time.res",
                FORMAL_EFFECT_AT_PLAIN_TIME,
            ),
            ("legal/formal.res", LEGAL_FORMAL),
            ("legal/nested_paths_formal.res", LEGAL_NESTED_PATHS_FORMAL),
            (
                "legal/record_literal_formal.res",
                LEGAL_RECORD_LITERAL_FORMAL,
            ),
            (
                "legal/record_literal_bad_thing_formal.res",
                LEGAL_RECORD_LITERAL_BAD_THING,
            ),
            (
                "legal/record_literal_bad_conduct_formal.res",
                LEGAL_RECORD_LITERAL_BAD_CONDUCT,
            ),
            (
                "legal/record_literal_bad_none_formal.res",
                LEGAL_RECORD_LITERAL_BAD_NONE,
            ),
            ("legal/give_formal.res", LEGAL_GIVE_FORMAL),
            ("legal/do_notdo_formal.res", LEGAL_DO_NOTDO_FORMAL),
            ("legal/sales_formal.res", LEGAL_SALES_FORMAL),
            (
                "legal/traditio_delivery_formal.res",
                LEGAL_TRADITIO_DELIVERY_FORMAL,
            ),
            (
                "legal/sales_delivery_formal.res",
                LEGAL_SALES_DELIVERY_FORMAL,
            ),
            (
                "legal/sales_retention_formal.res",
                LEGAL_SALES_RETENTION_FORMAL,
            ),
            (
                "legal/sales_retention_when_formal.res",
                LEGAL_SALES_RETENTION_WHEN_FORMAL,
            ),
            (
                "legal/sales_blocking_formal.res",
                LEGAL_SALES_BLOCKING_FORMAL,
            ),
            ("residual/horn_formal.res", RESIDUAL_HORN_FORMAL),
            ("residual/curried_formal.res", RESIDUAL_CURRIED_FORMAL),
            ("residual/unit_formal.res", RESIDUAL_UNIT_FORMAL),
        ];

        for (name, source) in fixtures {
            let chumsky = parse_formal(source)
                .unwrap_or_else(|error| panic!("chumsky parser failed for {name}: {error}"));
            let manual = crate::formal::parse_formal_manual(source)
                .unwrap_or_else(|error| panic!("manual parser failed for {name}: {error}"));
            assert_eq!(chumsky, manual, "surface AST mismatch for {name}");
        }
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn formal_and_cnl_elaborate_to_same_typed_ast() {
        let formal = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let cnl = parse_cnl_to_typed(CNL).expect("cnl elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn legal_formal_and_cnl_elaborate_to_same_subtype_hierarchy() {
        let formal = parse_formal_to_typed(LEGAL_FORMAL).expect("formal legal elaborates");
        let cnl = parse_cnl_to_typed(LEGAL_CNL).expect("cnl legal elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    fn legal_nominal_subtyping_is_reflexive_and_transitive() {
        let legal = parse_formal_to_typed(LEGAL_FORMAL).expect("formal legal elaborates");

        assert!(legal.is_subtype("Thing", "Thing"));
        assert!(legal.is_subtype("Money", "Fungible"));
        assert!(legal.is_subtype("Money", "Thing"));
        assert!(!legal.is_subtype("Thing", "Money"));
        assert!(!legal.is_subtype("Immovable", "Movable"));
    }

    #[test]
    fn legal_person_and_patrimony_records_elaborate_with_set_thing_assets() {
        let legal = parse_formal_to_typed(LEGAL_FORMAL).expect("formal legal elaborates");
        let patrimony = entity(&legal, "Patrimony");
        let person = entity(&legal, "Person");

        assert_eq!(
            patrimony.fields,
            vec![
                crate::typed::Field {
                    name: "assets".to_string(),
                    ty: Type::Set(Box::new(Type::Entity("Thing".to_string()))),
                },
                crate::typed::Field {
                    name: "rights".to_string(),
                    ty: Type::Set(Box::new(Type::Entity("Relation".to_string()))),
                },
                crate::typed::Field {
                    name: "obligations".to_string(),
                    ty: Type::Set(Box::new(Type::Entity("Relation".to_string()))),
                },
            ]
        );
        assert_eq!(
            person.fields,
            vec![crate::typed::Field {
                name: "patrimony".to_string(),
                ty: Type::Entity("Patrimony".to_string()),
            }]
        );
    }

    #[test]
    fn legal_conduct_is_nominal_and_relation_has_optional_fields() {
        let legal = parse_formal_to_typed(LEGAL_FORMAL).expect("formal legal elaborates");
        let conduct = entity(&legal, "Conduct");
        let relation = entity(&legal, "Relation");

        assert!(conduct.fields.is_empty());
        assert_eq!(
            relation.fields,
            vec![
                crate::typed::Field {
                    name: "debtor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                crate::typed::Field {
                    name: "thing".to_string(),
                    ty: Type::Optional(Box::new(Type::Entity("Thing".to_string()))),
                },
                crate::typed::Field {
                    name: "conduct".to_string(),
                    ty: Type::Optional(Box::new(Type::Entity("Conduct".to_string()))),
                },
                crate::typed::Field {
                    name: "creditor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
            ]
        );
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn legal_nested_field_paths_elaborate_equally_from_formal_and_cnl() {
        let formal =
            parse_formal_to_typed(LEGAL_NESTED_PATHS_FORMAL).expect("formal legal elaborates");
        let cnl = parse_cnl_to_typed(LEGAL_NESTED_PATHS_CNL).expect("cnl legal elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    fn legal_nested_field_paths_resolve_to_set_relation_targets() {
        let legal =
            parse_formal_to_typed(LEGAL_NESTED_PATHS_FORMAL).expect("formal legal elaborates");
        let registers = legal
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Verb(verb) if verb.name == "registers" => Some(verb),
                _ => None,
            })
            .expect("registers verb exists");
        let Some(Effect::StateTransform { operations, .. }) = &registers.effect else {
            panic!("registers has a state transform");
        };

        assert_eq!(
            operations,
            &vec![
                StateOperation::AddToSet {
                    target: FieldAccess {
                        base: Term::Var("subject".to_string()),
                        path: vec!["patrimony".to_string(), "obligations".to_string()],
                    },
                    value: Term::Var("object".to_string()),
                },
                StateOperation::AddToSet {
                    target: FieldAccess {
                        base: Term::Var("recipient".to_string()),
                        path: vec!["patrimony".to_string(), "rights".to_string()],
                    },
                    value: Term::Var("object".to_string()),
                },
            ]
        );
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn legal_sales_elaborates_equally_from_formal_and_cnl() {
        let formal = parse_formal_to_typed(LEGAL_SALES_FORMAL).expect("formal sales elaborates");
        let cnl = parse_cnl_to_typed(LEGAL_SALES_CNL).expect("cnl sales elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    fn legal_sales_preserves_conjunctive_antecedent() {
        let legal = parse_formal_to_typed(LEGAL_SALES_FORMAL).expect("formal sales elaborates");
        let sales = legal
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "sales" => Some(rule),
                _ => None,
            })
            .expect("sales rule exists");

        assert_eq!(
            sales.parameters,
            vec![
                Arg {
                    name: "debtor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "object".to_string(),
                    ty: Type::Entity("Thing".to_string()),
                },
                Arg {
                    name: "creditor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "t".to_string(),
                    ty: Type::PropositionTime,
                },
            ]
        );

        let clause = horn_clause(sales);
        let Antecedent::And(left, right) = &clause.antecedent else {
            panic!("sales has conjunctive antecedent");
        };
        let Antecedent::Event(price) = left.as_ref() else {
            panic!("left antecedent is an event");
        };
        let Antecedent::Event(object) = right.as_ref() else {
            panic!("right antecedent is an event");
        };

        assert_eq!(price.verb, "agreesPrice");
        assert_eq!(
            price.args,
            vec![
                Term::Var("debtor".to_string()),
                Term::Var("object".to_string()),
                Term::Var("creditor".to_string()),
            ]
        );
        assert_eq!(price.proposition.time, TimeExpr::At("t".to_string()));

        assert_eq!(object.verb, "agreesObject");
        assert_eq!(
            object.args,
            vec![
                Term::Var("creditor".to_string()),
                Term::Var("object".to_string()),
                Term::Var("debtor".to_string()),
            ]
        );
        assert_eq!(object.proposition.time, TimeExpr::At("t".to_string()));

        assert_eq!(clause.consequent.verb, "give");
        assert_eq!(
            clause.consequent.args,
            vec![
                Term::Var("debtor".to_string()),
                Term::Var("object".to_string()),
                Term::Var("creditor".to_string()),
            ]
        );
        assert_eq!(
            clause.consequent.proposition.time,
            TimeExpr::After(Box::new(TimeExpr::At("t".to_string())))
        );
    }

    #[test]
    fn conjunctive_sales_matching_derives_give_with_shared_substitution() {
        let legal = parse_formal_to_typed(LEGAL_SALES_FORMAL).expect("formal sales elaborates");
        let sales = rule(&legal, "sales");
        let price = seeded_event("agreesPrice", ["alice", "car", "bob"], "tau");
        let object = seeded_event("agreesObject", ["bob", "car", "alice"], "tau");

        let derivations = instantiate_from_known(sales, &[price.clone(), object.clone()]);

        assert_eq!(derivations.len(), 1);
        assert_eq!(
            derivations[0].event,
            give_effect_event("alice", "car", "bob", "tau")
        );
        assert_eq!(derivations[0].record.rule, "sales");
        assert_eq!(derivations[0].record.antecedents, vec![price, object]);
        assert_eq!(
            derivations[0].record.substitution,
            Substitution {
                bindings: vec![
                    SubstitutionBinding {
                        parameter: "creditor".to_string(),
                        value: SubstitutionValue::Term(Term::Var("bob".to_string())),
                    },
                    SubstitutionBinding {
                        parameter: "debtor".to_string(),
                        value: SubstitutionValue::Term(Term::Var("alice".to_string())),
                    },
                    SubstitutionBinding {
                        parameter: "object".to_string(),
                        value: SubstitutionValue::Term(Term::Var("car".to_string())),
                    },
                    SubstitutionBinding {
                        parameter: "t".to_string(),
                        value: SubstitutionValue::Time(TimeExpr::At("tau".to_string())),
                    },
                ],
            }
        );
    }

    #[test]
    fn conjunctive_sales_matching_rejects_incompatible_shared_substitution() {
        let legal = parse_formal_to_typed(LEGAL_SALES_FORMAL).expect("formal sales elaborates");
        let sales = rule(&legal, "sales");
        let price = seeded_event("agreesPrice", ["alice", "car", "bob"], "tau");
        let object = seeded_event("agreesObject", ["bob", "house", "alice"], "tau");

        assert_eq!(instantiate_from_known(sales, &[price, object]), vec![]);
    }

    #[test]
    fn legal_sales_residual_requirement_then_real_evidence_derives_give() {
        let legal = parse_formal_to_typed(LEGAL_SALES_FORMAL).expect("Legal sales elaborates");
        let original_module = legal.clone();
        let sales = rule(&legal, "sales");
        let residual = residuate(sales).expect("sales has a conjunctive antecedent");
        let price = seeded_event("agreesPrice", ["alice", "car", "bob"], "tau");
        let object = seeded_event("agreesObject", ["bob", "car", "alice"], "tau");
        let target = give_effect_event("alice", "car", "bob", "tau");
        let mut clock = TestClock::default();

        let initial = close(vec![price.clone()], &[sales.clone()], &mut clock);
        let original_closure = initial.clone();
        assert_eq!(initial.known, vec![price.clone()]);
        assert!(initial.derivations.is_empty());
        let pending = query_residual(&residual, &target, &initial.known).expect("query sales");
        let substitution = Substitution {
            bindings: vec![
                SubstitutionBinding {
                    parameter: "creditor".into(),
                    value: SubstitutionValue::Term(Term::Var("bob".into())),
                },
                SubstitutionBinding {
                    parameter: "debtor".into(),
                    value: SubstitutionValue::Term(Term::Var("alice".into())),
                },
                SubstitutionBinding {
                    parameter: "object".into(),
                    value: SubstitutionValue::Term(Term::Var("car".into())),
                },
                SubstitutionBinding {
                    parameter: "t".into(),
                    value: SubstitutionValue::Time(TimeExpr::At("tau".into())),
                },
            ],
        };
        assert_eq!(
            pending,
            vec![ResidualRequirement {
                rule: RuleRef {
                    name: "sales".into()
                },
                evidence: vec![price.clone()],
                goal_substitution: substitution.clone(),
                substitution: substitution.clone(),
                goal: target.clone(),
                required: Antecedent::Event(object.clone()),
                parameters: vec![],
            }]
        );
        assert_eq!(initial, original_closure);
        assert!(!initial.known.contains(&object));
        assert!(!initial.known.contains(&target));
        assert_eq!(clock.next, 0, "a requirement consumes no DerivationTime");

        // The caller supplies actual evidence independently of the requirement.
        let mut facts = initial.known.clone();
        facts.push(object.clone());
        assert!(
            query_residual(&residual, &target, &facts)
                .unwrap()
                .is_empty()
        );
        assert!(
            !facts.contains(&target),
            "satisfying B does not make the query assert C"
        );
        assert_eq!(clock.next, 0);
        let completed = close(facts.clone(), &[sales.clone()], &mut clock);
        assert_eq!(
            completed.known,
            vec![price.clone(), object.clone(), target.clone()]
        );
        assert_eq!(completed.derivations.len(), 1);
        let derived = &completed.derivations[0];
        assert_eq!(derived.event, target);
        assert_eq!(derived.record.rule, "sales");
        assert_eq!(derived.record.antecedents, facts);
        assert_eq!(derived.record.substitution, substitution);
        assert_eq!(derived.derived_at.name, "d1");
        assert!(
            query_residual(&residual, &target, &completed.known)
                .unwrap()
                .is_empty()
        );
        assert!(
            !completed
                .derivations
                .iter()
                .any(|derived| derived.event == object)
        );
        assert_eq!(initial, original_closure);
        assert_eq!(
            legal, original_module,
            "residuating sales leaves Legal unchanged"
        );
    }

    #[test]
    fn close_uses_conjunctive_sales_matching() {
        let legal = parse_formal_to_typed(LEGAL_SALES_FORMAL).expect("formal sales elaborates");
        let rules = legal
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Rule(rule) => Some(rule.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let price = seeded_event("agreesPrice", ["alice", "car", "bob"], "tau");
        let object = seeded_event("agreesObject", ["bob", "car", "alice"], "tau");
        let mut clock = TestClock::default();

        let closure = close(vec![price.clone(), object.clone()], &rules, &mut clock);

        assert_eq!(
            closure.known,
            vec![
                price.clone(),
                object.clone(),
                give_effect_event("alice", "car", "bob", "tau")
            ]
        );
        assert_eq!(closure.derivations.len(), 1);
        assert_eq!(
            closure.derivations[0].record.antecedents,
            vec![price, object]
        );
        assert_eq!(
            closure.derivations[0].derived_at,
            DerivationTime {
                name: "d1".to_string(),
            }
        );
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn legal_traditio_delivery_elaborates_equally_from_formal_and_cnl() {
        let formal = parse_formal_to_typed(LEGAL_TRADITIO_DELIVERY_FORMAL)
            .expect("formal traditio/delivery elaborates");
        let cnl = parse_cnl_to_typed(LEGAL_TRADITIO_DELIVERY_CNL)
            .expect("cnl traditio/delivery elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    fn legal_delivery_specializes_traditio_by_beta_reduction() {
        let legal = parse_formal_to_typed(LEGAL_TRADITIO_DELIVERY_FORMAL)
            .expect("formal traditio/delivery elaborates");

        let deliver = legal
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Const(constant) if constant.name == "deliver" => Some(constant),
                _ => None,
            })
            .expect("deliver constant exists");
        assert_eq!(
            deliver,
            &Constant {
                name: "deliver".to_string(),
                ty: Type::Entity("Conduct".to_string()),
            }
        );

        let traditio = rule(&legal, "traditio");
        assert_eq!(
            traditio.parameters,
            vec![
                Arg {
                    name: "conduct".to_string(),
                    ty: Type::Entity("Conduct".to_string()),
                },
                Arg {
                    name: "debtor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "object".to_string(),
                    ty: Type::Entity("Thing".to_string()),
                },
                Arg {
                    name: "creditor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "t".to_string(),
                    ty: Type::PropositionTime,
                },
            ]
        );

        let delivery = rule(&legal, "delivery");
        assert_eq!(
            delivery.parameters,
            vec![
                Arg {
                    name: "debtor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "object".to_string(),
                    ty: Type::Entity("Thing".to_string()),
                },
                Arg {
                    name: "creditor".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "t".to_string(),
                    ty: Type::PropositionTime,
                },
            ]
        );
        let clause = horn_clause(delivery);
        assert_eq!(unit_antecedent(clause).verb, "give");
        assert_eq!(clause.consequent.verb, "do");
        assert_eq!(
            clause.consequent.args,
            vec![
                Term::Var("debtor".to_string()),
                Term::Const("deliver".to_string()),
                Term::Var("object".to_string()),
                Term::Var("creditor".to_string()),
            ]
        );
    }

    #[test]
    fn close_uses_beta_reduced_delivery_rule() {
        let legal = parse_formal_to_typed(LEGAL_TRADITIO_DELIVERY_FORMAL)
            .expect("formal traditio/delivery elaborates");
        let rules = legal
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "delivery" => Some(rule.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let give = give_effect_event("alice", "car", "bob", "tau");
        let mut clock = TestClock::default();

        let closure = close(vec![give.clone()], &rules, &mut clock);

        assert_eq!(
            closure.known,
            vec![
                give.clone(),
                do_effect_event(
                    "alice",
                    "deliver",
                    "car",
                    "bob",
                    give.proposition.time.clone()
                )
            ]
        );
        assert_eq!(closure.derivations.len(), 1);
        assert_eq!(closure.derivations[0].record.antecedents, vec![give]);
    }

    #[test]
    fn legal_sales_and_delivery_preserve_composed_after_times() {
        let sales_module = parse_formal_to_typed(LEGAL_SALES_FORMAL).expect("sales elaborates");
        let delivery_module =
            parse_formal_to_typed(LEGAL_TRADITIO_DELIVERY_FORMAL).expect("delivery elaborates");
        let rules = vec![
            rule(&sales_module, "sales").clone(),
            rule(&delivery_module, "delivery").clone(),
        ];
        let seeds = vec![
            seeded_event("agreesPrice", ["alice", "car", "bob"], "tau"),
            seeded_event("agreesObject", ["bob", "car", "alice"], "tau"),
        ];
        let mut clock = TestClock::default();
        let closure = close(seeds, &rules, &mut clock);
        let give = give_effect_event("alice", "car", "bob", "tau");
        let expected_do = do_effect_event(
            "alice",
            "deliver",
            "car",
            "bob",
            give.proposition.time.clone(),
        );

        assert_eq!(closure.known.len(), 4);
        assert_eq!(closure.derivations.len(), 2);
        assert_eq!(closure.derivations[0].event, give);
        assert_eq!(closure.derivations[1].event, expected_do);
        assert_eq!(
            closure.derivations[1].record.antecedents,
            vec![give.clone()]
        );
        assert_eq!(
            closure.derivations[1].event.transition,
            Some(StateTransitionType {
                input: StateTime::At(give.proposition.time.clone()),
                output: StateTime::At(TimeExpr::After(Box::new(give.proposition.time))),
            }),
        );
    }

    #[test]
    fn instantiate_matches_nested_after_times_without_collapsing_them() {
        let transfer = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("transfer elaborates");
        let mut rule = rule(&transfer, "transfer").clone();
        let RuleBody::HornClause(clause) = &mut rule.body else {
            panic!("expected Horn clause");
        };
        let Antecedent::Event(premise) = &mut clause.antecedent else {
            panic!("expected unit antecedent");
        };
        premise.proposition.time = TimeExpr::After(Box::new(TimeExpr::At("t".to_string())));
        let mut seed = transfer_seed();
        let inner_time = TimeExpr::After(Box::new(TimeExpr::At("tau".to_string())));
        seed.proposition.time = TimeExpr::After(Box::new(inner_time.clone()));

        let derivation = instantiate(&rule, &seed).expect("nested time matches");

        assert_eq!(derivation.event.proposition.time, seed.proposition.time);
        assert_eq!(
            derivation.event.transition.as_ref().unwrap().input,
            StateTime::At(inner_time.clone()),
        );
        assert!(
            derivation
                .record
                .substitution
                .bindings
                .contains(&SubstitutionBinding {
                    parameter: "t".to_string(),
                    value: SubstitutionValue::Time(inner_time),
                })
        );
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn legal_do_and_not_do_elaborate_equally_from_formal_and_cnl() {
        let formal =
            parse_formal_to_typed(LEGAL_DO_NOTDO_FORMAL).expect("formal do/notDo elaborates");
        let cnl = parse_cnl_to_typed(LEGAL_DO_NOTDO_CNL).expect("cnl do/notDo elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    fn legal_do_and_not_do_insert_conduct_relations() {
        let legal =
            parse_formal_to_typed(LEGAL_DO_NOTDO_FORMAL).expect("formal do/notDo elaborates");

        for verb_name in ["do", "notDo"] {
            let verb = legal
                .declarations
                .iter()
                .find_map(|declaration| match declaration {
                    Declaration::Verb(verb) if verb.name == verb_name => Some(verb),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{verb_name} verb exists"));

            assert_eq!(
                verb.args,
                vec![
                    Arg {
                        name: "subject".to_string(),
                        ty: Type::Entity("Person".to_string()),
                    },
                    Arg {
                        name: "conduct".to_string(),
                        ty: Type::Entity("Conduct".to_string()),
                    },
                    Arg {
                        name: "indirectObject".to_string(),
                        ty: Type::Optional(Box::new(Type::Entity("Thing".to_string()))),
                    },
                    Arg {
                        name: "beneficiary".to_string(),
                        ty: Type::Entity("Person".to_string()),
                    },
                ]
            );

            let Some(Effect::StateTransform { operations, .. }) = &verb.effect else {
                panic!("{verb_name} has a state transform");
            };
            assert_eq!(operations.len(), 2);

            let relation = Term::RecordLiteral {
                entity: "Relation".to_string(),
                fields: vec![
                    RecordFieldValue {
                        field: "debtor".to_string(),
                        value: Term::Var("subject".to_string()),
                    },
                    RecordFieldValue {
                        field: "thing".to_string(),
                        value: Term::Var("indirectObject".to_string()),
                    },
                    RecordFieldValue {
                        field: "conduct".to_string(),
                        value: Term::Var("conduct".to_string()),
                    },
                    RecordFieldValue {
                        field: "creditor".to_string(),
                        value: Term::Var("beneficiary".to_string()),
                    },
                ],
            };

            assert_eq!(
                operations[0],
                StateOperation::AddToSet {
                    target: FieldAccess {
                        base: Term::Var("subject".to_string()),
                        path: vec!["patrimony".to_string(), "obligations".to_string()],
                    },
                    value: relation.clone(),
                },
                "{verb_name} debtor operation"
            );
            assert_eq!(
                operations[1],
                StateOperation::AddToSet {
                    target: FieldAccess {
                        base: Term::Var("beneficiary".to_string()),
                        path: vec!["patrimony".to_string(), "rights".to_string()],
                    },
                    value: relation,
                },
                "{verb_name} beneficiary operation"
            );
        }
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn legal_give_elaborates_equally_from_formal_and_cnl() {
        let formal = parse_formal_to_typed(LEGAL_GIVE_FORMAL).expect("formal give elaborates");
        let cnl = parse_cnl_to_typed(LEGAL_GIVE_CNL).expect("cnl give elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    fn legal_give_has_only_two_patrimonial_asset_operations() {
        let legal = parse_formal_to_typed(LEGAL_GIVE_FORMAL).expect("formal give elaborates");
        let give = legal
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Verb(verb) if verb.name == "give" => Some(verb),
                _ => None,
            })
            .expect("give verb exists");

        assert_eq!(
            give.args,
            vec![
                Arg {
                    name: "subject".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "object".to_string(),
                    ty: Type::Entity("Thing".to_string()),
                },
                Arg {
                    name: "recipient".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
            ]
        );

        let Some(Effect::StateTransform { operations, .. }) = &give.effect else {
            panic!("give has a state transform");
        };
        assert_eq!(operations.len(), 2);

        assert_eq!(
            operations[0],
            StateOperation::RemoveFromSet {
                target: FieldAccess {
                    base: Term::Var("subject".to_string()),
                    path: vec!["patrimony".to_string(), "assets".to_string()],
                },
                value: Term::Var("object".to_string()),
            }
        );
        assert_eq!(
            operations[1],
            StateOperation::AddToSet {
                target: FieldAccess {
                    base: Term::Var("recipient".to_string()),
                    path: vec!["patrimony".to_string(), "assets".to_string()],
                },
                value: Term::Var("object".to_string()),
            }
        );
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn legal_record_literals_elaborate_equally_from_formal_and_cnl() {
        let formal =
            parse_formal_to_typed(LEGAL_RECORD_LITERAL_FORMAL).expect("formal legal elaborates");
        let cnl = parse_cnl_to_typed(LEGAL_RECORD_LITERAL_CNL).expect("cnl legal elaborates");

        assert_eq!(formal, cnl);
    }

    #[test]
    fn legal_relation_record_literal_typechecks_optional_fields() {
        let legal =
            parse_formal_to_typed(LEGAL_RECORD_LITERAL_FORMAL).expect("formal legal elaborates");
        let records = legal
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Verb(verb) if verb.name == "records" => Some(verb),
                _ => None,
            })
            .expect("records verb exists");
        let Some(Effect::StateTransform { operations, .. }) = &records.effect else {
            panic!("records has a state transform");
        };

        assert_eq!(
            operations[0],
            StateOperation::AddToSet {
                target: FieldAccess {
                    base: Term::Var("subject".to_string()),
                    path: vec!["patrimony".to_string(), "obligations".to_string()],
                },
                value: Term::RecordLiteral {
                    entity: "Relation".to_string(),
                    fields: vec![
                        RecordFieldValue {
                            field: "debtor".to_string(),
                            value: Term::Var("subject".to_string()),
                        },
                        RecordFieldValue {
                            field: "thing".to_string(),
                            value: Term::Var("object".to_string()),
                        },
                        RecordFieldValue {
                            field: "conduct".to_string(),
                            value: Term::None,
                        },
                        RecordFieldValue {
                            field: "creditor".to_string(),
                            value: Term::Var("recipient".to_string()),
                        },
                    ],
                },
            }
        );
    }

    #[test]
    fn legal_relation_record_literal_rejects_wrong_optional_payload_types() {
        let thing_error = parse_formal_to_typed(LEGAL_RECORD_LITERAL_BAD_THING)
            .expect_err("Person is not assignable to Thing?");
        let conduct_error = parse_formal_to_typed(LEGAL_RECORD_LITERAL_BAD_CONDUCT)
            .expect_err("Thing is not assignable to Conduct?");

        assert_eq!(
            thing_error.to_string(),
            "cannot assign `recipient` to field `Relation.thing`"
        );
        assert_eq!(
            conduct_error.to_string(),
            "cannot assign `object` to field `Relation.conduct`"
        );
    }

    #[test]
    fn legal_relation_record_literal_rejects_none_for_non_optional_field() {
        let error = parse_formal_to_typed(LEGAL_RECORD_LITERAL_BAD_NONE)
            .expect_err("none is not assignable to Person");

        assert_eq!(
            error.to_string(),
            "cannot assign `none` to field `Relation.debtor`"
        );
    }

    #[test]
    fn timed_seeded_verb_application_elaborates_to_seeded_prop_at_time() {
        let module = parse_formal_to_typed(FORMAL).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "deliveryAcquiresAsset" => Some(rule),
                _ => None,
            })
            .expect("deliveryAcquiresAsset rule exists");

        let clause = horn_clause(rule);

        assert_eq!(rule.parameters[3].name, "t");
        assert_eq!(rule.parameters[3].ty, Type::PropositionTime);
        assert_eq!(unit_antecedent(clause).verb, "delivers");
        assert_eq!(
            unit_antecedent(clause).args,
            vec![
                Term::Var("giver".to_string()),
                Term::Var("object".to_string()),
                Term::Var("receiver".to_string())
            ]
        );
        assert_eq!(
            unit_antecedent(clause).proposition,
            PropositionType {
                kind: PropositionKind::Seeded,
                time: TimeExpr::At("t".to_string()),
            }
        );
    }

    #[test]
    fn timed_effect_verb_application_elaborates_to_effect_prop_at_time() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transfer" => Some(rule),
                _ => None,
            })
            .expect("transfer rule exists");

        let clause = horn_clause(rule);

        assert_eq!(clause.consequent.verb, "transfers");
        assert_eq!(
            clause.consequent.proposition,
            PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::After(Box::new(TimeExpr::At("t".to_string()))),
            }
        );
    }

    #[test]
    fn effect_event_after_time_resolves_state_transition_from_source_to_after_time() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transfer" => Some(rule),
                _ => None,
            })
            .expect("transfer rule exists");

        let clause = horn_clause(rule);

        assert_eq!(
            clause.consequent.transition,
            Some(StateTransitionType {
                input: StateTime::At(TimeExpr::At("t".to_string())),
                output: StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("t".to_string())))),
            })
        );
    }

    #[test]
    fn effect_event_at_plain_time_keeps_input_unresolved_and_sets_output_time() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_AT_PLAIN_TIME).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transferAtPlainTime" => Some(rule),
                _ => None,
            })
            .expect("transferAtPlainTime rule exists");

        let clause = horn_clause(rule);

        assert_eq!(
            unit_antecedent(clause).transition,
            Some(StateTransitionType {
                input: StateTime::Unresolved,
                output: StateTime::At(TimeExpr::At("t".to_string())),
            })
        );
    }

    #[test]
    fn transfer_rule_elaborates_to_seeded_to_effect_horn_clause() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transfer" => Some(rule),
                _ => None,
            })
            .expect("transfer rule exists");

        let clause = horn_clause(rule);

        assert_eq!(
            unit_antecedent(clause).proposition,
            PropositionType {
                kind: PropositionKind::Seeded,
                time: TimeExpr::At("t".to_string()),
            }
        );
        assert_eq!(
            clause.consequent.proposition,
            PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::After(Box::new(TimeExpr::At("t".to_string()))),
            }
        );
    }

    #[test]
    fn transfer_rule_is_parameterized_abstraction_over_horn_clause_body() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transfer" => Some(rule),
                _ => None,
            })
            .expect("transfer rule exists");

        assert_eq!(
            rule.parameters,
            vec![
                Arg {
                    name: "giver".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "asset".to_string(),
                    ty: Type::Entity("Asset".to_string()),
                },
                Arg {
                    name: "receiver".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "t".to_string(),
                    ty: Type::PropositionTime,
                },
            ]
        );
        assert!(matches!(rule.body, RuleBody::HornClause(_)));
    }

    #[test]
    fn transfer_default_is_supernormal_over_parameterized_transfer_rule() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let default = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Default(default) if default.name == "transferNormally" => {
                    Some(default)
                }
                _ => None,
            })
            .expect("transferNormally default exists");

        assert_eq!(
            default.parameters,
            vec![
                Arg {
                    name: "giver".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "asset".to_string(),
                    ty: Type::Entity("Asset".to_string()),
                },
                Arg {
                    name: "receiver".to_string(),
                    ty: Type::Entity("Person".to_string()),
                },
                Arg {
                    name: "t".to_string(),
                    ty: Type::PropositionTime,
                },
            ]
        );
        assert_eq!(
            default.body,
            DefaultBody::Supernormal {
                rule: RuleRef {
                    name: "transfer".to_string()
                },
            }
        );
    }

    #[test]
    fn transfer_rule_derives_single_effect_event_from_matching_seed() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transfer" => Some(rule),
                _ => None,
            })
            .expect("transfer rule exists");
        let seed = Event {
            verb: "delivers".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var("alice".to_string()),
                Term::Var("asset_a".to_string()),
                Term::Var("bob".to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Seeded,
                time: TimeExpr::At("tau".to_string()),
            },
            transition: None,
        };

        let derived = derive_once(
            rule,
            &seed,
            DerivationTime {
                name: "d1".to_string(),
            },
        )
        .expect("seed matches transfer rule");

        assert_eq!(
            derived.event,
            Event {
                verb: "transfers".to_string(),
                polarity: Polarity::Positive,
                args: vec![
                    Term::Var("alice".to_string()),
                    Term::Var("asset_a".to_string()),
                    Term::Var("bob".to_string()),
                ],
                proposition: PropositionType {
                    kind: PropositionKind::Effect,
                    time: TimeExpr::After(Box::new(TimeExpr::At("tau".to_string()))),
                },
                transition: Some(StateTransitionType {
                    input: StateTime::At(TimeExpr::At("tau".to_string())),
                    output: StateTime::At(TimeExpr::After(Box::new(TimeExpr::At(
                        "tau".to_string()
                    )))),
                }),
            }
        );
        assert_eq!(
            derived.derived_at,
            DerivationTime {
                name: "d1".to_string()
            }
        );
        assert_eq!(
            derived.record,
            DerivationRecord {
                rule: "transfer".to_string(),
                antecedents: vec![seed],
                substitution: Substitution {
                    bindings: vec![
                        SubstitutionBinding {
                            parameter: "asset".to_string(),
                            value: SubstitutionValue::Term(Term::Var("asset_a".to_string())),
                        },
                        SubstitutionBinding {
                            parameter: "giver".to_string(),
                            value: SubstitutionValue::Term(Term::Var("alice".to_string())),
                        },
                        SubstitutionBinding {
                            parameter: "receiver".to_string(),
                            value: SubstitutionValue::Term(Term::Var("bob".to_string())),
                        },
                        SubstitutionBinding {
                            parameter: "t".to_string(),
                            value: SubstitutionValue::Time(TimeExpr::At("tau".to_string())),
                        },
                    ],
                },
            }
        );
    }

    #[test]
    fn instantiate_and_stamp_are_separate_steps() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transfer" => Some(rule),
                _ => None,
            })
            .expect("transfer rule exists");
        let seed = transfer_seed();

        let unstamped = instantiate(rule, &seed).expect("seed matches transfer rule");
        assert_eq!(unstamped.record.rule, "transfer");

        let stamped = stamp(
            unstamped,
            DerivationTime {
                name: "d1".to_string(),
            },
        );
        assert_eq!(
            stamped.derived_at,
            DerivationTime {
                name: "d1".to_string()
            }
        );
    }

    #[test]
    fn close_accumulates_known_events_and_derivations_with_external_clock() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rules: Vec<_> = module
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Rule(rule) => Some(rule.clone()),
                _ => None,
            })
            .collect();
        let seed = transfer_seed();
        let expected_effect = Event {
            verb: "transfers".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var("alice".to_string()),
                Term::Var("asset_a".to_string()),
                Term::Var("bob".to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::After(Box::new(TimeExpr::At("tau".to_string()))),
            },
            transition: Some(StateTransitionType {
                input: StateTime::At(TimeExpr::At("tau".to_string())),
                output: StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("tau".to_string())))),
            }),
        };
        let mut clock = TestClock::default();

        let closure = close(vec![seed.clone()], &rules, &mut clock);

        assert_eq!(closure.known, vec![seed.clone(), expected_effect.clone()]);
        assert_eq!(closure.derivations.len(), 1);
        assert_eq!(closure.derivations[0].event, expected_effect);
        assert_eq!(
            closure.derivations[0].derived_at,
            DerivationTime {
                name: "d1".to_string()
            }
        );
        assert_eq!(closure.derivations[0].record.antecedents, vec![seed]);
    }

    #[test]
    fn close_deduplicates_known_events_but_keeps_duplicate_justifications() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rules: Vec<_> = module
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Rule(rule) => Some(rule.clone()),
                _ => None,
            })
            .collect();
        let seed = transfer_seed();
        let already_known_effect = Event {
            verb: "transfers".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var("alice".to_string()),
                Term::Var("asset_a".to_string()),
                Term::Var("bob".to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::After(Box::new(TimeExpr::At("tau".to_string()))),
            },
            transition: Some(StateTransitionType {
                input: StateTime::At(TimeExpr::At("tau".to_string())),
                output: StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("tau".to_string())))),
            }),
        };
        let mut clock = TestClock::default();

        let closure = close(
            vec![seed.clone(), already_known_effect.clone()],
            &rules,
            &mut clock,
        );

        assert_eq!(closure.known, vec![seed, already_known_effect.clone()]);
        assert_eq!(closure.derivations.len(), 1);
        assert_eq!(closure.derivations[0].event, already_known_effect);
    }

    #[test]
    fn execute_transfer_effect_is_functional_and_moves_asset_between_snapshots() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let transfers = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Verb(verb) if verb.name == "transfers" => Some(verb),
                _ => None,
            })
            .expect("transfers verb exists");
        let event = Event {
            verb: "transfers".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var("alice".to_string()),
                Term::Var("asset_a".to_string()),
                Term::Var("bob".to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::After(Box::new(TimeExpr::At("tau".to_string()))),
            },
            transition: Some(StateTransitionType {
                input: StateTime::At(TimeExpr::At("tau".to_string())),
                output: StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("tau".to_string())))),
            }),
        };
        let input = StateSnapshot {
            time: StateTime::At(TimeExpr::At("tau".to_string())),
            store: Store::new()
                .with_set(
                    ObjectId("alice".to_string()),
                    FieldName("assets".to_string()),
                    vec![ObjectId("asset_a".to_string())],
                )
                .with_set(
                    ObjectId("bob".to_string()),
                    FieldName("assets".to_string()),
                    vec![],
                ),
        };
        let original = input.clone();

        let output = execute_effect(&event, transfers, &input).expect("effect executes");

        assert_eq!(input, original);
        assert_eq!(
            output.time,
            StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("tau".to_string()))))
        );
        assert_eq!(
            output.store.set(
                &ObjectId("alice".to_string()),
                &FieldName("assets".to_string())
            ),
            Some(&[].into_iter().collect())
        );
        assert_eq!(
            output.store.set(
                &ObjectId("bob".to_string()),
                &FieldName("assets".to_string())
            ),
            Some(
                &[Value::Object(ObjectId("asset_a".to_string()))]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn legal_sales_delivery_derives_then_executes_functional_snapshots() {
        let legal = parse_formal_to_typed(LEGAL_SALES_DELIVERY_FORMAL).expect("Legal elaborates");
        let input = legal_snapshot();
        let original = input.clone();
        let rules = vec![
            rule(&legal, "sales").clone(),
            rule(&legal, "delivery").clone(),
        ];
        let closure = close(
            vec![
                seeded_event("agreesPrice", ["alice", "car", "bob"], "tau"),
                seeded_event("agreesObject", ["bob", "car", "alice"], "tau"),
            ],
            &rules,
            &mut TestClock::default(),
        );
        let give = give_effect_event("alice", "car", "bob", "tau");
        let do_event = do_effect_event(
            "alice",
            "deliver",
            "car",
            "bob",
            TimeExpr::After(Box::new(TimeExpr::At("tau".to_string()))),
        );
        assert_eq!(closure.known.len(), 4);
        assert!(closure.known.contains(&give));
        assert!(closure.known.contains(&do_event));
        assert_eq!(input, original, "derivation does not execute effects");

        let after_give =
            execute_effect(&give, verb(&legal, "give"), &input).expect("give executes");
        let saved_give = after_give.clone();
        let after_do =
            execute_effect(&do_event, verb(&legal, "do"), &after_give).expect("do executes");
        let expected_give = original
            .store
            .clone()
            .with_set(
                ObjectId("alice-patrimony".to_string()),
                FieldName("assets".to_string()),
                vec![],
            )
            .with_set(
                ObjectId("bob-patrimony".to_string()),
                FieldName("assets".to_string()),
                vec![ObjectId("car".to_string())],
            );
        assert_eq!(
            after_give.store, expected_give,
            "give moves only assets, preserving all rights and obligations"
        );
        let delivery_relation =
            runtime_relation(runtime_object("car"), Value::Const("deliver".to_string()));
        for (object, field) in [
            ("alice-patrimony", "obligations"),
            ("bob-patrimony", "rights"),
        ] {
            assert_eq!(
                after_do
                    .store
                    .set(&ObjectId(object.to_string()), &FieldName(field.to_string())),
                Some(&[delivery_relation.clone()].into_iter().collect()),
            );
        }
        // Compare the complete store to cover every unrelated field as well.
        let expected = after_give
            .store
            .clone()
            .with_value(
                ObjectId("alice-patrimony".to_string()),
                FieldName("obligations".to_string()),
                Value::Set([delivery_relation.clone()].into_iter().collect()),
            )
            .with_value(
                ObjectId("bob-patrimony".to_string()),
                FieldName("rights".to_string()),
                Value::Set([delivery_relation].into_iter().collect()),
            );
        assert_eq!(after_do.store, expected);
        assert_eq!(after_give.time, StateTime::At(give.proposition.time));
        assert_eq!(after_do.time, StateTime::At(do_event.proposition.time));
        assert_eq!(input, original);
        assert_eq!(after_give, saved_give);
    }

    #[test]
    fn legal_do_materializes_none_and_symbolic_conduct_constant() {
        let legal = parse_formal_to_typed(LEGAL_SALES_DELIVERY_FORMAL).expect("Legal elaborates");
        let input = legal_snapshot();
        let original = input.clone();
        let mut event = do_effect_event(
            "alice",
            "deliver",
            "car",
            "bob",
            TimeExpr::At("tau".to_string()),
        );
        event.args[2] = Term::None;
        let output =
            execute_effect(&event, verb(&legal, "do"), &input).expect("optional do executes");
        let relation = runtime_relation(Value::None, Value::Const("deliver".to_string()));
        for (object, field) in [
            ("alice-patrimony", "obligations"),
            ("bob-patrimony", "rights"),
        ] {
            assert_eq!(
                output
                    .store
                    .set(&ObjectId(object.to_string()), &FieldName(field.to_string())),
                Some(&[relation.clone()].into_iter().collect())
            );
        }
        assert_eq!(
            output.store,
            with_runtime_relation(input.store.clone(), relation),
            "do preserves assets and all unrelated fields"
        );
        assert_eq!(input, original);
    }

    #[test]
    fn legal_record_sets_deduplicate_and_remove_by_structural_value() {
        let legal = parse_formal_to_typed(LEGAL_SALES_DELIVERY_FORMAL).expect("Legal elaborates");
        let do_verb = verb(&legal, "do");
        let input = legal_snapshot();
        let event = do_effect_event(
            "alice",
            "deliver",
            "car",
            "bob",
            TimeExpr::At("tau".to_string()),
        );
        let once = execute_effect(&event, do_verb, &input).expect("first insertion");
        let relation = runtime_relation(runtime_object("car"), Value::Const("deliver".to_string()));
        let expected = with_runtime_relation(input.store.clone(), relation);
        assert_eq!(
            once.store, expected,
            "do creates a relation without moving assets"
        );
        let repeated_event = effect_after_snapshot(event.clone(), &once);
        let twice = execute_effect(&repeated_event, do_verb, &once).expect("second insertion");
        assert_eq!(twice.store, once.store, "sets deduplicate the same record");

        // Exercise the existing -= operation with the same typed record in a different field order.
        let mut retract = do_verb.clone();
        retract.name = "retract".to_string();
        let Some(Effect::StateTransform { operations, .. }) = &mut retract.effect else {
            panic!("do has a StateTransform");
        };
        for operation in operations {
            let StateOperation::AddToSet { target, value } = operation.clone() else {
                panic!("do inserts records");
            };
            let mut value = value;
            if let Term::RecordLiteral { fields, .. } = &mut value {
                fields.reverse();
            }
            *operation = StateOperation::RemoveFromSet { target, value };
        }
        let mut retract_event = effect_after_snapshot(event, &twice);
        retract_event.verb = "retract".to_string();
        let removed = execute_effect(&retract_event, &retract, &twice).expect("record removal");
        assert_eq!(removed.store, input.store);
        assert_eq!(twice.store, once.store, "removal preserves its input");
    }

    #[test]
    fn legal_effect_failure_on_second_operation_preserves_input() {
        let legal = parse_formal_to_typed(LEGAL_SALES_DELIVERY_FORMAL).expect("Legal elaborates");
        let mut input = legal_snapshot();
        input.store = input.store.with_value(
            ObjectId("bob-patrimony".to_string()),
            FieldName("assets".to_string()),
            Value::None,
        );
        let original = input.clone();
        let error = execute_effect(
            &give_effect_event("alice", "car", "bob", "tau"),
            verb(&legal, "give"),
            &input,
        )
        .expect_err("second target is not a set");
        assert_eq!(
            error.to_string(),
            "field `bob-patrimony.assets` is not a set"
        );
        assert_eq!(input, original, "no partial update escapes a failed effect");
    }

    #[test]
    fn legal_nested_paths_reject_missing_and_non_reference_intermediates() {
        let legal = parse_formal_to_typed(LEGAL_SALES_DELIVERY_FORMAL).expect("Legal elaborates");
        let mut inputs = vec![StateSnapshot {
            time: StateTime::At(TimeExpr::At("tau".to_string())),
            store: Store::new(),
        }];
        let mut non_reference = legal_snapshot();
        non_reference.store = non_reference.store.with_value(
            ObjectId("alice".to_string()),
            FieldName("patrimony".to_string()),
            Value::None,
        );
        inputs.push(non_reference);
        for (input, message) in inputs.into_iter().zip([
            "missing field `alice.patrimony`",
            "field `alice.patrimony` is not an object reference",
        ]) {
            let original = input.clone();
            let error = execute_effect(
                &give_effect_event("alice", "car", "bob", "tau"),
                verb(&legal, "give"),
                &input,
            )
            .expect_err("invalid intermediate reference");
            assert_eq!(error.to_string(), message);
            assert_eq!(input, original);
        }
    }

    #[test]
    fn legal_execution_requires_resolved_matching_state_times() {
        let legal = parse_formal_to_typed(LEGAL_SALES_DELIVERY_FORMAL).expect("Legal elaborates");
        let input = legal_snapshot();
        let original = input.clone();
        let event = give_effect_event("alice", "car", "bob", "tau");
        let mut wrong_input = event.clone();
        wrong_input.transition.as_mut().unwrap().input =
            StateTime::At(TimeExpr::At("other".to_string()));
        let mut unresolved_input = event.clone();
        unresolved_input.transition.as_mut().unwrap().input = StateTime::Unresolved;
        let mut unresolved_output = event.clone();
        unresolved_output.transition.as_mut().unwrap().output = StateTime::Unresolved;
        let mut wrong_output = event;
        wrong_output.transition.as_mut().unwrap().output =
            StateTime::At(TimeExpr::At("other".to_string()));
        for (event, message) in [
            (
                wrong_input,
                "input StateTime does not match effect transition",
            ),
            (unresolved_input, "effect input StateTime is unresolved"),
            (unresolved_output, "effect output StateTime is unresolved"),
            (
                wrong_output,
                "effect output StateTime differs from PropositionTime",
            ),
        ] {
            let error = execute_effect(&event, verb(&legal, "give"), &input)
                .expect_err("invalid transition");
            assert_eq!(error.to_string(), message);
            assert_eq!(input, original);
        }
    }

    #[test]
    fn transfer_rule_does_not_derive_from_non_matching_seed() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let rule = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == "transfer" => Some(rule),
                _ => None,
            })
            .expect("transfer rule exists");
        let seed = Event {
            verb: "transfers".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var("alice".to_string()),
                Term::Var("asset_a".to_string()),
                Term::Var("bob".to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::At("tau".to_string()),
            },
            transition: Some(StateTransitionType {
                input: StateTime::Unresolved,
                output: StateTime::At(TimeExpr::At("tau".to_string())),
            }),
        };

        assert_eq!(
            derive_once(
                rule,
                &seed,
                DerivationTime {
                    name: "d1".to_string(),
                },
            ),
            None
        );
    }

    #[test]
    fn effect_verb_state_transform_body_is_type_checked_and_represented() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let transfers = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Verb(verb) if verb.name == "transfers" => Some(verb),
                _ => None,
            })
            .expect("transfers verb exists");

        assert_eq!(
            transfers.effect,
            Some(Effect::StateTransform {
                transition: StateTransitionType {
                    input: StateTime::Unresolved,
                    output: StateTime::Unresolved,
                },
                operations: vec![
                    StateOperation::RemoveFromSet {
                        target: FieldAccess {
                            base: Term::Var("subject".to_string()),
                            path: vec!["assets".to_string()],
                        },
                        value: Term::Var("object".to_string()),
                    },
                    StateOperation::AddToSet {
                        target: FieldAccess {
                            base: Term::Var("recipient".to_string()),
                            path: vec!["assets".to_string()],
                        },
                        value: Term::Var("object".to_string()),
                    },
                ],
            })
        );
    }

    #[test]
    fn effect_verb_state_transform_is_temporally_indexed_but_unresolved_at_declaration() {
        let module = parse_formal_to_typed(FORMAL_EFFECT_HIERARCHY).expect("formal elaborates");
        let transfers = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Verb(verb) if verb.name == "transfers" => Some(verb),
                _ => None,
            })
            .expect("transfers verb exists");

        let Some(Effect::StateTransform { transition, .. }) = &transfers.effect else {
            panic!("transfers has a state transform interpretation");
        };

        assert_eq!(
            transition,
            &StateTransitionType {
                input: StateTime::Unresolved,
                output: StateTime::Unresolved,
            }
        );
    }

    #[test]
    fn effect_verb_state_transform_rejects_set_member_type_mismatch() {
        let error = parse_formal_to_typed(FORMAL_BAD_EFFECT_BODY)
            .expect_err("bad effect body should fail elaboration");

        assert_eq!(
            error.to_string(),
            "cannot update `subject.assets` with `recipient`"
        );
    }

    #[test]
    fn effect_is_usable_where_derived_or_prop_at_same_time_is_expected() {
        let effect = PropositionType {
            kind: PropositionKind::Effect,
            time: TimeExpr::At("t".to_string()),
        };
        let derived = PropositionType {
            kind: PropositionKind::Derived,
            time: TimeExpr::At("t".to_string()),
        };
        let prop = PropositionType {
            kind: PropositionKind::Prop,
            time: TimeExpr::At("t".to_string()),
        };

        assert!(effect.is_subtype_of(&derived));
        assert!(effect.is_subtype_of(&prop));
    }

    #[test]
    fn derived_and_seeded_do_not_flow_against_the_effect_hierarchy() {
        let effect = PropositionType {
            kind: PropositionKind::Effect,
            time: TimeExpr::At("t".to_string()),
        };
        let derived = PropositionType {
            kind: PropositionKind::Derived,
            time: TimeExpr::At("t".to_string()),
        };
        let seeded = PropositionType {
            kind: PropositionKind::Seeded,
            time: TimeExpr::At("t".to_string()),
        };

        assert!(!derived.is_subtype_of(&effect));
        assert!(!seeded.is_subtype_of(&derived));
    }

    #[test]
    fn formal_rejects_event_arguments_that_violate_grammatical_role_types() {
        let error = parse_formal_to_typed(FORMAL_BAD_GRAMMATICAL_ROLES)
            .expect_err("formal bad roles should fail elaboration");

        assert_eq!(error.to_string(), "conflicting inferred types for `thing`");
    }

    #[test]
    #[ignore = "CNL frontend is paused; Formal is the active semantic boundary"]
    fn cnl_rejects_event_arguments_that_violate_grammatical_role_types() {
        let error = parse_cnl_to_typed(CNL_BAD_GRAMMATICAL_ROLES)
            .expect_err("cnl bad roles should fail elaboration");

        assert_eq!(error.to_string(), "conflicting inferred types for `thing`");
    }

    #[test]
    fn formal_rejects_verb_application_without_resolved_proposition_time() {
        let error = parse_formal_to_typed(FORMAL_UNRESOLVED_PROP_TIME)
            .expect_err("unresolved PropositionTime should fail elaboration");

        assert_eq!(
            error.to_string(),
            "unresolved PropositionTime for `delivers`"
        );
    }

    #[test]
    fn legal_exception_default_preserves_rule_lambda_and_typed_priority_reference() {
        let surface = parse_formal(LEGAL_SALES_RETENTION_FORMAL).expect("Formal parses");
        let surface_default = surface
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                crate::surface::FormalDeclaration::Default(default)
                    if default.name == "RetentionSale" =>
                {
                    Some(default)
                }
                _ => None,
            })
            .expect("exception surface AST");
        assert_eq!(
            surface_default.body,
            crate::surface::SurfaceDefaultBody::Exception {
                rule: "retention".to_string(),
                to: "NormalSale".to_string(),
            }
        );
        let legal = elaborate_formal(surface).expect("Legal elaborates");
        let normal = default(&legal, "NormalSale");
        let exception = default(&legal, "RetentionSale");
        let sales = rule(&legal, "sales");
        let retention = rule(&legal, "retention");
        assert_eq!(
            horn_clause(sales).antecedent,
            horn_clause(retention).antecedent
        );
        assert_eq!(normal.parameters, sales.parameters);
        assert_eq!(exception.parameters, retention.parameters);
        assert_eq!(exception.parameters, normal.parameters);
        assert_eq!(
            normal.body,
            DefaultBody::Supernormal {
                rule: RuleRef {
                    name: "sales".to_string()
                }
            }
        );
        assert_eq!(
            exception.body,
            DefaultBody::Exception {
                rule: RuleRef {
                    name: "retention".to_string()
                },
                to: DefaultRef {
                    name: "NormalSale".to_string()
                },
            }
        );
        assert_eq!(
            select_active_rules(&legal, &["NormalSale"]).unwrap(),
            vec![sales.clone()]
        );
        assert_eq!(
            select_active_rules(&legal, &["NormalSale", "RetentionSale"]).unwrap(),
            vec![retention.clone()]
        );
        assert_eq!(
            retention.parameters.len(),
            4,
            "selection preserves the complete lambda"
        );
    }

    #[test]
    fn legal_default_selection_changes_derived_and_executed_effect_for_same_facts() {
        let legal = parse_formal_to_typed(LEGAL_SALES_RETENTION_FORMAL).expect("Legal elaborates");
        let seeds = vec![
            seeded_event("agreesPrice", ["alice", "car", "bob"], "tau"),
            seeded_event("agreesObject", ["bob", "car", "alice"], "tau"),
        ];
        let input = legal_snapshot();
        let original = input.clone();
        for (activated, expected_rule, expected_event) in [
            (
                vec!["NormalSale"],
                "sales",
                give_effect_event("alice", "car", "bob", "tau"),
            ),
            (
                vec!["RetentionSale"],
                "retention",
                do_effect_event(
                    "alice",
                    "deliver",
                    "car",
                    "bob",
                    TimeExpr::At("tau".to_string()),
                ),
            ),
            (
                vec!["NormalSale", "RetentionSale"],
                "retention",
                do_effect_event(
                    "alice",
                    "deliver",
                    "car",
                    "bob",
                    TimeExpr::At("tau".to_string()),
                ),
            ),
        ] {
            // Only the selected lambdas are passed to the unchanged Horn closure.
            let rules = select_active_rules(&legal, &activated).expect("defaults selected");
            assert_eq!(rules, vec![rule(&legal, expected_rule).clone()]);
            let closure = close(seeds.clone(), &rules, &mut TestClock::default());
            let mut expected_known = seeds.clone();
            expected_known.push(expected_event.clone());
            assert_eq!(
                closure.known, expected_known,
                "exactly one competing effect is derived"
            );
            assert_eq!(closure.derivations.len(), 1);
            let derived = &closure.derivations[0];
            assert_eq!(derived.record.rule, expected_rule);
            assert_eq!(derived.record.antecedents, seeds);
            assert_eq!(input, original, "selection and derivation do not execute");

            let output = execute_effect(&derived.event, verb(&legal, &derived.event.verb), &input)
                .expect("selected effect executes");
            let expected_store = if expected_rule == "sales" {
                input
                    .store
                    .clone()
                    .with_set(
                        ObjectId("alice-patrimony".to_string()),
                        FieldName("assets".to_string()),
                        vec![],
                    )
                    .with_set(
                        ObjectId("bob-patrimony".to_string()),
                        FieldName("assets".to_string()),
                        vec![ObjectId("car".to_string())],
                    )
            } else {
                with_runtime_relation(
                    input.store.clone(),
                    runtime_relation(runtime_object("car"), Value::Const("deliver".to_string())),
                )
            };
            assert_eq!(output.store, expected_store);
            assert_eq!(output.time, StateTime::At(expected_event.proposition.time));
            assert_eq!(input, original);
        }
    }

    #[test]
    fn default_selection_is_explicit_order_independent_and_deduplicated() {
        let legal = parse_formal_to_typed(LEGAL_SALES_RETENTION_FORMAL).expect("Legal elaborates");
        assert!(select_active_rules(&legal, &[]).unwrap().is_empty());
        let expected = vec![rule(&legal, "retention").clone()];
        assert_eq!(
            select_active_rules(&legal, &["RetentionSale"]).unwrap(),
            expected
        );
        assert_eq!(
            select_active_rules(&legal, &["RetentionSale", "NormalSale", "RetentionSale"]).unwrap(),
            expected
        );
        assert_eq!(
            select_active_rules(&legal, &["NormalSale", "RetentionSale"]).unwrap(),
            expected
        );
        let error = select_active_rules(&legal, &["Missing"]).expect_err("unknown activation");
        assert_eq!(error.to_string(), "unknown activated default `Missing`");
    }

    #[test]
    fn formal_exception_defaults_reject_unknown_references_and_incompatible_signatures() {
        for (source, message) in [
            (
                LEGAL_SALES_RETENTION_FORMAL.replace("exception retention", "exception missing"),
                "unknown rule `missing` in exception default",
            ),
            (
                LEGAL_SALES_RETENTION_FORMAL.replace("to NormalSale", "to Missing"),
                "unknown default `Missing` in exception default",
            ),
            (
                LEGAL_SALES_RETENTION_FORMAL.replace(
                    "rule retention(debtor, object, creditor, t)",
                    "rule retention(t, debtor, object, creditor)",
                ),
                "incompatible parameter types for exception default `RetentionSale` and target `NormalSale`",
            ),
            (
                format!("{LEGAL_SALES_RETENTION_FORMAL}\ndefault NormalSale = supernormal sales\n"),
                "duplicate default `NormalSale`",
            ),
        ] {
            let error = parse_formal_to_typed(&source).expect_err("invalid default");
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn formal_exception_default_requires_to_and_target() {
        for source in [
            LEGAL_SALES_RETENTION_FORMAL.replace("to NormalSale", "NormalSale"),
            LEGAL_SALES_RETENTION_FORMAL.replace("to NormalSale", "to"),
        ] {
            assert!(parse_formal(&source).is_err());
            assert!(crate::formal::parse_formal_manual(&source).is_err());
        }
    }

    #[test]
    fn default_selection_rejects_cyclic_priority_in_constructed_ast() {
        let mut legal =
            parse_formal_to_typed(LEGAL_SALES_RETENTION_FORMAL).expect("Legal elaborates");
        for declaration in &mut legal.declarations {
            if let Declaration::Default(default) = declaration {
                if default.name == "NormalSale" {
                    default.body = DefaultBody::Exception {
                        rule: RuleRef {
                            name: "sales".to_string(),
                        },
                        to: DefaultRef {
                            name: "RetentionSale".to_string(),
                        },
                    };
                }
            }
        }
        let error = select_active_rules(&legal, &["NormalSale", "RetentionSale"])
            .expect_err("cyclic priority");
        assert!(
            error
                .to_string()
                .starts_with("cyclic default priority at `")
        );
    }

    #[test]
    fn legal_default_when_elaborates_to_existing_typed_antecedent() {
        let surface = parse_formal(LEGAL_SALES_RETENTION_WHEN_FORMAL).expect("Formal parses");
        let parsed = surface
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                crate::surface::FormalDeclaration::Default(default)
                    if default.name == "RetentionSale" =>
                {
                    Some(default)
                }
                _ => None,
            })
            .expect("conditional default");
        assert!(matches!(
            parsed.condition,
            Some(crate::surface::SurfaceAntecedent::Event(_))
        ));
        let legal = elaborate_formal(surface).expect("Legal elaborates");
        let retention = default(&legal, "RetentionSale");
        assert_eq!(retention.parameters, rule(&legal, "retention").parameters);
        assert_eq!(
            retention.condition,
            Some(Antecedent::Event(seeded_event(
                "retentionAgreed",
                ["debtor", "object", "creditor"],
                "t",
            )))
        );
        assert!(default(&legal, "NormalSale").condition.is_none());
    }

    #[test]
    fn default_extension_distinguishes_applicability_priority_and_unchecked_consistency() {
        let legal =
            parse_formal_to_typed(LEGAL_SALES_RETENTION_WHEN_FORMAL).expect("Legal elaborates");
        let empty = evaluate_defaults(&legal, &[]).expect("empty context");
        assert_eq!(
            empty.applicable.len(),
            1,
            "unconditional supernormal is always a candidate"
        );
        assert_eq!(empty.applicable[0].default.name, "NormalSale");
        assert!(empty.applicable[0].matches[0].antecedents.is_empty());
        assert!(
            empty.applicable[0].matches[0]
                .substitution
                .bindings
                .is_empty()
        );
        assert!(empty.defeated.is_empty());
        assert_eq!(empty.rules(), &[rule(&legal, "sales").clone()]);
        assert_eq!(
            empty.consistency,
            crate::defaults::ConsistencyStatus::NotChecked
        );
        let closure = close(vec![], empty.rules(), &mut TestClock::default());
        assert!(
            closure.known.is_empty(),
            "a candidate lambda does not assert its consequent"
        );

        let fact = seeded_event("retentionAgreed", ["alice", "car", "bob"], "tau");
        let extension = evaluate_defaults(&legal, &[fact.clone()]).expect("matching condition");
        assert_eq!(extension.applicable.len(), 2);
        assert_eq!(
            extension.defeated,
            vec![DefaultRef {
                name: "NormalSale".to_string()
            }]
        );
        assert_eq!(
            extension.selected,
            vec![DefaultRef {
                name: "RetentionSale".to_string()
            }]
        );
        assert_eq!(extension.rules(), &[rule(&legal, "retention").clone()]);
        let witness = &extension.applicable[1].matches[0];
        assert_eq!(witness.antecedents, vec![fact]);
        assert_eq!(
            witness.substitution.bindings,
            vec![
                SubstitutionBinding {
                    parameter: "creditor".to_string(),
                    value: SubstitutionValue::Term(Term::Var("bob".to_string()))
                },
                SubstitutionBinding {
                    parameter: "debtor".to_string(),
                    value: SubstitutionValue::Term(Term::Var("alice".to_string()))
                },
                SubstitutionBinding {
                    parameter: "object".to_string(),
                    value: SubstitutionValue::Term(Term::Var("car".to_string()))
                },
                SubstitutionBinding {
                    parameter: "t".to_string(),
                    value: SubstitutionValue::Time(TimeExpr::At("tau".to_string()))
                },
            ]
        );
        assert_eq!(
            extension.consistency,
            crate::defaults::ConsistencyStatus::NotChecked
        );
    }

    #[test]
    fn legal_context_default_extension_selects_then_closes_and_executes() {
        let legal =
            parse_formal_to_typed(LEGAL_SALES_RETENTION_WHEN_FORMAL).expect("Legal elaborates");
        let sales_facts = vec![
            seeded_event("agreesPrice", ["alice", "car", "bob"], "tau"),
            seeded_event("agreesObject", ["bob", "car", "alice"], "tau"),
        ];
        let input = legal_snapshot();
        let original = input.clone();
        for agreed in [false, true] {
            let mut facts = sales_facts.clone();
            if agreed {
                facts.push(seeded_event(
                    "retentionAgreed",
                    ["alice", "car", "bob"],
                    "tau",
                ));
            }
            let extension = evaluate_defaults(&legal, &facts).expect("evaluate Known");
            let expected_rule = if agreed { "retention" } else { "sales" };
            assert_eq!(extension.rules(), &[rule(&legal, expected_rule).clone()]);
            let closure = close(facts.clone(), extension.rules(), &mut TestClock::default());
            let expected_event = if agreed {
                do_effect_event(
                    "alice",
                    "deliver",
                    "car",
                    "bob",
                    TimeExpr::At("tau".to_string()),
                )
            } else {
                give_effect_event("alice", "car", "bob", "tau")
            };
            let mut expected_known = facts;
            expected_known.push(expected_event.clone());
            assert_eq!(closure.known, expected_known);
            assert_eq!(closure.derivations.len(), 1);
            assert_eq!(closure.derivations[0].record.rule, expected_rule);
            assert_eq!(
                closure.derivations[0].record.antecedents, sales_facts,
                "condition evidence remains separate from rule provenance"
            );
            assert_eq!(input, original);
            let output = execute_effect(
                &closure.derivations[0].event,
                verb(&legal, &expected_event.verb),
                &input,
            )
            .expect("selected effect executes");
            let expected_store = if agreed {
                with_runtime_relation(
                    input.store.clone(),
                    runtime_relation(runtime_object("car"), Value::Const("deliver".to_string())),
                )
            } else {
                input
                    .store
                    .clone()
                    .with_set(
                        ObjectId("alice-patrimony".to_string()),
                        FieldName("assets".to_string()),
                        vec![],
                    )
                    .with_set(
                        ObjectId("bob-patrimony".to_string()),
                        FieldName("assets".to_string()),
                        vec![ObjectId("car".to_string())],
                    )
            };
            assert_eq!(output.store, expected_store);
            assert_eq!(output.time, StateTime::At(expected_event.proposition.time));
            assert_eq!(input, original);
        }
    }

    #[test]
    fn default_conditions_use_shared_substitution_for_conjunction_and_time() {
        let source = format!(
            "{LEGAL_SALES_RETENTION_WHEN_FORMAL}\n & agreesObject(creditor, object, debtor) @ t\n"
        );
        assert_eq!(
            parse_formal(&source).unwrap(),
            crate::formal::parse_formal_manual(&source).unwrap()
        );
        let legal = parse_formal_to_typed(&source).expect("conjunctive condition elaborates");
        assert!(matches!(
            default(&legal, "RetentionSale").condition,
            Some(Antecedent::And(_, _))
        ));
        let retention = seeded_event("retentionAgreed", ["alice", "car", "bob"], "tau");
        for incompatible in [
            seeded_event("agreesObject", ["bob", "house", "alice"], "tau"),
            seeded_event("agreesObject", ["bob", "car", "alice"], "other"),
        ] {
            let extension = evaluate_defaults(&legal, &[retention.clone(), incompatible]).unwrap();
            assert_eq!(extension.applicable.len(), 1);
            assert!(
                extension.defeated.is_empty(),
                "a non-applicable exception has no priority effect"
            );
            assert_eq!(extension.rules(), &[rule(&legal, "sales").clone()]);
        }
        let other = seeded_event("agreesObject", ["bob", "car", "alice"], "tau");
        let extension = evaluate_defaults(&legal, &[retention.clone(), other.clone()]).unwrap();
        assert_eq!(extension.rules(), &[rule(&legal, "retention").clone()]);
        assert_eq!(
            extension.applicable[1].matches[0].antecedents,
            vec![retention, other]
        );
    }

    #[test]
    fn default_conditions_match_derived_effects_and_constants_in_closure_known() {
        let source = LEGAL_SALES_RETENTION_WHEN_FORMAL.replace(
            "retentionAgreed(debtor, object, creditor) @ t",
            "do(debtor, deliver, object, creditor) @ after t",
        );
        let legal = parse_formal_to_typed(&source).expect("effect condition elaborates");
        let facts = vec![
            seeded_event("agreesPrice", ["alice", "car", "bob"], "tau"),
            seeded_event("agreesObject", ["bob", "car", "alice"], "tau"),
        ];
        let context = close(
            facts,
            &[rule(&legal, "retention").clone()],
            &mut TestClock::default(),
        );
        let extension = evaluate_defaults(&legal, &context.known).expect("Closure.known context");
        assert_eq!(extension.rules(), &[rule(&legal, "retention").clone()]);
        let effect = &context.derivations[0].event;
        assert_eq!(
            extension.applicable[1].matches[0].antecedents,
            vec![effect.clone()]
        );
        let mut different = effect.clone();
        different.args[1] = Term::Const("another_conduct".to_string());
        assert_eq!(
            evaluate_defaults(&legal, &[different]).unwrap().rules(),
            &[rule(&legal, "sales").clone()]
        );
    }

    #[test]
    fn formal_default_conditions_reject_invalid_terms_types_and_times() {
        let condition = "retentionAgreed(debtor, object, creditor) @ t";
        for (replacement, expected) in [
            (
                "unknown(debtor, object, creditor) @ t",
                "unknown verb `unknown`",
            ),
            (
                "retentionAgreed(debtor, object) @ t",
                "verb `retentionAgreed` expects 3 args, got 2",
            ),
            (
                "retentionAgreed(object, object, creditor) @ t",
                "default condition term `object` is not assignable to Entity(\"Person\")",
            ),
            (
                "retentionAgreed(missing, object, creditor) @ t",
                "unknown default condition term `missing`",
            ),
            (
                "retentionAgreed(debtor, object, creditor)",
                "unresolved PropositionTime for `retentionAgreed`",
            ),
            (
                "retentionAgreed(debtor, object, creditor) @ debtor",
                "default condition time `debtor` is not a PropositionTime parameter",
            ),
        ] {
            let error = parse_formal_to_typed(
                &LEGAL_SALES_RETENTION_WHEN_FORMAL.replace(condition, replacement),
            )
            .expect_err("invalid condition");
            assert_eq!(error.to_string(), expected);
        }
        let incomplete = LEGAL_SALES_RETENTION_WHEN_FORMAL.replace(condition, "");
        assert!(
            parse_formal(&incomplete).is_err(),
            "when requires a consuming antecedent"
        );
        assert!(crate::formal::parse_formal_manual(&incomplete).is_err());
    }

    #[test]
    fn legal_blocked_when_preserves_typed_antecedents_and_rule_lambdas() {
        let surface = parse_formal(LEGAL_SALES_BLOCKING_FORMAL).unwrap();
        assert_eq!(
            surface,
            crate::formal::parse_formal_manual(LEGAL_SALES_BLOCKING_FORMAL).unwrap()
        );
        let module = elaborate_formal(surface).unwrap();
        for (name, rule_name, marker) in [
            ("NormalSale", "sales", "inconsistentSale"),
            ("RetentionSale", "retention", "inconsistentRetention"),
        ] {
            let default = default(&module, name);
            assert_eq!(default.parameters, rule(&module, rule_name).parameters);
            let Some(Antecedent::Event(event)) = &default.blocking else {
                panic!("typed event blocker");
            };
            assert_eq!(event.verb, marker);
            assert_eq!(
                event.args,
                vec![
                    Term::Var("debtor".into()),
                    Term::Var("object".into()),
                    Term::Var("creditor".into())
                ]
            );
            assert_eq!(
                event.proposition,
                PropositionType {
                    kind: PropositionKind::Derived,
                    time: TimeExpr::At("t".into()),
                }
            );
            assert!(event.transition.is_none());
        }
        assert!(default(&module, "NormalSale").condition.is_none());
        assert!(default(&module, "RetentionSale").condition.is_some());
        let empty = evaluate_defaults(&module, &[]).unwrap();
        assert_eq!(empty.blocking_scope, BlockingScope::RuleLambda);
        assert!(empty.blocked.is_empty());
        assert_eq!(empty.rules(), &[rule(&module, "sales").clone()]);
    }

    #[test]
    fn legal_domain_markers_block_defaults_before_priority_and_effect_derivation() {
        let module = parse_formal_to_typed(LEGAL_SALES_BLOCKING_FORMAL).unwrap();
        let hard_rules = vec![
            rule(&module, "saleConflict").clone(),
            rule(&module, "retentionConflict").clone(),
        ];
        let sale_facts = vec![
            seeded_event("agreesPrice", ["alice", "car", "bob"], "tau"),
            seeded_event("agreesObject", ["bob", "car", "alice"], "tau"),
        ];
        let agreement = seeded_event("retentionAgreed", ["alice", "car", "bob"], "tau");
        let input = legal_snapshot();
        let original = input.clone();
        for (agreed, sale_blocked, retention_blocked, selected_rule) in [
            (false, false, false, Some("sales")),
            (false, true, false, None),
            (true, false, false, Some("retention")),
            (true, true, false, Some("retention")),
            (true, false, true, Some("sales")),
            (true, true, true, None),
        ] {
            let mut facts = sale_facts.clone();
            if agreed {
                facts.push(agreement.clone());
            }
            if sale_blocked {
                let mut negative = sale_facts[0].clone();
                negative.polarity = Polarity::EvidentialNot;
                facts.push(negative);
            }
            if retention_blocked {
                let mut negative = agreement.clone();
                negative.polarity = Polarity::EvidentialNot;
                facts.push(negative);
            }
            assert!(
                evaluate_defaults(&module, &facts)
                    .unwrap()
                    .blocked
                    .is_empty(),
                "opposition alone has no blocking privilege; evaluation does not run Horn"
            );
            let context = close(facts, &hard_rules, &mut TestClock::default());
            let original_context = context.clone();
            let extension = evaluate_defaults(&module, &context.known).unwrap();
            assert_eq!(extension.applicable.len(), if agreed { 2 } else { 1 });
            assert_eq!(
                extension.blocked.len(),
                usize::from(sale_blocked) + usize::from(retention_blocked)
            );
            assert_eq!(extension.blocking_scope, BlockingScope::RuleLambda);
            assert_eq!(
                extension.consistency,
                crate::defaults::ConsistencyStatus::NotChecked
            );
            for block in &extension.blocked {
                let marker = if block.default.name == "NormalSale" {
                    "inconsistentSale"
                } else {
                    "inconsistentRetention"
                };
                let derived = context
                    .derivations
                    .iter()
                    .find(|derived| derived.event.verb == marker)
                    .unwrap();
                assert_eq!(block.evidence, vec![derived.event.clone()]);
                assert_eq!(block.substitution, derived.record.substitution);
                assert_eq!(derived.record.antecedents.len(), 2);
                assert_eq!(
                    derived.record.antecedents[1].polarity,
                    Polarity::EvidentialNot
                );
            }
            let expected_rules: Vec<_> = selected_rule
                .into_iter()
                .map(|name| rule(&module, name).clone())
                .collect();
            assert_eq!(extension.rules(), expected_rules);
            let expected_default: Vec<_> = selected_rule
                .into_iter()
                .map(|name| DefaultRef {
                    name: if name == "sales" {
                        "NormalSale"
                    } else {
                        "RetentionSale"
                    }
                    .into(),
                })
                .collect();
            assert_eq!(extension.selected, expected_default);
            let defeated: Vec<_> = if agreed && !sale_blocked && !retention_blocked {
                vec![DefaultRef {
                    name: "NormalSale".into(),
                }]
            } else {
                vec![]
            };
            assert_eq!(
                extension.defeated, defeated,
                "blocking is distinct from priority defeat"
            );
            let closure = close(
                context.known.clone(),
                extension.rules(),
                &mut TestClock::default(),
            );
            assert_eq!(
                context, original_context,
                "default evaluation does not retract or modify Known"
            );
            if let Some(selected_rule) = selected_rule {
                assert_eq!(closure.derivations.len(), 1);
                let derived = &closure.derivations[0];
                assert_eq!(derived.record.rule, selected_rule);
                assert_eq!(
                    derived.record.antecedents, sale_facts,
                    "blocking evidence does not replace Horn provenance"
                );
                let expected_event = if selected_rule == "sales" {
                    give_effect_event("alice", "car", "bob", "tau")
                } else {
                    do_effect_event("alice", "deliver", "car", "bob", TimeExpr::At("tau".into()))
                };
                assert_eq!(derived.event, expected_event);
                let output =
                    execute_effect(&derived.event, verb(&module, &derived.event.verb), &input)
                        .unwrap();
                let expected_store = if selected_rule == "sales" {
                    input
                        .store
                        .clone()
                        .with_set(
                            ObjectId("alice-patrimony".into()),
                            FieldName("assets".into()),
                            vec![],
                        )
                        .with_set(
                            ObjectId("bob-patrimony".into()),
                            FieldName("assets".into()),
                            vec![ObjectId("car".into())],
                        )
                } else {
                    with_runtime_relation(
                        input.store.clone(),
                        runtime_relation(runtime_object("car"), Value::Const("deliver".into())),
                    )
                };
                assert_eq!(output.store, expected_store);
                assert_eq!(output.time, StateTime::At(expected_event.proposition.time));
            } else {
                assert!(closure.derivations.is_empty());
                assert_eq!(closure.known, context.known);
            }
            assert_eq!(input, original);
        }
    }

    #[test]
    fn default_blocking_is_global_and_preserves_every_witness() {
        let module = parse_formal_to_typed(LEGAL_SALES_BLOCKING_FORMAL).unwrap();
        let sales = vec![
            seeded_event("agreesPrice", ["alice", "car", "bob"], "tau"),
            seeded_event("agreesObject", ["bob", "car", "alice"], "tau"),
        ];
        let mut facts = sales.clone();
        for (debtor, object, creditor, time) in [
            ("charlie", "house", "dora", "other_time"),
            ("eve", "boat", "frank", "later"),
        ] {
            let positive = seeded_event("agreesPrice", [debtor, object, creditor], time);
            let mut negative = positive.clone();
            negative.polarity = Polarity::EvidentialNot;
            facts.extend([positive, negative]);
        }
        let context = close(
            facts,
            &[rule(&module, "saleConflict").clone()],
            &mut TestClock::default(),
        );
        let extension = evaluate_defaults(&module, &context.known).unwrap();
        assert_eq!(extension.blocking_scope, BlockingScope::RuleLambda);
        assert_eq!(extension.blocked.len(), 2);
        for (block, derived) in extension.blocked.iter().zip(&context.derivations) {
            assert_eq!(block.default.name, "NormalSale");
            assert_eq!(block.evidence, vec![derived.event.clone()]);
            assert_eq!(block.substitution, derived.record.substitution);
        }
        assert!(extension.selected.is_empty());
        assert!(
            extension.rules().is_empty(),
            "unrelated sales and times still block the entire lambda"
        );
        assert!(
            close(sales.clone(), extension.rules(), &mut TestClock::default())
                .derivations
                .is_empty()
        );
        let fresh = evaluate_defaults(&module, &sales).unwrap();
        assert!(
            fresh.blocked.is_empty(),
            "blocking belongs to one evaluation, not persistent state"
        );
        assert_eq!(fresh.rules(), &[rule(&module, "sales").clone()]);
        let mut inapplicable_context = context.known.clone();
        let mut irrelevant_marker = context.derivations[0].event.clone();
        irrelevant_marker.verb = "inconsistentRetention".into();
        inapplicable_context.push(irrelevant_marker);
        assert_eq!(
            evaluate_defaults(&module, &inapplicable_context).unwrap(),
            extension,
            "an inapplicable exception is not evaluated for blocking"
        );
    }

    #[test]
    fn default_blockers_share_substitution_time_polarity_and_typed_constants() {
        let blocker = "blocked when\n        inconsistentSale(debtor, object, creditor) @ t";
        let source = LEGAL_SALES_BLOCKING_FORMAL.replacen(blocker,
            "blocked when agreesPrice(debtor, object, creditor) @ t & ~agreesObject(creditor, object, debtor) @ t", 1);
        assert_eq!(
            parse_formal(&source).unwrap(),
            crate::formal::parse_formal_manual(&source).unwrap()
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let price = seeded_event("agreesPrice", ["alice", "car", "bob"], "tau");
        let positive = seeded_event("agreesObject", ["bob", "car", "alice"], "tau");
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let mut wrong_object = negative.clone();
        wrong_object.args[1] = Term::Var("house".into());
        let mut wrong_time = negative.clone();
        wrong_time.proposition.time = TimeExpr::At("later".into());
        let mut wrong_person = negative.clone();
        wrong_person.args[0] = Term::Var("charlie".into());
        for other in [positive, wrong_object, wrong_time, wrong_person] {
            assert!(
                evaluate_defaults(&module, &[price.clone(), other])
                    .unwrap()
                    .blocked
                    .is_empty()
            );
        }
        let extension = evaluate_defaults(&module, &[price.clone(), negative.clone()]).unwrap();
        assert_eq!(extension.blocked[0].evidence, vec![price, negative]);
        assert_eq!(extension.blocked[0].substitution.bindings.len(), 4);
        assert!(extension.rules().is_empty());
        let source = LEGAL_SALES_BLOCKING_FORMAL.replacen(
            blocker,
            "blocked when do(debtor, deliver, object, creditor) @ after t",
            1,
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let effect = do_effect_event("alice", "deliver", "car", "bob", TimeExpr::At("tau".into()));
        let extension = evaluate_defaults(&module, &[effect.clone()]).unwrap();
        assert_eq!(extension.blocked[0].evidence, vec![effect.clone()]);
        assert!(
            extension.blocked[0]
                .substitution
                .bindings
                .contains(&SubstitutionBinding {
                    parameter: "t".into(),
                    value: SubstitutionValue::Time(TimeExpr::At("tau".into()))
                })
        );
        let mut wrong_constant = effect;
        wrong_constant.args[1] = Term::Const("another_conduct".into());
        assert!(
            evaluate_defaults(&module, &[wrong_constant])
                .unwrap()
                .blocked
                .is_empty()
        );
    }

    #[test]
    fn formal_default_blockers_reject_invalid_terms_types_times_and_empty_clauses() {
        let blocker = "blocked when\n        inconsistentSale(debtor, object, creditor) @ t";
        for (replacement, expected) in [
            (
                "unknown(debtor, object, creditor) @ t",
                "unknown verb `unknown`",
            ),
            (
                "inconsistentSale(debtor, object) @ t",
                "expects 3 args, got 2",
            ),
            (
                "inconsistentSale(object, object, creditor) @ t",
                "not assignable",
            ),
            (
                "inconsistentSale(missing, object, creditor) @ t",
                "unknown default condition term `missing`",
            ),
            (
                "inconsistentSale(debtor, object, creditor)",
                "unresolved PropositionTime",
            ),
            (
                "inconsistentSale(debtor, object, creditor) @ debtor",
                "not a PropositionTime parameter",
            ),
        ] {
            let source = LEGAL_SALES_BLOCKING_FORMAL.replacen(
                blocker,
                &format!("blocked when {replacement}"),
                1,
            );
            assert!(
                parse_formal_to_typed(&source)
                    .unwrap_err()
                    .to_string()
                    .contains(expected)
            );
        }
        for suffix in [
            "blocked",
            "blocked when",
            "blocked when &",
            "blocked when inconsistentSale(debtor, object, creditor) @ t &",
        ] {
            let source = format!("{LEGAL_SALES_RETENTION_WHEN_FORMAL}\n{suffix}\n");
            assert!(parse_formal(&source).is_err(), "{suffix}");
            assert!(
                crate::formal::parse_formal_manual(&source).is_err(),
                "{suffix}"
            );
        }
    }

    #[test]
    fn blocked_defaults_do_not_contribute_transitive_priority_edges() {
        let source = format!(
            "{LEGAL_SALES_BLOCKING_FORMAL}\ndefault UpperSale = exception retention to RetentionSale\n"
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let agreement = seeded_event("retentionAgreed", ["alice", "car", "bob"], "tau");
        let mut negative = agreement.clone();
        negative.polarity = Polarity::EvidentialNot;
        let context = close(
            vec![agreement, negative],
            &[rule(&module, "retentionConflict").clone()],
            &mut TestClock::default(),
        );
        let extension = evaluate_defaults(&module, &context.known).unwrap();
        assert_eq!(extension.blocked.len(), 1);
        assert_eq!(extension.blocked[0].default.name, "RetentionSale");
        assert!(
            extension.defeated.is_empty(),
            "blocked RetentionSale cannot defeat NormalSale through UpperSale"
        );
        assert_eq!(
            extension.selected,
            vec![
                DefaultRef {
                    name: "NormalSale".into()
                },
                DefaultRef {
                    name: "UpperSale".into()
                }
            ]
        );
        assert_eq!(
            extension.rules(),
            &[
                rule(&module, "sales").clone(),
                rule(&module, "retention").clone()
            ]
        );
    }

    #[test]
    fn evidential_not_parses_and_elaborates_with_temporal_and_role_types() {
        let surface = parse_formal(FORMAL_EVIDENTIAL_NOT).expect("negative Formal parses");
        assert_eq!(
            surface,
            crate::formal::parse_formal_manual(FORMAL_EVIDENTIAL_NOT).unwrap()
        );
        let module = elaborate_formal(surface).expect("negative Formal elaborates");
        let denied = rule(&module, "deniedDelivery");
        assert_eq!(
            unit_antecedent(horn_clause(denied)).polarity,
            Polarity::EvidentialNot
        );
        let consequent = &horn_clause(denied).consequent;
        assert_eq!(consequent.polarity, Polarity::EvidentialNot);
        assert_eq!(consequent.proposition.kind, PropositionKind::Effect);
        assert!(consequent.proposition.is_subtype_of(&PropositionType {
            kind: PropositionKind::Derived,
            time: consequent.proposition.time.clone(),
        }));
        assert_eq!(denied.parameters, rule(&module, "transfer").parameters);
        assert_eq!(
            consequent.proposition.time,
            TimeExpr::After(Box::new(TimeExpr::At("t".to_string())))
        );
        let bad_roles = FORMAL_EVIDENTIAL_NOT.replace(
            "~delivers(giver, object, receiver)",
            "~delivers(object, giver, receiver)",
        );
        assert!(parse_formal_to_typed(&bad_roles).is_err());
        let unresolved = FORMAL_EVIDENTIAL_NOT.replace(
            "~delivers(giver, object, receiver) @ t",
            "~delivers(giver, object, receiver)",
        );
        assert!(parse_formal_to_typed(&unresolved).is_err());
        for bad in ["~", "~~delivers(giver, object, receiver) @ t"] {
            let source =
                FORMAL_EVIDENTIAL_NOT.replace("~delivers(giver, object, receiver) @ t", bad);
            assert!(parse_formal(&source).is_err());
            assert!(crate::formal::parse_formal_manual(&source).is_err());
        }
    }

    #[test]
    fn evidential_not_requires_explicit_evidence_and_matches_only_its_polarity() {
        let module = parse_formal_to_typed(FORMAL_EVIDENTIAL_NOT).unwrap();
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let denied = rule(&module, "deniedDelivery");
        let transfer = rule(&module, "transfer");
        assert!(instantiate(denied, &positive).is_none());
        assert!(instantiate(transfer, &negative).is_none());
        assert!(instantiate(denied, &negative).is_some());
        assert!(instantiate(transfer, &positive).is_some());
        let absent = close(vec![], &[denied.clone()], &mut TestClock::default());
        assert!(
            absent.known.is_empty(),
            "absence never manufactures EvidentialNot"
        );
        assert!(absent.derivations.is_empty());
        let present = close(
            vec![positive.clone()],
            &[denied.clone()],
            &mut TestClock::default(),
        );
        assert_eq!(present.known, vec![positive]);
        assert!(present.derivations.is_empty());
    }

    #[test]
    fn evidential_not_derivations_preserve_provenance_and_composed_times() {
        let module = parse_formal_to_typed(FORMAL_EVIDENTIAL_NOT).unwrap();
        let rules = vec![
            rule(&module, "deniedDelivery").clone(),
            rule(&module, "deniedAcquisition").clone(),
        ];
        let mut seed = transfer_seed();
        seed.polarity = Polarity::EvidentialNot;
        let mut clock = TestClock::default();
        let closure = close(vec![seed.clone()], &rules, &mut clock);
        assert_eq!(closure.known.len(), 3);
        assert_eq!(closure.derivations.len(), 2);
        assert_eq!(clock.next, 2);
        let first = &closure.derivations[0];
        assert_eq!(first.event.verb, "transfers");
        assert_eq!(first.event.polarity, Polarity::EvidentialNot);
        assert_eq!(first.record.rule, "deniedDelivery");
        assert_eq!(first.record.antecedents, vec![seed.clone()]);
        assert_eq!(
            first.derived_at,
            DerivationTime {
                name: "d1".to_string()
            }
        );
        assert_eq!(
            first.record.substitution,
            instantiate(&rules[0], &seed).unwrap().record.substitution
        );
        let second = &closure.derivations[1];
        assert_eq!(second.event.verb, "acquires");
        assert_eq!(second.event.polarity, Polarity::EvidentialNot);
        assert_eq!(second.record.rule, "deniedAcquisition");
        assert_eq!(second.record.antecedents, vec![first.event.clone()]);
        assert_eq!(
            second.derived_at,
            DerivationTime {
                name: "d2".to_string()
            }
        );
        assert_eq!(
            second.event.proposition.time,
            TimeExpr::After(Box::new(TimeExpr::After(Box::new(TimeExpr::At(
                "tau".to_string()
            )))))
        );
        assert!(
            closure
                .known
                .iter()
                .all(|event| event.polarity == Polarity::EvidentialNot)
        );
    }

    #[test]
    fn evidential_not_and_positive_events_coexist_in_monotonic_closure() {
        let module = parse_formal_to_typed(FORMAL_EVIDENTIAL_NOT).unwrap();
        let rules = vec![
            rule(&module, "transfer").clone(),
            rule(&module, "deniedDelivery").clone(),
            rule(&module, "deniedAcquisition").clone(),
        ];
        let seed = transfer_seed();
        let mut negative = seed.clone();
        negative.polarity = Polarity::EvidentialNot;
        let base = close(vec![seed.clone()], &rules, &mut TestClock::default());
        let expanded = close(
            vec![seed.clone(), negative.clone()],
            &rules,
            &mut TestClock::default(),
        );
        assert_eq!(expanded.known.len(), 5);
        assert!(expanded.known.contains(&seed));
        assert!(expanded.known.contains(&negative));
        assert!(
            base.known
                .iter()
                .all(|event| expanded.known.contains(event))
        );
        let positive_transfer = &base.derivations[0].event;
        let mut denied_transfer = positive_transfer.clone();
        denied_transfer.polarity = Polarity::EvidentialNot;
        assert!(expanded.known.contains(positive_transfer));
        assert!(expanded.known.contains(&denied_transfer));
        let repeated = close(expanded.known.clone(), &rules, &mut TestClock::default());
        assert_eq!(
            repeated.known, expanded.known,
            "logical identity includes polarity and deduplicates each separately"
        );
    }

    #[test]
    fn evidential_not_conjunction_requires_both_explicit_polarities() {
        let source = FORMAL_EVIDENTIAL_NOT.replace(
            "~delivers(giver, object, receiver) @ t",
            "~delivers(giver, object, receiver) @ t & delivers(giver, object, receiver) @ t",
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let denied = rule(&module, "deniedDelivery");
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        assert!(instantiate_from_known(denied, &[positive.clone()]).is_empty());
        assert!(instantiate_from_known(denied, &[negative.clone()]).is_empty());
        let matches = instantiate_from_known(denied, &[positive, negative]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].record.antecedents.len(), 2);
        assert_eq!(
            matches[0].record.antecedents[0].polarity,
            Polarity::EvidentialNot
        );
        assert_eq!(
            matches[0].record.antecedents[1].polarity,
            Polarity::Positive
        );
    }

    #[test]
    fn evidential_not_survives_rule_beta_reduction() {
        let source = LEGAL_SALES_DELIVERY_FORMAL.replace(
            "do(debtor, conduct, object, creditor) @ after t",
            "~do(debtor, conduct, object, creditor) @ after t",
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let delivery = rule(&module, "delivery");
        assert_eq!(
            horn_clause(delivery).consequent.polarity,
            Polarity::EvidentialNot
        );
        assert_eq!(
            horn_clause(delivery).consequent.args[1],
            Term::Const("deliver".to_string())
        );
        let derivation =
            instantiate(delivery, &give_effect_event("alice", "car", "bob", "tau")).unwrap();
        let mut expected = do_effect_event(
            "alice",
            "deliver",
            "car",
            "bob",
            TimeExpr::After(Box::new(TimeExpr::At("tau".to_string()))),
        );
        expected.polarity = Polarity::EvidentialNot;
        assert_eq!(derivation.event, expected);
    }

    #[test]
    fn evidential_not_rejects_seeded_consequents_and_positive_effect_execution() {
        let source = FORMAL_EVIDENTIAL_NOT.replace(
            "~transfers(giver, object, receiver) @ after t",
            "~delivers(giver, object, receiver) @ after t",
        );
        let error = parse_formal_to_typed(&source).expect_err("negative seed cannot be derived");
        assert!(error.to_string().contains("must be Derived"));
        let module = parse_formal_to_typed(FORMAL_EVIDENTIAL_NOT).unwrap();
        let mut seed = transfer_seed();
        seed.polarity = Polarity::EvidentialNot;
        let event = instantiate(rule(&module, "deniedDelivery"), &seed)
            .unwrap()
            .event;
        let input = StateSnapshot {
            time: StateTime::At(TimeExpr::At("tau".to_string())),
            store: Store::new()
                .with_set(
                    ObjectId("alice".to_string()),
                    FieldName("assets".to_string()),
                    vec![ObjectId("asset_a".to_string())],
                )
                .with_set(
                    ObjectId("bob".to_string()),
                    FieldName("assets".to_string()),
                    vec![],
                ),
        };
        let original = input.clone();
        let error = execute_effect(&event, verb(&module, "transfers"), &input)
            .expect_err("negative proposition does not run positive effect");
        assert_eq!(
            error.to_string(),
            "EvidentialNot has no executable positive StateTransform"
        );
        assert_eq!(input, original);
    }

    #[test]
    fn inconsistent_is_an_ordinary_temporally_indexed_derived_verb() {
        let module =
            parse_formal_to_typed(FORMAL_INCONSISTENT).expect("ordinary Derived elaborates");
        let declaration = verb(&module, "inconsistent");
        assert_eq!(declaration.kind, crate::typed::VerbKind::Derived);
        assert!(declaration.effect.is_none());
        let consequent = &horn_clause(rule(&module, "evidentialConflict")).consequent;
        let time = TimeExpr::After(Box::new(TimeExpr::At("t".to_string())));
        assert_eq!(consequent.polarity, Polarity::Positive);
        assert_eq!(
            consequent.proposition,
            PropositionType {
                kind: PropositionKind::Derived,
                time: time.clone()
            }
        );
        assert!(consequent.transition.is_none());
        assert!(consequent.proposition.is_subtype_of(&PropositionType {
            kind: PropositionKind::Prop,
            time: time.clone()
        }));
        assert!(!consequent.proposition.is_subtype_of(&PropositionType {
            kind: PropositionKind::Effect,
            time
        }));
        let renamed =
            parse_formal_to_typed(&FORMAL_INCONSISTENT.replace("inconsistent", "violation"))
                .unwrap();
        assert_eq!(
            verb(&renamed, "violation").kind,
            crate::typed::VerbKind::Derived,
            "the name inconsistent has no built-in meaning"
        );
    }

    #[test]
    fn explicit_opposition_derives_ordinary_inconsistency_with_provenance() {
        let module = parse_formal_to_typed(FORMAL_INCONSISTENT).unwrap();
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let facts = vec![positive.clone(), negative.clone()];
        let original = facts.clone();
        let mut clock = TestClock::default();
        let closure = close(
            facts.clone(),
            &[rule(&module, "evidentialConflict").clone()],
            &mut clock,
        );
        assert_eq!(facts, original);
        assert_eq!(closure.known.len(), 3);
        assert!(closure.known.contains(&positive));
        assert!(closure.known.contains(&negative));
        assert_eq!(closure.derivations.len(), 1);
        assert_eq!(clock.next, 1);
        let derived = &closure.derivations[0];
        assert_eq!(derived.event.verb, "inconsistent");
        assert_eq!(derived.event.args, positive.args);
        assert_eq!(derived.event.polarity, Polarity::Positive);
        assert_eq!(derived.event.proposition.kind, PropositionKind::Derived);
        assert_eq!(
            derived.event.proposition.time,
            TimeExpr::After(Box::new(TimeExpr::At("tau".to_string())))
        );
        assert!(derived.event.transition.is_none());
        assert_eq!(
            derived.derived_at,
            DerivationTime {
                name: "d1".to_string()
            }
        );
        assert_eq!(derived.record.rule, "evidentialConflict");
        assert_eq!(derived.record.antecedents, original);
        assert_eq!(derived.record.substitution.bindings.len(), 4);
        assert!(
            derived
                .record
                .substitution
                .bindings
                .contains(&SubstitutionBinding {
                    parameter: "t".to_string(),
                    value: SubstitutionValue::Time(TimeExpr::At("tau".to_string())),
                })
        );
        assert!(
            derived
                .record
                .substitution
                .bindings
                .contains(&SubstitutionBinding {
                    parameter: "object".to_string(),
                    value: SubstitutionValue::Term(Term::Var("asset_a".to_string())),
                })
        );
        let without_rule = close(facts, &[], &mut TestClock::default());
        assert_eq!(
            without_rule.known, original,
            "opposition has no consequence without a domain rule"
        );
        assert!(without_rule.derivations.is_empty());
    }

    #[test]
    fn inconsistency_rule_requires_both_polarities_with_shared_arguments_and_time() {
        let module = parse_formal_to_typed(FORMAL_INCONSISTENT).unwrap();
        let rules = vec![rule(&module, "evidentialConflict").clone()];
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let mut wrong_object = negative.clone();
        wrong_object.args[1] = Term::Var("other_asset".to_string());
        let mut wrong_time = negative.clone();
        wrong_time.proposition.time = TimeExpr::At("later".to_string());
        for facts in [
            vec![],
            vec![positive.clone()],
            vec![negative],
            vec![positive.clone(), wrong_object],
            vec![positive, wrong_time],
        ] {
            let closure = close(facts.clone(), &rules, &mut TestClock::default());
            assert_eq!(closure.known, facts);
            assert!(closure.derivations.is_empty());
        }
    }

    #[test]
    fn derived_verb_rejects_state_transform_and_cannot_execute_as_effect() {
        let source = FORMAL_INCONSISTENT.replace(
            ")\n\nrule evidentialConflict",
            ") =>\n    subject.assets += object\n\nrule evidentialConflict",
        );
        assert_eq!(
            parse_formal(&source).unwrap(),
            crate::formal::parse_formal_manual(&source).unwrap()
        );
        let error =
            parse_formal_to_typed(&source).expect_err("Derived has no operational interpretation");
        assert_eq!(
            error.to_string(),
            "derived verb `inconsistent` cannot have a StateTransform"
        );
        let module = parse_formal_to_typed(FORMAL_INCONSISTENT).unwrap();
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let closure = close(
            vec![positive, negative],
            &[rule(&module, "evidentialConflict").clone()],
            &mut TestClock::default(),
        );
        let input = StateSnapshot {
            time: StateTime::At(TimeExpr::At("tau".to_string())),
            store: Store::new(),
        };
        let original = input.clone();
        let error = execute_effect(
            &closure.derivations[0].event,
            verb(&module, "inconsistent"),
            &input,
        )
        .expect_err("ordinary Derived is not executable");
        assert_eq!(error.to_string(), "event is not an Effect");
        assert_eq!(input, original);
    }

    #[test]
    fn ordinary_inconsistency_marker_has_no_default_blocking_privilege() {
        let source =
            format!("{FORMAL_INCONSISTENT}\ndefault Detection = supernormal evidentialConflict\n");
        let module = parse_formal_to_typed(&source).unwrap();
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let facts = vec![positive, negative];
        let before = evaluate_defaults(&module, &facts).unwrap();
        let closure = close(facts, before.rules(), &mut TestClock::default());
        assert_eq!(closure.derivations[0].event.verb, "inconsistent");
        let after = evaluate_defaults(&module, &closure.known).unwrap();
        assert_eq!(
            before, after,
            "the marker is an ordinary fact and does not reject a default"
        );
        assert_eq!(
            after.consistency,
            crate::defaults::ConsistencyStatus::NotChecked
        );
    }

    #[test]
    fn ordinary_derived_marker_can_be_an_antecedent_of_another_domain_rule() {
        let source = format!(
            "{FORMAL_INCONSISTENT}\n\
            derived verb violation(subject : Person, object : Thing, recipient : Person)\n\
            rule report(giver, object, receiver, t) =\n\
                inconsistent(giver, object, receiver) @ t\n\
                <= violation(giver, object, receiver) @ after t\n"
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let rules = vec![
            rule(&module, "evidentialConflict").clone(),
            rule(&module, "report").clone(),
        ];
        let closure = close(vec![positive, negative], &rules, &mut TestClock::default());
        assert_eq!(closure.known.len(), 4);
        assert_eq!(closure.derivations.len(), 2);
        let violation = &closure.derivations[1];
        assert_eq!(violation.event.verb, "violation");
        assert_eq!(violation.event.proposition.kind, PropositionKind::Derived);
        assert!(violation.event.transition.is_none());
        assert_eq!(
            violation.record.antecedents,
            vec![closure.derivations[0].event.clone()]
        );
        assert_eq!(
            violation.event.proposition.time,
            TimeExpr::After(Box::new(TimeExpr::After(Box::new(TimeExpr::At(
                "tau".to_string()
            )))))
        );
    }

    #[test]
    fn ordinary_derived_consequents_also_support_evidential_not() {
        let source = FORMAL_INCONSISTENT.replace(
            "    inconsistent(giver, object, receiver)",
            "    ~inconsistent(giver, object, receiver)",
        );
        let module = parse_formal_to_typed(&source).unwrap();
        let positive = transfer_seed();
        let mut negative = positive.clone();
        negative.polarity = Polarity::EvidentialNot;
        let closure = close(
            vec![positive, negative],
            &[rule(&module, "evidentialConflict").clone()],
            &mut TestClock::default(),
        );
        let derived = &closure.derivations[0];
        assert_eq!(derived.event.polarity, Polarity::EvidentialNot);
        assert_eq!(derived.event.proposition.kind, PropositionKind::Derived);
        assert!(derived.event.transition.is_none());
        assert_eq!(derived.record.antecedents.len(), 2);
    }

    fn default<'a>(module: &'a Module, name: &str) -> &'a crate::typed::Default {
        module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Default(default) if default.name == name => Some(default),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{name} default exists"))
    }

    fn runtime_object(name: &str) -> Value {
        Value::Object(ObjectId(name.to_string()))
    }

    fn runtime_relation(thing: Value, conduct: Value) -> Value {
        Value::Record {
            entity: "Relation".to_string(),
            fields: [
                (FieldName("debtor".to_string()), runtime_object("alice")),
                (FieldName("thing".to_string()), thing),
                (FieldName("conduct".to_string()), conduct),
                (FieldName("creditor".to_string()), runtime_object("bob")),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn with_runtime_relation(store: Store, relation: Value) -> Store {
        store
            .with_value(
                ObjectId("alice-patrimony".to_string()),
                FieldName("obligations".to_string()),
                Value::Set([relation.clone()].into_iter().collect()),
            )
            .with_value(
                ObjectId("bob-patrimony".to_string()),
                FieldName("rights".to_string()),
                Value::Set([relation].into_iter().collect()),
            )
    }

    fn legal_snapshot() -> StateSnapshot {
        let mut store = Store::new();
        for (person, patrimony, assets) in [
            (
                "alice",
                "alice-patrimony",
                vec![ObjectId("car".to_string())],
            ),
            ("bob", "bob-patrimony", vec![]),
        ] {
            store = store
                .with_value(
                    ObjectId(person.to_string()),
                    FieldName("patrimony".to_string()),
                    runtime_object(patrimony),
                )
                .with_set(
                    ObjectId(patrimony.to_string()),
                    FieldName("assets".to_string()),
                    assets,
                )
                .with_set(
                    ObjectId(patrimony.to_string()),
                    FieldName("rights".to_string()),
                    vec![],
                )
                .with_set(
                    ObjectId(patrimony.to_string()),
                    FieldName("obligations".to_string()),
                    vec![],
                );
        }
        StateSnapshot {
            time: StateTime::At(TimeExpr::At("tau".to_string())),
            store,
        }
    }

    fn verb<'a>(module: &'a Module, name: &str) -> &'a crate::typed::Verb {
        module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Verb(verb) if verb.name == name => Some(verb),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{name} verb exists"))
    }

    fn effect_after_snapshot(mut event: Event, input: &StateSnapshot) -> Event {
        let StateTime::At(time) = &input.time else {
            panic!("resolved snapshot time");
        };
        let output = TimeExpr::After(Box::new(time.clone()));
        event.proposition.time = output.clone();
        event.transition = Some(StateTransitionType {
            input: input.time.clone(),
            output: StateTime::At(output),
        });
        event
    }

    fn horn_clause(rule: &Rule) -> &HornClause {
        match &rule.body {
            RuleBody::HornClause(clause) => clause,
            RuleBody::ResidualClause(_) => panic!("expected Horn clause"),
        }
    }

    fn rule<'a>(module: &'a Module, name: &str) -> &'a Rule {
        module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Rule(rule) if rule.name == name => Some(rule),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{name} rule exists"))
    }

    fn seeded_event<const N: usize>(verb: &str, args: [&str; N], time: &str) -> Event {
        Event {
            verb: verb.to_string(),
            polarity: Polarity::Positive,
            args: args
                .into_iter()
                .map(|arg| Term::Var(arg.to_string()))
                .collect(),
            proposition: PropositionType {
                kind: PropositionKind::Seeded,
                time: TimeExpr::At(time.to_string()),
            },
            transition: None,
        }
    }

    fn give_effect_event(debtor: &str, object: &str, creditor: &str, time: &str) -> Event {
        Event {
            verb: "give".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var(debtor.to_string()),
                Term::Var(object.to_string()),
                Term::Var(creditor.to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::After(Box::new(TimeExpr::At(time.to_string()))),
            },
            transition: Some(StateTransitionType {
                input: StateTime::At(TimeExpr::At(time.to_string())),
                output: StateTime::At(TimeExpr::After(Box::new(TimeExpr::At(time.to_string())))),
            }),
        }
    }

    fn do_effect_event(
        debtor: &str,
        conduct: &str,
        object: &str,
        creditor: &str,
        time: TimeExpr,
    ) -> Event {
        Event {
            verb: "do".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var(debtor.to_string()),
                Term::Const(conduct.to_string()),
                Term::Var(object.to_string()),
                Term::Var(creditor.to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Effect,
                time: TimeExpr::After(Box::new(time.clone())),
            },
            transition: Some(StateTransitionType {
                input: StateTime::At(time.clone()),
                output: StateTime::At(TimeExpr::After(Box::new(time))),
            }),
        }
    }

    fn unit_antecedent(clause: &HornClause) -> &Event {
        match &clause.antecedent {
            Antecedent::Unit => panic!("expected event antecedent, got monoidal unit"),
            Antecedent::Event(event) => event,
            Antecedent::And(_, _) => panic!("expected unit antecedent"),
        }
    }

    fn entity<'a>(module: &'a Module, name: &str) -> &'a crate::typed::Entity {
        module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Entity(entity) if entity.name == name => Some(entity),
                _ => None,
            })
            .expect("entity exists")
    }

    fn transfer_seed() -> Event {
        Event {
            verb: "delivers".to_string(),
            polarity: Polarity::Positive,
            args: vec![
                Term::Var("alice".to_string()),
                Term::Var("asset_a".to_string()),
                Term::Var("bob".to_string()),
            ],
            proposition: PropositionType {
                kind: PropositionKind::Seeded,
                time: TimeExpr::At("tau".to_string()),
            },
            transition: None,
        }
    }

    #[derive(Default)]
    struct TestClock {
        next: usize,
    }

    impl DerivationClock for TestClock {
        fn next_derivation_time(&mut self) -> DerivationTime {
            self.next += 1;
            DerivationTime {
                name: format!("d{}", self.next),
            }
        }
    }
}
