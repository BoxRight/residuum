//! Compare the extracted source program against the unchanged semantic oracle.
use super::*;
use residuum::typed::{Declaration, DerivedEvent, Event};
use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::Path;

const REFERENCE: &str = include_str!("../../../../examples/legal/case_monotonic_formal.res");

pub(super) fn equivalent_events(left: &[Event], right: &[Event]) -> bool {
    left.iter().cloned().collect::<BTreeSet<_>>() == right.iter().cloned().collect::<BTreeSet<_>>()
}

pub(super) fn same_proof(left: &DerivedEvent, right: &DerivedEvent) -> bool {
    // Specification §8: the external clock's labels are not a core law.
    // Ignore only derived_at, retaining the full event and ordered witnesses,
    // including operational metadata that Event's logical equality omits.
    let same_event = |a: &Event, b: &Event| {
        a == b && a.proposition == b.proposition && a.transition == b.transition
    };
    same_event(&left.event, &right.event)
        && left.record == right.record
        && left
            .record
            .antecedents
            .iter()
            .zip(&right.record.antecedents)
            .all(|(a, b)| same_event(a, b))
}

#[test]
fn complete_monotonic_extraction_preserves_all_sixteen_closures_and_provenance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let reference = residuum::parse_formal_to_typed(REFERENCE).unwrap();
    let modular =
        residuum::load_formal_module(root.join("examples/legal/modules/scenario.res")).unwrap();
    assert!(
        !modular
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Default(_)))
    );
    // Every original declaration is preserved, including all rule bodies and
    // the exact sixteen source-defined experiments. Resolution metadata differs.
    for declaration in &reference.declarations {
        assert_eq!(
            modular
                .declarations
                .iter()
                .find(|d| d.name() == declaration.name()),
            Some(declaration)
        );
    }
    let extras: Vec<_> = modular
        .declarations
        .iter()
        .filter(|d| !reference.declarations.contains(d))
        .map(|d| d.name())
        .collect();
    assert_eq!(
        extras,
        ["givingProgram", "sells", "selectedReciprocity"],
        "allow the associated Sales program and retain the Derived symbol without an automatic bridge"
    );
    for declaration in &modular.declarations {
        let origin = modular.origin(declaration.name()).unwrap();
        if let Declaration::Experiment(_) = declaration {
            assert_eq!(origin.module, "LegalMonotonicScenario");
        }
    }
    let before = run_forward_experiments(&reference).unwrap();
    let after = run_forward_experiments(&modular).unwrap();
    assert_eq!(before.len(), 16);
    assert_eq!(after.len(), 16);
    let mut report = String::from(
        "scenario | events | substitutions | proposition/state times | provenance | derivedAt (diagnostic only)\n",
    );
    let mut differences = Vec::new();
    for (old, new) in before.iter().zip(&after) {
        assert_eq!(old.experiment, new.experiment);
        let name = &old.experiment.name;
        let events = equivalent_events(&old.closure.known, &new.closure.known);
        let times = events
            && old.closure.known.iter().all(|a| {
                new.closure.known.iter().any(|b| {
                    a == b && a.proposition == b.proposition && a.transition == b.transition
                })
            });
        let substitutions = old.closure.derivations.len() == new.closure.derivations.len()
            && old.closure.derivations.iter().all(|a| {
                new.closure.derivations.iter().any(|b| {
                    a.event == b.event
                        && a.record.rule == b.record.rule
                        && a.record.antecedents == b.record.antecedents
                        && a.record.substitution == b.record.substitution
                })
            });
        let provenance = old.closure.derivations.len() == new.closure.derivations.len()
            && old.closure.derivations.iter().all(|a| {
                old.closure
                    .derivations
                    .iter()
                    .filter(|b| same_proof(a, b))
                    .count()
                    == new
                        .closure
                        .derivations
                        .iter()
                        .filter(|b| same_proof(a, b))
                        .count()
            });
        let stamps = provenance
            && old.closure.derivations.iter().all(|a| {
                new.closure
                    .derivations
                    .iter()
                    .any(|b| same_proof(a, b) && a.derived_at == b.derived_at)
            });
        writeln!(
            report,
            "{name} | {events} | {substitutions} | {times} | {provenance} | {stamps}"
        )
        .unwrap();
        if !(events && substitutions && times && provenance && old.horn_goal == new.horn_goal) {
            differences.push(name.clone());
        }
        // Independent executions may discover identical proofs in a different
        // order. Report their actual stamps without changing or equating them.
        if !stamps {
            for a in &old.closure.derivations {
                if let Some(b) = new.closure.derivations.iter().find(|b| same_proof(a, b)) {
                    if a.derived_at != b.derived_at {
                        writeln!(
                            report,
                            "  {}: original {}, modular {}",
                            a.record.rule, a.derived_at.name, b.derived_at.name
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
    let output = root.join("target/legal-monotonic-extraction-comparison.txt");
    std::fs::write(&output, &report).unwrap();
    println!("{report}\nReport: {}", output.display());
    assert!(
        differences.is_empty(),
        "extraction discrepancy in {differences:?}; classify it before changing any result"
    );
}

#[test]
fn sales_consumer_activates_imported_program_with_reference_provenance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("examples/legal/modules/sells_scenario.res");
    let module = residuum::load_formal_module(&path).unwrap();
    let inline =
        residuum::load_formal_module(root.join("examples/legal/modules/scenario_inline.res"))
            .unwrap();
    let actual = run_forward_experiments(&module).unwrap();
    let expected = residuum::experiments::run_default_experiments(&inline).unwrap();
    assert_eq!(actual.len(), 3);

    let sells = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == "sells" => Some(v),
            _ => None,
        })
        .unwrap();
    assert_eq!(module.origin("sells").unwrap().module, "Sales");
    let program = sells.program.as_ref().unwrap();
    assert_eq!(program.name, "givingProgram");
    assert_eq!(
        module.origin(&program.name).unwrap().module,
        "AssociatedGivingProgram"
    );
    assert_eq!(module.origin("give").unwrap().module, "LegalMonotonicCase");
    // The consumer defines only data and observations, no implementation rules.
    for declaration in &module.declarations {
        if module.origin(declaration.name()).unwrap().module == "SellsScenario" {
            assert!(matches!(
                declaration,
                Declaration::Const(_) | Declaration::Experiment(_)
            ));
        }
    }

    for result in actual.iter().take(2) {
        let reference = &expected
            .iter()
            .find(|r| r.observation.experiment.name == result.experiment.name)
            .unwrap()
            .observation;
        assert_eq!(result.experiment, reference.experiment);
        assert_eq!(result.horn_goal, reference.horn_goal);
        let left = &result.closure;
        let right = &reference.closure;
        assert!(equivalent_events(&left.known, &right.known));
        for a in &left.known {
            assert!(right.known.iter().any(|b| {
                a == b && a.proposition == b.proposition && a.transition == b.transition
            }));
        }
        assert_eq!(left.derivations.len(), right.derivations.len());
        for proof in &left.derivations {
            assert_eq!(
                left.derivations
                    .iter()
                    .filter(|p| same_proof(proof, p))
                    .count(),
                right
                    .derivations
                    .iter()
                    .filter(|p| same_proof(proof, p))
                    .count()
            );
        }
    }
    let real = &actual[0];
    assert!(real.horn_goal);
    assert_eq!(real.closure.derivations.len(), 1);
    let proof = &real.closure.derivations[0];
    assert_eq!(proof.record.rule, "givingProgram");
    assert_eq!(proof.record.antecedents, real.experiment.input);
    for binding in ["subject=juan", "object=car", "recipient=pedro", "t=tau"] {
        assert!(render_substitution(&proof.record.substitution).contains(binding));
    }
    assert!(actual[1].closure.known.is_empty());
    let negative = &actual[2];
    assert!(!negative.horn_goal);
    assert_eq!(negative.closure.known, negative.experiment.input);
    assert!(negative.closure.derivations.is_empty());
    // Derive effects without inventing historical acts or executing State.
    assert!(actual.iter().all(|r| {
        r.closure
            .known
            .iter()
            .all(|e| matches!(e.verb.as_str(), "sells" | "give"))
    }));
    let explanation = execute_file(Command::Explain, &path).unwrap();
    for text in [
        "verb origin: Sales (",
        "verb origin: LegalMonotonicCase (",
        "associated program: givingProgram | origin: AssociatedGivingProgram",
        "rule origin: AssociatedGivingProgram (",
    ] {
        assert!(explanation.contains(text), "{text}\n{explanation}");
    }
}
