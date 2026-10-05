use super::*;
use residuum::defaults::{BlockingScope, ConsistencyStatus};
use residuum::experiments::{DefaultExperimentResult, run_default_experiments};
use residuum::programs::program_rule_origin;
use residuum::typed::{Closure, DefaultBody, Module, Polarity, PropositionKind, Rule};
use std::collections::BTreeSet;
use std::path::Path;

fn load(name: &str) -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/legal/modules")
            .join(name),
    )
    .unwrap()
}
fn rule<'a>(module: &'a Module, name: &str) -> &'a Rule {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == name => Some(r),
            _ => None,
        })
        .unwrap()
}
fn source_record(module: &Module, proof: &DerivedEvent) -> residuum::typed::DerivationRecord {
    let Some(origin) = program_rule_origin(module, &proof.record.rule) else {
        return proof.record.clone();
    };
    let program = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Program(p) if p.identity == origin.program => Some(p),
            _ => None,
        })
        .unwrap();
    let compiled = residuum::programs::compile_program(program, module).unwrap();
    let member = &compiled[origin.rule_use];
    // Check the stored hygienic substitution before restoring source names.
    // Replaying the source alone must not conceal malformed runtime provenance.
    let replay = residuum::instantiate_from_known(member, &proof.record.antecedents);
    assert!(
        replay
            .iter()
            .any(|p| p.event == proof.event && p.record == proof.record)
    );
    let candidates =
        residuum::instantiate_from_known(rule(module, &origin.rule), &proof.record.antecedents);
    let matching: Vec<_> = candidates
        .into_iter()
        .filter(|p| p.event == proof.event && p.record.antecedents == proof.record.antecedents)
        .collect();
    assert_eq!(matching.len(), 1);
    matching[0].record.clone()
}
fn equivalent(module: &Module, actual: &Closure, expected: &Closure) {
    assert_eq!(
        actual.known.iter().cloned().collect::<BTreeSet<_>>(),
        expected.known.iter().cloned().collect::<BTreeSet<_>>()
    );
    for event in &actual.known {
        let old = expected.known.iter().find(|e| *e == event).unwrap();
        assert_eq!(event.proposition, old.proposition);
        assert_eq!(event.transition, old.transition);
    }
    assert_eq!(
        actual.derivations.len(),
        expected.derivations.len(),
        "no provenance duplicates may be discarded"
    );
    let mut unmatched: Vec<_> = expected.derivations.iter().collect();
    for proof in &actual.derivations {
        let record = source_record(module, proof);
        let index = unmatched
            .iter()
            .position(|old| old.event == proof.event && source_record(module, old) == record)
            .expect("same source rule, witnesses and complete substitution after beta");
        let old = unmatched.remove(index);
        assert_eq!(old.event.proposition, proof.event.proposition);
        assert_eq!(old.event.transition, proof.event.transition);
    }
    assert!(unmatched.is_empty());
}
fn same_selection(actual: &DefaultExperimentResult, expected: &DefaultExperimentResult) {
    assert_eq!(actual.extension.applicable, expected.extension.applicable);
    assert_eq!(actual.extension.blocked, expected.extension.blocked);
    assert_eq!(actual.extension.defeated, expected.extension.defeated);
    assert_eq!(actual.extension.selected, expected.extension.selected);
    assert_eq!(actual.extension.consistency, expected.extension.consistency);
}

