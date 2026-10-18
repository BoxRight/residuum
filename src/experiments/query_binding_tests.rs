use super::*;
use crate::typed::*;
use crate::{
    HornQueryScope, QueryBinding, QueryParameterScope,
    query_active_program_residual_in_scope_with_bindings,
    query_active_program_residual_with_bindings, query_program_residual_with_bindings,
};
use std::fmt::Debug;
use std::path::Path;

fn full_eq<T: Debug>(a: &T, b: &T) {
    assert_eq!(format!("{a:#?}"), format!("{b:#?}"));
}
fn load(file: &str) -> Module {
    crate::load_formal_module(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).unwrap()
}
fn experiment(m: &Module) -> &Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) => Some(e),
            _ => None,
        })
        .unwrap()
}

#[test]
fn source_compliance_scopes_match_direct_binding_apis_and_original_provenance() {
    let m = load("examples/experiments/legal_geometry/explicit_query_bindings.res");
    let original = m.clone();
    let results = run_experiments(&m).unwrap();
    assert_eq!(results.len(), 2);
    for result in &results {
        let e = &result.experiment;
        let program = e.name == "boundProgram";
        let identities = if program {
            vec!["ComplianceContext"]
        } else {
            vec!["obligationCompliance", "rightCompliance"]
        };
        let bindings: Vec<_> = identities
            .iter()
            .flat_map(|name| {
                [
                    (
                        "conduct",
                        Type::Entity("Conduct".into()),
                        SubstitutionValue::Term(Term::Const("gConduct".into())),
                    ),
                    (
                        "actAuthority",
                        Type::Entity("Authority".into()),
                        SubstitutionValue::Term(Term::Const("private".into())),
                    ),
                    (
                        "actEffective",
                        Type::PropositionTime,
                        SubstitutionValue::Time(TimeExpr::At("gDate".into())),
                    ),
                ]
                .into_iter()
                .map(move |(parameter, ty, value)| QueryBinding {
                    scope: if program {
                        QueryParameterScope::Program(ProgramRef {
                            name: (*name).into(),
                        })
                    } else {
                        QueryParameterScope::Rule(RuleRef {
                            name: (*name).into(),
                        })
                    },
                    parameter: parameter.into(),
                    ty,
                    value,
                })
            })
            .collect();
        full_eq(&e.query_options.bindings, &bindings);
        let scope = if program {
            HornQueryScope::Program(ProgramRef {
                name: "ComplianceContext".into(),
            })
        } else {
            HornQueryScope::Rules(
                ["obligationCompliance", "rightCompliance", "filedObligation"]
                    .map(|name| RuleRef { name: name.into() })
                    .to_vec(),
            )
        };
        let expected = query_program_residual_with_bindings(
            &m,
            &scope,
            &e.input,
            &e.goal,
            &bindings,
            SearchLimits::default(),
            None,
        )
        .unwrap()
        .query;
        full_eq(result.residual.as_ref().unwrap(), &expected);
        assert!(result.closure.known.is_empty() && !result.horn_goal);
        assert_eq!(expected.explanations.len(), 2);
        for branch in &expected.explanations {
            let crate::ResidualProof::Rule { rule, .. } = &branch.proof else {
                panic!("producer")
            };
            let origin = if program {
                crate::programs::program_rule_origin(&m, &rule.name)
                    .unwrap()
                    .rule_origin
            } else {
                m.origin(&rule.name).unwrap().clone()
            };
            assert_eq!(origin.module, "LegalGeneral");
            assert!(origin.file.is_some());
        }
    }
    full_eq(&m, &original);
}

