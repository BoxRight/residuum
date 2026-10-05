//! Experimental observations, not a consistency predicate or default policy.
//! All evidence and domain inference are declared in the imported .res fixture.
use super::*;
use residuum::defaults::ConsistencyStatus;
use residuum::experiments::{DefaultExperimentResult, run_default_experiments};
use residuum::programs::{compile_program, program_rule_origin};
use residuum::typed::{Closure, Default, Module, Polarity};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OppositionScope {
    AlreadyKnown,
    AgainstKnown,
    WithinHypothesis,
}

#[derive(Debug)]
pub(super) struct Opposition {
    pub(super) positive: Event,
    pub(super) negative: Event,
    pub(super) scope: OppositionScope,
}

pub(super) struct Probe {
    pub(super) closure: Closure,
    pub(super) added: BTreeSet<Event>,
    pub(super) oppositions: Vec<Opposition>,
}

fn source() -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/legal/modules/default_opposition_scenario.res"),
    )
    .unwrap()
}

fn target(default: &Default) -> &str {
    default.body.target_name()
}

fn probe(module: &Module, context: &DefaultExperimentResult, candidate: &Default) -> Probe {
    probe_candidates(module, context, &[candidate])
}

pub(super) fn probe_candidates(
    module: &Module,
    context: &DefaultExperimentResult,
    candidates: &[&Default],
) -> Probe {
    let controlled: BTreeSet<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Default(default) => Some(default),
            _ => None,
        })
        .flat_map(|default| {
            if default.body.is_program() {
                let program = module
                    .declarations
                    .iter()
                    .find_map(|d| match d {
                        Declaration::Program(p) if p.identity == target(default) => Some(p),
                        _ => None,
                    })
                    .unwrap();
                program
                    .rule_uses
                    .iter()
                    .map(|u| u.rule.name.as_str())
                    .collect::<Vec<_>>()
            } else {
                vec![target(default)]
            }
        })
        .collect();
    let active: Vec<_> = candidates
        .iter()
        .flat_map(|candidate| {
            if candidate.body.is_program() {
                let program = module
                    .declarations
                    .iter()
                    .find_map(|d| match d {
                        Declaration::Program(p) if p.identity == target(candidate) => Some(p),
                        _ => None,
                    })
                    .unwrap();
                compile_program(program, module).unwrap()
            } else {
                vec![
                    module
                        .declarations
                        .iter()
                        .find_map(|d| match d {
                            Declaration::Rule(r) if r.name == target(candidate) => Some(r.clone()),
                            _ => None,
                        })
                        .unwrap(),
                ]
            }
        })
        .collect();
    let mut hypothetical = module.clone();
    hypothetical.declarations.retain(|d| match d {
        Declaration::Default(_) | Declaration::Experiment(_) => false,
        Declaration::Rule(rule) => !controlled.contains(rule.name.as_str()),
        _ => true,
    });
    hypothetical
        .declarations
        .extend(active.into_iter().map(Declaration::Rule));
    let mut observation = context.observation.experiment.clone();
    // A private copy of the current mandatory Known, including Derived facts.
    // This is not source seed injection or a mutation of the real experiment.
    // Reuse the forward runner's termination checks and close_program API.
    observation.input = context.context.known.clone();
    hypothetical
        .declarations
        .push(Declaration::Experiment(observation));
    let closure = run_forward_experiments(&hypothetical)
        .unwrap()
        .remove(0)
        .closure;
    let known: BTreeSet<_> = context.context.known.iter().cloned().collect();
    let added: BTreeSet<_> = closure
        .known
        .iter()
        .filter(|e| !known.contains(*e))
        .cloned()
        .collect();
    let mut oppositions = Vec::new();
    for positive in closure
        .known
        .iter()
        .filter(|e| e.polarity == Polarity::Positive)
    {
        let mut opposite = positive.clone();
        opposite.polarity = Polarity::EvidentialNot;
        if let Some(negative) = closure.known.iter().find(|e| **e == opposite) {
            let scope = match (known.contains(positive), known.contains(negative)) {
                (true, true) => OppositionScope::AlreadyKnown,
                (false, false) => OppositionScope::WithinHypothesis,
                _ => OppositionScope::AgainstKnown,
            };
            oppositions.push(Opposition {
                positive: positive.clone(),
                negative: negative.clone(),
                scope,
            });
        }
    }
    Probe {
        closure,
        added,
        oppositions,
    }
}

