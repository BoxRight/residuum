//! Experimental evidence contract, never a rejection predicate or selector input.
use super::opposition::{OppositionScope, probe_candidates};
use super::*;
use residuum::defaults::ConsistencyStatus;
use residuum::experiments::{DefaultExperimentResult, run_default_experiments};
use residuum::programs::program_rule_origin;
use residuum::typed::{Closure, Default, DefaultRef, Module, ProgramRef};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug)]
struct EvidenceSide {
    event: Event,
    in_known: bool,
    known_justifications: Vec<DerivedEvent>,
    hypothetical_justifications: Vec<DerivedEvent>,
}

#[derive(Debug)]
struct EvidenceOpposition {
    positive: EvidenceSide,
    negative: EvidenceSide,
}

// All three collections may be nonempty. No Consistent/Inconsistent, Blocked,
// Defeated or Accepted label can be inferred from this observation alone.
#[derive(Debug)]
struct ObservedConsistencyStatus {
    candidate: DefaultRef,
    program: ProgramRef,
    prior: Vec<EvidenceOpposition>,
    against_known: Vec<EvidenceOpposition>,
    within_delta: Vec<EvidenceOpposition>,
    delta: BTreeSet<Event>,
    hypothetical: Closure,
}

fn load(file: &str) -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/legal/modules")
            .join(file),
    )
    .unwrap()
}

fn observe(
    module: &Module,
    context: &DefaultExperimentResult,
    candidate: &Default,
) -> ObservedConsistencyStatus {
    assert!(candidate.body.is_program());
    // Probe ignores applicability/blocking/priority deliberately: this is H_D,
    // including counterfactual candidates, not the result of default selection.
    let probe = probe_candidates(module, context, &[candidate]);
    let side = |event: &Event| EvidenceSide {
        event: event.clone(),
        in_known: context.context.known.contains(event),
        known_justifications: context
            .context
            .derivations
            .iter()
            .filter(|d| d.event == *event)
            .cloned()
            .collect(),
        hypothetical_justifications: probe
            .closure
            .derivations
            .iter()
            .filter(|d| d.event == *event)
            .cloned()
            .collect(),
    };
    let pairs = |scope| {
        probe
            .oppositions
            .iter()
            .filter(|p| p.scope == scope)
            .map(|p| EvidenceOpposition {
                positive: side(&p.positive),
                negative: side(&p.negative),
            })
            .collect()
    };
    ObservedConsistencyStatus {
        candidate: DefaultRef {
            name: candidate.name.clone(),
        },
        program: ProgramRef {
            name: candidate.body.target_name().into(),
        },
        prior: pairs(OppositionScope::AlreadyKnown),
        against_known: pairs(OppositionScope::AgainstKnown),
        within_delta: pairs(OppositionScope::WithinHypothesis),
        delta: probe.added,
        hypothetical: probe.closure,
    }
}

fn append_side(report: &mut String, module: &Module, side: &EvidenceSide) {
    writeln!(
        report,
        "    {} | in Known: {}",
        render_event(&side.event),
        side.in_known
    )
    .unwrap();
    for (location, proofs) in [
        ("Known", &side.known_justifications),
        ("hypothesis", &side.hypothetical_justifications),
    ] {
        for proof in proofs {
            let origin = program_rule_origin(module, &proof.record.rule);
            let (program, rule, owner) = if let Some(origin) = &origin {
                (
                    origin.program.as_str(),
                    origin.rule.as_str(),
                    origin.rule_origin.module.as_str(),
                )
            } else {
                (
                    "environment",
                    proof.record.rule.as_str(),
                    module.origin(&proof.record.rule).unwrap().module.as_str(),
                )
            };
            writeln!(report, "      {location}: program {program} | rule {rule} | module {owner} | derivedAt {} | {}", proof.derived_at.name, render_substitution(&proof.record.substitution)).unwrap();
            for witness in &proof.record.antecedents {
                writeln!(report, "        witness: {}", render_event(witness)).unwrap();
            }
        }
    }
}

