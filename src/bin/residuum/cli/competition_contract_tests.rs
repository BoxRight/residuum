//! Proposed competition contract only. No language syntax or selector policy.
//! Fixtures define actual applicability, priorities, lambdas and consequences.
use super::default_comparison::{case, load, same_closure};
use super::*;
use residuum::defaults::{ApplicableDefault, ConsistencyStatus, DefaultExtension};
use residuum::experiments::run_default_experiments;
use residuum::typed::{Default, DefaultBody, DefaultRef, DerivedEvent, Module};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Competes(DefaultRef, DefaultRef);

#[derive(Clone, Debug, PartialEq, Eq)]
struct CompetitionConflict {
    // Preserve both defaults and every applicability match; no selected loser.
    candidates: Vec<ApplicableDefault>,
    declaration: Competes,
}

#[derive(Debug, PartialEq, Eq)]
enum Observation {
    Selected(Vec<DefaultRef>),
    Conflict(CompetitionConflict),
}

fn default<'a>(module: &'a Module, reference: &DefaultRef) -> &'a Default {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Default(d) if d.name == reference.name => Some(d),
            _ => None,
        })
        .expect("experimental references must resolve to declared defaults")
}

fn references(module: &Module) -> Competes {
    // This is the explicit experiment declaration, not a universal inference
    // from having two defaults, shared consequences, or P/~P opposition.
    let pair = Competes(
        DefaultRef {
            name: "NormalSale".into(),
        },
        DefaultRef {
            name: "RetentionSale".into(),
        },
    );
    default(module, &pair.0);
    default(module, &pair.1);
    pair
}

fn observes_priority(module: &Module, higher: &DefaultRef, lower: &DefaultRef) -> bool {
    // Deliberately the direct, two-default fragment of this experiment only.
    // No new transitive or blocked-edge priority algorithm is introduced.
    matches!(&default(module, higher).body,
        DefaultBody::Exception { to, .. } if to == lower)
}

fn observe(
    module: &Module,
    extension: &DefaultExtension,
    competition: Option<&Competes>,
) -> Observation {
    if let Some(pair) = competition {
        default(module, &pair.0);
        default(module, &pair.1);
        let live = |reference: &DefaultRef| {
            extension.applicable.iter().any(|a| a.default == *reference)
                && !extension.blocked.iter().any(|b| b.default == *reference)
        };
        if live(&pair.0)
            && live(&pair.1)
            && !observes_priority(module, &pair.0, &pair.1)
            && !observes_priority(module, &pair.1, &pair.0)
        {
            let mut candidates: Vec<_> = extension
                .applicable
                .iter()
                .filter(|a| a.default == pair.0 || a.default == pair.1)
                .cloned()
                .collect();
            candidates.sort_by(|a, b| a.default.name.cmp(&b.default.name));
            let mut declaration = pair.clone();
            if declaration.0.name > declaration.1.name {
                declaration = Competes(declaration.1, declaration.0);
            }
            return Observation::Conflict(CompetitionConflict {
                candidates,
                declaration,
            });
        }
    }
    // Priority/blocking outcomes are taken from the existing selector.
    let mut selected = extension.selected.clone();
    selected.sort_by(|a, b| a.name.cmp(&b.name));
    Observation::Selected(selected)
}