fn append_evidence(
    report: &mut String,
    module: &Module,
    known: &Closure,
    probe: &Probe,
    event: &Event,
) {
    let location = if known.known.contains(event) {
        "Known"
    } else {
        "hypothetical delta"
    };
    writeln!(report, "    {location}: {}", render_event(event)).unwrap();
    let proofs = if known.known.contains(event) {
        &known.derivations
    } else {
        &probe.closure.derivations
    };
    let mut supported = false;
    for proof in proofs.iter().filter(|p| p.event == *event) {
        supported = true;
        let origin = program_rule_origin(module, &proof.record.rule);
        let source_rule = origin
            .as_ref()
            .map(|o| o.rule.as_str())
            .unwrap_or(&proof.record.rule);
        writeln!(
            report,
            "      rule {} | origin {} | derivedAt {} | {}",
            proof.record.rule,
            module.origin(source_rule).unwrap().module,
            proof.derived_at.name,
            render_substitution(&proof.record.substitution)
        )
        .unwrap();
        if let Some(origin) = origin {
            writeln!(
                report,
                "      program {} | module {} | source rule {}",
                origin.program, origin.defining_module, origin.rule
            )
            .unwrap();
        }
        for antecedent in &proof.record.antecedents {
            writeln!(report, "        witness: {}", render_event(antecedent)).unwrap();
        }
    }
    if !supported {
        writeln!(report, "      source evidence").unwrap();
    }
}

