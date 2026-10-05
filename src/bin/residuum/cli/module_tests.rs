use super::*;
use residuum::experiments::run_default_experiments;
use residuum::typed::{Closure, Declaration, Polarity, Type, VerbKind};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/legal/modules")
        .join(name)
}

struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "residuum-imports-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, source).unwrap();
        path
    }
    fn error(&self, name: &str, source: &str) -> String {
        residuum::load_formal_module(self.write(name, source))
            .unwrap_err()
            .to_string()
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn assert_same_closure(left: &Closure, right: &Closure) {
    assert!(super::extraction::equivalent_events(
        &left.known,
        &right.known
    ));
    for a in &left.known {
        assert!(
            right.known.iter().any(|b| {
                a == b && a.proposition == b.proposition && a.transition == b.transition
            })
        );
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

#[test]
fn modular_sale_matches_inline_derivations_times_substitutions_and_provenance() {
    // Rule-level import oracle; native Program selection is checked separately.
    let modular = super::default_comparison::legacy_rule_reference(
        residuum::load_formal_module(fixture("defaults_scenario.res")).unwrap(),
    );
    let inline = residuum::load_formal_module(fixture("scenario_inline.res")).unwrap();
    // The reference ontology is now complete; the earlier inline oracle
    // remains a smaller program. Its declarations must stay unchanged.
    for declaration in &inline.declarations {
        assert!(
            modular.declarations.contains(declaration),
            "{}",
            declaration.name()
        );
    }
    assert_ne!(modular.origins, inline.origins);
    let results = run_default_experiments(&modular).unwrap();
    let expected = run_default_experiments(&inline).unwrap();
    assert_eq!(results.len(), expected.len());
    for (left, right) in results.iter().zip(&expected) {
        assert_eq!(left.extension, right.extension);
        assert_eq!(left.observation.experiment, right.observation.experiment);
        assert_eq!(left.observation.horn_goal, right.observation.horn_goal);
        assert_same_closure(&left.context, &right.context);
        assert_same_closure(&left.observation.closure, &right.observation.closure);
    }
    // Compare the original four scenarios too. Rule scheduling may change the
    // external clock stamps; logical times and complete justifications must not.
    let previous = cases_previous();
    assert_eq!(previous.len(), 4);
    let mut report = String::from(
        "scenario | selected | blocked | legal results/times/substitutions/provenance | derivedAt (diagnostic only)\n",
    );
    for (result, old) in results.iter().take(4).zip(previous) {
        assert_eq!(result.extension, old.extension);
        assert_same_closure(&result.context, &old.context);
        assert!(
            result
                .context
                .known
                .iter()
                .all(|e| e.verb != "selectedReciprocity")
        );
        assert!(
            result
                .context
                .derivations
                .iter()
                .all(|p| { !matches!(p.record.rule.as_str(), "normalSale" | "retentionSale") })
        );
        let current = &result.observation;
        assert_eq!(current.experiment, old.observation.experiment);
        assert_eq!(current.horn_goal, old.observation.horn_goal);
        assert_same_closure(&current.closure, &old.observation.closure);
        assert!(
            current
                .closure
                .known
                .iter()
                .all(|e| e.verb != "reciprocity")
        );
        for proof in &current.closure.derivations {
            if proof.event.verb == "selectedReciprocity" {
                assert!(matches!(
                    proof.record.rule.as_str(),
                    "normalSale" | "retentionSale"
                ));
                assert_eq!(modular.origin(&proof.record.rule).unwrap().module, "Sales");
            }
        }
        let chosen = &result.extension.selected[0].name;
        let normal = current.experiment.name == "noRetention";
        assert_eq!(
            chosen,
            if normal {
                "NormalSale"
            } else {
                "RetentionSale"
            }
        );
        if !normal {
            assert!(
                result
                    .extension
                    .blocked
                    .iter()
                    .any(|b| b.default.name == "NormalSale")
            );
            assert!(
                current
                    .closure
                    .derivations
                    .iter()
                    .all(|p| p.record.rule != "normalSale")
            );
        }
        let stamps = current.closure.derivations.iter().all(|p| {
            old.observation
                .closure
                .derivations
                .iter()
                .any(|q| super::extraction::same_proof(p, q) && p.derived_at == q.derived_at)
        });
        writeln!(
            report,
            "{} | {} | {} | equal | {}",
            current.experiment.name,
            chosen,
            if normal { "none" } else { "NormalSale" },
            stamps
        )
        .unwrap();
    }
    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/legal-sale-defaults-comparison.txt");
    std::fs::write(&output, &report).unwrap();
    println!("{report}\nReport: {}", output.display());
    for (name, owner) in [
        ("Person", "LegalMonotonicCase"),
        ("give", "LegalMonotonicCase"),
        ("obligationCreatesRelation", "LegalMonotonicCase"),
        ("sells", "Sales"),
        ("reciprocity", "Sales"),
        ("selectedReciprocity", "Sales"),
        ("normalSale", "Sales"),
        ("retentionSale", "Sales"),
        ("NormalSale", "SaleDefaults"),
        ("juan", "DefaultScenario"),
    ] {
        assert_eq!(modular.origin(name).unwrap().module, owner);
        assert!(modular.origin(name).unwrap().file.is_some());
    }
    for declaration in &modular.declarations {
        assert_eq!(
            modular
                .declarations
                .iter()
                .filter(|d| d.name() == declaration.name())
                .count(),
            1
        );
    }
    for (name, kind) in [
        ("reciprocity", VerbKind::Seeded),
        ("selectedReciprocity", VerbKind::Derived),
    ] {
        let v = modular
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Verb(v) if v.name == name => Some(v),
                _ => None,
            })
            .unwrap();
        assert_eq!(v.kind, kind);
    }
    let opposition = results
        .iter()
        .find(|r| r.observation.experiment.name == "suppliedOpposition")
        .unwrap();
    assert!(!opposition.observation.horn_goal);
    assert!(
        opposition
            .observation
            .closure
            .known
            .iter()
            .any(|e| e.verb == "Act" && e.polarity == Polarity::EvidentialNot)
    );
}

fn cases_previous() -> Vec<residuum::experiments::DefaultExperimentResult> {
    let source = include_str!("../../../../examples/legal/case_sale_defaults_formal.res");
    run_default_experiments(&residuum::parse_formal_to_typed(source).unwrap()).unwrap()
}

#[test]
fn importing_sale_defaults_only_loads_declarations_and_imported_lambdas() {
    let imported = residuum::load_formal_module(fixture("sale_defaults_import_only.res")).unwrap();
    let defaults = residuum::load_formal_module(fixture("sale_defaults.res")).unwrap();
    assert_eq!(imported.declarations, defaults.declarations);
    assert!(
        !imported
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Experiment(_)))
    );
    let mut count = 0;
    for declaration in &imported.declarations {
        let Declaration::Default(default) = declaration else {
            continue;
        };
        count += 1;
        assert_eq!(
            imported.origin(&default.name).unwrap().module,
            "SaleDefaults"
        );
        let reference = default.body.target_name();
        assert_eq!(imported.origin(reference).unwrap().module, "Sales");
        assert_eq!(
            imported
                .declarations
                .iter()
                .filter(|d| d.name() == reference)
                .count(),
            1
        );
    }
    assert_eq!(count, 2);
    assert!(imported.declarations.iter().all(|d| {
        !matches!(d, Declaration::Rule(_))
            || imported.origin(d.name()).unwrap().module != "SaleDefaults"
    }));
    assert_eq!(
        run_default_experiments(&imported).unwrap_err().to_string(),
        "module contains no experiments"
    );
}

