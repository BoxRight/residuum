//! Only observations of the case: its data and domain rules live in one .res.
use super::*;
use crate::typed::{PropositionKind, VerbKind};
use crate::{parse_formal, parse_formal_to_typed};

const SOURCE: &str = include_str!("../../examples/legal/case_monotonic_formal.res");

fn cases() -> Vec<ExperimentResult> {
    let module = parse_formal_to_typed(SOURCE).expect("the legal case elaborates");
    let original = module.clone();
    let results = run_forward_experiments(&module).expect("finite monotonic forward case");
    assert_eq!(module, original);
    assert_eq!(results.len(), 8);
    for result in &results {
        for seed in &result.experiment.input {
            assert!(result.closure.known.contains(seed), "no seed is retracted");
        }
        assert!(result.residual.is_none());
        assert!(result.four.is_none());
        for event in &result.closure.known {
            assert_eq!(event.polarity, Polarity::Positive);
            assert_eq!(event.proposition.time, TimeExpr::At("tau".into()));
        }
        for derivation in &result.closure.derivations {
            assert_eq!(derivation.event.proposition.kind, PropositionKind::Derived);
            assert!(!derivation.record.antecedents.is_empty());
            assert!(
                derivation
                    .record
                    .antecedents
                    .iter()
                    .all(|e| result.closure.known.contains(e))
            );
        }
    }
    results
}

fn case<'a>(results: &'a [ExperimentResult], name: &str) -> &'a ExperimentResult {
    results
        .iter()
        .find(|r| r.experiment.name == name)
        .expect("source-defined scenario")
}

fn has(result: &ExperimentResult, event: &str) -> bool {
    result
        .closure
        .known
        .iter()
        .any(|e| render_event(e) == event)
}

fn proof<'a>(result: &'a ExperimentResult, rule: &str) -> &'a crate::typed::DerivedEvent {
    result
        .closure
        .derivations
        .iter()
        .find(|d| d.record.rule == rule)
        .expect("the expected source rule actually fired")
}

#[test]
fn legal_case_fulfillment_and_patrimony_chain() {
    assert_eq!(
        parse_formal(SOURCE).unwrap(),
        crate::formal::parse_formal_manual(SOURCE).unwrap()
    );
    let module = parse_formal_to_typed(SOURCE).unwrap();
    assert!(!module.declarations.iter().any(|d| match d {
        Declaration::Default(_) => true,
        Declaration::Verb(v) => v.kind == VerbKind::Effect,
        _ => false,
    }));
    let results = cases();
    let fulfilled = case(&results, "fulfilledTransfer");
    assert!(fulfilled.horn_goal);
    for event in [
        "compliance(transferDuty, transferAct, juan, car, pedro, private, oldDate) @ tau",
        "compliance(transferRight, transferAct, juan, car, pedro, private, oldDate) @ tau",
        "patrimonialAsset(pedro, car, transferDuty, private, oldDate) @ tau",
        "owns(pedro, car, transferDuty, private, oldDate) @ tau",
        "owns(pedro, car, transferAct, private, newDate) @ tau",
        "capacity(pedro, use, car, transferDuty, private, oldDate) @ tau",
    ] {
        assert!(has(fulfilled, event), "{event}");
    }
    for (rule, antecedent) in [
        ("obligationCompliance", "Act"),
        ("rightCompliance", "Act"),
        ("complianceAsset", "compliance"),
        ("patrimonyTitle", "patrimonialAsset"),
        ("transferTitle", "transferred"),
        ("ownershipCapacity", "owns"),
    ] {
        assert!(
            proof(fulfilled, rule)
                .record
                .antecedents
                .iter()
                .any(|e| e.verb == antecedent)
        );
    }
    assert!(
        !fulfilled
            .closure
            .known
            .iter()
            .any(|e| e.verb == "violation")
    );
}

