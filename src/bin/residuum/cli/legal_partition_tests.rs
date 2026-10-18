//! Literal partition oracle. No inference or default policy is implemented here.
use super::*;
use residuum::surface::FormalDeclaration;
use residuum::typed::{Closure, DeclarationOrigin, Module};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const REFERENCE: &str = include_str!("../../../../examples/legal/modules/legal_monotonic.res");
const GENERAL: &str = include_str!("../../../../examples/legal/modules/legal_general.res");
const PATRIMONIAL: &str = include_str!("../../../../examples/legal/modules/legal_patrimonial.res");
const PRIVATE: [&str; 3] = ["deliver", "molest", "complianceAsset"];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/legal/modules")
        .canonicalize()
        .unwrap()
}

// Extract complete declaration text independently of AST equality. End-of-block
// blank lines and comments are outside declarations; bodies remain byte-exact.
fn literal_declarations(source: &str) -> BTreeMap<String, String> {
    let lines: Vec<_> = source.split_inclusive('\n').collect();
    let starts: Vec<_> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            [
                "entity ",
                "const ",
                "seeded verb ",
                "derived verb ",
                "effect verb ",
                "rule ",
            ]
            .iter()
            .find_map(|prefix| line.strip_prefix(prefix))
            .map(|rest| (i, rest.split([' ', '(', '\n']).next().unwrap().to_string()))
        })
        .collect();
    let mut result = BTreeMap::new();
    for (position, (start, name)) in starts.iter().enumerate() {
        let mut end = starts.get(position + 1).map_or(lines.len(), |s| s.0);
        while end > *start
            && (lines[end - 1].trim().is_empty() || lines[end - 1].trim_start().starts_with("//"))
        {
            end -= 1;
        }
        assert!(
            result
                .insert(name.clone(), lines[*start..end].concat())
                .is_none()
        );
    }
    result
}

#[test]
fn partition_inventory_preserves_every_literal_and_surface_declaration_once() {
    let original = residuum::parse_formal(REFERENCE).unwrap();
    let general = residuum::parse_formal(GENERAL).unwrap();
    let patrimonial = residuum::parse_formal(PATRIMONIAL).unwrap();
    assert_eq!(general.name, "LegalGeneral");
    assert_eq!(patrimonial.name, "LegalPatrimonial");
    assert_eq!(
        patrimonial.declarations.first(),
        Some(&FormalDeclaration::Import("./legal_general.res".into()))
    );
    let originals = literal_declarations(REFERENCE);
    let mut combined = literal_declarations(GENERAL);
    for (name, declaration) in literal_declarations(PATRIMONIAL) {
        assert!(
            combined.insert(name, declaration).is_none(),
            "duplicate declaration"
        );
    }
    assert_eq!(
        combined, originals,
        "literal copy, including signatures and temporal syntax"
    );
    let mut rule_counts = [0, 0];
    for (i, partition) in [&general, &patrimonial].into_iter().enumerate() {
        let actual: Vec<_> = partition
            .declarations
            .iter()
            .filter(|d| d.name().is_some())
            .collect();
        let expected: Vec<_> = original
            .declarations
            .iter()
            .filter(|d| PRIVATE.contains(&d.name().unwrap()) == (i == 1))
            .collect();
        assert_eq!(
            actual, expected,
            "preserve every declaration and its relative order"
        );
        rule_counts[i] = actual
            .iter()
            .filter(|d| matches!(d, FormalDeclaration::Rule(_)))
            .count();
    }
    assert_eq!(rule_counts, [27, 1]);
    assert_eq!(patrimonial.declarations.len(), 4); // one import, two constants, one rule
    let linked = residuum::load_formal_module(fixtures().join("legal_patrimonial.res")).unwrap();
    let before = residuum::load_formal_module(fixtures().join("legal_monotonic.res")).unwrap();
    assert_eq!(linked.declarations.len(), before.declarations.len());
    for declaration in &before.declarations {
        assert_eq!(
            linked
                .declarations
                .iter()
                .find(|d| d.name() == declaration.name()),
            Some(declaration)
        );
        let origin = linked.origin(declaration.name()).unwrap();
        let (module, file) = if PRIVATE.contains(&declaration.name()) {
            ("LegalPatrimonial", "legal_patrimonial.res")
        } else {
            ("LegalGeneral", "legal_general.res")
        };
        assert_eq!(origin.module, module);
        assert_eq!(origin.file.as_ref().unwrap(), &fixtures().join(file));
    }
    println!(
        "Inventory: {} declarations, 28 rules; 27 general + 1 patrimonial; literal and SurfaceAST equality",
        originals.len()
    );
}

