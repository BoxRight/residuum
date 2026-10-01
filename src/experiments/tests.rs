use super::*;
use crate::typed::{ExperimentInputKind, Type};
use crate::{parse_formal, parse_formal_to_typed};

const RESIDUAL: &str = include_str!("../../examples/experiments/residual_proof.res");
const FOUR: &str = include_str!("../../examples/experiments/four_counterexample.res");

#[test]
fn fixtures_define_data_and_queries_in_equal_formal_surface_asts() {
    for source in [RESIDUAL, FOUR] {
        assert_eq!(
            parse_formal(source).unwrap(),
            crate::formal::parse_formal_manual(source).unwrap()
        );
        let module = parse_formal_to_typed(source).unwrap();
        assert!(module.declarations.iter().any(|d| matches!(d, Declaration::Const(c) if c.name == "tau" && c.ty == Type::PropositionTime)));
    }
    let module = parse_formal_to_typed(FOUR).unwrap();
    assert!(
        module
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Rule(rule) if rule.parameters.is_empty()))
    );
}

#[test]
fn source_defined_residual_scenarios_have_observable_requirements() {
    let module = parse_formal_to_typed(RESIDUAL).unwrap();
    let before = module.clone();
    let results = run_experiments(&module).unwrap();
    assert_eq!(results.len(), 3);
    let requirements: Vec<_> = results
        .iter()
        .map(|r| render_requirement(r.residual.as_ref().unwrap().required.as_ref().unwrap()))
        .collect();
    assert_eq!(
        requirements,
        [
            "(agreesPrice(alice, car, bob) @ tau ⊗ agreesObject(bob, car, alice) @ tau)",
            "agreesObject(bob, car, alice) @ tau",
            "I",
        ]
    );
    assert_eq!(
        results.iter().map(|r| r.horn_goal).collect::<Vec<_>>(),
        [false, false, true]
    );
    assert_eq!(
        results
            .iter()
            .map(|r| r.experiment.input.len())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!(
        module, before,
        "queries do not assert hypotheses or execute effects"
    );
    let output = render_results(&results);
    assert!(output.contains("experiment empty\n"));
    assert!(output.contains("requirement: I\n"));
}

#[test]
fn source_defined_four_query_distinguishes_entailment_from_horn() {
    let module = parse_formal_to_typed(FOUR).unwrap();
    let results = run_experiments(&module).unwrap();
    let result = &results[0];
    assert_eq!(result.experiment.input_kind, ExperimentInputKind::Premises);
    assert!(!result.horn_goal);
    assert_eq!(
        result.four,
        Some(FourEntailment {
            interpretations: 16,
            models: 3,
            entailed: true
        })
    );
    assert_eq!(result.closure.known, result.experiment.input);
    assert!(result.closure.derivations.is_empty());
    assert!(render_results(&results).contains("FOUR fusion: true (3 models / 16 interpretations)"));
}

#[test]
fn experiment_data_are_ground_typed_and_seeds_are_not_derived() {
    for (source, message) in [
        (
            RESIDUAL.replace(
                "give(alice, car, bob) @ after tau",
                "give(car, car, bob) @ after tau",
            ),
            "not assignable",
        ),
        (
            RESIDUAL.replace(
                "give(alice, car, bob) @ after tau",
                "give(alice, car) @ after tau",
            ),
            "expects 3 args",
        ),
        (
            RESIDUAL.replace(
                "give(alice, car, bob) @ after tau",
                "give(alice, unknown, bob) @ after tau",
            ),
            "unknown ground constant",
        ),
        (
            RESIDUAL.replace("@ after tau", "@ after unknownTime"),
            "PropositionTime constant",
        ),
        (
            RESIDUAL.replace(
                "agreesPrice(alice, car, bob) @ tau,",
                "give(alice, car, bob) @ tau,",
            ),
            "seeded verbs",
        ),
        (FOUR.replace("premises", "seeds"), "logical premises"),
        (RESIDUAL.replace("seeds", "premises"), "require seeds"),
        (
            RESIDUAL.replace("@ after tau", ""),
            "unresolved PropositionTime",
        ),
    ] {
        let error = parse_formal_to_typed(&source).unwrap_err().to_string();
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn experiment_parser_requires_progress_and_internal_commas() {
    let missing = RESIDUAL.replace(
        "agreesPrice(alice, car, bob) @ tau,",
        "agreesPrice(alice, car, bob) @ tau",
    );
    let incomplete = RESIDUAL.trim_end().trim_end_matches('}');
    for source in [missing.as_str(), incomplete] {
        assert!(parse_formal(source).is_err());
        assert!(crate::formal::parse_formal_manual(source).is_err());
    }
    assert!(
        parse_formal_to_typed(&FOUR.replace("~Q() @ tau,", "~Q() @ tau")).is_ok(),
        "only trailing comma is optional"
    );
}

#[test]
fn fixture_runner_rejects_unbounded_or_unsupported_comparisons() {
    let cyclic = format!(
        "{RESIDUAL}\nrule repeat(subject, object, recipient, t) = give(subject, object, recipient) @ t <= give(subject, object, recipient) @ after t\n"
    );
    assert!(
        run_experiments(&parse_formal_to_typed(&cyclic).unwrap())
            .unwrap_err()
            .to_string()
            .contains("acyclic")
    );
    let parameterized = FOUR
        .replace("rule implication()", "rule implication(t)")
        .replace("P() @ tau <= Q() @ tau", "P() @ t <= Q() @ t");
    assert!(
        run_experiments(&parse_formal_to_typed(&parameterized).unwrap())
            .unwrap_err()
            .to_string()
            .contains("ground rules")
    );
    let times = FOUR
        .replace(
            "const tau : PropositionTime",
            "const tau : PropositionTime\nconst sigma : PropositionTime",
        )
        .replace(
            "query fourFusion ~P() @ tau",
            "query fourFusion ~P() @ sigma",
        );
    assert!(
        run_experiments(&parse_formal_to_typed(&times).unwrap())
            .unwrap_err()
            .to_string()
            .contains("shared proposition time")
    );
    let too_many = format!(
        "{FOUR}\nderived verb R()\nderived verb S()\nderived verb T()\nrule r() = P() @ tau <= R() @ tau\nrule s() = R() @ tau <= S() @ tau\nrule u() = S() @ tau <= T() @ tau\n"
    );
    assert!(
        run_experiments(&parse_formal_to_typed(&too_many).unwrap())
            .unwrap_err()
            .to_string()
            .contains("at most four")
    );
}
