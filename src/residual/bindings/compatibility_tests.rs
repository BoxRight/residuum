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

type Fixed = BTreeMap<String, BTreeMap<String, SubstitutionValue>>;

fn ordered_atoms(a: &Antecedent, out: &mut Vec<Event>) {
    match a {
        Antecedent::Unit => {}
        Antecedent::Event(e) => out.push(e.clone()),
        Antecedent::And(a, b) => {
            ordered_atoms(a, out);
            ordered_atoms(b, out);
        }
    }
}

fn fixed_compliance() -> Fixed {
    let values = BTreeMap::from([
        (
            "conduct".into(),
            SubstitutionValue::Term(Term::Const("gConduct".into())),
        ),
        (
            "actAuthority".into(),
            SubstitutionValue::Term(Term::Const("private".into())),
        ),
        (
            "actEffective".into(),
            SubstitutionValue::Time(TimeExpr::At("gDate".into())),
        ),
    ]);
    BTreeMap::from([
        ("obligationCompliance".into(), values.clone()),
        ("rightCompliance".into(), values),
    ])
}

fn supplied(m: &Module, fixed: &Fixed) -> Vec<QueryBinding> {
    fixed
        .iter()
        .flat_map(|(name, values)| {
            values.iter().map(move |(parameter, value)| QueryBinding {
                scope: QueryParameterScope::Rule(RuleRef { name: name.clone() }),
                parameter: parameter.clone(),
                ty: find_rule(m, name)
                    .unwrap()
                    .parameters
                    .iter()
                    .find(|p| p.name == *parameter)
                    .unwrap()
                    .ty
                    .clone(),
                value: value.clone(),
            })
        })
        .collect()
}

// Independent oracle: beta-reduce manually specified typed values, then invoke
// legacy APIs. It deliberately does not call resolve or constrained_goal.
fn specialize_rules(m: &Module, fixed: &Fixed) -> Module {
    let mut result = m.clone();
    for d in &mut result.declarations {
        if let Declaration::Rule(r) = d {
            if let Some(values) = fixed.get(&r.name) {
                r.body = crate::elab::substitute_rule_body(&r.body, values);
                r.parameters.retain(|p| !values.contains_key(&p.name));
            }
        }
    }
    result
}
fn restore_substitution(s: &mut Substitution, values: &BTreeMap<String, SubstitutionValue>) {
    let mut restored = values.clone();
    for b in &s.bindings {
        assert!(
            restored
                .insert(b.parameter.clone(), b.value.clone())
                .is_none(),
            "oracle parameter was not eliminated"
        );
    }
    *s = crate::derive::substitution_record(&restored);
}
fn restore_proof(p: &mut super::super::ResidualProof, fixed: &Fixed) {
    use super::super::ResidualProof;
    match p {
        ResidualProof::Rule {
            rule,
            substitution,
            antecedent,
            ..
        } => {
            if let Some(values) = fixed.get(&rule.name) {
                restore_substitution(substitution, values);
            }
            restore_proof(antecedent, fixed);
        }
        ResidualProof::Tensor(a, b) => {
            restore_proof(a, fixed);
            restore_proof(b, fixed);
        }
        _ => {}
    }
}