#[test]
fn declared_competition_reports_conflict_without_changing_selection() {
    let mut report = String::from(
        "Experimental contract only; not implemented as Residuum selection semantics.\nD1=NormalSale; D2=RetentionSale.\ncompetes(D1,D2): symmetric, explicitly declared in this harness using resolved DefaultRef.\npriority(D1,D2): directional, read from existing exception syntax.\nscenario | actual selected | experimental observation\n",
    );
    for (file, name, conflict, both) in [
        (
            "default_comparison/priority_scenario.res",
            "normalOnly",
            false,
            false,
        ),
        (
            "default_comparison/competition_priority_scenario.res",
            "normalHasPriority",
            false,
            true,
        ),
        (
            "default_comparison/unordered_scenario.res",
            "noPriority",
            true,
            true,
        ),
    ] {
        let module = load(file);
        let original = module.clone();
        let results = run_default_experiments(&module).unwrap();
        let result = case(&results, name);
        let before = result.clone();
        let pair = references(&module);
        assert_eq!(result.extension.applicable.len(), if both { 2 } else { 1 });
        assert!(result.extension.blocked.is_empty());
        assert_eq!(result.extension.consistency, ConsistencyStatus::NotChecked);
        let observation = observe(&module, &result.extension, Some(&pair));
        let selected: BTreeSet<_> = result
            .extension
            .selected
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        if conflict {
            assert_eq!(selected, BTreeSet::from(["NormalSale", "RetentionSale"]));
            assert!(result.extension.defeated.is_empty());
            let Observation::Conflict(details) = &observation else {
                panic!("expected Conflict")
            };
            assert_eq!(details.candidates.len(), 2);
            assert!(details.candidates.iter().all(|c| !c.matches.is_empty()));
            assert_eq!(details.candidates, result.extension.applicable);
            // Withholding the explicit competition declaration gives no
            // Conflict, even though the same two defaults remain selected.
            assert!(matches!(observe(&module, &result.extension, None),
                Observation::Selected(ref defaults) if defaults.len() == 2));
            // Symmetry changes neither candidates nor their evidence.
            assert_eq!(
                observation,
                observe(
                    &module,
                    &result.extension,
                    Some(&Competes(pair.1.clone(), pair.0.clone()))
                )
            );
        } else {
            assert_eq!(selected, BTreeSet::from(["NormalSale"]));
            assert_eq!(observation, Observation::Selected(vec![pair.0.clone()]));
        }
        if name == "normalHasPriority" {
            assert!(observes_priority(&module, &pair.0, &pair.1));
            assert!(!observes_priority(&module, &pair.1, &pair.0));
            assert_eq!(result.extension.defeated, vec![pair.1.clone()]);
        }
        let candidates: Vec<_> = result
            .extension
            .selected
            .iter()
            .map(|r| default(&module, r))
            .collect();
        let hypothetical = super::opposition::probe_candidates(&module, result, &candidates);
        assert!(
            hypothetical.oppositions.is_empty(),
            "Conflict needs no P/~P"
        );
        same_closure(&hypothetical.closure, &result.observation.closure);
        writeln!(report, "{name} | {selected:?} | {observation:?}").unwrap();
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
            for witness in &proof.record.antecedents {
                writeln!(report, "    witness: {}", render_event(witness)).unwrap();
            }
        }
        // A competing unordered pair retains both program justifications.
        assert_eq!(
            result
                .observation
                .closure
                .derivations
                .iter()
                .filter(|p| p.event.verb == "selectedReciprocity")
                .count(),
            selected.len()
        );
        let mut reversed = module.clone();
        reversed.declarations.reverse();
        let reordered = run_default_experiments(&reversed).unwrap();
        let reordered = case(&reordered, name);
        assert_eq!(
            observation,
            observe(&reversed, &reordered.extension, Some(&pair))
        );
        same_closure(&result.observation.closure, &reordered.observation.closure);
        assert_eq!(*result, before);
        assert_eq!(module, original);
    }
    // Opposition alone is neither competition nor an instruction to resolve it.
    let opposed = load("default_comparison/unordered_scenario.res");
    let results = run_default_experiments(&opposed).unwrap();
    let result = case(&results, "opposedConsequences");
    assert!(matches!(observe(&opposed, &result.extension, None),
        Observation::Selected(ref defaults) if defaults.len() == 2));
    assert!(matches!(
        observe(&opposed, &result.extension, Some(&references(&opposed))),
        Observation::Conflict(_)
    ));
    writeln!(report, "P/~P without declared competition: Selected(both); with competition: Conflict.\nConflict is an observation only: no filtering of lambdas, no blocked/defeated mark, no mutation of Known.\nLimit: two-default, whole-lambda contract; not a general competition graph or ground-instance selector.\nNo source-level competes declaration yet: defaults are not reified terms.\n").unwrap();
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/default-competition-contract.txt");
    std::fs::write(&path, &report).unwrap();
    println!("{report}\nReport: {}", path.display());
}

