//! Domain policy and experiment inputs live in the separate Formal fixture.
use super::*;
use residuum::defaults::{BlockingScope, ConsistencyStatus};
use residuum::experiments::{DefaultExperimentResult, run_default_experiments};
use residuum::typed::{Polarity, PropositionKind};

const SOURCE: &str = include_str!("../../../../examples/legal/case_sale_defaults_formal.res");

fn cases(source: &str) -> Vec<DefaultExperimentResult> {
    run_default_experiments(&residuum::parse_formal_to_typed(source).unwrap()).unwrap()
}

#[test]
fn sale_defaults_select_packages_before_horn_and_require_structural_payment() {
    let results = cases(SOURCE);
    assert_eq!(results.len(), 4);
    for (result, (name, chosen, lambda, obligation)) in results.iter().zip([
        ("noRetention", "NormalSale", "normalSale", true),
        ("retentionUnpaid", "RetentionSale", "retentionSale", false),
        ("retentionPaid", "RetentionSale", "retentionSale", true),
        ("normalBlocked", "RetentionSale", "retentionSale", false),
    ]) {
        assert_eq!(result.observation.experiment.name, name);
        assert_eq!(result.extension.blocking_scope, BlockingScope::RuleLambda);
        assert_eq!(result.extension.consistency, ConsistencyStatus::NotChecked);
        assert_eq!(result.extension.selected.len(), 1);
        assert_eq!(result.extension.selected[0].name, chosen);
        assert_eq!(result.extension.rules().len(), 1);
        assert_eq!(result.extension.rules()[0].name, lambda);
        assert_eq!(
            result.extension.blocked.len(),
            usize::from(chosen == "RetentionSale")
        );
        assert!(
            result.extension.defeated.is_empty(),
            "blocked is not priority defeat"
        );
        let known = &result.observation.closure.known;
        let norm =
            "obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau";
        assert_eq!(known.iter().any(|e| render_event(e) == norm), obligation);
        assert!(known.iter().any(|e| render_event(e) == "permission(saleTransferPermission, juan, transfer, car, pedro, private, oldDate) @ tau"));
        assert_eq!(known.iter().any(|e| e.verb == "do"), obligation);
        assert!(
            !known
                .iter()
                .any(|e| e.verb == "give" || e.polarity == Polarity::EvidentialNot)
        );
        let package = if obligation && chosen == "NormalSale" {
            "Plain"
        } else {
            "Retention"
        };
        assert!(
            known
                .iter()
                .any(|e| render_event(e)
                    == format!("selectedReciprocity(saleCase, {package}) @ tau"))
        );
        assert_eq!(
            known
                .iter()
                .filter(|e| e.verb == "selectedReciprocity")
                .count(),
            1
        );
        assert!(!result.context.known.iter().any(|e| matches!(
            e.verb.as_str(),
            "selectedReciprocity" | "obligation" | "permission" | "do"
        )));
        for seed in &result.observation.experiment.input {
            assert!(known.contains(seed));
        }
        for event in known.iter().filter(|e| e.verb == "Act") {
            assert!(result.observation.experiment.input.contains(event));
        }
        if !result.extension.blocked.is_empty() {
            let block = &result.extension.blocked[0];
            assert_eq!(block.default.name, "NormalSale");
            assert_eq!(block.evidence.len(), 1);
            assert_eq!(
                render_event(&block.evidence[0]),
                "retentionAgreed(saleCase) @ tau"
            );
            let substitution = render_substitution(&block.substitution);
            assert!(substitution.contains("saleId=saleCase"));
            assert!(substitution.contains("t=tau"));
            assert!(
                !result
                    .observation
                    .closure
                    .derivations
                    .iter()
                    .any(|d| d.record.rule == "normalSale")
            );
        }
    }
    let paid = &results[2].observation;
    let proof = paid
        .closure
        .derivations
        .iter()
        .find(|d| d.record.rule == "paidRetentionObligation")
        .unwrap();
    assert_eq!(proof.record.antecedents.len(), 4);
    for value in [
        "seller=juan",
        "buyer=pedro",
        "priceObject=price",
        "effective=oldDate",
        "paymentEffective=newDate",
    ] {
        assert!(render_substitution(&proof.record.substitution).contains(value));
    }
    assert!(results[3].observation.horn_goal);
    let module = residuum::parse_formal_to_typed(SOURCE).unwrap();
    assert!(
        run_forward_experiments(&module).is_err(),
        "default rules must not be treated as ordinary Horn rules"
    );
    // Alter only source evidence: neither a wrong payee nor negative payment
    // satisfies the positive, structurally matched payment condition.
    for source in [
        SOURCE.replace(
            "Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau",
            "Act(pricePayment, pedro, pay, price, pedro, private, newDate) @ tau",
        ),
        SOURCE.replace(
            "        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau",
            "        ~Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau",
        ),
    ] {
        let changed = cases(&source);
        let paid = &changed[2];
        assert_eq!(paid.extension.selected[0].name, "RetentionSale");
        assert!(!paid.observation.horn_goal);
        assert!(
            !paid
                .observation
                .closure
                .known
                .iter()
                .any(|e| { matches!(e.verb.as_str(), "obligation" | "do") })
        );
        assert!(
            paid.observation
                .closure
                .known
                .iter()
                .any(|e| e.verb == "permission")
        );
    }
}