// Test-only decoding back to source Rules. The Program alias and use index are
// asserted before projecting them; event metadata and branch multiplicity stay
// intact. The independent mapping is not used by the bindings implementation.
fn source_proof(p: &mut super::super::ResidualProof, m: &Module, identity: &str) {
    use super::super::ResidualProof;
    match p {
        ResidualProof::Rule {
            rule,
            substitution,
            antecedent,
            ..
        } => {
            let origin = program_rule_origin(m, &rule.name).unwrap();
            assert_eq!(origin.program, identity);
            let program = find_program(m, identity).unwrap();
            let member = &program.rule_uses[origin.rule_use];
            assert_eq!(member.rule.name, origin.rule);
            let original = find_rule(m, &origin.rule).unwrap();
            let values: BTreeMap<_, _> = substitution
                .bindings
                .iter()
                .map(|b| (b.parameter.clone(), b.value.clone()))
                .collect();
            let shared: BTreeMap<_, _> = program
                .parameters
                .iter()
                .map(|p| {
                    (
                        p.name.clone(),
                        SubstitutionValue::Term(Term::Var(format!(
                            "#{identity}:shared:{}",
                            p.name
                        ))),
                    )
                })
                .collect();
            let source_values = original
                .parameters
                .iter()
                .map(|p| {
                    let term = member
                        .bindings
                        .iter()
                        .find(|b| b.parameter == p.name)
                        .map(|b| crate::derive::substitute_term(&b.value, &shared))
                        .unwrap_or_else(|| {
                            Term::Var(format!("#{identity}:use{}:{}", origin.rule_use, p.name))
                        });
                    let value =
                        SubstitutionValue::Term(crate::derive::substitute_term(&term, &values))
                            .canonical_for_type(&p.ty)
                            .unwrap();
                    (p.name.clone(), value)
                })
                .collect();
            *substitution = crate::derive::substitution_record(&source_values);
            rule.name = origin.rule;
            full_eq(&m.origin(&rule.name), &Some(&origin.rule_origin));
            source_proof(antecedent, m, identity);
        }
        ResidualProof::Tensor(a, b) => {
            source_proof(a, m, identity);
            source_proof(b, m, identity);
        }
        _ => {}
    }
}

#[test]
fn nonempty_compliance_matches_independent_rules_and_named_program_full_proofs() {
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let selected = view(
        &m,
        &["obligationCompliance", "rightCompliance", "filedObligation"],
    );
    let fixed = fixed_compliance();
    let bindings = supplied(&m, &fixed);
    let independent = specialize_rules(&selected, &fixed);
    let mut program_view = view(&m, &[]);
    program_view.declarations.extend(
        compile_program(find_program(&m, "ComplianceBound").unwrap(), &m)
            .unwrap()
            .into_iter()
            .map(Declaration::Rule),
    );
    let g = module("examples/experiments/legal_geometry/general.res");
    let input = &experiment(&g, "performance").input;
    let inputs = [input[0].clone(), input[1].clone(), input[5].clone()];
    let goal = &experiment(&m, "suppliedContext").goal;
    for mask in 0..8 {
        let known: Vec<_> = inputs
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect();
        let actual = query_program_residual_with_bindings(
            &m,
            &rules_scope(&selected),
            &known,
            goal,
            &bindings,
            SearchLimits::default(),
            None,
        )
        .unwrap()
        .query;
        let mut expected =
            query_program_residual(&independent, &known, goal, SearchLimits::default()).unwrap();
        for e in &mut expected.explanations {
            restore_proof(&mut e.proof, &fixed);
        }
        full_eq(&actual, &expected);
        let mut named =
            query_program_residual(&program_view, &known, goal, SearchLimits::default()).unwrap();
        for e in &mut named.explanations {
            source_proof(&mut e.proof, &m, "ComplianceBound");
        }
        full_eq(&actual, &named);
        assert_eq!(actual.explanations.len(), 2);
        // Certification uses original producers, including both compliance paths.
        for e in &actual.explanations {
            let mut hypothetical = known.clone();
            ordered_atoms(&e.required, &mut hypothetical);
            let closure = crate::close_program(hypothetical, &m, &mut Clock::default()).unwrap();
            assert!(closure.known.contains(goal));
            assert!(
                closure
                    .derivations
                    .iter()
                    .any(|d| d.record.rule == "obligationCompliance"
                        || d.record.rule == "rightCompliance")
            );
        }
    }
    full_eq(
        &m,
        &module("examples/experiments/legal_geometry/explicit_substitutions.res"),
    );
}