#[derive(Debug)]
struct SlotHint {
    left: DefaultRef,
    right: DefaultRef,
    varying_argument: usize,
    left_proof: DerivedEvent,
    right_proof: DerivedEvent,
}

fn target_rule(candidate: &Default) -> &str {
    candidate.body.target_name()
}

fn slot_hints(
    module: &Module,
    context: &residuum::experiments::DefaultExperimentResult,
) -> Vec<SlotHint> {
    // A hypothesis about structure, NOT an inference of incompatibility.
    // Inspect individual unblocked candidates before priority selection.
    let mut candidates: Vec<_> = context
        .extension
        .applicable
        .iter()
        .filter(|a| {
            !context
                .extension
                .blocked
                .iter()
                .any(|b| b.default == a.default)
        })
        .map(|a| default(module, &a.default))
        .collect();
    candidates.sort_by(|a, b| a.name.cmp(&b.name));
    let probes: Vec<_> = candidates
        .iter()
        .map(|d| super::opposition::probe_candidates(module, context, &[*d]))
        .collect();
    let mut hints = Vec::new();
    for left in 0..candidates.len() {
        for right in left + 1..candidates.len() {
            // Attribute the direct consequence using actual derivation records,
            // not a table relating default names to a particular domain verb.
            for lp in probes[left]
                .closure
                .derivations
                .iter()
                .filter(|p| p.record.rule == target_rule(candidates[left]))
            {
                for rp in probes[right]
                    .closure
                    .derivations
                    .iter()
                    .filter(|p| p.record.rule == target_rule(candidates[right]))
                {
                    let a = &lp.event;
                    let b = &rp.event;
                    if a.verb != b.verb
                        || a.polarity != b.polarity
                        || a.proposition.time != b.proposition.time
                        || a.args.len() != b.args.len()
                    {
                        continue;
                    }
                    let different: Vec<_> = a
                        .args
                        .iter()
                        .zip(&b.args)
                        .enumerate()
                        .filter(|(_, (x, y))| x != y)
                        .map(|(index, _)| index)
                        .collect();
                    if different.len() == 1 {
                        hints.push(SlotHint {
                            left: DefaultRef {
                                name: candidates[left].name.clone(),
                            },
                            right: DefaultRef {
                                name: candidates[right].name.clone(),
                            },
                            varying_argument: different[0],
                            left_proof: lp.clone(),
                            right_proof: rp.clone(),
                        });
                    }
                }
            }
        }
    }
    hints
}

