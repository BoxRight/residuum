use super::*;

#[path = "legal_geometry_tests.rs"]
mod legal_geometry;

#[path = "effect_tests.rs"]
mod effects;

#[path = "default_tests.rs"]
mod defaults;

#[path = "opposition_tests.rs"]
mod opposition;

#[path = "default_comparison_tests.rs"]
mod default_comparison;

#[path = "competition_contract_tests.rs"]
mod competition_contract;

#[path = "partial_package_tests.rs"]
mod partial_package;

#[path = "program_denotation_tests.rs"]
mod program_denotation;

#[path = "autonomous_program_tests.rs"]
mod autonomous_program;

#[path = "program_default_tests.rs"]
mod program_defaults;

#[path = "program_consistency_tests.rs"]
mod program_consistency;

#[path = "program_tests.rs"]
mod programs;

#[path = "temporal_beta_tests.rs"]
mod temporal_beta;

#[path = "query_binding_syntax_tests.rs"]
mod query_binding_syntax;

#[path = "module_tests.rs"]
mod modules;

#[path = "extraction_tests.rs"]
mod extraction;

#[path = "legal_partition_tests.rs"]
mod legal_partition;

const LEGAL: &str = include_str!("../../../../examples/legal/case_monotonic_formal.res");
const RESIDUAL: &str = include_str!("../../../../examples/experiments/residual_proof.res");
const FOUR: &str = include_str!("../../../../examples/experiments/four_counterexample.res");
const PROOFS: &str = include_str!("../../../../examples/experiments/cli_explain.res");

#[test]
fn run_observes_the_legal_case_without_invoking_queries() {
    let output = execute(Command::Run, LEGAL).unwrap();
    assert_eq!(output.matches("experiment ").count(), 16);
    assert_eq!(output.matches("Horn: true").count(), 10);
    assert_eq!(output.matches("Horn: false").count(), 6);
    assert!(output.contains("owns(pedro, car, transferDuty, private, oldDate) @ tau"));
    assert!(
        output.contains("annulled(carProhibition, carPermission, constitutional, newDate) @ tau")
    );
    assert!(!output.contains("requirement:"));
    assert!(!output.contains("FOUR fusion:"));
    // A false goal is an observation, not a failed program execution.
    let missing = output.split("experiment unperformed\n").nth(1).unwrap();
    assert!(missing.starts_with("  Seeds:"));
    assert!(missing.contains("Horn: false"));
}

#[test]
fn legal_reciprocity_uses_payment_evidence_and_accumulates_packages() {
    let module = residuum::parse_formal_to_typed(LEGAL).unwrap();
    let cases = run_forward_experiments(&module).unwrap();
    let case = |name: &str| cases.iter().find(|r| r.experiment.name == name).unwrap();
    let permit =
        "permission(saleTransferPermission, juan, transfer, car, pedro, private, oldDate) @ tau";
    for (name, obligated, permitted) in [
        ("plainWithoutPayment", true, true),
        ("retentionWithoutPayment", false, true),
        ("retentionPaid", true, true),
        ("retentionNegativePayment", false, true),
        ("retentionPaidAndNegative", true, true),
        ("bothReciprocities", true, true),
        ("retentionWrongPayment", false, true),
        ("noReciprocityPackage", false, false),
    ] {
        let result = case(name);
        assert_eq!(result.horn_goal, obligated, "{name}");
        assert_eq!(
            result
                .closure
                .known
                .iter()
                .any(|e| render_event(e) == permit),
            permitted,
            "{name}"
        );
        for seed in &result.experiment.input {
            assert!(result.closure.known.contains(seed), "{name}: seed retained");
        }
        // A norm's birth supplies neither historical transmission evidence nor
        // execution of a patrimonial effect. All Act evidence is source input.
        for event in &result.closure.known {
            assert!(!matches!(
                event.verb.as_str(),
                "give" | "transferred" | "owns" | "compliance" | "violation"
            ));
            if event.verb == "Act" {
                assert!(result.experiment.input.contains(event));
            }
        }
    }
    let unpaid = case("retentionWithoutPayment");
    let paid = case("retentionPaid");
    let opposed = case("retentionPaidAndNegative");
    let both = case("bothReciprocities");
    assert!(
        unpaid
            .closure
            .known
            .iter()
            .all(|e| paid.closure.known.contains(e))
    );
    assert!(
        paid.closure
            .known
            .iter()
            .all(|e| opposed.closure.known.contains(e))
    );
    assert!(
        paid.closure
            .known
            .iter()
            .all(|e| both.closure.known.contains(e))
    );
    assert!(
        opposed
            .closure
            .known
            .iter()
            .any(|e| e.verb == "inconsistent")
    );

    let proof = paid
        .closure
        .derivations
        .iter()
        .find(|d| d.record.rule == "paidRetentionObligation")
        .unwrap();
    assert_eq!(proof.record.antecedents.len(), 4);
    for verb in ["sale", "reciprocity", "paymentFor", "Act"] {
        assert!(proof.record.antecedents.iter().any(|e| e.verb == verb));
    }
    let substitution = render_substitution(&proof.record.substitution);
    for binding in [
        "seller=juan",
        "buyer=pedro",
        "object=car",
        "priceObject=price",
        "authority=private",
        "effective=oldDate",
        "paymentEffective=newDate",
        "t=tau",
    ] {
        assert!(substitution.contains(binding), "{binding}: {substitution}");
    }
    let justifications: Vec<_> = both
        .closure
        .derivations
        .iter()
        .filter(|d| d.event == both.experiment.goal)
        .collect();
    assert_eq!(justifications.len(), 2);
    for rule in ["plainTransferObligation", "paidRetentionObligation"] {
        assert!(justifications.iter().any(|d| d.record.rule == rule));
    }
    assert_eq!(
        both.closure
            .known
            .iter()
            .filter(|e| **e == both.experiment.goal)
            .count(),
        1
    );
    assert_eq!(
        both.closure
            .derivations
            .iter()
            .filter(|d| render_event(&d.event) == permit)
            .count(),
        2
    );
}