#[test]
fn nonempty_program_scope_matches_independent_specialization_and_partial_application() {
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let goal = &experiment(&m, "suppliedContext").goal;
    let constants = m
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Const(c) => Some((c.name.clone(), c.ty.clone())),
            _ => None,
        })
        .collect();
    for (identity, args, names) in [
        (
            "ComplianceContext",
            vec!["gConduct", "private", "gDate"],
            vec!["conduct", "actAuthority", "actEffective"],
        ),
        (
            "CompliancePartial",
            vec!["private", "gDate"],
            vec!["actAuthority", "actEffective"],
        ),
    ] {
        let program = find_program(&m, identity).unwrap();
        let specialized = crate::programs::specialize_program(
            program,
            &args.iter().map(|a| (*a).into()).collect::<Vec<_>>(),
            &constants,
        )
        .unwrap();
        let bindings: Vec<_> = names
            .iter()
            .zip(&args)
            .map(|(name, value)| {
                let p = program.parameters.iter().find(|p| p.name == *name).unwrap();
                QueryBinding {
                    scope: QueryParameterScope::Program(ProgramRef {
                        name: identity.into(),
                    }),
                    parameter: (*name).into(),
                    ty: p.ty.clone(),
                    value: SubstitutionValue::Term(Term::Const((*value).into())),
                }
            })
            .collect();
        let mut oracle = view(&m, &[]);
        let rules = compile_program(&specialized, &m).unwrap();
        let fixed: Fixed = rules
            .iter()
            .map(|r| {
                (
                    r.name.clone(),
                    bindings
                        .iter()
                        .map(|b| {
                            (
                                format!("#{identity}:shared:{}", b.parameter),
                                b.value.canonical_for_type(&b.ty).unwrap(),
                            )
                        })
                        .collect(),
                )
            })
            .collect();
        oracle
            .declarations
            .extend(rules.into_iter().map(Declaration::Rule));
        let mut expected =
            query_program_residual(&oracle, &[], goal, SearchLimits::default()).unwrap();
        for e in &mut expected.explanations {
            restore_proof(&mut e.proof, &fixed);
        }
        let actual = query_program_residual_with_bindings(
            &m,
            &HornQueryScope::Program(ProgramRef {
                name: identity.into(),
            }),
            &[],
            goal,
            &bindings,
            SearchLimits::default(),
            None,
        )
        .unwrap()
        .query;
        full_eq(&actual, &expected);
    }
}

#[test]
fn nonempty_local_partial_and_temporal_queries_match_independent_rules() {
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let goal = &experiment(&m, "suppliedContext").goal;
    let fixed = BTreeMap::from([(
        "rightCompliance".into(),
        BTreeMap::from([(
            "conduct".into(),
            SubstitutionValue::Term(Term::Const("gConduct".into())),
        )]),
    )]);
    let independent = specialize_rules(&m, &fixed);
    let rule = find_rule(&m, "rightCompliance").unwrap();
    let mut expected = query_residual(
        find_rule(&independent, "rightCompliance").unwrap(),
        goal,
        &[],
    )
    .unwrap();
    for e in &mut expected {
        restore_substitution(&mut e.goal_substitution, &fixed["rightCompliance"]);
        restore_substitution(&mut e.substitution, &fixed["rightCompliance"]);
    }
    let actual = query_residual_with_bindings(&m, rule, goal, &[], &supplied(&m, &fixed))
        .unwrap()
        .query;
    full_eq(&actual, &expected);
    assert_eq!(actual[0].parameters.len(), 2);
    assert!(
        actual[0]
            .parameters
            .iter()
            .any(|p| p.ty == Type::PropositionTime)
    );

    let m=crate::parse_formal_to_typed("module OracleTemporal\nentity Thing\nconst x : Thing\nconst tau : PropositionTime\nseeded verb source(object : Thing, effective : PropositionTime)\nseeded verb needed(object : Thing)\nderived verb target(object : Thing)\nrule r(x, effective, t) = source(x, effective) @ t & needed(x) @ t <= target(x) @ after t\nexperiment e { seeds {} query residual target(x) @ after tau }").unwrap();
    for term in [Term::Const("x".into()), Term::Var("x".into())] {
        let fixed = BTreeMap::from([(
            "r".into(),
            BTreeMap::from([
                ("x".into(), SubstitutionValue::Term(term.clone())),
                (
                    "effective".into(),
                    SubstitutionValue::Time(TimeExpr::After(Box::new(TimeExpr::At("tau".into())))),
                ),
            ]),
        )]);
        let independent = specialize_rules(&m, &fixed);
        let rule = find_rule(&m, "r").unwrap();
        let bindings = supplied(&m, &fixed);
        let mut goal = experiment(&m, "e").goal.clone();
        goal.args[0] = term;
        let initial = query_residual(find_rule(&independent, "r").unwrap(), &goal, &[]).unwrap();
        let mut atoms = Vec::new();
        ordered_atoms(&initial[0].required, &mut atoms);
        // Evidence metadata is preserved in witnesses even when not logical identity.
        atoms[0].transition = Some(StateTransitionType {
            input: StateTime::Unresolved,
            output: StateTime::At(TimeExpr::At("tau".into())),
        });
        for known in [vec![], vec![atoms[0].clone(), atoms[0].clone()]] {
            let mut expected =
                query_residual(find_rule(&independent, "r").unwrap(), &goal, &known).unwrap();
            for e in &mut expected {
                restore_substitution(&mut e.goal_substitution, &fixed["r"]);
                restore_substitution(&mut e.substitution, &fixed["r"]);
            }
            let actual = query_residual_with_bindings(&m, rule, &goal, &known, &bindings)
                .unwrap()
                .query;
            full_eq(&actual, &expected);
        }
        let unit = query_residual_with_bindings(&m, rule, &goal, &atoms, &bindings)
            .unwrap()
            .query;
        assert_eq!(unit.len(), 1);
        assert!(
            query_residual(find_rule(&independent, "r").unwrap(), &goal, &atoms)
                .unwrap()
                .is_empty()
        );
        assert_eq!(unit[0].required, Antecedent::Unit);
        full_eq(&unit[0].evidence, &atoms);
        let witness = crate::derive::instantiate_from_witness(rule, &atoms).unwrap();
        full_eq(&unit[0].substitution, &witness.record.substitution);
        full_eq(&unit[0].goal, &witness.event);
    }
}

