//! Stage-one specialization through existing Program APIs; no new query semantics.
use super::*;
use residuum::programs::{compile_program, program_rule_origin, specialize_program};
use residuum::{ResidualProof, SearchLimits, query_program_residual, query_residual};

fn program<'a>(m: &'a Module, name: &str) -> &'a Program {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Program(p) if p.identity == name => Some(p),
            _ => None,
        })
        .unwrap()
}

fn constants(m: &Module) -> BTreeMap<String, Type> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Const(c) => Some((c.name.clone(), c.ty.clone())),
            _ => None,
        })
        .collect()
}

// A visible harness adapter: the query API accepts Module, not ProgramRef.
// Only the caller-selected RuleUses participate. Original rules remain unchanged.
fn query_scope(m: &Module, name: &str) -> Module {
    let rules = compile_program(program(m, name), m).unwrap();
    let mut scope = m.clone();
    scope
        .declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    for r in rules {
        let origin = program_rule_origin(m, &r.name).unwrap();
        scope.origins.insert(r.name.clone(), origin.rule_origin);
        scope.declarations.push(Declaration::Rule(r));
    }
    scope
}

fn required(a: &Antecedent) -> BTreeSet<Event> {
    let mut v = Vec::new();
    atoms(a, &mut v);
    v.into_iter().cloned().collect()
}

fn check_origins(m: &Module, proof: &ResidualProof) {
    match proof {
        ResidualProof::Rule {
            rule, antecedent, ..
        } => {
            let origin = program_rule_origin(m, &rule.name).unwrap();
            assert_eq!(origin.rule_origin.module, "LegalGeneral");
            assert!(matches!(
                origin.rule.as_str(),
                "obligationCompliance" | "rightCompliance" | "filedObligation"
            ));
            check_origins(m, antecedent);
        }
        ResidualProof::Tensor(a, b) => {
            check_origins(m, a);
            check_origins(m, b);
        }
        _ => (),
    }
}

#[test]
fn explicit_bindings_preserve_typed_pending_parameters_and_temporal_scope() {
    let m = module("explicit_substitutions");
    let goal = &experiment(&m, "suppliedContext").goal;
    let partial = program(&m, "CompliancePartial");
    assert_eq!(
        partial
            .parameters
            .iter()
            .map(|a| (&*a.name, &a.ty))
            .collect::<Vec<_>>(),
        vec![
            ("actAuthority", &Type::Entity("Authority".into())),
            ("actEffective", &Type::PropositionTime),
            ("object", &Type::Entity("Thing".into())),
            ("t", &Type::PropositionTime),
        ]
    );
    let rules = compile_program(partial, &m).unwrap();
    let r = rules
        .iter()
        .find(|r| r.name.ends_with("::rightCompliance"))
        .unwrap();
    let pending = query_residual(r, goal, &[]).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].parameters,
        vec![
            Arg {
                name: "#CompliancePartial:shared:actAuthority".into(),
                ty: Type::Entity("Authority".into())
            },
            Arg {
                name: "#CompliancePartial:shared:actEffective".into(),
                ty: Type::PropositionTime
            },
        ]
    );
    let mut terms = Vec::new();
    atoms(&pending[0].required, &mut terms);
    assert!(
        terms
            .iter()
            .all(|e| e.args[2] == Term::Const("gConduct".into()))
    );
    assert!(
        query_program_residual(
            &query_scope(&m, "CompliancePartial"),
            &[],
            goal,
            SearchLimits::default()
        )
        .is_err()
    );

    let bound = program(&m, "ComplianceBound");
    assert_eq!(
        bound
            .parameters
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>(),
        vec!["object", "t"]
    );
    let direct = specialize_program(
        program(&m, "ComplianceContext"),
        &[
            "gConduct".into(),
            "private".into(),
            "gDate".into(),
            "gObject".into(),
            "gTau".into(),
        ],
        &constants(&m),
    )
    .unwrap();
    let ground = program(&m, "ComplianceGround");
    assert!(ground.parameters.is_empty());
    assert_eq!(direct.rule_uses, ground.rule_uses);
    assert_eq!(direct.defining_module, ground.defining_module);
    for r in compile_program(ground, &m).unwrap() {
        let RuleBody::HornClause(c) = &r.body else {
            panic!("expected Horn clause")
        };
        let mut es = Vec::new();
        atoms(&c.antecedent, &mut es);
        es.push(&c.consequent);
        assert!(
            es.iter()
                .all(|e| e.proposition.time == TimeExpr::At("gTau".into()))
        );
        assert!(
            es.iter()
                .all(|e| e.args.contains(&Term::Const("gObject".into())))
        );
        if let Some(a) = es.iter().find(|e| e.verb == "Act") {
            assert_eq!(a.args[5], Term::Const("private".into()));
            assert_eq!(a.args[6], Term::Const("gDate".into()));
        }
    }
    assert!(
        specialize_program(
            program(&m, "ComplianceContext"),
            &["private".into()],
            &constants(&m)
        )
        .is_err()
    );
    let mut other_time = goal.clone();
    other_time.proposition.time = TimeExpr::At("gOtherTau".into());
    let fixed = query_program_residual(
        &query_scope(&m, "ComplianceGround"),
        &[],
        &other_time,
        SearchLimits::default(),
    )
    .unwrap();
    assert!(fixed.explanations.is_empty());
    assert!(fixed.failure.is_some());
    let open = query_program_residual(
        &query_scope(&m, "ComplianceBound"),
        &[],
        &other_time,
        SearchLimits::default(),
    )
    .unwrap();
    assert_eq!(open.explanations.len(), 2);
    for answer in &open.explanations {
        assert!(
            required(&answer.required)
                .iter()
                .all(|e| e.proposition.time == other_time.proposition.time)
        );
    }
    save(
        "explicit-query-pending.tsv",
        &format!(
            "case\ttyped_parameters\trequirement\npartial\t{:?}\t{:?}\n",
            pending[0].parameters, pending[0]
        ),
    );
}

