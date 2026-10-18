use super::*;
use crate::{ResidualProof, query_active_program_residual, query_program_residual, query_residual};
use std::path::Path;

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("bindings-{}", self.0),
        }
    }
}
fn module(file: &str) -> Module {
    crate::load_formal_module(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).unwrap()
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
fn rb(rule: &str, parameter: &str, constant: &str, ty: Type) -> QueryBinding {
    QueryBinding {
        scope: QueryParameterScope::Rule(RuleRef { name: rule.into() }),
        parameter: parameter.into(),
        ty,
        value: SubstitutionValue::Term(Term::Const(constant.into())),
    }
}
fn scope(names: &[&str]) -> HornQueryScope {
    HornQueryScope::Rules(
        names
            .iter()
            .map(|n| RuleRef { name: (*n).into() })
            .collect(),
    )
}
fn atoms(a: &Antecedent) -> BTreeSet<Event> {
    match a {
        Antecedent::Unit => BTreeSet::new(),
        Antecedent::Event(e) => BTreeSet::from([e.clone()]),
        Antecedent::And(a, b) => atoms(a).union(&atoms(b)).cloned().collect(),
    }
}
fn compliance_bindings() -> Vec<QueryBinding> {
    ["obligationCompliance", "rightCompliance"]
        .into_iter()
        .flat_map(|r| {
            [
                rb(r, "conduct", "gConduct", Type::Entity("Conduct".into())),
                rb(
                    r,
                    "actAuthority",
                    "private",
                    Type::Entity("Authority".into()),
                ),
                rb(r, "actEffective", "gDate", Type::PropositionTime),
            ]
        })
        .collect()
}

#[test]
fn compliance_constraints_match_verified_program_experiment_without_wrapper() {
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let original = m.clone();
    let goal = &experiment(&m, "suppliedContext").goal;
    let selected = scope(&["obligationCompliance", "rightCompliance", "filedObligation"]);
    let bindings = compliance_bindings();
    let mut oracle = m.clone();
    oracle
        .declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    oracle.declarations.extend(
        compile_program(find_program(&m, "ComplianceBound").unwrap(), &m)
            .unwrap()
            .into_iter()
            .map(Declaration::Rule),
    );
    let g = module("examples/experiments/legal_geometry/general.res");
    let evidence = &experiment(&g, "performance").input;
    let inputs = [
        evidence[0].clone(),
        evidence[1].clone(),
        evidence[5].clone(),
    ];
    for mask in 0..8 {
        let known: Vec<_> = inputs
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect();
        let before = known.clone();
        let answer = query_program_residual_with_bindings(
            &m,
            &selected,
            &known,
            goal,
            &bindings,
            SearchLimits::default(),
            None,
        )
        .unwrap();
        let expected =
            query_program_residual(&oracle, &known, goal, SearchLimits::default()).unwrap();
        let requirements = |q: &ProgramResidualQuery| {
            q.explanations
                .iter()
                .map(|a| atoms(&a.required))
                .collect::<BTreeSet<_>>()
        };
        assert_eq!(requirements(&answer.query), requirements(&expected));
        assert_eq!(answer.query.explanations.len(), 2);
        let mut producers = BTreeSet::new();
        for branch in &answer.query.explanations {
            let ResidualProof::Rule {
                rule, substitution, ..
            } = &branch.proof
            else {
                panic!("rule proof")
            };
            producers.insert(rule.name.clone());
            assert_eq!(m.origin(&rule.name).unwrap().module, "LegalGeneral");
            for binding in bindings
                .iter()
                .filter(|b| matches!(&b.scope, QueryParameterScope::Rule(r) if r.name == rule.name))
            {
                let value = binding.value.canonical_for_type(&binding.ty).unwrap();
                assert!(
                    substitution
                        .bindings
                        .iter()
                        .any(|b| b.parameter == binding.parameter && b.value == value)
                );
            }
            let mut hypothetical = known.clone();
            hypothetical.extend(atoms(&branch.required));
            assert!(
                crate::close_program(hypothetical, &m, &mut Clock::default())
                    .unwrap()
                    .known
                    .contains(goal)
            );
        }
        assert_eq!(
            producers,
            BTreeSet::from(["obligationCompliance".into(), "rightCompliance".into()])
        );
        assert_eq!(answer.bindings.len(), 6);
        assert_eq!(known, before);
    }
    assert_eq!(m, original);
    assert!(query_program_residual(&m, &[], goal, SearchLimits::default()).is_err());
}

#[test]
fn single_rule_partial_bindings_keep_typed_unresolved_parameters_and_unit() {
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let goal = &experiment(&m, "suppliedContext").goal;
    let r = find_rule(&m, "rightCompliance").unwrap();
    let conduct = [rb(
        "rightCompliance",
        "conduct",
        "gConduct",
        Type::Entity("Conduct".into()),
    )];
    let pending = query_residual_with_bindings(&m, r, goal, &[], &conduct).unwrap();
    assert_eq!(pending.query.len(), 1);
    assert_eq!(
        pending.query[0]
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["actAuthority", "actEffective"])
    );
    assert_eq!(
        pending.query[0]
            .parameters
            .iter()
            .find(|p| p.name == "actEffective")
            .unwrap()
            .ty,
        Type::PropositionTime
    );
    let err = query_program_residual_with_bindings(
        &m,
        &scope(&["rightCompliance"]),
        &[],
        goal,
        &conduct,
        SearchLimits::default(),
        None,
    )
    .unwrap_err()
    .to_string();
    assert!(
        err.contains("unresolved query parameters")
            && err.contains("actAuthority")
            && err.contains("PropositionTime")
    );
    let all: Vec<_> = compliance_bindings()
        .into_iter()
        .filter(|b| matches!(&b.scope, QueryParameterScope::Rule(r) if r.name == "rightCompliance"))
        .collect();
    let q = query_residual_with_bindings(&m, r, goal, &[], &all).unwrap();
    let known: Vec<_> = atoms(&q.query[0].required).into_iter().collect();
    let unit = query_residual_with_bindings(&m, r, goal, &known, &all).unwrap();
    assert_eq!(unit.query[0].required, Antecedent::Unit);
    assert!(unit.query[0].parameters.is_empty());
    assert!(query_residual(r, goal, &known).unwrap().is_empty());
    let mut wrong = goal.clone();
    wrong.verb = "Act".into();
    assert!(
        query_residual_with_bindings(&m, r, &wrong, &[], &all)
            .unwrap()
            .query
            .is_empty()
    );
}