#[test]
fn inferred_slot_shape_does_not_establish_exclusive_competition() {
    let mut report = String::from(
        "Explicit competition vs inferred candidate slot: finite experiment only.\nShape heuristic: same predicate, polarity, proposition time and all but one argument.\nThe varying argument is a guessed value position, not a declared functional dependency.\nscenario | shape hints | evidential opposition | explicit competition | priority | outcome\n",
    );
    for (file, name, hint_count, opposition_count, explicit) in [
        (
            "default_comparison/priority_scenario.res",
            "normalOnly",
            0,
            0,
            true,
        ),
        (
            "default_comparison/competition_priority_scenario.res",
            "normalHasPriority",
            1,
            0,
            true,
        ),
        (
            "default_comparison/unordered_scenario.res",
            "noPriority",
            1,
            0,
            true,
        ),
        (
            "default_comparison/cumulative_rights_scenario.res",
            "cumulativeRights",
            1,
            0,
            false,
        ),
        (
            "default_joint_opposition_scenario.res",
            "jointCandidates",
            0,
            1,
            false,
        ),
    ] {
        let module = load(file);
        let original = module.clone();
        let results = run_default_experiments(&module).unwrap();
        let context = case(&results, name);
        let before = context.clone();
        let hints = slot_hints(&module, context);
        assert_eq!(hints.len(), hint_count, "{name}");
        let pair = if explicit {
            Some(references(&module))
        } else {
            None
        };
        let outcome = observe(&module, &context.extension, pair.as_ref());
        assert_eq!(
            matches!(outcome, Observation::Conflict(_)),
            name == "noPriority"
        );
        let active: Vec<_> = context
            .extension
            .selected
            .iter()
            .map(|r| default(&module, r))
            .collect();
        let joint = super::opposition::probe_candidates(&module, context, &active);
        assert_eq!(joint.oppositions.len(), opposition_count);
        same_closure(&joint.closure, &context.observation.closure);
        assert_eq!(context.extension.consistency, ConsistencyStatus::NotChecked);
        writeln!(
            report,
            "{name} | {} | {opposition_count} | {explicit} | {:?} | {outcome:?}",
            hints.len(),
            context.extension.defeated
        )
        .unwrap();
        for hint in &hints {
            writeln!(
                report,
                "  candidate slot: {} @ {:?}, key arguments except {}, defaults {} / {}",
                hint.left_proof.event.verb,
                hint.left_proof.event.proposition.time,
                hint.varying_argument,
                hint.left.name,
                hint.right.name
            )
            .unwrap();
            for proof in [&hint.left_proof, &hint.right_proof] {
                writeln!(
                    report,
                    "    {} | rule {} | origin {} | {}",
                    render_event(&proof.event),
                    proof.record.rule,
                    module.origin(&proof.record.rule).unwrap().module,
                    render_substitution(&proof.record.substitution)
                )
                .unwrap();
                for witness in &proof.record.antecedents {
                    writeln!(report, "      witness: {}", render_event(witness)).unwrap();
                }
            }
        }
        if name == "cumulativeRights" {
            // Structurally identical to a two-valued slot hint, yet two rights
            // are allowed. Treating every hint as competes yields a false positive.
            let hint = &hints[0];
            assert_eq!(hint.left_proof.event.verb, "entitledTo");
            assert_eq!(hint.varying_argument, 1);
            assert!(
                context
                    .observation
                    .closure
                    .known
                    .contains(&hint.left_proof.event)
            );
            assert!(
                context
                    .observation
                    .closure
                    .known
                    .contains(&hint.right_proof.event)
            );
            assert!(matches!(outcome, Observation::Selected(ref selected) if selected.len() == 2));
            let inferred_pair = Competes(hint.left.clone(), hint.right.clone());
            assert!(matches!(
                observe(&module, &context.extension, Some(&inferred_pair)),
                Observation::Conflict(_)
            ));
            // The exact same program/closure allows either experimental reading.
            // Exclusivity is extra semantic input, not proved by this shape.
            writeln!(report, "  false positive: promoting the slot hint to competition diagnoses Conflict for cumulative rights.\n  Same AST and Known; only the harness's exclusivity assumption differs.").unwrap();
        }
        let mut reversed = module.clone();
        reversed.declarations.reverse();
        let reversed_results = run_default_experiments(&reversed).unwrap();
        let reversed_context = case(&reversed_results, name);
        let reversed_hints = slot_hints(&reversed, reversed_context);
        assert_eq!(hints.len(), reversed_hints.len());
        for (a, b) in hints.iter().zip(&reversed_hints) {
            assert_eq!(a.left, b.left);
            assert_eq!(a.right, b.right);
            assert_eq!(a.varying_argument, b.varying_argument);
            assert!(super::extraction::same_proof(&a.left_proof, &b.left_proof));
            assert!(super::extraction::same_proof(
                &a.right_proof,
                &b.right_proof
            ));
        }
        assert_eq!(
            outcome,
            observe(&reversed, &reversed_context.extension, pair.as_ref())
        );
        same_closure(
            &context.observation.closure,
            &reversed_context.observation.closure,
        );
        assert_eq!(*context, before);
        assert_eq!(module, original);
    }
    writeln!(report, "Conclusion (limited): shape identifies candidate slots, not exclusivity.\nExisting predicates carry no declared key/value cardinality or incompatibility contract.\nAn exclusive slot could support inference without pairwise competes declarations, but that exclusivity is not present in today's AST.\nNo new syntax, selector changes, Known mutations or consistency policy.\n").unwrap();
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/default-slot-inference-comparison.txt");
    std::fs::write(&path, &report).unwrap();
    println!("{report}\nReport: {}", path.display());
}
