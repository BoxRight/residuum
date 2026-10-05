//! Observations of the existing selector, not a new ambiguity/consistency policy.
use super::*;
use residuum::defaults::ConsistencyStatus;
use residuum::experiments::{DefaultExperimentResult, run_default_experiments};
use residuum::typed::{Closure, DefaultRef, Module, Polarity};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn load(file: &str) -> Module {
    legacy_rule_reference(
        residuum::load_formal_module(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples/legal/modules")
                .join(file),
        )
        .unwrap(),
    )
}

/// Historical Rule-level oracle, never used by the engine or native Program tests.
/// Recover the entry Rule from the declared first member, not a name registry.
pub(super) fn legacy_rule_reference(mut module: Module) -> Module {
    let entries: std::collections::BTreeMap<_, _> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Program(p) => p
                .rule_uses
                .first()
                .map(|u| (p.identity.clone(), u.rule.clone())),
            _ => None,
        })
        .collect();
    for declaration in &mut module.declarations {
        if let Declaration::Default(default) = declaration {
            default.body = match &default.body {
                residuum::typed::DefaultBody::SupernormalProgram { program } => {
                    residuum::typed::DefaultBody::Supernormal {
                        rule: entries[&program.name].clone(),
                    }
                }
                residuum::typed::DefaultBody::ExceptionProgram { program, to } => {
                    residuum::typed::DefaultBody::Exception {
                        rule: entries[&program.name].clone(),
                        to: to.clone(),
                    }
                }
                other => other.clone(),
            };
        }
    }
    module
        .declarations
        .retain(|d| !matches!(d, Declaration::Program(_)));
    module
}

fn names(items: &[DefaultRef]) -> BTreeSet<&str> {
    items.iter().map(|d| d.name.as_str()).collect()
}

pub(super) fn case<'a>(
    results: &'a [DefaultExperimentResult],
    name: &str,
) -> &'a DefaultExperimentResult {
    results
        .iter()
        .find(|r| r.observation.experiment.name == name)
        .unwrap()
}

pub(super) fn same_closure(left: &Closure, right: &Closure) {
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
}

fn append_proofs(report: &mut String, module: &Module, result: &DefaultExperimentResult) {
    for applicable in &result.extension.applicable {
        writeln!(report, "  applicable {}", applicable.default.name).unwrap();
        for matched in &applicable.matches {
            writeln!(report, "    {}", render_substitution(&matched.substitution)).unwrap();
            for event in &matched.antecedents {
                writeln!(report, "    witness: {}", render_event(event)).unwrap();
            }
        }
    }
    for proof in &result.observation.closure.derivations {
        writeln!(
            report,
            "  {} | rule {} | origin {} | {}",
            render_event(&proof.event),
            proof.record.rule,
            module.origin(&proof.record.rule).unwrap().module,
            render_substitution(&proof.record.substitution)
        )
        .unwrap();
        for event in &proof.record.antecedents {
            writeln!(report, "    witness: {}", render_event(event)).unwrap();
        }
    }
}