#[test]
fn source_active_scopes_preserve_guards_and_original_closure_multiplicity() {
    let m = load("examples/experiments/active_query_bindings.res");
    let original = m.clone();
    let results = run_experiments(&m).unwrap();
    assert_eq!(results.len(), 3);
    for result in &results {
        let e = &result.experiment;
        let rule_scope = e.name == "satisfied";
        let scope = if rule_scope {
            HornQueryScope::Rules(vec![RuleRef {
                name: "finish".into(),
            }])
        } else {
            HornQueryScope::Program(ProgramRef { name: "P".into() })
        };
        let bindings = [QueryBinding {
            scope: if rule_scope {
                QueryParameterScope::Rule(RuleRef {
                    name: "finish".into(),
                })
            } else {
                QueryParameterScope::Program(ProgramRef { name: "P".into() })
            },
            parameter: "authority".into(),
            ty: Type::Entity("Authority".into()),
            value: SubstitutionValue::Term(Term::Const("private".into())),
        }];
        let expected = query_active_program_residual_in_scope_with_bindings(
            &m,
            &scope,
            &e.input,
            &e.goal,
            &bindings,
            SearchLimits::default(),
            None,
            &mut Clock::default(),
        )
        .unwrap()
        .query;
        let actual = result.active_residual.as_ref().unwrap();
        full_eq(actual, &expected);
        assert_eq!(actual.explanations.len(), 2);
        for branch in &actual.explanations {
            assert_eq!(branch.activations.len(), 1);
            let guard = &branch.activations[0].trigger;
            assert_eq!(guard.verb, "starts");
            assert_eq!(guard.args[1], Term::Const("private".into()));
            assert_eq!(
                branch.activations[0]
                    .program_origin
                    .as_ref()
                    .unwrap()
                    .program,
                "P"
            );
            let count = if e.name == "otherActivation" { 4 } else { 2 };
            assert_eq!(branch.hypothetical.derivations.len(), count);
            assert!(
                branch
                    .hypothetical
                    .derivations
                    .iter()
                    .all(|d| d.record.rule.starts_with("P::") && d.record.antecedents.len() == 1)
            );
            if rule_scope {
                assert_eq!(branch.candidate.required, Antecedent::Unit);
            } else {
                full_eq(
                    &branch.candidate.required,
                    &Antecedent::Event(guard.clone()),
                );
            }
        }
        // Query bindings and private hypotheses never enter the observed Known.
        if e.name == "pending" {
            assert!(result.closure.known.is_empty());
        }
        if e.name == "otherActivation" {
            assert!(
                result
                    .closure
                    .known
                    .iter()
                    .filter(|event| event.verb == "starts")
                    .all(|event| event.args[1] == Term::Const("public".into()))
            );
        }
        full_eq(
            &result.closure,
            &crate::close_program(e.input.clone(), &m, &mut Clock::default()).unwrap(),
        );
    }
    full_eq(&m, &original);
}

const SIMPLE: &str = "module SourceBindings\nentity Thing\nentity Authority\nconst item : Thing\nconst other : Thing\nconst private : Authority\nconst tau : PropositionTime\nseeded verb source(object : Thing, authority : Authority, effective : PropositionTime)\nderived verb target(object : Thing)\nrule r(x, authority, effective, t) = source(x, authority, effective) @ t <= target(x) @ after t\nrule s(x, authority, effective, t) = source(x, authority, effective) @ t <= target(x) @ after t\nprogram P(object : Thing, authority : Authority, effective : PropositionTime, t : PropositionTime) { use r { x = object, authority = authority, effective = effective, t = t } }\nprogram Partial = P(item)\n";
fn source(clauses: &str) -> String {
    format!(
        "{SIMPLE}experiment e {{ seeds {{}} query residual target(item) @ after tau {clauses} }}"
    )
}

#[test]
fn source_bindings_reject_bad_scopes_values_duplicates_and_conflicts() {
    for (clauses, message) in [
        ("with {}", "explicit search scope"),
        (
            "with { rule r.authority = private }",
            "explicit search scope",
        ),
        ("using rules { missing }", "unknown query rule"),
        ("using program Missing", "unknown query program"),
        ("using rules { r, r }", "duplicate query scope rule"),
        (
            "using rules { r } with { rule r.missing = private }",
            "unknown query parameter",
        ),
        (
            "using rules { r } with { rule missing.authority = private }",
            "unknown query rule",
        ),
        (
            "using rules { r } with { rule r.authority = private, rule r.authority = private }",
            "duplicate query binding",
        ),
        (
            "using rules { r } with { rule r.authority = missing }",
            "unknown effect argument or constant",
        ),
        (
            "using rules { r } with { rule r.authority = item }",
            "has type",
        ),
        (
            "using rules { r } with { rule s.authority = private }",
            "outside the selected query scope",
        ),
        (
            "using rules { r } with { program P.authority = private }",
            "outside the selected query scope",
        ),
        (
            "using rules { r } with { rule r.x = other }",
            "inconsistent with the goal",
        ),
        (
            "using program P with { program P.object = other }",
            "inconsistent with the goal",
        ),
        (
            "using program Partial with { program Partial.object = item }",
            "unknown query parameter",
        ),
    ] {
        let error = crate::parse_formal_to_typed(&source(clauses))
            .unwrap_err()
            .to_string();
        assert!(error.contains(message), "{clauses}: {error}");
    }
    for query in [
        "active fourFusion target(item) @ after tau",
        "fourFusion target(item) @ after tau using rules { r }",
    ] {
        let text = format!("{SIMPLE}experiment e {{ premises {{}} query {query} }}");
        assert!(
            crate::parse_formal_to_typed(&text)
                .unwrap_err()
                .to_string()
                .contains("fourFusion does not accept")
        );
    }
}

