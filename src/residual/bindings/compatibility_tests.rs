//! Compare full structures, not Event::PartialEq (which omits metadata).
use super::*;
use crate::{
    query_active_program_residual, query_active_program_residual_with_abducibles,
    query_program_residual, query_program_residual_with_abducibles, query_residual,
};
use std::fmt::Debug;
use std::path::Path;

fn full_eq<T: Debug>(actual: &T, expected: &T) {
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"));
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
fn rules_scope(m: &Module) -> HornQueryScope {
    HornQueryScope::Rules(
        m.declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) => Some(RuleRef {
                    name: r.name.clone(),
                }),
                _ => None,
            })
            .collect(),
    )
}
fn view(m: &Module, names: &[&str]) -> Module {
    let mut v = m.clone();
    v.declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    v.declarations.extend(
        names
            .iter()
            .map(|n| Declaration::Rule(find_rule(m, n).unwrap().clone())),
    );
    v
}
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("compatibility-{}", self.0),
        }
    }
}

#[test]
fn empty_local_bindings_preserve_satisfied_legacy_result() {
    let m = module("examples/experiments/residual_proof.res");
    let e = experiment(&m, "both");
    let r = find_rule(&m, "sales").unwrap();
    let expected = query_residual(r, &e.goal, &e.input).unwrap();
    assert!(expected.is_empty());
    let actual = query_residual_with_bindings(&m, r, &e.goal, &e.input, &[]).unwrap();
    full_eq(&actual.query, &expected);
    assert!(actual.bindings.is_empty());
}

#[test]
fn empty_horn_bindings_preserve_unresolved_legacy_error() {
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let goal = &experiment(&m, "suppliedContext").goal;
    let scoped = view(
        &m,
        &["obligationCompliance", "rightCompliance", "filedObligation"],
    );
    let expected = query_program_residual(&scoped, &[], goal, SearchLimits::default()).unwrap_err();
    let actual = query_program_residual_with_bindings(
        &m,
        &rules_scope(&scoped),
        &[],
        goal,
        &[],
        SearchLimits::default(),
        None,
    )
    .unwrap_err();
    full_eq(&actual, &expected);
}

#[test]
fn empty_horn_bindings_preserve_default_errors_and_limit_precedence() {
    let m = module("examples/legal/case_sale_defaults_formal.res");
    let goal = &experiment(&m, "noRetention").goal;
    for limits in [
        SearchLimits::default(),
        SearchLimits {
            max_nodes: 0,
            ..SearchLimits::default()
        },
    ] {
        let expected = query_program_residual(&m, &[], goal, limits).unwrap_err();
        let actual = query_program_residual_with_bindings(
            &m,
            &rules_scope(&m),
            &[],
            goal,
            &[],
            limits,
            None,
        )
        .unwrap_err();
        full_eq(&actual, &expected);
    }
}

fn hidden_active() -> Module {
    crate::parse_formal_to_typed("module CompatibilityActive\nentity Thing\nentity Authority\nconst item : Thing\nconst private : Authority\nconst tau : PropositionTime\nseeded verb starts(object : Thing, authority : Authority) = program P\nderived verb done(object : Thing)\nrule r(object, authority, t) = I <= done(object) @ after t\nprogram P(object : Thing, authority : Authority, t : PropositionTime) { use r { object = object, authority = authority, t = t } }\nexperiment e { seeds { starts(item, private) @ tau } query residual done(item) @ after tau }").unwrap()
}

#[test]
fn empty_active_bindings_preserve_unresolved_legacy_error() {
    let m = hidden_active();
    let goal = &experiment(&m, "e").goal;
    let expected = query_active_program_residual(
        &m,
        &[],
        goal,
        SearchLimits::default(),
        &mut Clock::default(),
    )
    .unwrap_err();
    let actual = query_active_program_residual_with_bindings(
        &m,
        &[],
        goal,
        &[],
        SearchLimits::default(),
        None,
        &mut Clock::default(),
    )
    .unwrap_err();
    full_eq(&actual, &expected);
}

#[test]
fn empty_local_bindings_preserve_metadata_pending_multiplicity_and_mismatch() {
    let m = module("examples/experiments/residual_proof.res");
    let e = experiment(&m, "priceOnly");
    let r = find_rule(&m, "sales").unwrap();
    let mut known = e.input.clone();
    known[0].transition = e.goal.transition.clone();
    known.push(known[0].clone());
    let expected = query_residual(r, &e.goal, &known).unwrap();
    assert_eq!(expected.len(), 2);
    assert!(expected[0].evidence[0].transition.is_some());
    full_eq(
        &query_residual_with_bindings(&m, r, &e.goal, &known, &[])
            .unwrap()
            .query,
        &expected,
    );
    let mut goal = e.goal.clone();
    goal.proposition.kind = PropositionKind::Seeded;
    full_eq(
        &query_residual_with_bindings(&m, r, &goal, &known, &[])
            .unwrap()
            .query,
        &query_residual(r, &goal, &known).unwrap(),
    );
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let r = find_rule(&m, "rightCompliance").unwrap();
    let goal = &experiment(&m, "suppliedContext").goal;
    let expected = query_residual(r, goal, &[]).unwrap();
    assert!(!expected[0].parameters.is_empty());
    full_eq(
        &query_residual_with_bindings(&m, r, goal, &[], &[])
            .unwrap()
            .query,
        &expected,
    );
}