#[test]
fn program_defaults_preserve_four_sale_cases_and_complete_provenance() {
    let module = load("defaults_scenario.res");
    let expected_module = residuum::parse_formal_to_typed(include_str!(
        "../../../../examples/legal/case_sale_defaults_formal.res"
    ))
    .unwrap();
    let results = run_default_experiments(&module).unwrap();
    let expected = run_default_experiments(&expected_module).unwrap();
    for (actual, old) in results.iter().take(4).zip(&expected) {
        assert_eq!(actual.observation.experiment, old.observation.experiment);
        assert_eq!(actual.observation.horn_goal, old.observation.horn_goal);
        same_selection(actual, old);
        assert_eq!(
            actual.extension.blocking_scope,
            BlockingScope::ProgramLambda
        );
        assert_eq!(actual.extension.consistency, ConsistencyStatus::NotChecked);
        assert_eq!(actual.extension.programs().len(), 1);
        assert_eq!(actual.extension.rules().len(), 3);
        let target = if actual.observation.experiment.name == "noRetention" {
            "normalSaleProgram"
        } else {
            "retentionSaleProgram"
        };
        assert_eq!(actual.extension.programs()[0].name, target);
        assert_eq!(module.origin(target).unwrap().module, "Sales");
        equivalent(&module, &actual.context, &old.context);
        equivalent(
            &module,
            &actual.observation.closure,
            &old.observation.closure,
        );
        assert!(
            !actual
                .context
                .known
                .iter()
                .any(|e| e.verb == "selectedReciprocity"
                    || e.verb == "obligation"
                    || e.verb == "permission")
        );
        for proof in &actual.observation.closure.derivations {
            if let Some(origin) = program_rule_origin(&module, &proof.record.rule) {
                assert_eq!(origin.program, target);
                assert_eq!(origin.defining_module, "Sales");
                assert_eq!(origin.rule_origin.module, "Sales");
                assert_ne!(origin.rule, "obligationCreatesRelation");
            } else {
                assert_eq!(proof.record.rule, "obligationCreatesRelation");
                assert_eq!(
                    module.origin(&proof.record.rule).unwrap().module,
                    "LegalMonotonicCase"
                );
            }
        }
        assert!(
            !actual
                .observation
                .closure
                .known
                .iter()
                .any(|e| e.polarity == Polarity::EvidentialNot)
        );
    }
    let reciprocity = |name: &str| {
        module
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Verb(v) if v.name == name => Some(v.kind),
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(
        reciprocity("reciprocity"),
        residuum::typed::VerbKind::Seeded
    );
    assert_eq!(
        reciprocity("selectedReciprocity"),
        residuum::typed::VerbKind::Derived
    );
    assert!(
        results
            .iter()
            .take(4)
            .flat_map(|r| &r.observation.closure.known)
            .filter(|e| e.verb == "selectedReciprocity")
            .all(|e| e.proposition.kind == PropositionKind::Derived)
    );
    // Source-only wrong/negative payment variants must retain structural matching.
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/legal/modules/defaults_scenario.res");
    let text = std::fs::read_to_string(&path).unwrap();
    for bad in [
        "Act(pricePayment, pedro, pay, price, pedro, private, newDate)",
        "~Act(pricePayment, pedro, pay, price, juan, private, newDate)",
    ] {
        let temporary = path
            .parent()
            .unwrap()
            .join(format!(".program-payment-{}.res", std::process::id()));
        std::fs::write(
            &temporary,
            text.replace(
                "        Act(pricePayment, pedro, pay, price, juan, private, newDate)",
                &format!("        {bad}"),
            ),
        )
        .unwrap();
        let changed = residuum::load_formal_module(&temporary);
        std::fs::remove_file(&temporary).unwrap();
        let changed = run_default_experiments(&changed.unwrap()).unwrap();
        assert!(
            !changed
                .iter()
                .find(|r| r.observation.experiment.name == "retentionPaid")
                .unwrap()
                .observation
                .horn_goal
        );
    }
}

#[test]
fn default_programs_keep_general_relation_rule_in_environment_without_duplicate_provenance() {
    let module = load("program_environment_scenario.res");
    let results = run_default_experiments(&module).unwrap();
    for result in &results {
        assert!(result.observation.horn_goal);
        let proof = result
            .observation
            .closure
            .derivations
            .iter()
            .filter(|p| p.event == result.observation.experiment.goal)
            .collect::<Vec<_>>();
        assert_eq!(proof.len(), 1);
        assert_eq!(proof[0].record.rule, "obligationCreatesRelation");
        assert!(program_rule_origin(&module, &proof[0].record.rule).is_none());
        assert_eq!(
            module.origin(&proof[0].record.rule).unwrap().module,
            "LegalMonotonicCase"
        );
        if result.observation.experiment.name != "saleAndGeneralObligation" {
            assert!(
                result
                    .context
                    .known
                    .contains(&result.observation.experiment.goal)
            );
        }
    }
    for d in &module.declarations {
        if let Declaration::Program(p) = d {
            assert!(
                p.rule_uses
                    .iter()
                    .all(|u| u.rule.name != "obligationCreatesRelation")
            );
        }
    }
}

#[test]
fn importing_program_defaults_is_passive_and_explain_identifies_whole_selected_unit() {
    let imported = load("sale_defaults_import_only.res");
    assert!(
        !imported
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Experiment(_)))
    );
    let bodies: Vec<_> = imported
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Default(d) => Some(&d.body),
            _ => None,
        })
        .collect();
    assert!(
        matches!(bodies[0], DefaultBody::SupernormalProgram { program } if program.name == "normalSaleProgram")
    );
    assert!(
        matches!(bodies[1], DefaultBody::ExceptionProgram { program, to } if program.name == "retentionSaleProgram" && to.name == "NormalSale")
    );
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/legal/modules/defaults_scenario.res");
    let output = execute_file(Command::Explain, &path).unwrap();
    for text in [
        "default NormalSale -> normalSaleProgram: selected",
        "default NormalSale -> normalSaleProgram: blocked",
        "default RetentionSale -> retentionSaleProgram: selected",
        "selected programs: normalSaleProgram",
        "selected programs: retentionSaleProgram",
        "program normalSaleProgram | module Sales",
        "program retentionSaleProgram | module Sales",
    ] {
        assert!(output.contains(text), "missing {text}");
    }
    let environment = execute_file(
        Command::Explain,
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/legal/modules/program_environment_scenario.res"),
    )
    .unwrap();
    assert!(environment.contains("rule obligationCreatesRelation | derivedAt"));
    assert!(environment.contains("rule origin: LegalMonotonicCase"));
}