/// Mirror existing consumers in a disposable graph. Only the old library import
/// is replaced by two real imports. Original fixtures are never overwritten.
struct PartitionGraph(PathBuf);
impl PartitionGraph {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "residuum-legal-partition-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        copy_partition_graph(&fixtures(), &path);
        Self(path.canonicalize().unwrap())
    }

    fn expected_origin(&self, name: &str, before: &DeclarationOrigin) -> DeclarationOrigin {
        let file = before.file.as_ref().unwrap();
        if before.module == "LegalMonotonicCase" {
            assert_eq!(file, &fixtures().join("legal_monotonic.res"));
            let (module, destination) = if PRIVATE.contains(&name) {
                ("LegalPatrimonial", "legal_patrimonial.res")
            } else {
                ("LegalGeneral", "legal_general.res")
            };
            DeclarationOrigin {
                module: module.into(),
                file: Some(self.0.join(destination)),
            }
        } else {
            // The physical temp prefix is checked explicitly, never discarded
            // together with the source module identity or relative file path.
            DeclarationOrigin {
                module: before.module.clone(),
                file: Some(self.0.join(file.strip_prefix(fixtures()).unwrap())),
            }
        }
    }
}
impl Drop for PartitionGraph {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn copy_partition_graph(source: &Path, target: &Path) {
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let destination = target.join(entry.file_name());
        if path.is_dir() {
            std::fs::create_dir(&destination).unwrap();
            copy_partition_graph(&path, &destination);
        } else if path.extension().is_some_and(|e| e == "res")
            && entry.file_name() != "legal_monotonic.res"
        {
            let text = std::fs::read_to_string(&path).unwrap();
            let mut variant = String::new();
            for line in text.split_inclusive('\n') {
                if let Some(relative) = line
                    .trim()
                    .strip_prefix("import \"")
                    .and_then(|s| s.strip_suffix('"'))
                {
                    if let Some(prefix) = relative.strip_suffix("legal_monotonic.res") {
                        variant.push_str(&format!("import \"{prefix}legal_general.res\"\nimport \"{prefix}legal_patrimonial.res\"\n"));
                        continue;
                    }
                }
                variant.push_str(line);
            }
            // No declarations or experiment data are changed in any consumer.
            if variant != text {
                let old = residuum::parse_formal(&text).unwrap();
                let new = residuum::parse_formal(&variant).unwrap();
                assert_eq!(old.name, new.name);
                let declarations = |m: residuum::surface::FormalModule| {
                    m.declarations
                        .into_iter()
                        .filter(|d| !matches!(d, FormalDeclaration::Import(_)))
                        .collect::<Vec<_>>()
                };
                assert_eq!(declarations(old), declarations(new));
            }
            std::fs::write(destination, variant).unwrap();
        }
    }
}

fn compare_modules(before: &Module, after: &Module, graph: &PartitionGraph) {
    assert_eq!(before.name, after.name);
    assert_eq!(before.declarations.len(), after.declarations.len());
    assert_eq!(before.origins.len(), after.origins.len());
    for declaration in &before.declarations {
        let name = declaration.name();
        let copies: Vec<_> = after
            .declarations
            .iter()
            .filter(|d| d.name() == name)
            .collect();
        assert_eq!(copies, [declaration], "typed declaration {name}");
        assert_eq!(
            after.origin(name).unwrap(),
            &graph.expected_origin(name, before.origin(name).unwrap()),
            "origin of {name}"
        );
    }
    assert!(
        after
            .origins
            .values()
            .all(|o| o.module != "LegalMonotonicCase")
    );
}