#[test]
fn empty_horn_bindings_preserve_complete_results_policy_and_errors() {
    let m = module("examples/experiments/residual_proof.res");
    let scoped = rules_scope(&m);
    for name in ["empty", "priceOnly", "both"] {
        let e = experiment(&m, name);
        for allowed in [None, Some(&[][..])] {
            let expected = match allowed {
                None => query_program_residual(&m, &e.input, &e.goal, SearchLimits::default()),
                Some(a) => query_program_residual_with_abducibles(
                    &m,
                    &e.input,
                    &e.goal,
                    a,
                    SearchLimits::default(),
                ),
            };
            let actual = query_program_residual_with_bindings(
                &m,
                &scoped,
                &e.input,
                &e.goal,
                &[],
                SearchLimits::default(),
                allowed,
            )
            .map(|a| a.query);
            full_eq(&actual, &expected);
        }
    }
    let e = experiment(&m, "empty");
    for limits in [
        SearchLimits {
            max_nodes: 0,
            ..SearchLimits::default()
        },
        SearchLimits {
            max_nodes: 1,
            ..SearchLimits::default()
        },
    ] {
        full_eq(
            &query_program_residual_with_bindings(&m, &scoped, &[], &e.goal, &[], limits, None)
                .map(|a| a.query),
            &query_program_residual(&m, &[], &e.goal, limits),
        );
    }
    let mut goal = e.goal.clone();
    goal.verb = "unknown".into();
    full_eq(
        &query_program_residual_with_bindings(
            &m,
            &scoped,
            &[],
            &goal,
            &[],
            SearchLimits::default(),
            None,
        )
        .map(|a| a.query),
        &query_program_residual(&m, &[], &goal, SearchLimits::default()),
    );
    // The Program API's legacy counterpart is the explicitly compiled Horn view.
    let m = module("examples/experiments/associated_multi_rule_program.res");
    let goal = &experiment(&m, "real").goal;
    let scope = HornQueryScope::Program(ProgramRef {
        name: "Ground".into(),
    });
    let mut v = view(&m, &[]);
    v.declarations.extend(
        compile_program(find_program(&m, "Ground").unwrap(), &m)
            .unwrap()
            .into_iter()
            .map(Declaration::Rule),
    );
    full_eq(
        &query_program_residual_with_bindings(
            &m,
            &scope,
            &[],
            goal,
            &[],
            SearchLimits::default(),
            None,
        )
        .unwrap()
        .query,
        &query_program_residual(&v, &[], goal, SearchLimits::default()).unwrap(),
    );
}

fn without_derived_at(mut q: ActiveProgramResidualQuery) -> ActiveProgramResidualQuery {
    for e in &mut q.explanations {
        for d in &mut e.hypothetical.derivations {
            d.derived_at.name = "independent clock".into();
        }
    }
    q
}

#[test]
fn empty_active_bindings_preserve_all_guards_provenance_and_policies() {
    let m = module("examples/experiments/associated_multi_rule_program.res");
    for name in ["real", "derived", "absent", "negative"] {
        let e = experiment(&m, name);
        for allowed in [None, Some(&[][..])] {
            let expected = match allowed {
                None => query_active_program_residual(
                    &m,
                    &e.input,
                    &e.goal,
                    SearchLimits::default(),
                    &mut Clock(100),
                ),
                Some(a) => query_active_program_residual_with_abducibles(
                    &m,
                    &e.input,
                    &e.goal,
                    a,
                    SearchLimits::default(),
                    &mut Clock(100),
                ),
            }
            .unwrap();
            let actual = query_active_program_residual_with_bindings(
                &m,
                &e.input,
                &e.goal,
                &[],
                SearchLimits::default(),
                allowed,
                &mut Clock(0),
            )
            .unwrap();
            full_eq(
                &without_derived_at(actual.query),
                &without_derived_at(expected),
            );
        }
    }
    let goal = &experiment(&m, "real").goal;
    for limits in [
        SearchLimits {
            max_nodes: 0,
            ..SearchLimits::default()
        },
        SearchLimits {
            max_nodes: 1,
            ..SearchLimits::default()
        },
    ] {
        full_eq(
            &query_active_program_residual_with_bindings(
                &m,
                &[],
                goal,
                &[],
                limits,
                None,
                &mut Clock::default(),
            )
            .map(|a| a.query),
            &query_active_program_residual(&m, &[], goal, limits, &mut Clock::default()),
        );
    }
}