#[test]
fn imported_verb_program_and_beta_reduction_keep_their_bound_targets() {
    let module = residuum::load_formal_module(fixture("defaults_scenario.res")).unwrap();
    let sells = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == "sells" => Some(v),
            _ => None,
        })
        .unwrap();
    assert_eq!(module.origin("sells").unwrap().module, "Sales");
    let reference = sells.program.as_ref().unwrap();
    assert_eq!(reference.name, "givingProgram");
    assert_eq!(
        module.origin(&reference.name).unwrap().module,
        "AssociatedGivingProgram"
    );
    let beta_module = residuum::load_formal_module(fixture("beta_scenario.res")).unwrap();
    let specialized = beta_module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == "juanSale" => Some(r),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        specialized
            .parameters
            .iter()
            .map(|a| &a.ty)
            .collect::<Vec<_>>(),
        vec![
            &Type::Entity("Thing".into()),
            &Type::Entity("Person".into()),
            &Type::PropositionTime
        ]
    );
    let results = run_default_experiments(&module).unwrap();
    let case = |name: &str| {
        &results
            .iter()
            .find(|r| r.observation.experiment.name == name)
            .unwrap()
            .observation
    };
    let real = case("sellsReal");
    let beta_results = run_forward_experiments(&beta_module).unwrap();
    let beta = &beta_results[0];
    assert!(real.horn_goal && beta.horn_goal);
    assert_eq!(real.experiment.goal, beta.experiment.goal);
    assert!(case("sellsAbsent").closure.known.is_empty());
    assert!(case("sellsAbsent").closure.derivations.is_empty());
    let proof = &real.closure.derivations[0];
    assert_eq!(proof.record.rule, "givingProgram");
    assert_eq!(proof.record.antecedents, real.experiment.input);
    for binding in ["subject=juan", "object=car", "recipient=pedro", "t=tau"] {
        assert!(render_substitution(&proof.record.substitution).contains(binding));
    }
    assert_eq!(beta.closure.derivations[0].record.rule, "juanSale");
    assert!(real.closure.known.iter().all(|e| e.verb != "Act"));
    let explanation = execute_file(Command::Explain, &fixture("defaults_scenario.res")).unwrap();
    for text in [
        "default origin: SaleDefaults",
        "lambda origin: Sales",
        "rule origin: Sales",
        "verb origin: LegalMonotonicCase",
        "associated program: givingProgram | origin: AssociatedGivingProgram",
    ] {
        assert!(explanation.contains(text), "{text}\n{explanation}");
    }
    assert_eq!(
        execute_file(Command::Run, &fixture("defaults_scenario.res"))
            .unwrap()
            .matches("experiment ")
            .count(),
        8
    );
}

