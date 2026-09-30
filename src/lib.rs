pub mod cnl;
pub mod derive;
pub mod elab;
pub mod formal;
pub mod runtime;
pub mod surface;
pub mod typed;

pub use cnl::parse_cnl;
pub use derive::{DerivationClock, close, derive_once, instantiate, instantiate_from_known, stamp};
pub use elab::{Error, elaborate_cnl, elaborate_formal};
pub use formal::parse_formal;
pub use runtime::execute_effect;
pub use typed::Module;

pub fn parse_formal_to_typed(source: &str) -> Result<Module, Error> {
    elaborate_formal(parse_formal(source)?)
}

pub fn parse_cnl_to_typed(source: &str) -> Result<Module, Error> {
    elaborate_cnl(parse_cnl(source)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{FieldName, ObjectId, StateSnapshot, Store};
    use crate::typed::{
        Antecedent, Arg, Constant, Declaration, DefaultBody, DerivationRecord, DerivationTime,
        Effect, Event, FieldAccess, HornClause, PropositionKind, PropositionType, RecordFieldValue,
        Rule, RuleBody, RuleRef, StateOperation, StateTime, StateTransitionType, Substitution,
        SubstitutionBinding, SubstitutionValue, Term, TimeExpr, Type,
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

    #[test]
    fn chumsky_formal_parser_matches_manual_surface_ast_for_existing_fixtures() {
        let fixtures = [
            ("transfer/formal.res", FORMAL),
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
    fn legal_conduct_and_relation_records_elaborate_with_optional_fields() {
        let legal = parse_formal_to_typed(LEGAL_FORMAL).expect("formal legal elaborates");
        let conduct = entity(&legal, "Conduct");
        let relation = entity(&legal, "Relation");

        assert_eq!(
            conduct.fields,
            vec![
                crate::typed::Field {
                    name: "conduct".to_string(),
                    ty: Type::Entity("Thing".to_string()),
                },
                crate::typed::Field {
                    name: "indirectObject".to_string(),
                    ty: Type::Optional(Box::new(Type::Entity("Thing".to_string()))),
                },
            ]
        );
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
        let RuleBody::HornClause(clause) = &mut rule.body;
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
    fn legal_give_has_two_relation_insertions() {
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

        let relation = Term::RecordLiteral {
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
        };

        assert_eq!(
            operations[0],
            StateOperation::AddToSet {
                target: FieldAccess {
                    base: Term::Var("subject".to_string()),
                    path: vec!["patrimony".to_string(), "obligations".to_string()],
                },
                value: relation.clone(),
            }
        );
        assert_eq!(
            operations[1],
            StateOperation::AddToSet {
                target: FieldAccess {
                    base: Term::Var("recipient".to_string()),
                    path: vec!["patrimony".to_string(), "rights".to_string()],
                },
                value: relation,
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
            Some(&[ObjectId("asset_a".to_string())].into_iter().collect())
        );
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

    fn horn_clause(rule: &Rule) -> &HornClause {
        match &rule.body {
            RuleBody::HornClause(clause) => clause,
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