#[test]
fn hypothetical_defaults_observe_opposition_without_changing_selection() {
    let module = source();
    let unchanged = module.clone();
    let contexts = run_default_experiments(&module).unwrap();
    assert_eq!(contexts.len(), 9);
    let expected = [
        ("clearSale", None, None),
        ("opposedNormal", Some(OppositionScope::AgainstKnown), None),
        (
            "opposedRetentionUnpaid",
            Some(OppositionScope::AgainstKnown),
            None,
        ),
        (
            "opposedRetentionPaid",
            Some(OppositionScope::AgainstKnown),
            Some(OppositionScope::AgainstKnown),
        ),
        (
            "alreadyOpposed",
            Some(OppositionScope::AlreadyKnown),
            Some(OppositionScope::AlreadyKnown),
        ),
        ("differentAuthority", None, None),
        ("differentBeneficiary", None, None),
        ("differentTime", None, None),
        (
            "internalOpposition",
            Some(OppositionScope::WithinHypothesis),
            None,
        ),
    ];
    let mut report = String::from(
        "Experimental evidence opposition; consistency remains NotChecked.\nEach declared lambda is inspected independently, including unselected/inapplicable counterfactuals.\nscenario | candidate | applicable | selected (unchanged policy) | added events | already known | against Known | within hypothesis\n",
    );
    for (context, (name, normal, retention)) in contexts.iter().zip(expected) {
        assert_eq!(context.observation.experiment.name, name);
        assert_eq!(context.extension.consistency, ConsistencyStatus::NotChecked);
        let before = context.clone();
        for declaration in &module.declarations {
            let Declaration::Default(candidate) = declaration else {
                continue;
            };
            let observed = probe(&module, context, candidate);
            let wanted = if candidate.name == "NormalSale" {
                normal
            } else {
                retention
            };
            let wanted_count = if name == "alreadyOpposed" {
                2
            } else {
                usize::from(wanted.is_some())
            };
            assert_eq!(
                observed.oppositions.len(),
                wanted_count,
                "{name}: {}",
                candidate.name
            );
            if let Some(scope) = wanted {
                assert!(observed.oppositions.iter().all(|p| p.scope == scope));
            }
            assert!(
                context
                    .context
                    .known
                    .iter()
                    .all(|e| observed.closure.known.contains(e))
            );
            assert!(
                observed
                    .added
                    .iter()
                    .all(|e| !context.context.known.contains(e))
            );
            assert!(
                observed
                    .closure
                    .known
                    .iter()
                    .all(|e| e.verb != "inconsistent")
            );
            assert!(observed.closure.derivations.iter().all(|p| {
                !matches!(p.record.rule.as_str(), "normalSale" | "retentionSale")
                    || p.record.rule == target(candidate)
            }));
            for event in observed.closure.known.iter().filter(|e| e.verb == "Act") {
                assert!(
                    context.context.known.contains(event),
                    "no historical act is manufactured"
                );
            }
            let applicable = context
                .extension
                .applicable
                .iter()
                .any(|d| d.default.name == candidate.name);
            let selected = context
                .extension
                .selected
                .iter()
                .any(|d| d.name == candidate.name);
            let count = |scope| {
                observed
                    .oppositions
                    .iter()
                    .filter(|p| p.scope == scope)
                    .count()
            };
            writeln!(
                report,
                "{name} | {} | {applicable} | {selected} | {} | {} | {} | {}",
                candidate.name,
                observed.added.len(),
                count(OppositionScope::AlreadyKnown),
                count(OppositionScope::AgainstKnown),
                count(OppositionScope::WithinHypothesis)
            )
            .unwrap();
            for event in &observed.added {
                writeln!(report, "  added: {}", render_event(event)).unwrap();
            }
            for pair in &observed.oppositions {
                writeln!(report, "  opposition: {:?}", pair.scope).unwrap();
                append_evidence(
                    &mut report,
                    &module,
                    &context.context,
                    &observed,
                    &pair.positive,
                );
                append_evidence(
                    &mut report,
                    &module,
                    &context.context,
                    &observed,
                    &pair.negative,
                );
            }
        }
        assert_eq!(
            *context, before,
            "hypotheses do not mutate Known, provenance or selection"
        );
    }
    assert_eq!(module, unchanged);
    assert_eq!(run_default_experiments(&module).unwrap(), contexts);
    // Opposition is experimental data, not an additional rejection condition.
    let opposed = &contexts[1].extension;
    assert_eq!(opposed.selected[0].name, "NormalSale");
    assert!(opposed.blocked.is_empty() && opposed.defeated.is_empty());
    let paid = &contexts[3].extension;
    assert_eq!(paid.selected[0].name, "RetentionSale");
    assert!(paid.defeated.is_empty());
    assert_eq!(paid.blocked[0].default.name, "NormalSale");
    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/default-opposition-experiment.txt");
    std::fs::write(&output, &report).unwrap();
    println!("{report}\nReport: {}", output.display());
}