#[test]
fn imported_experiments_are_local_and_diamond_imports_are_unique() {
    let files = Files::new();
    files.write("base.res", "module Base\nentity T\nconst a : T\nconst tau : PropositionTime\nseeded verb p(x : T)\nderived verb q(x : T)\nrule r(x,t) = p(x) @ t <= q(x) @ t\nexperiment local { seeds { p(a) @ tau, } query residual q(a) @ tau }");
    files.write(
        "left.res",
        "module Left\nimport \"./base.res\"\nentity LeftOnly",
    );
    files.write(
        "right.res",
        "module Right\nimport \"./base.res\"\nentity RightOnly",
    );
    let root = files.write("root.res", "module Root\nimport \"./left.res\"\nimport \"./right.res\"\nimport \"./base.res\"\nimport \"./base.res\"\nexperiment local { seeds {} query residual q(a) @ tau }");
    let module = residuum::load_formal_module(&root).unwrap();
    assert_eq!(
        module
            .declarations
            .iter()
            .filter(|d| d.name() == "r")
            .count(),
        1
    );
    assert_eq!(module.origin("r").unwrap().module, "Base");
    assert_eq!(module.origin("local").unwrap().module, "Root");
    let results = run_forward_experiments(&module).unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        results[0].closure.known.is_empty(),
        "importing does not assert library seeds"
    );
    assert!(!results[0].horn_goal);
    let only_import = files.write("only.res", "module Only\nimport \"./base.res\"");
    let only_import = residuum::load_formal_module(only_import).unwrap();
    assert!(
        !only_import
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Experiment(_)))
    );
    assert!(run_forward_experiments(&only_import).is_err());
}