#[test]
fn partial_source_bindings_keep_unresolved_parameters_and_do_not_filter_rules() {
    for scope in ["using rules { r }", "using rules { r, s }"] {
        let m = crate::parse_formal_to_typed(&source(&format!(
            "{scope} with {{ rule r.authority = private }}"
        )))
        .unwrap();
        let e = experiment(&m);
        let r = m
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Rule(r) if r.name == "r" => Some(r),
                _ => None,
            })
            .unwrap();
        let local =
            crate::query_residual_with_bindings(&m, r, &e.goal, &[], &e.query_options.bindings)
                .unwrap();
        assert_eq!(
            local.query[0].parameters,
            vec![Arg {
                name: "effective".into(),
                ty: Type::PropositionTime
            }]
        );
        let error = run_experiments(&m).unwrap_err().to_string();
        assert!(error.contains("effective") && error.contains("PropositionTime"));
    }
    let m = crate::parse_formal_to_typed(&source(
        "using rules { r, s } with { rule r.authority = private, rule r.effective = tau }",
    ))
    .unwrap();
    let error = run_experiments(&m).unwrap_err().to_string();
    assert!(error.contains("rule `s`") && error.contains("unresolved query parameters"));
    // A source-defined single-Rule query needs no Program declarations at all.
    let header = SIMPLE.split("\nprogram P").next().unwrap();
    let m=crate::parse_formal_to_typed(&format!("{header}\nexperiment e {{ seeds {{}} query residual target(item) @ after tau using rules {{ r }} with {{ rule r.authority = private, rule r.effective = after(tau) }} }}")).unwrap();
    assert!(
        !m.declarations
            .iter()
            .any(|d| matches!(d, Declaration::Program(_)))
    );
    assert_eq!(
        run_experiments(&m).unwrap()[0]
            .residual
            .as_ref()
            .unwrap()
            .explanations
            .len(),
        1
    );
}