#[test]
fn program_consistency_observes_three_scopes_without_altering_selection() {
    let mut report = String::from(
        "Experimental ObservedConsistencyStatus; runtime ConsistencyStatus = NotChecked.\nH_D = Close(mandatory rules + compiled Program_D, Known). Delta_D = H_D minus Known.\nNo opposition implies blocked/defeated; probes do not feed selection.\nscenario | candidate -> Program | applicable | selected by unchanged policy | prior | against_known | within_delta | delta size\n",
    );
    for (file, expected) in [
        (
            "default_opposition_scenario.res",
            vec![
                ("clearSale", [0, 0, 0], [0, 0, 0]),
                ("opposedNormal", [0, 1, 0], [0, 0, 0]),
                ("opposedRetentionUnpaid", [0, 1, 0], [0, 0, 0]),
                ("opposedRetentionPaid", [0, 1, 0], [0, 1, 0]),
                ("alreadyOpposed", [2, 0, 0], [2, 0, 0]),
                ("differentAuthority", [0, 0, 0], [0, 0, 0]),
                ("differentBeneficiary", [0, 0, 0], [0, 0, 0]),
                ("differentTime", [0, 0, 0], [0, 0, 0]),
                ("internalOpposition", [0, 0, 1], [0, 0, 0]),
            ],
        ),
        (
            "program_consistency_scenario.res",
            vec![
                ("emptyKnown", [0, 0, 0], [0, 0, 0]),
                ("newNegativeAgainstKnown", [0, 1, 0], [0, 1, 0]),
                ("mixedOpposition", [2, 1, 1], [2, 0, 0]),
            ],
        ),
    ] {
        let module = load(file);
        let original = module.clone();
        let contexts = run_default_experiments(&module).unwrap();
        assert_eq!(contexts.len(), expected.len());
        for (context, (name, normal, retention)) in contexts.iter().zip(expected) {
            assert_eq!(context.observation.experiment.name, name);
            let before = context.clone();
            for candidate in module.declarations.iter().filter_map(|d| match d {
                Declaration::Default(d) => Some(d),
                _ => None,
            }) {
                let status = observe(&module, context, candidate);
                let expected = if candidate.name == "NormalSale" {
                    normal
                } else {
                    retention
                };
                assert_eq!(
                    [
                        status.prior.len(),
                        status.against_known.len(),
                        status.within_delta.len()
                    ],
                    expected,
                    "{name}: {}",
                    candidate.name
                );
                assert_eq!(status.candidate.name, candidate.name);
                let actual_delta: BTreeSet<_> = status
                    .hypothetical
                    .known
                    .iter()
                    .filter(|e| !context.context.known.contains(e))
                    .cloned()
                    .collect();
                assert_eq!(status.delta, actual_delta);
                assert!(
                    context
                        .context
                        .known
                        .iter()
                        .all(|e| status.hypothetical.known.contains(e))
                );
                for proof in &status.hypothetical.derivations {
                    if let Some(origin) = program_rule_origin(&module, &proof.record.rule) {
                        assert_eq!(
                            origin.program, status.program.name,
                            "no other default Program may leak into H_D"
                        );
                    }
                    assert!(
                        proof
                            .record
                            .antecedents
                            .iter()
                            .all(|e| status.hypothetical.known.contains(e))
                    );
                }
                let applicable = context
                    .extension
                    .applicable
                    .iter()
                    .any(|d| d.default == status.candidate);
                let selected = context.extension.selected.contains(&status.candidate);
                writeln!(
                    report,
                    "{name} | {} -> {} | {applicable} | {selected} | {} | {} | {} | {}",
                    candidate.name,
                    status.program.name,
                    status.prior.len(),
                    status.against_known.len(),
                    status.within_delta.len(),
                    status.delta.len()
                )
                .unwrap();
                for (scope, pairs) in [
                    ("prior", &status.prior),
                    ("against_known", &status.against_known),
                    ("within_delta", &status.within_delta),
                ] {
                    for pair in pairs {
                        assert_eq!(pair.positive.event.args, pair.negative.event.args);
                        assert_eq!(pair.positive.event.verb, pair.negative.event.verb);
                        assert_eq!(
                            pair.positive.event.proposition.time,
                            pair.negative.event.proposition.time
                        );
                        assert_ne!(pair.positive.event.polarity, pair.negative.event.polarity);
                        match scope {
                            "prior" => assert!(pair.positive.in_known && pair.negative.in_known),
                            "against_known" => {
                                assert_ne!(pair.positive.in_known, pair.negative.in_known)
                            }
                            _ => assert!(!pair.positive.in_known && !pair.negative.in_known),
                        }
                        writeln!(report, "  scope: {scope}").unwrap();
                        append_side(&mut report, &module, &pair.positive);
                        append_side(&mut report, &module, &pair.negative);
                    }
                }
                assert_eq!(
                    *context, before,
                    "Known and default selection remain intact"
                );
                assert_eq!(context.extension.consistency, ConsistencyStatus::NotChecked);
            }
        }
        assert_eq!(module, original);
        assert_eq!(run_default_experiments(&module).unwrap(), contexts);
    }
    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/program-consistency-observations.txt");
    std::fs::write(&output, &report).unwrap();
    println!("{report}\nReport: {}", output.display());
}

#[test]
fn program_consistency_keeps_prior_and_hypothetical_justifications_for_known_events() {
    let module = load("program_consistency_scenario.res");
    let contexts = run_default_experiments(&module).unwrap();
    let context = contexts
        .iter()
        .find(|c| c.observation.experiment.name == "newNegativeAgainstKnown")
        .unwrap();
    let candidate = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Default(d) if d.name == "NormalSale" => Some(d),
            _ => None,
        })
        .unwrap();
    let status = observe(&module, context, candidate);
    let pair = &status.against_known[0];
    assert!(pair.positive.in_known && !pair.negative.in_known);
    assert_eq!(pair.positive.known_justifications.len(), 1);
    assert_eq!(
        pair.positive.known_justifications[0].record.rule,
        "filedObligation"
    );
    assert_eq!(pair.positive.hypothetical_justifications.len(), 2);
    assert!(pair.positive.hypothetical_justifications.iter().any(|p| {
        program_rule_origin(&module, &p.record.rule).is_some_and(|o| {
            o.program == "normalSaleProgram" && o.rule == "plainTransferObligation"
        })
    }));
    assert!(!status.delta.contains(&pair.positive.event));
    assert!(status.delta.contains(&pair.negative.event));
    assert!(pair.negative.known_justifications.is_empty());
    assert_eq!(pair.negative.hypothetical_justifications.len(), 1);
    assert_eq!(
        pair.negative.hypothetical_justifications[0].record.rule,
        "conditionalNegativeDuty"
    );
    let selected = pair.negative.hypothetical_justifications[0]
        .record
        .antecedents
        .iter()
        .find(|e| e.verb == "selectedReciprocity")
        .unwrap();
    assert!(status.hypothetical.derivations.iter().any(|p| {
        p.event == *selected
            && program_rule_origin(&module, &p.record.rule)
                .is_some_and(|o| o.program == "normalSaleProgram" && o.rule == "normalSale")
    }));
}