#[test]
fn module_resolution_rejects_bad_paths_cycles_and_duplicate_names() {
    let files = Files::new();
    assert!(
        files
            .error("missing.res", "module Missing\nimport \"./absent.res\"")
            .contains("cannot resolve module")
    );
    assert!(
        files
            .error("empty.res", "module Empty\nimport \"\"")
            .contains("relative path")
    );
    assert!(
        files
            .error("absolute.res", "module Absolute\nimport \"/tmp/no.res\"")
            .contains("relative path")
    );
    assert!(
        files
            .error("self.res", "module Self\nimport \"./self.res\"")
            .contains("cyclic import")
    );
    files.write("a.res", "module A\nimport \"./b.res\"");
    assert!(
        files
            .error("b.res", "module B\nimport \"./a.res\"")
            .contains("cyclic import")
    );
    files.write("left.res", "module Left\nentity Shared");
    files.write("right.res", "module Right\nentity Shared");
    let error = files.error(
        "duplicate.res",
        "module Duplicate\nimport \"./left.res\"\nimport \"./right.res\"",
    );
    assert!(
        error.contains("duplicate symbol `Shared`")
            && error.contains("Left")
            && error.contains("Right")
    );
    assert!(
        files
            .error("local.res", "module Local\nentity X\nentity X")
            .contains("duplicate symbol `X`")
    );
    assert!(
        files
            .error(
                "shadow.res",
                "module Shadow\nimport \"./left.res\"\nentity Shared"
            )
            .contains("duplicate symbol `Shared`")
    );
    assert!(
        files
            .error("crosskind.res", "module CrossKind\nentity X\nconst X : X")
            .contains("duplicate symbol `X`")
    );
}

#[test]
fn module_elaboration_rejects_unknown_symbols_and_consumer_scope_leaks() {
    let files = Files::new();
    for (name, source, expected) in [
        (
            "type.res",
            "module Bad\nentity T { field : Missing }",
            "unknown entity `Missing`",
        ),
        (
            "constant_type.res",
            "module Bad\nconst a : Missing",
            "unknown entity `Missing`",
        ),
        (
            "verb.res",
            "module Bad\nrule r(x,t) = missing(x) @ t <= missing(x) @ t",
            "unknown verb `missing`",
        ),
        (
            "program.res",
            "module Bad\nentity T\nseeded verb p(x : T) = program absent",
            "unknown associated rule `absent`",
        ),
        (
            "application.res",
            "module Bad\nrule r = absent()",
            "unknown rule `absent`",
        ),
        (
            "value.res",
            "module Bad\nentity T\nconst tau : PropositionTime\nseeded verb p(x : T)\nexperiment test { seeds { p(absent) @ tau, } query residual p(absent) @ tau }",
            "unknown ground constant `absent`",
        ),
    ] {
        let error = files.error(name, source);
        assert!(error.contains(expected), "{expected}: {error}");
    }
    files.write(
        "dependency.res",
        "module Dependency\nentity T\nrule r(x,t) = consumerOnly(x) @ t <= consumerOnly(x) @ t",
    );
    let consumer = files.error(
        "consumer.res",
        "module Consumer\nimport \"./dependency.res\"\nseeded verb consumerOnly(x : T)",
    );
    assert!(consumer.contains("unknown verb `consumerOnly`"));
    assert!(
        residuum::parse_formal_to_typed("module M\nimport \"./base.res\"")
            .unwrap_err()
            .to_string()
            .contains("unresolved import")
    );
    assert!(residuum::parse_formal("module M\nimport base").is_err());
    assert!(residuum::parse_formal("module M\nimport \"unterminated").is_err());
}

#[test]
fn module_graph_keeps_the_existing_source_budget() {
    let files = Files::new();
    let large = format!("module Large\n//{}", "x".repeat(40 * 1024));
    files.write("large.res", &large);
    let root = format!(
        "module Root\nimport \"./large.res\"\n//{}",
        "x".repeat(30 * 1024)
    );
    assert!(files.error("root.res", &root).contains("64 KiB"));
}