#[test]
fn program_scope_and_staged_specialization_preserve_hygiene_and_provenance() {
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let goal = &experiment(&m, "suppliedContext").goal;
    let shared: Vec<_> = compliance_bindings()
        .into_iter()
        .take(3)
        .map(|b| QueryBinding {
            scope: QueryParameterScope::Program(ProgramRef {
                name: "ComplianceContext".into(),
            }),
            ..b
        })
        .collect();
    let q = query_program_residual_with_bindings(
        &m,
        &HornQueryScope::Program(ProgramRef {
            name: "ComplianceContext".into(),
        }),
        &[],
        goal,
        &shared,
        SearchLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(q.query.explanations.len(), 2);
    for e in &q.query.explanations {
        let ResidualProof::Rule {
            rule, substitution, ..
        } = &e.proof
        else {
            panic!("producer")
        };
        assert_eq!(
            program_rule_origin(&m, &rule.name).unwrap().program,
            "ComplianceContext"
        );
        assert!(
            substitution
                .bindings
                .iter()
                .any(|b| b.parameter == "#ComplianceContext:shared:conduct"
                    && b.value == SubstitutionValue::Term(Term::Const("gConduct".into())))
        );
    }
    let partial: Vec<_> = shared
        .into_iter()
        .skip(1)
        .map(|b| QueryBinding {
            scope: QueryParameterScope::Program(ProgramRef {
                name: "CompliancePartial".into(),
            }),
            ..b
        })
        .collect();
    let staged = query_program_residual_with_bindings(
        &m,
        &HornQueryScope::Program(ProgramRef {
            name: "CompliancePartial".into(),
        }),
        &[],
        goal,
        &partial,
        SearchLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(staged.query.required, q.query.required);
    let compiled_rules =
        compile_program(find_program(&m, "CompliancePartial").unwrap(), &m).unwrap();
    let mut bound_program = find_program(&m, "ComplianceBound").unwrap().clone();
    // Compare beta-reduced bodies under the same hygienic Program identity.
    bound_program.identity = "CompliancePartial".into();
    let resolved = compile_program(&bound_program, &m).unwrap();
    let (maps, _) = resolve(&m, &compiled_rules, &BTreeMap::new(), &partial).unwrap();
    for (a, b) in compiled_rules.iter().zip(resolved.iter()) {
        // Existing typed beta-reduction yields the same constrained body.
        assert_eq!(
            crate::elab::substitute_rule_body(&a.body, &maps[&a.name]),
            b.body
        );
    }
}

#[test]
fn bindings_reject_unknown_types_scope_conflicts_and_goal_inconsistency() {
    let m = module("examples/experiments/associated_multi_rule_program.res");
    let goal = &experiment(&m, "real").goal;
    let valid = rb("second", "local", "item", Type::Entity("Thing".into()));
    for invalid in [
        QueryBinding {
            parameter: "missing".into(),
            ..valid.clone()
        },
        QueryBinding {
            ty: Type::PropositionTime,
            ..valid.clone()
        },
        QueryBinding {
            value: SubstitutionValue::Term(Term::Const("missing".into())),
            ..valid.clone()
        },
        QueryBinding {
            scope: QueryParameterScope::Rule(RuleRef {
                name: "first".into(),
            }),
            ..valid.clone()
        },
        rb("second", "local", "other", Type::Entity("Thing".into())),
    ] {
        assert!(
            query_program_residual_with_bindings(
                &m,
                &scope(&["second"]),
                &[goal.clone()],
                goal,
                &[invalid],
                SearchLimits::default(),
                None
            )
            .is_err()
        );
    }
    assert!(
        query_program_residual_with_bindings(
            &m,
            &scope(&["second"]),
            &[],
            goal,
            &[valid.clone(), valid],
            SearchLimits::default(),
            None
        )
        .unwrap_err()
        .to_string()
        .contains("duplicate")
    );
    let fixed = QueryBinding {
        scope: QueryParameterScope::Rule(RuleRef {
            name: "first".into(),
        }),
        parameter: "local".into(),
        ty: Type::Entity("Thing".into()),
        value: SubstitutionValue::Term(Term::Const("other".into())),
    };
    assert!(
        query_program_residual_with_bindings(
            &m,
            &HornQueryScope::Program(ProgramRef {
                name: "Ground".into()
            }),
            &[],
            goal,
            &[fixed],
            SearchLimits::default(),
            None
        )
        .unwrap_err()
        .to_string()
        .contains("specialized RuleUse")
    );
}

#[test]
fn temporal_constructors_and_homonymous_runtime_values_remain_distinct() {
    let m = crate::parse_formal_to_typed("module BindingTemporal\nentity Thing\nconst x : Thing\nconst tau : PropositionTime\nseeded verb source(object : Thing, effective : PropositionTime)\nderived verb target(object : Thing)\nrule r(x, effective, t) = source(x, effective) @ t <= target(x) @ after t\nexperiment e { seeds {} query residual target(x) @ after tau }").unwrap();
    let r = find_rule(&m, "r").unwrap();
    let mut goal = experiment(&m, "e").goal.clone();
    let effective = QueryBinding {
        scope: QueryParameterScope::Rule(RuleRef { name: "r".into() }),
        parameter: "effective".into(),
        ty: Type::PropositionTime,
        value: SubstitutionValue::Time(TimeExpr::After(Box::new(TimeExpr::At("tau".into())))),
    };
    let known_before = Vec::<Event>::new();
    for term in [Term::Const("x".into()), Term::Var("x".into())] {
        goal.args[0] = term.clone();
        let binding = QueryBinding {
            scope: QueryParameterScope::Rule(RuleRef { name: "r".into() }),
            parameter: "x".into(),
            ty: Type::Entity("Thing".into()),
            value: SubstitutionValue::Term(term.clone()),
        };
        let q = query_residual_with_bindings(
            &m,
            r,
            &goal,
            &known_before,
            &[binding, effective.clone()],
        )
        .unwrap();
        let event = atoms(&q.query[0].required).into_iter().next().unwrap();
        assert_eq!(event.args[0], term);
        assert_eq!(
            event.args[1],
            Term::Constructor {
                constructor: Constructor::After,
                argument: Box::new(Term::Const("tau".into()))
            }
        );
        assert_eq!(event.proposition.time, TimeExpr::At("tau".into()));
        let unit =
            query_residual_with_bindings(&m, r, &goal, &[event], &[effective.clone()]).unwrap();
        assert_eq!(unit.query[0].required, Antecedent::Unit);
    }
    assert!(known_before.is_empty());
    assert!(
        query_residual_with_bindings(
            &m,
            r,
            &goal,
            &[],
            &[QueryBinding {
                value: SubstitutionValue::Time(TimeExpr::Var("tau".into())),
                ..effective
            }]
        )
        .is_err()
    );
}

#[test]
fn active_bound_queries_preserve_guards_and_original_hypothetical_producers() {
    let m = module("examples/experiments/associated_multi_rule_program.res");
    let original = m.clone();
    let e = experiment(&m, "real");
    let binding = QueryBinding {
        scope: QueryParameterScope::Program(ProgramRef {
            name: "Chain".into(),
        }),
        parameter: "object".into(),
        ty: Type::Entity("Thing".into()),
        value: SubstitutionValue::Term(Term::Const("item".into())),
    };
    let bound = query_active_program_residual_with_bindings(
        &m,
        &[],
        &e.goal,
        &[binding.clone()],
        SearchLimits::default(),
        None,
        &mut Clock::default(),
    )
    .unwrap();
    let expected = query_active_program_residual(
        &m,
        &[],
        &e.goal,
        SearchLimits::default(),
        &mut Clock::default(),
    )
    .unwrap();
    assert_eq!(bound.query, expected);
    assert_eq!(bound.query.explanations.len(), 4);
    for a in &bound.query.explanations {
        assert!(a.hypothetical.known.contains(&e.goal));
        assert!(!a.activations.is_empty());
        assert!(a.activations.iter().all(|a| {
            a.substitution
                .bindings
                .iter()
                .any(|b| b.parameter == "#Chain:shared:object" && b.value == binding.value)
        }));
        assert!(
            a.hypothetical
                .derivations
                .iter()
                .all(|d| !d.record.rule.starts_with("@residual:"))
        );
    }
    let unit = query_active_program_residual_with_bindings(
        &m,
        &e.input,
        &e.goal,
        &[binding.clone()],
        SearchLimits::default(),
        None,
        &mut Clock::default(),
    )
    .unwrap();
    assert!(
        unit.query
            .explanations
            .iter()
            .any(|a| a.candidate.required == Antecedent::Unit)
    );
    let mut negative = e.input[0].clone();
    negative.polarity = Polarity::EvidentialNot;
    let none = query_active_program_residual_with_bindings(
        &m,
        &[negative],
        &e.goal,
        &[binding.clone()],
        SearchLimits::default(),
        Some(&[]),
        &mut Clock::default(),
    )
    .unwrap();
    assert!(none.query.required.is_none());
    assert_eq!(
        none.query.failure,
        Some(super::super::AbductionFailure::NotAbducible)
    );
    assert!(
        query_active_program_residual_with_bindings(
            &m,
            &[],
            &e.goal,
            &[binding],
            SearchLimits {
                max_nodes: 1,
                ..SearchLimits::default()
            },
            None,
            &mut Clock::default()
        )
        .is_err()
    );
    assert_eq!(m, original);
}

#[test]
fn ordinary_rule_names_do_not_share_constraints_and_empty_bindings_preserve_api() {
    let m = crate::parse_formal_to_typed("module ScopedBindings\nentity Thing\nconst a : Thing\nconst b : Thing\nconst tau : PropositionTime\nseeded verb source(object : Thing)\nderived verb target()\nrule r(x) = source(x) @ tau <= target() @ tau\nrule s(x) = source(x) @ tau <= target() @ tau\nexperiment e { seeds {} query residual target() @ tau }").unwrap();
    let goal = &experiment(&m, "e").goal;
    let binding = rb("r", "x", "a", Type::Entity("Thing".into()));
    let error = query_program_residual_with_bindings(
        &m,
        &scope(&["r", "s"]),
        &[],
        goal,
        &[binding],
        SearchLimits::default(),
        None,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("rule `s`") && error.contains("unresolved"));
    let bindings = [
        rb("r", "x", "a", Type::Entity("Thing".into())),
        rb("s", "x", "a", Type::Entity("Thing".into())),
    ];
    let answer = query_program_residual_with_bindings(
        &m,
        &scope(&["r", "s"]),
        &[],
        goal,
        &bindings,
        SearchLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(answer.query.explanations.len(), 2);
    assert!(!matches!(
        answer.query.required,
        Some(super::super::RequirementFormula::Join(_, _))
    ));
    let m = module("examples/experiments/associated_program_waits.res");
    let goal = &experiment(&m, "later").goal;
    let q = query_program_residual_with_bindings(
        &m,
        &scope(&["onStart", "prepare"]),
        &[],
        goal,
        &[],
        SearchLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(
        q.query,
        query_program_residual(&m, &[], goal, SearchLimits::default()).unwrap()
    );
}

#[test]
fn active_hidden_shared_parameter_constrains_the_guard_without_inventing_activation() {
    let m = crate::parse_formal_to_typed("module HiddenActive\nentity Thing\nentity Authority\nconst item : Thing\nconst private : Authority\nconst public : Authority\nconst tau : PropositionTime\nseeded verb starts(object : Thing, authority : Authority) = program P\nderived verb done(object : Thing)\nrule r(object, authority, t) = I <= done(object) @ after t\nprogram P(object : Thing, authority : Authority, t : PropositionTime) { use r { object = object, authority = authority, t = t } use r { object = object, authority = authority, t = t } }\nexperiment e { seeds { starts(item, private) @ tau } query residual done(item) @ after tau }").unwrap();
    let original = m.clone();
    let e = experiment(&m, "e");
    assert!(
        query_active_program_residual(
            &m,
            &[],
            &e.goal,
            SearchLimits::default(),
            &mut Clock::default()
        )
        .is_err()
    );
    let binding = QueryBinding {
        scope: QueryParameterScope::Program(ProgramRef { name: "P".into() }),
        parameter: "authority".into(),
        ty: Type::Entity("Authority".into()),
        value: SubstitutionValue::Term(Term::Const("private".into())),
    };
    let pending = query_active_program_residual_with_bindings(
        &m,
        &[],
        &e.goal,
        &[binding.clone()],
        SearchLimits::default(),
        None,
        &mut Clock::default(),
    )
    .unwrap();
    assert_eq!(pending.query.explanations.len(), 2);
    for branch in &pending.query.explanations {
        assert_eq!(
            atoms(&branch.candidate.required),
            e.input.iter().cloned().collect()
        );
        assert_eq!(branch.activations.len(), 1);
        assert_eq!(branch.activations[0].trigger, e.input[0]);
        assert!(branch.hypothetical.known.contains(&e.goal));
        assert_eq!(branch.hypothetical.derivations.len(), 2);
    }
    let proven = query_active_program_residual_with_bindings(
        &m,
        &e.input,
        &e.goal,
        &[binding.clone()],
        SearchLimits::default(),
        None,
        &mut Clock::default(),
    )
    .unwrap();
    assert!(
        proven
            .query
            .explanations
            .iter()
            .all(|a| a.candidate.required == Antecedent::Unit)
    );
    let mut wrong = e.input[0].clone();
    wrong.args[1] = Term::Const("public".into());
    let none = query_active_program_residual_with_bindings(
        &m,
        &[wrong],
        &e.goal,
        &[binding],
        SearchLimits::default(),
        Some(&[]),
        &mut Clock::default(),
    )
    .unwrap();
    assert!(none.query.explanations.is_empty());
    assert_eq!(m, original);
}

#[test]
fn structured_bindings_use_declared_field_types_and_preserve_nested_time_values() {
    let m = module("examples/experiments/associated_multi_rule_program.res");
    let r = find_rule(&m, "seal").unwrap();
    let record = Term::RecordLiteral {
        entity: "Envelope".into(),
        fields: vec![RecordFieldValue {
            field: "payload".into(),
            value: Term::RecordLiteral {
                entity: "Box".into(),
                fields: vec![RecordFieldValue {
                    field: "value".into(),
                    value: Term::Const("item".into()),
                }],
            },
        }],
    };
    let binding = QueryBinding {
        scope: QueryParameterScope::Rule(RuleRef {
            name: "seal".into(),
        }),
        parameter: "record".into(),
        ty: Type::Entity("Envelope".into()),
        value: SubstitutionValue::Term(record.clone()),
    };
    let RuleBody::HornClause(clause) = &r.body else {
        unreachable!()
    };
    let mut goal = clause.consequent.clone();
    goal.args[0] = record;
    let q = query_residual_with_bindings(&m, r, &goal, &[], &[binding.clone()]).unwrap();
    assert_eq!(q.query[0].required, Antecedent::Unit);
    let invalid = QueryBinding {
        value: SubstitutionValue::Term(Term::RecordLiteral {
            entity: "Envelope".into(),
            fields: vec![RecordFieldValue {
                field: "payload".into(),
                value: Term::Const("item".into()),
            }],
        }),
        ..binding
    };
    assert!(query_residual_with_bindings(&m, r, &goal, &[], &[invalid]).is_err());
}