fn compare_closures(
    before: &Closure,
    after: &Closure,
    old: &Module,
    new: &Module,
    graph: &PartitionGraph,
) {
    // Same existing oracle: only the concrete derived_at label may differ.
    assert_eq!(before.known.len(), after.known.len());
    assert!(super::extraction::equivalent_events(
        &before.known,
        &after.known
    ));
    for event in &before.known {
        assert!(
            after.known.iter().any(|other| {
                event == other
                    && event.proposition == other.proposition
                    && event.transition == other.transition
            }),
            "event type/time/transition: {}",
            render_event(event)
        );
    }
    assert_eq!(before.derivations.len(), after.derivations.len());
    for proof in &before.derivations {
        let count = |closure: &Closure| {
            closure
                .derivations
                .iter()
                .filter(|p| super::extraction::same_proof(proof, p))
                .count()
        };
        assert_eq!(
            count(before),
            count(after),
            "proof multiplicity: {}",
            proof.record.rule
        );
        let producer = &proof.record.rule;
        if let Some(origin) = old.origin(producer) {
            assert_eq!(
                new.origin(producer).unwrap(),
                &graph.expected_origin(producer, origin)
            );
        } else {
            let a = residuum::programs::program_rule_origin(old, producer).unwrap();
            let b = residuum::programs::program_rule_origin(new, producer).unwrap();
            assert_eq!(a.program, b.program);
            assert_eq!(a.defining_module, b.defining_module);
            assert_eq!(a.rule, b.rule);
            assert_eq!(a.rule_use, b.rule_use);
            assert_eq!(
                b.rule_origin,
                graph.expected_origin(&a.rule, &a.rule_origin)
            );
        }
    }
}

fn compare_observation(
    a: &ExperimentResult,
    b: &ExperimentResult,
    old: &Module,
    new: &Module,
    graph: &PartitionGraph,
) {
    assert_eq!(a.experiment, b.experiment);
    for (x, y) in a
        .experiment
        .input
        .iter()
        .chain([&a.experiment.goal])
        .zip(b.experiment.input.iter().chain([&b.experiment.goal]))
    {
        assert_eq!(x.proposition, y.proposition);
        assert_eq!(x.transition, y.transition);
    }
    assert_eq!(a.horn_goal, b.horn_goal);
    assert_eq!(a.residual, b.residual);
    assert_eq!(a.four, b.four);
    compare_closures(&a.closure, &b.closure, old, new, graph);
}

fn equivalence(target: &str, consumer: &str, expected_experiments: usize) {
    let graph = PartitionGraph::new();
    let original_root = fixtures();
    let before_target = residuum::load_formal_module(original_root.join(target)).unwrap();
    let after_target = residuum::load_formal_module(graph.0.join(target)).unwrap();
    compare_modules(&before_target, &after_target, &graph);
    let before = residuum::load_formal_module(original_root.join(consumer)).unwrap();
    let after = residuum::load_formal_module(graph.0.join(consumer)).unwrap();
    compare_modules(&before, &after, &graph);
    let has_defaults = before
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Default(_)));
    let count = if has_defaults {
        let old = run_default_experiments(&before).unwrap();
        let new = run_default_experiments(&after).unwrap();
        assert_eq!(old.len(), new.len());
        for (a, b) in old.iter().zip(&new) {
            assert_eq!(
                a.extension, b.extension,
                "default policy, witnesses and selected programs"
            );
            compare_closures(&a.context, &b.context, &before, &after, &graph);
            compare_observation(&a.observation, &b.observation, &before, &after, &graph);
        }
        old.len()
    } else {
        let old = run_forward_experiments(&before).unwrap();
        let new = run_forward_experiments(&after).unwrap();
        assert_eq!(old.len(), new.len());
        for (a, b) in old.iter().zip(&new) {
            compare_observation(a, b, &before, &after, &graph);
        }
        old.len()
    };
    assert_eq!(count, expected_experiments);
    let report = format!(
        "{target} | consumer: {consumer} | {count} experiments | same events, types, times, rules, ordered witnesses, substitutions and proof multiplicities | origins: exact partition | derivedAt: operational\n"
    );
    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("legal-partition-{}.txt", target.replace('/', "_"))),
        &report,
    )
    .unwrap();
    println!("{report}");
}