#[test]
fn program_exception_priority_selects_whole_unit_without_conflating_blocking() {
    let module = load("program_priority_scenario.res");
    let results = run_default_experiments(&module).unwrap();
    let actual = &results[0];
    assert!(actual.observation.horn_goal);
    assert_eq!(actual.extension.applicable.len(), 2);
    assert!(actual.extension.blocked.is_empty());
    assert_eq!(
        actual
            .extension
            .defeated
            .iter()
            .map(|d| d.name.as_str())
            .collect::<Vec<_>>(),
        vec!["NormalSale"]
    );
    assert_eq!(
        actual
            .extension
            .selected
            .iter()
            .map(|d| d.name.as_str())
            .collect::<Vec<_>>(),
        vec!["RetentionSale"]
    );
    assert_eq!(actual.extension.programs()[0].name, "retentionSaleProgram");
    assert_eq!(actual.extension.rules().len(), 3);
    assert_eq!(actual.extension.consistency, ConsistencyStatus::NotChecked);
    let reference = load("defaults_scenario.res");
    let expected = run_default_experiments(&reference).unwrap();
    let expected = expected
        .iter()
        .find(|r| r.observation.experiment.name == "normalBlocked")
        .unwrap();
    equivalent(&module, &actual.context, &expected.context);
    equivalent(
        &module,
        &actual.observation.closure,
        &expected.observation.closure,
    );
    assert!(!expected.extension.blocked.is_empty());
    assert!(expected.extension.defeated.is_empty());
    assert!(actual.observation.closure.derivations.iter().all(|p| {
        program_rule_origin(&module, &p.record.rule)
            .is_none_or(|o| o.program == "retentionSaleProgram")
    }));
}

#[test]
fn program_default_references_reject_unknown_targets_and_incompatible_signatures() {
    let source = include_str!("../../../../examples/experiments/associated_multi_rule_program.res");
    for (declarations, message) in [
        (
            "default Base = supernormal Missing",
            "unknown rule or program `Missing` in supernormal default",
        ),
        (
            "default Base = supernormal Chain\ndefault Wrong = exception Partial to Base",
            "incompatible parameter types for exception default `Wrong` and target `Base`",
        ),
    ] {
        let error =
            residuum::parse_formal_to_typed(&format!("{source}\n{declarations}\n")).unwrap_err();
        assert_eq!(error.to_string(), message);
    }
}