#[test]
fn hypothetical_oppositions_preserve_exact_event_keys_and_provenance() {
    let module = source();
    let contexts = run_default_experiments(&module).unwrap();
    let candidate = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Default(default) if default.name == "NormalSale" => Some(default),
            _ => None,
        })
        .unwrap();
    for name in ["opposedNormal", "alreadyOpposed", "internalOpposition"] {
        let context = contexts
            .iter()
            .find(|c| c.observation.experiment.name == name)
            .unwrap();
        let observed = probe(&module, context, candidate);
        let pair = observed
            .oppositions
            .iter()
            .find(|p| p.positive.verb == "obligation")
            .unwrap();
        assert_eq!(pair.positive.verb, "obligation");
        assert_eq!(pair.positive.args, pair.negative.args);
        assert_eq!(
            pair.positive.proposition.time,
            pair.negative.proposition.time
        );
        assert_eq!(pair.positive.polarity, Polarity::Positive);
        assert_eq!(pair.negative.polarity, Polarity::EvidentialNot);
        for (event, rule) in [
            (
                &pair.positive,
                if name == "alreadyOpposed" {
                    "filedObligation"
                } else {
                    "plainTransferObligation"
                },
            ),
            (
                &pair.negative,
                if name == "internalOpposition" {
                    "conditionalNegativeDuty"
                } else {
                    "recordNegativeFiledObligation"
                },
            ),
        ] {
            let closure = if context.context.known.contains(event) {
                &context.context
            } else {
                &observed.closure
            };
            let proof = closure
                .derivations
                .iter()
                .find(|p| {
                    p.event == *event
                        && program_rule_origin(&module, &p.record.rule)
                            .map(|o| o.rule)
                            .unwrap_or_else(|| p.record.rule.clone())
                            == rule
                })
                .unwrap();
            assert!(!proof.derived_at.name.is_empty());
            assert!(
                proof
                    .record
                    .antecedents
                    .iter()
                    .all(|w| closure.known.contains(w))
            );
            assert!(!proof.record.substitution.bindings.is_empty());
        }
    }
    for name in [
        "differentAuthority",
        "differentBeneficiary",
        "differentTime",
    ] {
        let context = contexts
            .iter()
            .find(|c| c.observation.experiment.name == name)
            .unwrap();
        let observed = probe(&module, context, candidate);
        assert!(observed.oppositions.is_empty());
        assert!(
            observed
                .added
                .iter()
                .any(|e| e.verb == "obligation" && e.polarity == Polarity::Positive)
        );
        assert!(
            context
                .context
                .known
                .iter()
                .any(|e| e.verb == "obligation" && e.polarity == Polarity::EvidentialNot)
        );
    }
}