#[test]
fn partition_scenario_preserves_all_sixteen_closures_and_provenance() {
    equivalence("scenario.res", "scenario.res", 16);
}
#[test]
fn partition_sales_baseline_preserves_declarations_and_consumer_closures() {
    equivalence("sales_monotonic_baseline.res", "scenario.res", 16);
}
#[test]
fn partition_sales_preserves_declarations_programs_and_default_closures() {
    equivalence("sales.res", "defaults_scenario.res", 8);
}
#[test]
fn partition_associated_program_preserves_bindings_and_activation_provenance() {
    equivalence("associated_program.res", "sells_scenario.res", 3);
}
#[test]
fn partition_defaults_scenario_preserves_selection_context_and_provenance() {
    equivalence("defaults_scenario.res", "defaults_scenario.res", 8);
}
#[test]
fn partition_joint_opposition_preserves_both_default_justifications() {
    equivalence(
        "default_joint_opposition_scenario.res",
        "default_joint_opposition_scenario.res",
        1,
    );
}
#[test]
fn partition_partial_normal_sale_preserves_beta_and_provenance() {
    equivalence(
        "default_comparison/partial_normal_sale_scenario.res",
        "default_comparison/partial_normal_sale_scenario.res",
        1,
    );
}

#[test]
fn general_only_keeps_liability_but_does_not_acquire_an_asset_by_non_transfer_compliance() {
    let module =
        residuum::load_formal_module(fixtures().join("partition_general_scenario.res")).unwrap();
    assert!(PRIVATE.iter().all(|name| module.origin(name).is_none()));
    assert!(
        module
            .origins
            .values()
            .all(|o| o.module != "LegalPatrimonial" && o.module != "LegalMonotonicCase")
    );
    let results = run_forward_experiments(&module).unwrap();
    assert_eq!(results.len(), 2);
    let negative = results
        .iter()
        .find(|r| r.experiment.name == "negativeUse")
        .unwrap();
    let fulfilled = results
        .iter()
        .find(|r| r.experiment.name == "fulfilledUse")
        .unwrap();
    assert!(negative.horn_goal);
    assert!(fulfilled.horn_goal);
    for rule in ["obligationViolation", "violationLiability"] {
        let proofs: Vec<_> = negative
            .closure
            .derivations
            .iter()
            .filter(|p| p.record.rule == rule)
            .collect();
        assert_eq!(proofs.len(), 1);
        assert_eq!(module.origin(rule).unwrap().module, "LegalGeneral");
    }
    fn verbs(result: &ExperimentResult) -> BTreeSet<&str> {
        result
            .closure
            .known
            .iter()
            .map(|e| e.verb.as_str())
            .collect()
    }
    assert_eq!(
        verbs(negative),
        BTreeSet::from([
            "Act",
            "obligationFiled",
            "obligation",
            "do",
            "violation",
            "patrimonialLiability"
        ])
    );
    assert_eq!(
        verbs(fulfilled),
        BTreeSet::from(["Act", "obligationFiled", "obligation", "do", "compliance"])
    );
    for result in &results {
        for event in result.closure.known.iter().filter(|e| e.verb == "Act") {
            assert!(result.experiment.input.contains(event));
        }
    }
    println!(
        "General only: obligation + ~Act => violation + patrimonialLiability; obligation + Act(use) => compliance, no patrimonialAsset/owns or transfer"
    );
}