#[test]
fn source_values_reuse_temporal_record_none_and_named_subtype_elaboration() {
    let header = "module SourceValueTypes\nentity Thing\nentity Special <: Thing\nentity TimeBox { first : PropositionTime second : PropositionTime }\nconst x : Thing\nconst child : Special\nconst tau : PropositionTime\nseeded verb source(object : Thing, effective : PropositionTime, box : TimeBox, maybe : Thing?)\nderived verb target(object : Thing)\nrule r(x, effective, box, maybe, t) = source(x, effective, box, maybe) @ t <= target(x) @ after t\n";
    let query = "using rules { r } with { rule r.x = child, rule r.effective = after(after(tau)), rule r.box = TimeBox { second = after(tau), first = tau }, rule r.maybe = none }";
    let m = crate::parse_formal_to_typed(&format!(
        "{header}experiment e {{ seeds {{}} query residual target(child) @ after tau {query} }}"
    ))
    .unwrap();
    let e = experiment(&m);
    let b = &e.query_options.bindings;
    assert_eq!(b[0].ty, Type::Entity("Special".into()));
    assert_eq!(
        b[0].value,
        SubstitutionValue::Term(Term::Const("child".into()))
    );
    assert_eq!(
        b[1].value,
        SubstitutionValue::Time(TimeExpr::After(Box::new(TimeExpr::After(Box::new(
            TimeExpr::At("tau".into())
        )))))
    );
    let SubstitutionValue::Term(Term::RecordLiteral { fields, .. }) = &b[2].value else {
        panic!("record")
    };
    assert_eq!(
        fields.iter().map(|f| f.field.as_str()).collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    assert_eq!(
        b[3].ty,
        Type::Optional(Box::new(Type::Entity("Thing".into())))
    );
    assert_eq!(b[3].value, SubstitutionValue::Term(Term::None));
    let results = run_experiments(&m).unwrap();
    assert_eq!(results[0].residual.as_ref().unwrap().explanations.len(), 1);
    assert!(results[0].closure.known.is_empty());
    // A homonymous parameter does not capture the module constant on the RHS.
    let m=crate::parse_formal_to_typed(&format!("{header}experiment e {{ seeds {{}} query residual target(x) @ after tau using rules {{ r }} with {{ rule r.x = x }} }}")).unwrap();
    assert_eq!(
        experiment(&m).query_options.bindings[0].value,
        SubstitutionValue::Term(Term::Const("x".into()))
    );
    // A resolved partial Program keeps its fixed item and remaining shared names.
    let m=crate::parse_formal_to_typed(&source("using program Partial with { program Partial.authority = private, rule r.effective = after(tau) }")).unwrap();
    let query = run_experiments(&m).unwrap().remove(0).residual.unwrap();
    assert_eq!(query.explanations.len(), 1);
    let crate::ResidualProof::Rule {
        rule, substitution, ..
    } = &query.explanations[0].proof
    else {
        panic!("producer")
    };
    assert_eq!(rule.name, "Partial::0::r");
    assert!(
        !substitution
            .bindings
            .iter()
            .any(|b| b.parameter == "#Partial:shared:object")
    );
    assert!(
        substitution
            .bindings
            .iter()
            .any(|b| b.parameter == "#Partial:shared:effective"
                && b.value
                    == SubstitutionValue::Time(TimeExpr::After(Box::new(TimeExpr::At(
                        "tau".into()
                    )))))
    );
}

#[test]
fn unscoped_active_and_empty_scoped_queries_delegate_without_changing_legacy_modes() {
    let active = "module SourceActive\nentity Thing\nconst item : Thing\nconst tau : PropositionTime\nseeded verb starts(object : Thing) = program P\nderived verb done(object : Thing)\nrule finish(object, t) = I <= done(object) @ after t\nprogram P(object : Thing, t : PropositionTime) { use finish { object = object, t = t } }\nexperiment e { seeds {} query active residual done(item) @ after tau }";
    let m = crate::parse_formal_to_typed(active).unwrap();
    let e = experiment(&m);
    let result = run_experiments(&m).unwrap().remove(0);
    full_eq(
        result.active_residual.as_ref().unwrap(),
        &query_active_program_residual_with_bindings(
            &m,
            &e.input,
            &e.goal,
            &[],
            SearchLimits::default(),
            None,
            &mut Clock::default(),
        )
        .unwrap()
        .query,
    );
    let legacy =
        crate::parse_formal_to_typed(&active.replace("query active residual", "query residual"))
            .unwrap();
    assert!(
        run_experiments(&legacy)
            .unwrap_err()
            .to_string()
            .contains("Program currently supports forward execution")
    );
    let m = load("examples/experiments/residual_proof.res");
    let mut scoped = m.clone();
    for d in &mut scoped.declarations {
        if let Declaration::Experiment(e) = d {
            e.query_options.scope = Some(HornQueryScope::Rules(vec![RuleRef {
                name: "sales".into(),
            }]));
        }
    }
    let expected = run_experiments(&m).unwrap();
    let actual = run_experiments(&scoped).unwrap();
    for (a, b) in actual.iter().zip(&expected) {
        full_eq(&a.residual, &b.residual);
        full_eq(&a.closure, &b.closure);
        assert_eq!(a.horn_goal, b.horn_goal);
    }
}

#[test]
fn active_scope_selects_guarded_producers_but_certifies_the_original_module() {
    let source = "module ScopedActive\nentity Thing\nconst item : Thing\nconst tau : PropositionTime\nseeded verb starts(object : Thing) = program P\nseeded verb requests(object : Thing) = program Q\nderived verb done(object : Thing)\nrule p(object, t) = I <= done(object) @ after t\nrule q(object, t) = I <= done(object) @ after t\nprogram P(object : Thing, t : PropositionTime) { use p { object = object, t = t } }\nprogram Q(object : Thing, t : PropositionTime) { use q { object = object, t = t } }\n";
    let m=crate::parse_formal_to_typed(&format!("{source}experiment e {{ seeds {{ requests(item) @ tau }} query active residual done(item) @ after tau using program P }}")).unwrap();
    let result = run_experiments(&m).unwrap().remove(0);
    assert!(result.horn_goal);
    let query = result.active_residual.unwrap();
    assert_eq!(query.explanations.len(), 1);
    let branch = &query.explanations[0];
    assert_eq!(branch.activations[0].rule.name, "P::0::p");
    assert!(matches!(&branch.candidate.required,Antecedent::Event(e) if e.verb=="starts"));
    assert!(
        branch
            .hypothetical
            .derivations
            .iter()
            .any(|d| d.record.rule == "P::0::p")
    );
    assert!(
        branch
            .hypothetical
            .derivations
            .iter()
            .any(|d| d.record.rule == "Q::0::q")
    );
    let empty=crate::parse_formal_to_typed(&format!("{source}experiment e {{ seeds {{}} query active residual done(item) @ after tau using rules {{}} }}")).unwrap();
    assert!(
        run_experiments(&empty).unwrap()[0]
            .active_residual
            .as_ref()
            .unwrap()
            .required
            .is_none()
    );
}
