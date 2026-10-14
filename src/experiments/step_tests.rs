use super::*;
use std::path::Path;
fn module(file: &str) -> Module {
    crate::load_formal_module(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).unwrap()
}
fn experiments(m: &Module) -> Vec<&Experiment> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) => Some(e),
            _ => None,
        })
        .collect()
}
fn rules(m: &Module) -> Vec<Rule> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn four_steps_filter_models_and_final_result_matches_fusion_counterexample() {
    let m = module("examples/experiments/four_counterexample.res");
    let e = experiments(&m)[0];
    let r = rules(&m);
    let mut s = FourSession::new(e, &r).unwrap();
    let first = s.step();
    assert!(!first.complete);
    assert!(first.entailed.is_none());
    assert!(s.result().is_none());
    let second = s.step();
    assert!(second.complete);
    assert_eq!(second.entailed, Some(true));
    assert_eq!(second.surviving.len(), 3);
    assert_eq!(s.result().unwrap(), &four::entailment(e, &r).unwrap());
    let rejected: BTreeSet<_> = first
        .eliminated
        .iter()
        .chain(&second.eliminated)
        .copied()
        .collect();
    assert_eq!(
        rejected.len() + second.surviving.len(),
        second.interpretations
    );
    let mut reverse = r.clone();
    reverse.reverse();
    let mut s = FourSession::new(e, &reverse).unwrap();
    s.trace(10);
    assert_eq!(s.result().unwrap(), &four::entailment(e, &r).unwrap());
}

#[test]
fn four_empty_theory_and_finite_domain_restrictions_are_preserved() {
    let m = module("examples/experiments/four_counterexample.res");
    let mut e = experiments(&m)[0].clone();
    e.input.clear();
    let mut s = FourSession::new(&e, &[]).unwrap();
    assert!(s.step().complete);
    assert_eq!(s.result().unwrap().entailed, false);
    let m = module("examples/experiments/residual_proof.res");
    assert!(
        FourSession::new(experiments(&m)[0], &rules(&m))
            .unwrap_err()
            .to_string()
            .contains("ground rules")
    );
}

#[test]
fn default_steps_preserve_one_selection_and_existing_final_extension() {
    let m = module("examples/legal/case_sale_defaults_formal.res");
    let oracle = run_default_experiments(&m).unwrap();
    for e in experiments(&m) {
        let mut s = DefaultSession::new(&m, e).unwrap();
        let steps = s.trace(64, &mut Clock::default()).unwrap();
        assert!(steps.last().unwrap().complete);
        assert_eq!(
            steps
                .iter()
                .filter(|s| s.phase == DefaultPhase::Selection)
                .count(),
            1
        );
        let actual = s.result().unwrap();
        let expected = oracle
            .iter()
            .find(|a| a.observation.experiment.name == e.name)
            .unwrap();
        assert_eq!(actual.extension, expected.extension);
        assert_eq!(
            actual
                .observation
                .closure
                .known
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected.observation.closure.known.iter().cloned().collect()
        );
        let proofs = |c: &Closure| {
            let mut p: Vec<_> = c
                .derivations
                .iter()
                .map(|d| format!("{:?} {:?}", d.event, d.record))
                .collect();
            p.sort();
            p
        };
        assert_eq!(
            proofs(&actual.observation.closure),
            proofs(&expected.observation.closure)
        );
    }
}