#[test]
fn explicitly_specialized_query_recovers_two_compliance_frontiers_in_its_declared_scope() {
    let m = module("explicit_substitutions");
    let g = module("general");
    let p = module("patrimonial");
    let goal = &experiment(&m, "suppliedContext").goal;
    // Independent frontiers computed with every original general rule, not the query scope.
    let e = universe(&p);
    let sources: BTreeSet<_> = e
        .iter()
        .enumerate()
        .filter(|(_, x)| x.proposition.kind == PropositionKind::Seeded)
        .map(|(i, _)| i)
        .collect();
    let (_, frontiers) = finite_frontiers(&g, &e, &sources);
    let q = e.iter().position(|x| x == goal).unwrap();
    assert_eq!(frontiers[q].len(), 2);
    let performance = &experiment(&g, "performance").input;
    let inputs = [
        performance[0].clone(),
        performance[1].clone(),
        performance[5].clone(),
    ];
    let mut report = String::from(
        "program\tevidence_mask\tcanonical_requirements\tminimal_support_ids\tbranches\tproofs\n",
    );
    for name in ["ComplianceBound", "ComplianceGround"] {
        let scope = query_scope(&m, name);
        assert_eq!(
            scope
                .declarations
                .iter()
                .filter(|d| matches!(d, Declaration::Rule(_)))
                .count(),
            3
        );
        for mask in 0..8 {
            let known = subset(&inputs, mask);
            let before = known.clone();
            let known_set: BTreeSet<_> = known.iter().cloned().collect();
            let query =
                query_program_residual(&scope, &known, goal, SearchLimits::default()).unwrap();
            assert!(query.failure.is_none());
            let mut actual: BTreeSet<BTreeSet<Event>> = BTreeSet::new();
            for answer in &query.explanations {
                check_origins(&m, &answer.proof);
                let h = required(&answer.required);
                let hypothetical: Vec<_> = known_set.union(&h).cloned().collect();
                for model in [&scope, &g, &p] {
                    let c = close(model, hypothetical.clone());
                    assert!(c.known.contains(goal));
                    audit_proofs(model, &c);
                }
                actual.insert(h);
            }
            let expected: BTreeSet<_> = frontiers[q]
                .iter()
                .map(|s| {
                    s.iter()
                        .map(|i| e[*i].clone())
                        .filter(|x| !known_set.contains(x))
                        .collect::<BTreeSet<_>>()
                })
                .collect();
            // The engine retains sufficient branches, including non-minimal ones.
            assert_eq!(actual, expected);
            let mut actual_minimal = Supports::new();
            for h in &actual {
                insert_minimal(
                    &mut actual_minimal,
                    h.iter()
                        .map(|x| e.iter().position(|y| y == x).unwrap())
                        .collect(),
                );
            }
            let known_ids: BTreeSet<_> = known_set
                .iter()
                .map(|x| e.iter().position(|y| y == x).unwrap())
                .collect();
            let mut expected_minimal = Supports::new();
            for s in &frontiers[q] {
                insert_minimal(
                    &mut expected_minimal,
                    s.difference(&known_ids).copied().collect(),
                );
            }
            assert_eq!(actual_minimal, expected_minimal);
            let producers: BTreeSet<_> = query
                .explanations
                .iter()
                .map(|a| {
                    let ResidualProof::Rule { rule, .. } = &a.proof else {
                        panic!("expected rule proof")
                    };
                    program_rule_origin(&m, &rule.name).unwrap().rule
                })
                .collect();
            assert_eq!(
                producers,
                BTreeSet::from(["obligationCompliance".into(), "rightCompliance".into()])
            );
            assert_eq!(known, before);
            assert_eq!(query.explanations.len(), 2);
            writeln!(
                report,
                "{name}\t{mask}\t{:?}\t{actual_minimal:?}\t{}\t{:?}",
                actual
                    .iter()
                    .map(|s| s.iter().map(render_event).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                query.explanations.len(),
                query
                    .explanations
                    .iter()
                    .map(|a| &a.proof)
                    .collect::<Vec<_>>()
            )
            .unwrap();
        }
    }
    // Specialization is not an implicit replacement for whole-module queries.
    assert!(
        query_program_residual(&m, &[], goal, SearchLimits::default())
            .unwrap_err()
            .to_string()
            .contains("not determined by the goal")
    );
    save("explicit-query-frontiers.tsv", &report);
    println!(
        "16 scoped queries preserve two proof alternatives and certify their hypotheses against both unchanged ontologies"
    );
}