#[test]
fn modular_defaults_distinguish_applicability_priority_and_unordered_candidates() {
    let priority = load("default_comparison/priority_scenario.res");
    let unordered = load("default_comparison/unordered_scenario.res");
    let priority_results = run_default_experiments(&priority).unwrap();
    let unordered_results = run_default_experiments(&unordered).unwrap();
    assert_eq!(priority_results.len(), 3);
    assert_eq!(unordered_results.len(), 3);
    let mut report = String::from(
        "Existing Residuum selection; no new conflict policy.\nCatala comparison is documentary, not an execution of Catala.\nscenario | applicable | blocked | defeated | selected | packages | explicit opposition\n",
    );
    for (module, results) in [
        (&priority, &priority_results),
        (&unordered, &unordered_results),
    ] {
        assert_eq!(
            module.origin("Person").unwrap().module,
            "LegalMonotonicCase"
        );
        for rule in ["normalSale", "retentionSale"] {
            assert_eq!(module.origin(rule).unwrap().module, "Sales");
        }
        for result in results {
            let name = result.observation.experiment.name.as_str();
            let both = matches!(
                name,
                "explicitPriority"
                    | "noPriority"
                    | "opposedConsequences"
                    | "opposedNegativeConsequences"
            );
            let applicable: BTreeSet<_> = result
                .extension
                .applicable
                .iter()
                .map(|a| a.default.name.as_str())
                .collect();
            let expected = if both {
                BTreeSet::from(["NormalSale", "RetentionSale"])
            } else {
                BTreeSet::from(["NormalSale"])
            };
            assert_eq!(applicable, expected, "{name}");
            assert!(result.extension.blocked.is_empty());
            assert_eq!(
                names(&result.extension.defeated),
                if name == "explicitPriority" {
                    BTreeSet::from(["NormalSale"])
                } else {
                    BTreeSet::new()
                }
            );
            let selected = if name == "explicitPriority" {
                BTreeSet::from(["RetentionSale"])
            } else {
                expected
            };
            assert_eq!(names(&result.extension.selected), selected);
            assert_eq!(result.extension.consistency, ConsistencyStatus::NotChecked);
            // No controlled lambda has fired while computing applicability Known.
            assert_eq!(result.context.known, result.observation.experiment.input);
            assert!(result.context.derivations.is_empty());
            assert!(
                result
                    .context
                    .known
                    .iter()
                    .all(|e| e.verb != "selectedReciprocity")
            );
            let candidates: Vec<_> = module
                .declarations
                .iter()
                .filter_map(|d| match d {
                    Declaration::Default(d) if selected.contains(d.name.as_str()) => Some(d),
                    _ => None,
                })
                .collect();
            let probe = super::opposition::probe_candidates(module, result, &candidates);
            same_closure(&probe.closure, &result.observation.closure);
            let opposite_count = usize::from(matches!(
                name,
                "opposedConsequences" | "opposedNegativeConsequences"
            ));
            assert_eq!(probe.oppositions.len(), opposite_count, "{name}");
            let packages: Vec<_> = result
                .observation
                .closure
                .known
                .iter()
                .filter(|e| e.verb == "selectedReciprocity")
                .collect();
            let firing = if name == "emptyKnown" {
                0
            } else {
                selected.len()
            };
            assert_eq!(packages.len(), firing);
            assert_eq!(result.observation.horn_goal, name != "emptyKnown");
            assert_eq!(
                result
                    .observation
                    .closure
                    .known
                    .iter()
                    .any(|e| e.verb == "obligation"),
                matches!(
                    name,
                    "normalOnly"
                        | "noPriority"
                        | "opposedConsequences"
                        | "opposedNegativeConsequences"
                )
            );
            for (default, rule, package) in [
                ("NormalSale", "normalSale", "Plain"),
                ("RetentionSale", "retentionSale", "Retention"),
            ] {
                let proofs: Vec<_> = result
                    .observation
                    .closure
                    .derivations
                    .iter()
                    .filter(|p| p.record.rule == rule)
                    .collect();
                assert_eq!(
                    proofs.len(),
                    usize::from(selected.contains(default) && name != "emptyKnown")
                );
                if let Some(proof) = proofs.first() {
                    assert_eq!(
                        render_event(&proof.event),
                        format!("selectedReciprocity(saleCase, {package}) @ tau")
                    );
                    assert_eq!(proof.record.antecedents.len(), 1);
                    assert_eq!(proof.record.antecedents[0].verb, "sale");
                    for binding in ["saleId=saleCase", "seller=juan", "buyer=pedro", "t=tau"] {
                        assert!(render_substitution(&proof.record.substitution).contains(binding));
                    }
                }
            }
            // Both unprioritized packages support the same permission; retain
            // both witnesses, even though logical Event identity deduplicates it.
            assert_eq!(
                result
                    .observation
                    .closure
                    .derivations
                    .iter()
                    .filter(|p| p.record.rule == "reciprocalTransferPermission")
                    .count(),
                firing
            );
            assert_eq!(
                result
                    .observation
                    .closure
                    .known
                    .iter()
                    .filter(|e| e.verb == "permission")
                    .count(),
                usize::from(firing > 0)
            );
            writeln!(
                report,
                "{name} | {applicable:?} | none | {:?} | {selected:?} | {} | {opposite_count}",
                names(&result.extension.defeated),
                packages.len()
            )
            .unwrap();
            append_proofs(&mut report, module, result);
        }
    }
    // Applicability names coincide, but not its evidence: RetentionSale is
    // conditional in the priority fixture and unconditional in the other.
    // Removing priority while preserving that condition is not expressible.
    assert_eq!(
        case(&priority_results, "explicitPriority")
            .extension
            .applicable
            .iter()
            .map(|a| &a.default)
            .collect::<Vec<_>>(),
        case(&unordered_results, "noPriority")
            .extension
            .applicable
            .iter()
            .map(|a| &a.default)
            .collect::<Vec<_>>()
    );
    // Canonical SaleDefaults remains unchanged: blocking precedes priority.
    let canonical = load("defaults_scenario.res");
    let reference = run_default_experiments(&canonical).unwrap();
    let blocked = case(&reference, "retentionUnpaid");
    assert_eq!(blocked.extension.applicable.len(), 2);
    assert_eq!(blocked.extension.blocked.len(), 1);
    assert_eq!(blocked.extension.blocked[0].default.name, "NormalSale");
    assert!(blocked.extension.defeated.is_empty());
    assert_eq!(
        names(&blocked.extension.selected),
        BTreeSet::from(["RetentionSale"])
    );
    writeln!(report, "Canonical retentionUnpaid: NormalSale blocked, not defeated; RetentionSale selected.\nEmpty Known: NormalSale applicable and selected, with no matching sale and no derived package.\nCapability missing: conditional default without an exception edge (supernormal when is unsupported).\nNo applicable defaults is not expressible in a nonempty well-founded source default graph: its supernormal roots are unconditional. Zero surviving lambdas after blocking is a different state.\nUnordered variant uses two unconditional supernormals; this is not an isolated edge removal preserving RetentionSale's condition.\nMultiple selected lambdas is observable, not a built-in ambiguity/conflict status.\n").unwrap();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/catala-default-comparison.txt");
    std::fs::write(&output, &report).unwrap();
    println!("{report}\nReport: {}", output.display());
}

