use super::*;
use std::path::Path;
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("search-{}", self.0),
        }
    }
}
fn module(file: &str) -> Module {
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
fn equal(a: &ProgramResidualQuery, b: &ProgramResidualQuery) {
    assert_eq!(a.required, b.required);
    assert_eq!(a.explanations, b.explanations);
    assert_eq!(a.failure, b.failure);
}

#[test]
fn pending_layers_are_explicit_and_completed_search_matches_existing_branches() {
    let m = module("examples/experiments/residual_proof.res");
    let e = experiment(&m);
    let original = m.clone();
    let mut s = ResidualSession::new(&m, &e.input, &e.goal, SearchLimits::default(), None).unwrap();
    let first = s.step().unwrap();
    assert!(!first.complete);
    assert_eq!(first.pending.len(), 2);
    assert!(s.result().is_none());
    let second = s.step().unwrap();
    assert!(second.complete);
    assert!(second.pending.is_empty());
    equal(
        s.result().unwrap(),
        &query_program_residual(&m, &e.input, &e.goal, SearchLimits::default()).unwrap(),
    );
    assert_eq!(s.evidence(), e.input);
    assert_eq!(m, original);
}

#[test]
fn joins_cartesian_products_and_proof_multiplicity_match_existing_search() {
    let m = module("examples/residual/alternatives_formal.res");
    let e = Event {
        verb: "result".into(),
        args: vec![],
        polarity: Polarity::Positive,
        proposition: PropositionType {
            kind: PropositionKind::Derived,
            time: TimeExpr::At("tau".into()),
        },
        transition: None,
    };
    // Resolve the actual source goal's verb without introducing a new fixture.
    let mut goal = m
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r),
            _ => None,
        })
        .last()
        .map(|r| match &r.body {
            RuleBody::HornClause(c) => c.consequent.clone(),
            RuleBody::ResidualClause(c) => c.consequent.consequent.clone(),
        })
        .unwrap_or(e);
    goal.proposition.time = TimeExpr::At("tau".into());
    let mut s = ResidualSession::new(&m, &[], &goal, SearchLimits::default(), None).unwrap();
    assert!(s.trace(32).unwrap().last().unwrap().complete);
    equal(
        s.result().unwrap(),
        &query_program_residual(&m, &[], &goal, SearchLimits::default()).unwrap(),
    );
    let m = crate::parse_formal_to_typed("module Multiplicity\nconst tau : PropositionTime\nseeded verb A()\nderived verb B()\nrule first() = A() @ tau <= B() @ tau\nrule second() = A() @ tau <= B() @ tau\nexperiment e { seeds {} query residual B() @ tau }").unwrap();
    let e = experiment(&m);
    let mut s = ResidualSession::new(&m, &[], &e.goal, SearchLimits::default(), None).unwrap();
    s.trace(10).unwrap();
    assert_eq!(s.result().unwrap().explanations.len(), 2);
    assert!(!matches!(
        s.result().unwrap().required,
        Some(RequirementFormula::Join(_, _))
    ));
}

#[test]
fn unit_pending_absence_and_budget_errors_are_distinct_and_errors_are_terminal() {
    let m = module("examples/experiments/residual_proof.res");
    let e = experiment(&m);
    let mut none =
        ResidualSession::new(&m, &[], &e.goal, SearchLimits::default(), Some(&[])).unwrap();
    none.trace(10).unwrap();
    assert!(none.result().unwrap().required.is_none());
    let mut unit = ResidualSession::new(
        &m,
        &[e.goal.clone()],
        &e.goal,
        SearchLimits::default(),
        None,
    )
    .unwrap();
    assert!(unit.step().unwrap().complete);
    assert_eq!(
        unit.result().unwrap().required,
        Some(RequirementFormula::Unit)
    );
    let mut limited = ResidualSession::new(
        &m,
        &[],
        &e.goal,
        SearchLimits {
            max_nodes: 1,
            ..SearchLimits::default()
        },
        None,
    )
    .unwrap();
    assert!(
        limited
            .step()
            .unwrap_err()
            .to_string()
            .contains("limit exceeded")
    );
    assert!(limited.result().is_none());
    assert!(limited.step().is_err());
    let mut prefix = ResidualSession::new(&m, &[], &e.goal, SearchLimits::default(), None).unwrap();
    assert!(!prefix.trace(1).unwrap()[0].complete);
    assert!(prefix.result().is_none());
}

#[test]
fn cyclic_and_unresolved_parameters_error_without_partial_explanations() {
    let m = crate::parse_formal_to_typed("module CyclicSearch\nconst tau : PropositionTime\nderived verb A()\nrule cycle() = A() @ tau <= A() @ tau\nexperiment e { seeds {} query residual A() @ tau }").unwrap();
    let e = experiment(&m);
    let mut s = ResidualSession::new(&m, &[], &e.goal, SearchLimits::default(), None).unwrap();
    assert!(!s.step().unwrap().complete);
    assert!(s.step().unwrap_err().to_string().contains("cyclic"));
    assert!(s.result().is_none());
    let m = module("examples/experiments/legal_geometry/explicit_substitutions.res");
    let e = experiment(&m);
    let mut s = ResidualSession::new(&m, &[], &e.goal, SearchLimits::default(), None).unwrap();
    assert!(
        s.step()
            .unwrap_err()
            .to_string()
            .contains("not determined by the goal")
    );
}

#[test]
fn active_layered_search_preserves_guards_and_certifies_original_module() {
    let m = module("examples/experiments/associated_multi_rule_program.res");
    let e = experiment(&m);
    let original = m.clone();
    let mut s =
        ActiveResidualSession::new(&m, &[], &e.goal, SearchLimits::default(), None).unwrap();
    let first = s.step(&mut Clock::default()).unwrap();
    assert!(!first.complete);
    assert!(s.result().is_none());
    assert!(
        s.trace(32, &mut Clock::default())
            .unwrap()
            .last()
            .unwrap()
            .complete
    );
    let q = s.result().unwrap();
    let oracle = query_active_program_residual(
        &m,
        &[],
        &e.goal,
        SearchLimits::default(),
        &mut Clock::default(),
    )
    .unwrap();
    assert_eq!(q.required, oracle.required);
    assert_eq!(q.explanations, oracle.explanations);
    assert_eq!(q.explanations.len(), 4);
    assert!(
        q.explanations
            .iter()
            .all(|a| !a.activations.is_empty() && a.hypothetical.known.contains(&e.goal))
    );
    assert_eq!(m, original);
}
