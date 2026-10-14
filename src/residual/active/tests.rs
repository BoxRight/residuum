//! Public API regressions using source fixtures, independent of the legal harness.
use super::*;
use crate::derive::close_program_reference;
use std::path::Path;

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("active-api-{}", self.0),
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

fn same_closure(a: &Closure, b: &Closure) {
    let events = |c: &Closure| {
        c.known
            .iter()
            .map(|e| format!("{e:?}"))
            .collect::<BTreeSet<_>>()
    };
    let proofs = |c: &Closure| {
        let mut p: Vec<_> = c
            .derivations
            .iter()
            .map(|d| format!("{:?}\t{:?}", d.event, d.record))
            .collect();
        p.sort();
        p
    };
    assert_eq!(events(a), events(b));
    assert_eq!(proofs(a), proofs(b));
}

#[test]
fn activation_guards_are_certified_without_changing_horn_queries() {
    let m = module("examples/experiments/associated_program_waits.res");
    let original = m.clone();
    let ex = experiment(&m, "later");
    let q = query_active_program_residual(
        &m,
        &[],
        &ex.goal,
        SearchLimits::default(),
        &mut Clock::default(),
    )
    .unwrap();
    assert_eq!(q.explanations.len(), 1);
    let mut hypotheses = Vec::new();
    leaves(&q.explanations[0].candidate.required, &mut hypotheses);
    assert_eq!(
        hypotheses
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["starts", "signals"])
    );
    let real = close_program_reference(ex.input.clone(), &m, &mut Clock::default()).unwrap();
    same_closure(&q.explanations[0].hypothetical, &real);
    let ready = real
        .known
        .iter()
        .find(|e| e.verb == "ready")
        .unwrap()
        .clone();
    let horn = super::super::query_program_residual_with_abducibles(
        &m,
        &[ready.clone()],
        &ex.goal,
        &[],
        SearchLimits::default(),
    )
    .unwrap();
    assert_eq!(horn.required, Some(RequirementFormula::Unit));
    let active = query_active_program_residual_with_abducibles(
        &m,
        &[ready.clone()],
        &ex.goal,
        &[],
        SearchLimits::default(),
        &mut Clock::default(),
    )
    .unwrap();
    assert!(active.required.is_none());
    assert!(active.explanations.is_empty());
    let mut negative = ex.input[0].clone();
    negative.polarity = Polarity::EvidentialNot;
    let mut wrong_time = ex.input[0].clone();
    wrong_time.proposition.time = TimeExpr::At("sigma".into());
    for trigger in [negative, wrong_time] {
        let q = query_active_program_residual_with_abducibles(
            &m,
            &[trigger, ready.clone()],
            &ex.goal,
            &[],
            SearchLimits::default(),
            &mut Clock::default(),
        )
        .unwrap();
        assert!(q.explanations.is_empty());
        assert_ne!(q.required, Some(RequirementFormula::Unit));
    }
    assert_eq!(m, original);
}

#[test]
fn active_results_distinguish_pending_unit_absence_and_errors() {
    let m = module("examples/experiments/associated_multi_rule_program.res");
    let original = m.clone();
    let ex = experiment(&m, "real");
    for (evidence, expected_unit, expected_pending) in [
        (Vec::new(), false, true),
        (ex.input.clone(), true, true),
        (vec![ex.goal.clone()], true, false),
    ] {
        let before = evidence.clone();
        let q = query_active_program_residual(
            &m,
            &evidence,
            &ex.goal,
            SearchLimits::default(),
            &mut Clock::default(),
        )
        .unwrap();
        assert!(q.required.is_some());
        assert!(q.failure.is_none());
        assert_eq!(
            q.explanations
                .iter()
                .any(|a| a.candidate.required == Antecedent::Unit),
            expected_unit
        );
        assert_eq!(
            q.explanations
                .iter()
                .any(|a| a.candidate.required != Antecedent::Unit),
            expected_pending
        );
        for a in &q.explanations {
            assert!(a.hypothetical.known.contains(&ex.goal));
        }
        if expected_unit && expected_pending {
            assert!(matches!(q.required, Some(RequirementFormula::Join(_, _))));
            assert_eq!(q.explanations.len(), 4);
        }
        assert_eq!(evidence, before);
    }
    let none = query_active_program_residual_with_abducibles(
        &m,
        &[],
        &ex.goal,
        &[],
        SearchLimits::default(),
        &mut Clock::default(),
    )
    .unwrap();
    assert!(none.required.is_none());
    assert!(none.explanations.is_empty());
    assert_eq!(none.failure, Some(AbductionFailure::NotAbducible));
    let err = query_active_program_residual(
        &m,
        &[],
        &ex.goal,
        SearchLimits {
            max_nodes: 1,
            ..SearchLimits::default()
        },
        &mut Clock::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("limit exceeded"));
    assert_eq!(m, original);
}

#[test]
fn imported_chained_programs_retain_actual_producers_and_proofs() {
    let m = module("examples/experiments/legal_geometry/residual_activation_chain_scenario.res");
    let original = m.clone();
    let ex = experiment(&m, "real");
    let q = query_active_program_residual(
        &m,
        &[],
        &ex.goal,
        SearchLimits::default(),
        &mut Clock::default(),
    )
    .unwrap();
    assert_eq!(q.explanations.len(), 1);
    let a = &q.explanations[0];
    let mut required = Vec::new();
    leaves(&a.candidate.required, &mut required);
    assert_eq!(
        required.iter().cloned().collect::<BTreeSet<_>>(),
        ex.input.iter().cloned().collect()
    );
    let real = close_program_reference(ex.input.clone(), &m, &mut Clock::default()).unwrap();
    same_closure(&a.hypothetical, &real);
    assert_eq!(real.known.len(), 5);
    assert_eq!(real.derivations.len(), 4);
    assert_eq!(
        a.activations
            .iter()
            .map(|a| a.rule.name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "Prepare::0::prepareFirst",
            "Prepare::1::prepareSecond",
            "Finish::0::finishFirst",
            "Finish::1::finishSecond"
        ])
    );
    for activation in &a.activations {
        let origin = activation.program_origin.as_ref().unwrap();
        assert_eq!(origin.defining_module, "ResidualActivationChain");
        assert_eq!(origin.rule_origin.module, "ResidualActivationChain");
        assert!(matches!(origin.program.as_str(), "Prepare" | "Finish"));
    }
    assert_eq!(m, original);
}