#[test]
fn explain_defaults_reports_selection_blocking_witnesses_and_missing_obligation() {
    let output = execute(Command::Explain, SOURCE).unwrap();
    let normal = output.split("experiment retentionUnpaid\n").next().unwrap();
    assert!(normal.contains("default NormalSale -> normalSale: selected"));
    assert!(normal.contains("default RetentionSale -> retentionSale: not applicable"));
    assert!(normal.contains("applicability: unconditional"));
    assert!(normal.contains("selected default lambdas: normalSale"));
    assert!(normal.contains("rule normalSale | derivedAt"));
    let unpaid = output
        .split("experiment retentionUnpaid\n")
        .nth(1)
        .unwrap()
        .split("experiment retentionPaid\n")
        .next()
        .unwrap();
    for expected in [
        "default NormalSale -> normalSale: blocked",
        "blocking evidence: retentionAgreed(saleCase) @ tau",
        "default RetentionSale -> retentionSale: selected",
        "applicability evidence: retentionAgreed(saleCase) @ tau",
        "substitution: {saleId=saleCase, t=tau}",
        "declared priority: exception to NormalSale",
        "selected default lambdas: retentionSale",
        "not derived from these seeds",
        "blocking scope: RuleLambda",
        "consistency: NotChecked",
    ] {
        assert!(unpaid.contains(expected), "{expected}\n{unpaid}");
    }
    let blocked = output.split("experiment normalBlocked\n").nth(1).unwrap();
    assert!(blocked.contains("rule retentionSale | derivedAt"));
    assert!(!blocked.contains("rule normalSale | derivedAt"));
    let run = execute(Command::Run, SOURCE).unwrap();
    assert_eq!(run.matches("experiment ").count(), 4);
    assert_eq!(run.matches("Horn: true").count(), 3);
    assert_eq!(run.matches("Horn: false").count(), 1);
    assert!(
        execute(Command::Query, SOURCE)
            .unwrap_err()
            .to_string()
            .contains("without defaults")
    );
}

#[test]
fn default_runner_uses_mandatory_derived_context_and_keeps_priority_distinct() {
    let no_blocker = SOURCE.replace(
        "    blocked when\n        retentionAgreed(saleId) @ t\n",
        "",
    );
    let unblocked = cases(&no_blocker);
    let blocked = cases(SOURCE);
    for (priority, explicit) in unblocked.iter().zip(&blocked).skip(1) {
        assert!(priority.extension.blocked.is_empty());
        assert_eq!(priority.extension.defeated[0].name, "NormalSale");
        assert_eq!(priority.extension.selected, explicit.extension.selected);
        assert_eq!(priority.observation.closure, explicit.observation.closure);
    }
    let explanation = execute(Command::Explain, &no_blocker).unwrap();
    assert!(explanation.contains("default NormalSale -> normalSale: defeated"));
    assert!(explanation.contains("priority: displaced by an applicable, unblocked exception"));

    // A source-level variation derives the reservation from a filed document.
    // The evaluator must see its mandatory closure, not just raw input seeds.
    let (declarations, experiments) = SOURCE.split_once("experiment noRetention").unwrap();
    let derived_context = declarations.replace(
        "seeded verb retentionAgreed(sale : Fact)",
        "seeded verb reservationDocument(sale : Fact)\nderived verb retentionAgreed(sale : Fact)",
    );
    let derived_context = format!(
        "{derived_context}\nrule filedReservation(saleId, t) =\n    reservationDocument(saleId) @ t\n    <= retentionAgreed(saleId) @ t\n\nexperiment noRetention{}",
        experiments.replace("retentionAgreed(saleCase)", "reservationDocument(saleCase)")
    );
    let derived = cases(&derived_context);
    assert_eq!(derived[1].extension.selected[0].name, "RetentionSale");
    assert!(!derived[1].observation.horn_goal);
    let evidence = &derived[1].extension.blocked[0].evidence[0];
    assert_eq!(evidence.proposition.kind, PropositionKind::Derived);
    assert!(
        derived[1]
            .context
            .derivations
            .iter()
            .any(|d| d.event == *evidence && d.record.rule == "filedReservation")
    );
    let cyclic = format!(
        "{SOURCE}\nrule cycle(saleId, package, t) = selectedReciprocity(saleId, package) @ t <= selectedReciprocity(saleId, package) @ t\n"
    );
    assert!(
        execute(Command::Run, &cyclic)
            .unwrap_err()
            .to_string()
            .contains("acyclic")
    );
    let associated = SOURCE.replace(
        "effective : PropositionTime)\nseeded verb retentionAgreed",
        "effective : PropositionTime) = program normalSale\nseeded verb retentionAgreed",
    );
    assert_ne!(associated, SOURCE);
    assert!(
        execute(Command::Run, &associated)
            .unwrap_err()
            .to_string()
            .contains("default-controlled rules cannot also be associated verb programs")
    );
}