#[test]
fn individually_compatible_defaults_reveal_joint_opposition_without_resolution() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let module = residuum::load_formal_module(
        root.join("examples/legal/modules/default_joint_opposition_scenario.res"),
    )
    .unwrap();
    let unchanged = module.clone();
    let results = run_default_experiments(&module).unwrap();
    assert_eq!(results.len(), 1);
    let context = &results[0];
    let before = context.clone();
    assert_eq!(context.context.known, context.observation.experiment.input);
    assert!(context.context.derivations.is_empty());
    assert_eq!(context.context.known.len(), 1);
    assert_eq!(context.extension.consistency, ConsistencyStatus::NotChecked);
    assert!(context.extension.blocked.is_empty());
    assert!(context.extension.defeated.is_empty());
    let selected: BTreeSet<_> = context
        .extension
        .selected
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert_eq!(selected, BTreeSet::from(["PositiveDuty", "NegativeDuty"]));
    let candidates: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Default(default) if selected.contains(default.name.as_str()) => {
                Some(default)
            }
            _ => None,
        })
        .collect();
    assert_eq!(candidates.len(), 2);

    let mut report = String::from(
        "Joint consistency experiment; selection remains unresolved by consistency (NotChecked).\nKnown is the mandatory closure, copied for each probe.\nprobe | activated default lambdas | new opposition against Known | new internal opposition\n",
    );
    for candidate in &candidates {
        let individual = probe(&module, context, candidate);
        assert!(
            individual.oppositions.is_empty(),
            "individual NoNewOpposition check passes"
        );
        let polarity = if candidate.name == "PositiveDuty" {
            Polarity::Positive
        } else {
            Polarity::EvidentialNot
        };
        assert!(
            individual
                .added
                .iter()
                .any(|e| e.verb == "obligation" && e.polarity == polarity)
        );
        assert!(
            individual
                .closure
                .known
                .iter()
                .all(|e| e.verb != "obligation" || e.polarity == polarity)
        );
        writeln!(
            report,
            "individual | {} -> {} | 0 | 0",
            candidate.name,
            target(candidate)
        )
        .unwrap();
    }

    let joint = probe_candidates(&module, context, &candidates);
    assert_eq!(joint.oppositions.len(), 1);
    let pair = &joint.oppositions[0];
    assert_eq!(pair.scope, OppositionScope::WithinHypothesis);
    assert_eq!(pair.positive, context.observation.experiment.goal);
    assert_eq!(pair.positive.polarity, Polarity::Positive);
    assert_eq!(pair.negative.polarity, Polarity::EvidentialNot);
    assert_eq!(pair.positive.args, pair.negative.args);
    assert_eq!(
        pair.positive.proposition.time,
        pair.negative.proposition.time
    );
    assert!(joint.added.contains(&pair.positive) && joint.added.contains(&pair.negative));
    assert!(
        joint
            .closure
            .known
            .iter()
            .all(|e| e.verb != "inconsistent" && e.verb != "Act")
    );
    let justifications: Vec<_> = joint
        .closure
        .derivations
        .iter()
        .filter(|p| p.event == pair.positive || p.event == pair.negative)
        .collect();
    assert_eq!(justifications.len(), 2);
    assert_eq!(
        justifications
            .iter()
            .map(|p| p.record.rule.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["positiveDuty", "negativeDuty"])
    );
    writeln!(report, "joint | PositiveDuty + NegativeDuty | 0 | 1").unwrap();
    for event in [&pair.positive, &pair.negative] {
        let proof = justifications.iter().find(|p| p.event == *event).unwrap();
        let contributors: Vec<_> = candidates
            .iter()
            .filter(|d| target(d) == proof.record.rule)
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(contributors.len(), 1);
        assert_eq!(proof.record.antecedents, context.context.known);
        assert_eq!(proof.record.substitution.bindings.len(), 8);
        for binding in ["id=duty", "subject=juan", "beneficiary=pedro", "t=tau"] {
            assert!(render_substitution(&proof.record.substitution).contains(binding));
        }
        writeln!(
            report,
            "  contributing default: {} -> {}",
            contributors[0], proof.record.rule
        )
        .unwrap();
        append_evidence(&mut report, &module, &context.context, &joint, event);
    }

    // The existing selector still retains both defaults and their consequences.
    // Compare complete proofs rather than assigning precedence to a first proof.
    let same_closure = |left: &Closure, right: &Closure| {
        assert!(super::extraction::equivalent_events(
            &left.known,
            &right.known
        ));
        for event in &left.known {
            assert!(right.known.iter().any(|e| e == event
                && e.proposition == event.proposition
                && e.transition == event.transition));
        }
        assert_eq!(left.derivations.len(), right.derivations.len());
        for proof in &left.derivations {
            let count = |closure: &Closure| {
                closure
                    .derivations
                    .iter()
                    .filter(|p| super::extraction::same_proof(proof, p))
                    .count()
            };
            assert_eq!(count(left), count(right));
        }
    };
    same_closure(&joint.closure, &context.observation.closure);
    // Reverse declaration order, including rule and default discovery order.
    // Stamps may differ, but no order may arbitrarily choose one default.
    let mut reordered = module.clone();
    reordered.declarations.reverse();
    let reversed = run_default_experiments(&reordered).unwrap().remove(0);
    assert_eq!(
        reversed
            .extension
            .selected
            .iter()
            .map(|d| d.name.as_str())
            .collect::<BTreeSet<_>>(),
        selected
    );
    assert!(reversed.extension.blocked.is_empty() && reversed.extension.defeated.is_empty());
    let reversed_candidates: Vec<_> = reordered
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Default(default) => Some(default),
            _ => None,
        })
        .collect();
    let reversed_joint = probe_candidates(&reordered, &reversed, &reversed_candidates);
    assert_eq!(reversed_joint.oppositions.len(), 1);
    assert_eq!(
        reversed_joint.oppositions[0].scope,
        OppositionScope::WithinHypothesis
    );
    same_closure(&joint.closure, &reversed_joint.closure);
    assert_eq!(*context, before);
    assert_eq!(module, unchanged);
    assert_eq!(run_default_experiments(&module).unwrap(), results);
    writeln!(report, "Both justifications retained; both defaults remain selected.\nblocked: none; defeated: none; consistency: NotChecked.\nReversed declaration order: same opposition and provenance.\nKnown unchanged: {} events, {} derivations.", context.context.known.len(), context.context.derivations.len()).unwrap();
    let output = root.join("target/default-joint-opposition-experiment.txt");
    std::fs::write(&output, &report).unwrap();
    println!("{report}\nReport: {}", output.display());
}