#[test]
fn unordered_sale_defaults_preserve_opposed_proofs_without_choosing_a_winner() {
    let module = load("default_comparison/unordered_scenario.res");
    let original = module.clone();
    let results = run_default_experiments(&module).unwrap();
    let result = case(&results, "opposedConsequences");
    let before = result.clone();
    let candidates: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Default(d) => Some(d),
            _ => None,
        })
        .collect();
    assert_eq!(candidates.len(), 2);
    for candidate in &candidates {
        let individual = super::opposition::probe_candidates(&module, result, &[*candidate]);
        assert!(individual.oppositions.is_empty());
    }
    let joint = super::opposition::probe_candidates(&module, result, &candidates);
    assert_eq!(joint.oppositions.len(), 1);
    let pair = &joint.oppositions[0];
    assert_eq!(
        pair.scope,
        super::opposition::OppositionScope::WithinHypothesis
    );
    assert_eq!(pair.positive.verb, "comparisonP");
    assert_eq!(pair.positive, result.observation.experiment.goal);
    assert_eq!(pair.negative.polarity, Polarity::EvidentialNot);
    assert!(joint.added.contains(&pair.positive) && joint.added.contains(&pair.negative));
    for (event, rule, package, root) in [
        (&pair.positive, "plainConsequence", "Plain", "normalSale"),
        (
            &pair.negative,
            "retentionConsequence",
            "Retention",
            "retentionSale",
        ),
    ] {
        let proofs: Vec<_> = joint
            .closure
            .derivations
            .iter()
            .filter(|p| p.event == *event)
            .collect();
        assert_eq!(proofs.len(), 1);
        assert_eq!(proofs[0].record.rule, rule);
        let witness = proofs[0]
            .record
            .antecedents
            .iter()
            .find(|e| e.verb == "selectedReciprocity")
            .unwrap();
        assert_eq!(
            render_event(witness),
            format!("selectedReciprocity(saleCase, {package}) @ tau")
        );
        let origin = joint
            .closure
            .derivations
            .iter()
            .find(|p| p.event == *witness)
            .unwrap();
        assert_eq!(origin.record.rule, root);
        assert_eq!(module.origin(root).unwrap().module, "Sales");
        assert_eq!(origin.record.antecedents[0].verb, "sale");
    }
    // Evidence opposition changes neither applicability nor selection.
    let nonopposed = case(&results, "noPriority");
    assert_eq!(result.extension, nonopposed.extension);
    assert!(result.extension.blocked.is_empty() && result.extension.defeated.is_empty());
    assert_eq!(result.extension.consistency, ConsistencyStatus::NotChecked);
    // Reverse rule/default declaration order: compare logical events and full
    // provenance, allowing only external clock stamps to differ.
    let mut reversed = module.clone();
    reversed.declarations.reverse();
    let rerun = run_default_experiments(&reversed).unwrap();
    let reordered = case(&rerun, "opposedConsequences");
    assert_eq!(
        names(&reordered.extension.selected),
        names(&result.extension.selected)
    );
    same_closure(&reordered.observation.closure, &result.observation.closure);
    assert_eq!(*result, before);
    assert_eq!(module, original);
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/legal/modules/default_comparison/unordered_scenario.res");
    let explanation = execute_file(Command::Explain, &file).unwrap();
    for rule in [
        "normalSale",
        "retentionSale",
        "plainConsequence",
        "retentionConsequence",
    ] {
        assert!(
            explanation.contains(&format!("rule {rule} |")),
            "{explanation}"
        );
    }
    assert!(explanation.contains("~comparisonP(saleCase) @ tau"));
}