#[test]
fn query_reports_residual_requirements_and_the_four_counterexample() {
    let output = execute(Command::Query, RESIDUAL).unwrap();
    assert!(output.contains(
        "requirement: (agreesPrice(alice, car, bob) @ tau ⊗ agreesObject(bob, car, alice) @ tau)"
    ));
    assert!(output.contains("requirement: agreesObject(bob, car, alice) @ tau"));
    assert!(output.contains("requirement: I"));
    let empty = output.split("experiment priceOnly\n").next().unwrap();
    assert!(empty.contains("Horn: false"));
    assert!(empty.contains("Known after close: {}"));
    let output = execute(Command::Query, FOUR).unwrap();
    assert!(output.contains("Horn: false"));
    assert!(output.contains("FOUR fusion: true (3 models / 16 interpretations)"));
    assert!(output.contains("Known after close: {~Q() @ tau}"));
}

#[test]
fn explain_preserves_all_justifications_seeds_units_and_missing_goals() {
    let output = execute(Command::Explain, PROOFS).unwrap();
    let both = output.split("experiment seedGoal\n").next().unwrap();
    for expected in [
        "#1 result() @ tau",
        "#2 shared() @ tau",
        "#3 A() @ tau",
        "#4 B() @ tau",
        "rule consequence | derivedAt",
        "rule fromA | derivedAt",
        "rule fromB | derivedAt",
        "substitution: {t=tau}",
        "antecedents: #2",
        "antecedents: #3",
        "antecedents: #4",
    ] {
        assert!(both.contains(expected), "{expected}\n{both}");
    }
    assert_eq!(both.matches("#2 shared() @ tau").count(), 1);
    assert_eq!(both.matches("    seed\n").count(), 2);
    assert!(output.contains("#1 A() @ tau\n    seed\n"));
    assert!(output.contains("rule axiom | derivedAt"));
    assert!(output.contains("substitution: {}\n      antecedents: I"));
    let missing = output.split("experiment missing\n").nth(1).unwrap();
    assert!(missing.contains("not derived from these seeds"));
    assert!(!missing.contains("rule "));
    assert!(!missing.contains("hypothesis"));
    assert!(!missing.contains("derivedAt"));
    let legal = execute(Command::Explain, LEGAL).unwrap();
    assert!(legal.contains("rule patrimonyTitle | derivedAt"));
    assert!(legal.contains("authority=private"));
    assert!(legal.contains("effective=oldDate"));
    assert!(legal.contains("rule permissionAnnulsProhibition | derivedAt"));
}

#[test]
fn cli_keeps_existing_errors_and_associated_program_semantics() {
    assert!(
        Command::parse("execute")
            .unwrap_err()
            .to_string()
            .contains("Usage:")
    );
    assert!(
        execute(Command::Run, "module Empty")
            .unwrap_err()
            .to_string()
            .contains("no experiments")
    );
    assert!(
        execute(Command::Run, FOUR)
            .unwrap_err()
            .to_string()
            .contains("seed inputs")
    );
    let associated = include_str!("../../../../examples/experiments/associated_program_waits.res");
    let run = execute(Command::Run, associated).unwrap();
    assert_eq!(run.matches("Horn: true").count(), 1);
    assert_eq!(run.matches("Horn: false").count(), 2);
    // The existing residual query contract rejects unresolved rule parameters.
    assert!(execute(Command::Query, LEGAL).is_err());
    let cyclic = format!("{PROOFS}\nrule cycle(t) = result() @ t <= shared() @ t\n");
    assert!(
        execute(Command::Run, &cyclic)
            .unwrap_err()
            .to_string()
            .contains("acyclic")
    );
    let ill_typed = RESIDUAL.replace("give(alice, car, bob)", "give(car, car, bob)");
    assert!(
        execute(Command::Run, &ill_typed)
            .unwrap_err()
            .to_string()
            .contains("not assignable")
    );
}

#[path = "peano_after_tests.rs"]
mod peano_after;

#[path = "peano_multiplication_tests.rs"]
mod peano_multiplication;

#[path = "temporal_identity_tests.rs"]
mod temporal_identity;

#[path = "temporal_constructor_tests.rs"]
mod temporal_constructors;

#[path = "pilot004_tests.rs"]
mod pilot004;
#[path = "pilot004r_tests.rs"]
mod pilot004r;
#[path = "semantic_domain_tests.rs"]
mod semantic_domain;
#[path = "temporal_history_tests.rs"]
mod temporal_history;
#[path = "temporal_order_tests.rs"]
mod temporal_order;

#[path = "peano_polynomial_tests.rs"]
mod peano_polynomials;

#[path = "peano_polynomial_product_tests.rs"]
mod peano_polynomial_products;

#[path = "dynamic_incremental_contract_tests.rs"]
mod dynamic_incremental_contract;

#[path = "program_validation_tests.rs"]
mod program_validation;