#[test]
fn selecting_one_program_keeps_satisfiable_members_of_the_other_inactive() {
    let module = load("program_isolation_scenario.res");
    let programs = ["normalSaleProgram", "retentionSaleProgram"].map(|name| {
        module
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Program(p) if p.identity == name => Some(p),
                _ => None,
            })
            .unwrap()
    });
    let members = programs.map(|p| {
        p.rule_uses
            .iter()
            .map(|u| u.rule.name.as_str())
            .collect::<BTreeSet<_>>()
    });
    assert_eq!(
        members[0]
            .intersection(&members[1])
            .copied()
            .collect::<Vec<_>>(),
        vec!["reciprocalTransferPermission"]
    );
    assert_ne!(members[0], members[1]);
    assert!(
        members
            .iter()
            .all(|m| !m.contains("obligationCreatesRelation"))
    );

    let results = run_default_experiments(&module).unwrap();
    assert_eq!(results.len(), 6);
    for (name, selected, unused_obligation, obligated) in [
        (
            "normalWithRetentionEvidence",
            "normalSaleProgram",
            "paidRetentionObligation",
            true,
        ),
        (
            "retentionUnpaidWithPlainEvidence",
            "retentionSaleProgram",
            "plainTransferObligation",
            false,
        ),
        (
            "retentionPaidWithPlainEvidence",
            "retentionSaleProgram",
            "plainTransferObligation",
            true,
        ),
    ] {
        let result = results
            .iter()
            .find(|r| r.observation.experiment.name == name)
            .unwrap();
        let closure = &result.observation.closure;
        assert_eq!(result.extension.programs().len(), 1);
        assert_eq!(result.extension.programs()[0].name, selected);
        assert_eq!(result.extension.rules().len(), 3);
        assert_eq!(result.extension.consistency, ConsistencyStatus::NotChecked);
        assert_eq!(result.observation.horn_goal, obligated, "{name}");
        assert!(
            !result
                .context
                .known
                .contains(&result.observation.experiment.goal)
        );
        assert!(!result.context.known.iter().any(|e| e.verb == "permission"));

        // These members WOULD fire if they leaked into the active rule set.
        // In the unpaid case they would even derive an otherwise absent norm.
        let witnesses =
            residuum::instantiate_from_known(rule(&module, unused_obligation), &closure.known);
        assert!(
            witnesses
                .iter()
                .any(|d| d.event == result.observation.experiment.goal),
            "{name}: unselected member must be enabled by evidence"
        );
        assert!(
            !residuum::instantiate_from_known(
                rule(&module, "reciprocalTransferPermission"),
                &closure.known
            )
            .is_empty()
        );

        for compiled in result.extension.rules() {
            assert_eq!(
                program_rule_origin(&module, &compiled.name)
                    .unwrap()
                    .program,
                selected
            );
        }
        for proof in &closure.derivations {
            if let Some(origin) = program_rule_origin(&module, &proof.record.rule) {
                assert_eq!(origin.program, selected);
                assert_eq!(origin.defining_module, "Sales");
                assert_eq!(origin.rule_origin.module, "Sales");
            } else {
                assert!(
                    !members
                        .iter()
                        .any(|m| m.contains(proof.record.rule.as_str())),
                    "raw member bypassed ProgramRef selection"
                );
            }
        }
        // Shared source member, differently captured in each Program: one proof.
        let permits: Vec<_> = closure
            .derivations
            .iter()
            .filter(|d| d.event.verb == "permission")
            .collect();
        assert_eq!(
            permits.len(),
            1,
            "{name}: no duplicate shared permission justification"
        );
        let origin = program_rule_origin(&module, &permits[0].record.rule).unwrap();
        assert_eq!(origin.program, selected);
        assert_eq!(origin.rule, "reciprocalTransferPermission");

        let norms = closure
            .derivations
            .iter()
            .filter(|d| d.event == result.observation.experiment.goal)
            .count();
        assert_eq!(
            norms,
            usize::from(obligated),
            "{name}: no duplicate alternative justification"
        );
        for conduct in ["deliver", "transfer"] {
            let effects: Vec<_> = closure
                .derivations
                .iter()
                .filter(|d| d.event.verb == "do" && d.event.args[1] == Term::Const(conduct.into()))
                .collect();
            assert_eq!(
                effects.len(),
                usize::from(conduct == "deliver" || obligated)
            );
            for proof in effects {
                assert_eq!(proof.record.rule, "obligationCreatesRelation");
                assert_eq!(
                    module.origin(&proof.record.rule).unwrap().module,
                    "LegalMonotonicCase"
                );
            }
        }
        for seed in &result.observation.experiment.input {
            assert!(closure.known.contains(seed));
        }
        assert!(
            closure
                .known
                .iter()
                .filter(|e| e.verb == "Act")
                .all(|e| result.observation.experiment.input.contains(e))
        );
    }
}

#[test]
fn evidence_for_unselected_program_adds_only_its_explicit_probe_provenance() {
    let module = load("program_isolation_scenario.res");
    let results = run_default_experiments(&module).unwrap();
    for (control, challenge) in [
        ("normalControl", "normalWithRetentionEvidence"),
        ("retentionUnpaidControl", "retentionUnpaidWithPlainEvidence"),
        ("retentionPaidControl", "retentionPaidWithPlainEvidence"),
    ] {
        let control = results
            .iter()
            .find(|r| r.observation.experiment.name == control)
            .unwrap();
        let challenge = results
            .iter()
            .find(|r| r.observation.experiment.name == challenge)
            .unwrap();
        same_selection(challenge, control);
        assert_eq!(challenge.extension.programs(), control.extension.programs());
        let added: BTreeSet<_> = challenge
            .context
            .known
            .iter()
            .filter(|e| !control.context.known.contains(e))
            .cloned()
            .collect();
        assert_eq!(added.len(), 2);
        assert_eq!(
            added
                .iter()
                .map(|e| e.verb.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["testPackageEvidence", "selectedReciprocity"])
        );
        let mut projected = challenge.observation.closure.clone();
        projected.known.retain(|e| !added.contains(e));
        let probe: Vec<_> = projected
            .derivations
            .iter()
            .filter(|d| added.contains(&d.event))
            .collect();
        assert_eq!(probe.len(), 1);
        assert_eq!(probe[0].record.rule, "testExposePackage");
        assert_eq!(
            module.origin(&probe[0].record.rule).unwrap().module,
            "ProgramIsolationScenario"
        );
        projected.derivations.retain(|d| !added.contains(&d.event));
        // Projection is only a comparison of experiment results. The real Known
        // retains both intermediate facts; there is no retraction or preference.
        equivalent(&module, &projected, &control.observation.closure);
        assert!(
            added
                .iter()
                .all(|e| challenge.observation.closure.known.contains(e))
        );
    }
}