#[test]
fn explicit_conflicts_differ_from_incompatible_specialized_heads_and_pending_errors() {
    let m = module("examples/experiments/associated_multi_rule_program.res");
    let goal = &experiment(&m, "real").goal;
    let fixed = BTreeMap::from([(
        "second".into(),
        BTreeMap::from([(
            "local".into(),
            SubstitutionValue::Term(Term::Const("other".into())),
        )]),
    )]);
    let independent = specialize_rules(&view(&m, &["second"]), &fixed);
    let bindings = supplied(&m, &fixed);
    for known in [vec![], vec![goal.clone()]] {
        assert!(
            query_residual_with_bindings(
                &m,
                find_rule(&m, "second").unwrap(),
                goal,
                &known,
                &bindings
            )
            .unwrap_err()
            .to_string()
            .contains("inconsistent with the goal")
        );
        assert!(
            query_residual(find_rule(&independent, "second").unwrap(), goal, &known)
                .unwrap()
                .is_empty()
        );
        assert!(
            query_program_residual_with_bindings(
                &m,
                &rules_scope(&independent),
                &known,
                goal,
                &bindings,
                SearchLimits::default(),
                None
            )
            .is_err()
        );
    }
    assert!(
        query_program_residual(&independent, &[], goal, SearchLimits::default())
            .unwrap()
            .required
            .is_none()
    );
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let goal = &experiment(&m, "suppliedContext").goal;
    let partial = BTreeMap::from([(
        "rightCompliance".into(),
        BTreeMap::from([(
            "conduct".into(),
            SubstitutionValue::Term(Term::Const("gConduct".into())),
        )]),
    )]);
    let independent = specialize_rules(&view(&m, &["rightCompliance"]), &partial);
    let old = query_program_residual(&independent, &[], goal, SearchLimits::default())
        .unwrap_err()
        .to_string();
    let new = query_program_residual_with_bindings(
        &m,
        &rules_scope(&independent),
        &[],
        goal,
        &supplied(&m, &partial),
        SearchLimits::default(),
        None,
    )
    .unwrap_err()
    .to_string();
    assert!(old.contains("actAuthority"));
    assert!(
        new.contains("actAuthority")
            && new.contains("Entity(\"Authority\")")
            && new.contains("actEffective")
            && new.contains("PropositionTime"),
        "{new}"
    );
    let constants = m
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Const(c) => Some((c.name.clone(), c.ty.clone())),
            _ => None,
        })
        .collect();
    assert!(
        crate::programs::specialize_program(
            find_program(&m, "ComplianceContext").unwrap(),
            &["private".into()],
            &constants
        )
        .is_err()
    );
    let invalid = QueryBinding {
        scope: QueryParameterScope::Program(ProgramRef {
            name: "ComplianceContext".into(),
        }),
        parameter: "conduct".into(),
        ty: Type::Entity("Authority".into()),
        value: SubstitutionValue::Term(Term::Const("private".into())),
    };
    assert!(
        query_program_residual_with_bindings(
            &m,
            &HornQueryScope::Program(ProgramRef {
                name: "ComplianceContext".into()
            }),
            &[],
            goal,
            &[invalid],
            SearchLimits::default(),
            None
        )
        .is_err()
    );
}