#[test]
fn legal_case_counter_and_prohibition_remain_monotonic() {
    let results = cases();
    let counter = case(&results, "counterOnly");
    assert!(counter.horn_goal);
    for event in [
        "violation(deliveryDuty, deliveryAct, juan, car, pedro, private, oldDate) @ tau",
        "violation(noOmission, deliveryAct, juan, car, pedro, judicial, oldDate) @ tau",
        "enforceability(deliveryRight, deliveryAct, juan, car, pedro, private, oldDate) @ tau",
        "patrimonialLiability(juan, car, deliveryDuty, private, oldDate) @ tau",
    ] {
        assert!(has(counter, event), "{event}");
    }
    assert!(
        !counter
            .closure
            .known
            .iter()
            .any(|e| e.verb == "Act" || e.verb == "compliance")
    );
    let both = case(&results, "actAndCounter");
    assert!(both.horn_goal);
    for verb in [
        "Act",
        "CounterAct",
        "extinguished",
        "obligation",
        "compliance",
        "violation",
        "patrimonialAsset",
        "patrimonialLiability",
    ] {
        assert!(both.closure.known.iter().any(|e| e.verb == verb));
    }
    let prohibited = case(&results, "prohibitedOccurrence");
    assert!(prohibited.horn_goal);
    assert_eq!(
        proof(prohibited, "prohibitedAct").record.antecedents.len(),
        2
    );
    let absent = case(&results, "unperformed");
    assert!(!absent.horn_goal);
    assert!(
        !absent
            .closure
            .known
            .iter()
            .any(|e| matches!(e.verb.as_str(), "CounterAct" | "violation" | "compliance"))
    );
}

#[test]
fn legal_case_authority_dates_and_annulment_preserve_facts() {
    let results = cases();
    let authority = case(&results, "authorityOverrides");
    assert!(authority.horn_goal);
    let annuls: BTreeSet<_> = authority
        .closure
        .known
        .iter()
        .filter(|e| e.verb == "annulled")
        .map(render_event)
        .collect();
    assert_eq!(
        annuls,
        [
            "annulled(carProhibition, carPermission, constitutional, newDate) @ tau",
            "annulled(boatProhibition, boatDuty, legislative, newDate) @ tau",
            "annulled(housePermission, houseProhibition, judicial, newDate) @ tau",
            "annulled(seatDuty, seatProhibition, constitutional, newDate) @ tau",
            "annulled(sameDateProhibition, sameDatePermission, constitutional, oldDate) @ tau",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    // The older superior permission has no qualifying date witness.
    assert!(has(
        authority,
        "permission(olderPermission, juan, transfer, book, pedro, judicial, oldDate) @ tau"
    ));
    assert!(has(
        authority,
        "prohibition(newerProhibition, juan, transfer, book, pedro, private, newDate) @ tau"
    ));
    // In particular, normalization to Derived does not erase the lower duty.
    assert!(has(
        authority,
        "obligation(seatDuty, juan, transfer, legislativeSeat, pedro, private, oldDate) @ tau"
    ));
    for rule in [
        "permissionAnnulsProhibition",
        "prohibitionAnnulsPermission",
        "obligationAnnulsProhibition",
        "prohibitionAnnulsObligation",
    ] {
        let record = &proof(authority, rule).record;
        assert_eq!(record.antecedents.len(), 4);
        for verb in ["higherAuthority", "notEarlier"] {
            assert!(record.antecedents.iter().any(|e| e.verb == verb));
        }
    }
}

#[test]
fn legal_case_legislation_requires_power_and_retains_metadata() {
    let results = cases();
    let legislation = case(&results, "legislation");
    assert!(legislation.horn_goal);
    for event in [
        "owns(legislator, legislativeSeat, holding, constitutional, oldDate) @ tau",
        "capacity(legislator, use, legislativeSeat, holding, constitutional, oldDate) @ tau",
        "legislativeCapacity(legislator, legislativeSeat, grantedPower, constitutional, oldDate) @ tau",
        "legislativePower(legislator, grantedPower, constitutional, oldDate) @ tau",
        "law(statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau",
        "obligation(statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau",
    ] {
        assert!(has(legislation, event), "{event}");
    }
    let law = proof(legislation, "enactLaw");
    assert_eq!(law.record.antecedents.len(), 3);
    assert!(
        law.record
            .antecedents
            .iter()
            .any(|e| e.verb == "legislativePower")
    );
    assert_eq!(
        proof(legislation, "statutoryObligation").record.antecedents[0],
        law.event
    );
    let powerless = case(&results, "enactmentWithoutPower");
    assert!(!powerless.horn_goal);
    assert!(
        !powerless
            .closure
            .known
            .iter()
            .any(|e| matches!(e.verb.as_str(), "law" | "obligation" | "legislativePower"))
    );
}
