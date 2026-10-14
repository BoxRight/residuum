use super::*;
use crate::ResidualProof;
use std::path::Path;
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("integrated-{}", self.0),
        }
    }
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
fn rb(rule: &str, name: &str, value: &str, ty: Type) -> QueryBinding {
    QueryBinding {
        scope: QueryParameterScope::Rule(RuleRef { name: rule.into() }),
        parameter: name.into(),
        ty,
        value: SubstitutionValue::Term(Term::Const(value.into())),
    }
}

#[test]
fn constrained_compliance_sessions_match_full_queries_and_preserve_supplied_provenance() {
    let m = crate::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/legal_geometry/explicit_substitutions.res"),
    )
    .unwrap();
    let original = m.clone();
    let goal = &experiment(&m, "suppliedContext").goal;
    let scope = HornQueryScope::Rules(
        ["obligationCompliance", "rightCompliance", "filedObligation"]
            .map(|name| RuleRef { name: name.into() })
            .to_vec(),
    );
    let bindings: Vec<_> = ["obligationCompliance", "rightCompliance"]
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
        .collect();
    let general = crate::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/legal_geometry/general.res"),
    )
    .unwrap();
    let inputs = &experiment(&general, "performance").input;
    let inputs = [inputs[0].clone(), inputs[1].clone(), inputs[5].clone()];
    for mask in 0..8 {
        let known: Vec<_> = inputs
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect();
        let expected = query_program_residual_with_bindings(
            &m,
            &scope,
            &known,
            goal,
            &bindings,
            SearchLimits::default(),
            None,
        )
        .unwrap();
        let mut session = begin_residual_query_with_bindings(
            &m,
            &scope,
            &known,
            goal,
            &bindings,
            SearchLimits::default(),
            None,
        )
        .unwrap();
        let first = session.query.step().unwrap();
        assert!(!first.complete);
        assert!(session.query.result().is_none());
        assert!(session.query.trace(32).unwrap().last().unwrap().complete);
        let actual = session.query.result().unwrap();
        assert_eq!(actual.required, expected.query.required);
        assert_eq!(actual.explanations, expected.query.explanations);
        assert_eq!(actual.explanations.len(), 2);
        assert_eq!(session.bindings, expected.bindings);
        for branch in &actual.explanations {
            let ResidualProof::Rule { substitution, .. } = &branch.proof else {
                panic!("rule proof")
            };
            assert!(
                substitution
                    .bindings
                    .iter()
                    .any(|b| b.parameter == "conduct"
                        && b.value == SubstitutionValue::Term(Term::Const("gConduct".into())))
            );
        }
        assert_eq!(session.query.evidence(), known);
    }
    assert_eq!(m, original);
}

#[test]
fn constrained_active_sessions_keep_hidden_program_guards_and_original_certificates() {
    let m = crate::parse_formal_to_typed("module IntegratedActive\nentity Thing\nentity Authority\nconst item : Thing\nconst private : Authority\nconst tau : PropositionTime\nseeded verb starts(object : Thing, authority : Authority) = program P\nderived verb done(object : Thing)\nrule r(object, authority, t) = I <= done(object) @ after t\nprogram P(object : Thing, authority : Authority, t : PropositionTime) { use r { object = object, authority = authority, t = t } use r { object = object, authority = authority, t = t } }\nexperiment e { seeds { starts(item, private) @ tau } query residual done(item) @ after tau }").unwrap();
    let original = m.clone();
    let e = experiment(&m, "e");
    let binding = QueryBinding {
        scope: QueryParameterScope::Program(ProgramRef { name: "P".into() }),
        parameter: "authority".into(),
        ty: Type::Entity("Authority".into()),
        value: SubstitutionValue::Term(Term::Const("private".into())),
    };
    let expected = query_active_program_residual_with_bindings(
        &m,
        &[],
        &e.goal,
        &[binding.clone()],
        SearchLimits::default(),
        None,
        &mut Clock::default(),
    )
    .unwrap();
    let mut session = begin_active_residual_query_with_bindings(
        &m,
        &[],
        &e.goal,
        &[binding.clone()],
        SearchLimits::default(),
        None,
    )
    .unwrap();
    assert!(!session.query.step(&mut Clock::default()).unwrap().complete);
    assert!(session.query.result().is_none());
    assert!(
        session
            .query
            .trace(32, &mut Clock::default())
            .unwrap()
            .last()
            .unwrap()
            .complete
    );
    let actual = session.query.result().unwrap();
    assert_eq!(actual.required, expected.query.required);
    assert_eq!(actual.explanations, expected.query.explanations);
    assert_eq!(actual.explanations.len(), 2);
    assert!(
        actual
            .explanations
            .iter()
            .all(|a| a.activations[0].trigger == e.input[0]
                && a.hypothetical.known.contains(&e.goal))
    );
    let mut no_hypotheses = begin_active_residual_query_with_bindings(
        &m,
        &[],
        &e.goal,
        &[binding],
        SearchLimits::default(),
        Some(&[]),
    )
    .unwrap();
    no_hypotheses
        .query
        .trace(32, &mut Clock::default())
        .unwrap();
    assert!(no_hypotheses.query.result().unwrap().required.is_none());
    assert_eq!(m, original);
}

#[test]
fn constrained_session_errors_remain_explicit_and_cannot_be_resumed_as_success() {
    let m = crate::parse_formal_to_typed("module IntegratedErrors\nentity Thing\nconst a : Thing\nconst b : Thing\nconst tau : PropositionTime\nseeded verb source(object : Thing)\nderived verb target()\nrule r(x) = source(x) @ tau <= target() @ tau\nexperiment e { seeds {} query residual target() @ tau }").unwrap();
    let e = experiment(&m, "e");
    let scope = HornQueryScope::Rules(vec![RuleRef { name: "r".into() }]);
    let mut pending = begin_residual_query_with_bindings(
        &m,
        &scope,
        &[],
        &e.goal,
        &[],
        SearchLimits::default(),
        None,
    )
    .unwrap();
    let error = pending.query.step().unwrap_err().to_string();
    assert!(error.contains("x") && error.contains("Thing"));
    assert!(pending.query.result().is_none());
    assert!(pending.query.step().is_err());
    let unknown = rb("r", "unknown", "a", Type::Entity("Thing".into()));
    assert!(
        begin_residual_query_with_bindings(
            &m,
            &scope,
            &[],
            &e.goal,
            &[unknown],
            SearchLimits::default(),
            None
        )
        .is_err()
    );
    let m = crate::parse_formal_to_typed("module IntegratedHead\nentity Thing\nconst a : Thing\nconst b : Thing\nconst tau : PropositionTime\nseeded verb source(object : Thing)\nderived verb target(object : Thing)\nrule r(x) = source(x) @ tau <= target(x) @ tau\nexperiment e { seeds {} query residual target(a) @ tau }").unwrap();
    let e = experiment(&m, "e");
    assert!(
        begin_residual_query_with_bindings(
            &m,
            &scope,
            &[e.goal.clone()],
            &e.goal,
            &[rb("r", "x", "b", Type::Entity("Thing".into()))],
            SearchLimits::default(),
            None
        )
        .unwrap_err()
        .to_string()
        .contains("inconsistent with the goal")
    );
}