#[test]
fn active_bindings_match_independent_guarded_specialization_and_original_certification() {
    let m=crate::parse_formal_to_typed("module OracleActive\nentity Thing\nentity Authority\nconst item : Thing\nconst private : Authority\nconst public : Authority\nconst tau : PropositionTime\nseeded verb starts(object : Thing, authority : Authority) = program P\nderived verb done(object : Thing)\nrule r(object, authority, t) = I <= done(object) @ after t\nprogram P(object : Thing, authority : Authority, t : PropositionTime) { use r { object = object, authority = authority, t = t } use r { object = object, authority = authority, t = t } }\nexperiment e { seeds { starts(item, private) @ tau } query residual done(item) @ after tau }").unwrap();
    let original = m.clone();
    let e = experiment(&m, "e");
    let (v, templates) = guarded_view(&m).unwrap();
    let fixed: Fixed = templates
        .keys()
        .map(|name| {
            (
                name.clone(),
                BTreeMap::from([(
                    "#P:shared:authority".into(),
                    SubstitutionValue::Term(Term::Const("private".into())),
                )]),
            )
        })
        .collect();
    let independent = specialize_rules(&v, &fixed);
    let bindings = [QueryBinding {
        scope: QueryParameterScope::Program(ProgramRef { name: "P".into() }),
        parameter: "authority".into(),
        ty: Type::Entity("Authority".into()),
        value: SubstitutionValue::Term(Term::Const("private".into())),
    }];
    let mut wrong = e.input[0].clone();
    wrong.args[1] = Term::Const("public".into());
    for known in [vec![], e.input.clone(), vec![wrong]] {
        for allowed in [None, Some(&[][..])] {
            let mut search = match allowed {
                None => {
                    query_program_residual(&independent, &known, &e.goal, SearchLimits::default())
                }
                Some(a) => query_program_residual_with_abducibles(
                    &independent,
                    &known,
                    &e.goal,
                    a,
                    SearchLimits::default(),
                ),
            }
            .unwrap();
            for e in &mut search.explanations {
                restore_proof(&mut e.proof, &fixed);
            }
            let expected =
                certify(&m, &known, &e.goal, search, &templates, &mut Clock(100)).unwrap();
            let actual = query_active_program_residual_with_bindings(
                &m,
                &known,
                &e.goal,
                &bindings,
                SearchLimits::default(),
                allowed,
                &mut Clock(0),
            )
            .unwrap()
            .query;
            full_eq(
                &without_derived_at(actual.clone()),
                &without_derived_at(expected),
            );
            for explanation in &actual.explanations {
                assert_eq!(explanation.activations.len(), 1);
                full_eq(&explanation.activations[0].trigger, &e.input[0]);
                assert_eq!(explanation.hypothetical.derivations.len(), 2);
                for (i, d) in explanation.hypothetical.derivations.iter().enumerate() {
                    assert_eq!(d.record.rule, format!("P::{i}::r"));
                    let origin = program_rule_origin(&m, &d.record.rule).unwrap();
                    assert_eq!(origin.rule_use, i);
                    assert_eq!(origin.rule, "r");
                    full_eq(&origin.rule_origin, m.origin("r").unwrap());
                    full_eq(&d.record.antecedents, &e.input);
                    assert!(
                        d.record
                            .substitution
                            .bindings
                            .iter()
                            .any(|b| b.parameter == "#P:shared:authority"
                                && b.value == bindings[0].value)
                    );
                }
            }
        }
    }
    full_eq(&m, &original);
}
