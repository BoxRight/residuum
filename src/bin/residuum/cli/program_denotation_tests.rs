//! Semantic interpretations over existing Rules; no new program AST or selector.
use super::default_comparison::{case, load, same_closure};
use super::*;
use residuum::experiments::run_default_experiments;
use residuum::typed::{
    Antecedent, Closure, DerivationTime, Module, Rule, RuleBody, SubstitutionBinding,
};
use residuum::{DerivationClock, close};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("denotation-{}", self.0),
        }
    }
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

fn common_rules(module: &Module) -> Vec<Rule> {
    let controlled: BTreeSet<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Default(d) => Some(d.body.target_name()),
            _ => None,
        })
        .collect();
    let associated: BTreeSet<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) => v.program.as_ref().map(|r| r.name.as_str()),
            _ => None,
        })
        .collect();
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r)
                if !controlled.contains(r.name.as_str())
                    && !associated.contains(r.name.as_str()) =>
            {
                Some(r.clone())
            }
            _ => None,
        })
        .collect()
}

fn run(input: &[Event], rules: &[Rule]) -> Closure {
    close(input.to_vec(), rules, &mut Clock::default())
}

fn merge_proofs(previous: &Closure, next: &mut Closure) {
    // Known alone does not transport the previous run's DerivationRecords.
    for proof in &previous.derivations {
        if !next
            .derivations
            .iter()
            .any(|p| super::extraction::same_proof(proof, p))
        {
            next.derivations.push(proof.clone());
        }
    }
}

fn reaches_root(
    event: &Event,
    root: &str,
    closure: &Closure,
    visiting: &mut BTreeSet<Event>,
) -> bool {
    if !visiting.insert(event.clone()) {
        return false;
    }
    let result = closure
        .derivations
        .iter()
        .filter(|p| &p.event == event)
        .any(|p| {
            p.record.rule == root
                || p.record
                    .antecedents
                    .iter()
                    .any(|a| reaches_root(a, root, closure, visiting))
        });
    visiting.remove(event);
    result
}

fn write_report(name: &str, report: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(name);
    std::fs::write(&path, report).unwrap();
    println!("{report}\nReport: {}", path.display());
}

#[test]
fn sale_entry_rules_denote_contextual_programs_without_a_compound_ast() {
    let module = load("defaults_scenario.res");
    let before = module.clone();
    let results = run_default_experiments(&module).unwrap();
    let inline = residuum::parse_formal_to_typed(include_str!(
        "../../../../examples/legal/case_sale_defaults_formal.res"
    ))
    .unwrap();
    let reference = run_default_experiments(&inline).unwrap();
    let common = common_rules(&module);
    let specialized = load("default_comparison/program_specialization.res");
    let mut report = String::from(
        "Program interpretation = selected entry Rule + explicit fixed common environment.\nscenario | entry | permission | obligation | do | same events/times/substitutions/provenance\n",
    );
    for (result, old) in results.iter().take(4).zip(&reference) {
        let input = &result.observation.experiment.input;
        assert!(input.iter().all(|e| e.verb != "sells"));
        let [entry] = result.extension.rules() else {
            panic!("one selected entry")
        };
        assert!(matches!(entry.body, RuleBody::HornClause(_)));
        let mut active = common.clone();
        active.push(entry.clone());
        let whole = run(input, &active);
        same_closure(&whole, &result.observation.closure);
        same_closure(&whole, &old.observation.closure);

        // This factoring is valid for these sale entries: common rules cannot
        // derive new Seeded sale inputs, so no feedback can re-enable an entry.
        let first = run(input, std::slice::from_ref(entry));
        let mut staged = run(&first.known, &common);
        merge_proofs(&first, &mut staged);
        same_closure(&whole, &staged);
        assert!(
            result
                .context
                .known
                .iter()
                .all(|e| e.verb != "selectedReciprocity")
        );

        let alias = if entry.name == "normalSale" {
            "normalForJuan"
        } else {
            "retentionForJuan"
        };
        let partial = rule(&specialized, alias);
        assert_eq!(partial.parameters, entry.parameters[2..]);
        let mut partial_rules = common.clone();
        partial_rules.push(partial.clone());
        let mut partial_closure = run(input, &partial_rules);
        assert!(super::extraction::equivalent_events(
            &whole.known,
            &partial_closure.known
        ));
        let entry_witness = whole
            .derivations
            .iter()
            .find(|p| p.record.rule == entry.name)
            .unwrap();
        let applied: Vec<SubstitutionBinding> = entry_witness
            .record
            .substitution
            .bindings
            .iter()
            .filter(|b| entry.parameters[..2].iter().any(|p| p.name == b.parameter))
            .cloned()
            .collect();
        assert_eq!(applied.len(), 2);
        for proof in &mut partial_closure.derivations {
            if proof.record.rule == alias {
                // Test-only correspondence: beta consumed these two binders.
                // Production provenance keeps the actual alias and residual match.
                assert!(
                    proof
                        .record
                        .substitution
                        .bindings
                        .iter()
                        .all(|b| !applied.iter().any(|a| a.parameter == b.parameter))
                );
                proof.record.rule = entry.name.clone();
                proof.record.substitution.bindings.extend(applied.clone());
                proof
                    .record
                    .substitution
                    .bindings
                    .sort_by(|a, b| a.parameter.cmp(&b.parameter));
            }
        }
        same_closure(&whole, &partial_closure);
        let has = |verb: &str| whole.known.iter().any(|e| e.verb == verb);
        assert!(has("permission"));
        assert_eq!(
            has("obligation"),
            matches!(
                result.observation.experiment.name.as_str(),
                "noRetention" | "retentionPaid"
            )
        );
        assert_eq!(has("do"), has("obligation"));
        writeln!(
            report,
            "{} | {} | {} | {} | {} | equal (beta provenance modulo explicit specialization)",
            result.observation.experiment.name,
            entry.name,
            has("permission"),
            has("obligation"),
            has("do")
        )
        .unwrap();
        for proof in &whole.derivations {
            let supported = proof.record.rule == entry.name
                || proof
                    .record
                    .antecedents
                    .iter()
                    .any(|a| reaches_root(a, &entry.name, &whole, &mut BTreeSet::new()));
            assert!(supported, "this scenario's consequences trace to its entry");
            writeln!(
                report,
                "  context program=({}, common imports), producing rule={} [{}], {}",
                entry.name,
                proof.record.rule,
                module.origin(&proof.record.rule).unwrap().module,
                render_event(&proof.event)
            )
            .unwrap();
        }
    }
    // A shared rule can also be enabled by an independent Seeded channel.
    // It has no intrinsic unique package owner in DerivationRecord.
    let supplied = case(&results, "suppliedPlain");
    let independent = supplied
        .observation
        .closure
        .derivations
        .iter()
        .find(|p| p.record.rule == "suppliedPlainObligation")
        .unwrap();
    assert!(independent.record.antecedents.iter().all(|a| !reaches_root(
        a,
        "normalSale",
        &supplied.observation.closure,
        &mut BTreeSet::new()
    )));
    assert_eq!(module, before);
    report.push_str("Independent suppliedPlainObligation proof has no normalSale ancestor; program labels require execution context, not a fabricated unique owner.\nKnown fed to a second close preserves events, but previous proofs must be retained by the harness. No DerivationRecord fields changed.\n");
    write_report("sale-program-denotation.txt", &report);
}

#[test]
fn fixed_sale_programs_are_monotone_on_a_finite_signed_evidence_universe() {
    let module = load("defaults_scenario.res");
    let results = run_default_experiments(&module).unwrap();
    let mut universe = case(&results, "retentionPaid")
        .observation
        .experiment
        .input
        .clone();
    let negative = case(&results, "suppliedOpposition")
        .observation
        .experiment
        .input
        .iter()
        .find(|e| e.verb == "Act" && e.polarity == residuum::typed::Polarity::EvidentialNot)
        .unwrap();
    universe.push(negative.clone());
    assert_eq!(universe.len(), 5);
    let common = common_rules(&module);
    let mut checks = 0;
    for entry in ["normalSale", "retentionSale"] {
        let mut active = common.clone();
        active.push(rule(&module, entry).clone());
        let closures: Vec<_> = (0usize..32)
            .map(|mask| {
                let input: Vec<_> = universe
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, e)| e.clone())
                    .collect();
                run(&input, &active)
            })
            .collect();
        for (a, left) in closures.iter().enumerate() {
            for (b, right) in closures.iter().enumerate() {
                if a & b == a {
                    assert!(
                        left.known.iter().all(|e| right.known.contains(e)),
                        "{entry}: {a} <= {b}"
                    );
                    for proof in &left.derivations {
                        assert!(
                            right
                                .derivations
                                .iter()
                                .any(|p| super::extraction::same_proof(proof, p))
                        );
                    }
                    checks += 1;
                }
            }
        }
    }
    assert_eq!(checks, 486);
    // Fixed program monotonicity is not monotonicity of default selection.
    let plain = case(&results, "noRetention");
    let retained = case(&results, "retentionUnpaid");
    assert!(
        plain.observation.experiment.input.iter().all(|e| retained
            .observation
            .experiment
            .input
            .contains(e))
    );
    let lost = plain
        .observation
        .closure
        .known
        .iter()
        .find(|e| e.verb == "obligation")
        .unwrap();
    assert!(!retained.observation.closure.known.contains(lost));
    write_report(
        "sale-program-monotonicity.txt",
        "MODEL RESULT: 64 fixed-program closures, 486 comparable input pairs; logical events and complete justifications persist, ignoring only external derivedAt.\nCounterexample for selection + closure: adding retentionAgreed switches entry and removes the normal obligation. This is not a claim that defaults are monotone.\n",
    );
}

#[test]
fn specializing_an_entry_does_not_beta_reduce_its_shared_environment() {
    let base = load("defaults_scenario.res");
    let module = load("default_comparison/program_specialization.res");
    let experiment = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == "specializationScope" => Some(e),
            _ => None,
        })
        .unwrap();
    let common = common_rules(&base);
    let mut entry_only = common.clone();
    entry_only.push(rule(&module, "normalForJuan").clone());
    let unscoped = run(&experiment.input, &entry_only);
    assert!(unscoped.known.contains(&experiment.goal));

    // This is a comparison of existing rule applications, not an implementation
    // of automatic capture, a compound lambda, or global parameter substitution.
    let mut scoped: Vec<_> = common
        .into_iter()
        .filter(|r| {
            !matches!(
                r.name.as_str(),
                "reciprocalTransferPermission" | "plainTransferObligation"
            )
        })
        .collect();
    for name in ["normalForJuan", "permissionForJuan", "obligationForJuan"] {
        scoped.push(rule(&module, name).clone());
    }
    let specialized_components = run(&experiment.input, &scoped);
    assert!(!specialized_components.known.contains(&experiment.goal));
    assert!(
        specialized_components
            .known
            .iter()
            .any(|e| e.verb == "obligation")
    );
    write_report(
        "sale-program-specialization-scope.txt",
        "Entry normalSale(saleCase,juan) emits only selectedReciprocity(saleCase,Plain).\nWith two sale inputs sharing saleCase and different sellers, the unspecialized shared environment also derives the second seller's obligation.\nExplicitly specializing the existing permission/obligation rules to the same prefix excludes that consequence.\nConclusion: beta works on each Rule, but entry specialization is not automatic beta over all related Rules. A shared binder/captured environment is not represented today; saleId uniqueness is not assumed.\n",
    );
}

#[test]
fn one_pass_program_composition_can_miss_feedback_into_the_first_program() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/experiments/rule_composition_feedback.res");
    let module = residuum::load_formal_module(path).unwrap();
    // The fixture also defines beta aliases. Compare their execution separately,
    // rather than enabling both definitions and aliases in the same baseline.
    let aliases = ["pAtTau", "qForItem", "feedbackForItem"];
    let mut baseline = module.clone();
    baseline
        .declarations
        .retain(|d| !matches!(d, Declaration::Rule(r) if aliases.contains(&r.name.as_str())));
    let result = run_forward_experiments(&baseline).unwrap().remove(0);
    let p = vec![
        rule(&module, "pEntry").clone(),
        rule(&module, "pFeedback").clone(),
        rule(&module, "observeCompletion").clone(),
    ];
    let q = vec![rule(&module, "qStep").clone()];
    let first = run(&result.experiment.input, &p);
    let mut second = run(&first.known, &q);
    merge_proofs(&first, &mut second);
    assert!(!second.known.contains(&result.experiment.goal));
    assert!(result.closure.known.contains(&result.experiment.goal));
    let mut third = run(&second.known, &p);
    merge_proofs(&second, &mut third);
    same_closure(&third, &result.closure);

    let tau = TimeExpr::At("tau".into());
    let after = |time: &TimeExpr| TimeExpr::After(Box::new(time.clone()));
    let one = after(&tau);
    let two = after(&one);
    let three = after(&two);
    for (verb, expected) in [
        ("start", &tau),
        ("intermediate", &one),
        ("returned", &two),
        ("completed", &three),
        ("completionObserved", &tau),
    ] {
        let events: Vec<_> = result
            .closure
            .known
            .iter()
            .filter(|e| e.verb == verb)
            .collect();
        assert_eq!(events.len(), 1);
        assert_eq!(&events[0].proposition.time, expected, "{verb}");
    }
    assert_ne!(one, two);
    assert_ne!(two, three);
    for (name, expected) in [
        ("pEntry", &tau),
        ("qStep", &one),
        ("pFeedback", &two),
        ("observeCompletion", &three),
    ] {
        let proof = result
            .closure
            .derivations
            .iter()
            .find(|p| p.record.rule == name)
            .unwrap();
        assert!(
            proof
                .record
                .substitution
                .bindings
                .contains(&SubstitutionBinding {
                    parameter: "t".into(),
                    value: SubstitutionValue::Time(expected.clone()),
                }),
            "{name}: preserve the entire matched temporal expression"
        );
    }

    for (base, alias, before, after_time) in [
        ("pEntry", "pAtTau", tau.clone(), one.clone()),
        (
            "qStep",
            "qForItem",
            TimeExpr::At("t".into()),
            after(&TimeExpr::At("t".into())),
        ),
        (
            "pFeedback",
            "feedbackForItem",
            TimeExpr::At("t".into()),
            after(&TimeExpr::At("t".into())),
        ),
    ] {
        let original = rule(&module, base);
        let partial = rule(&module, alias);
        assert_eq!(partial.parameters, original.parameters[1..]);
        let RuleBody::HornClause(clause) = &partial.body else {
            panic!("Horn alias")
        };
        let Antecedent::Event(input) = &clause.antecedent else {
            panic!("one antecedent")
        };
        assert_eq!(input.proposition.time, before);
        assert_eq!(clause.consequent.proposition.time, after_time);
        if alias == "pAtTau" {
            assert_eq!(partial.parameters[0].name, "object");
        } else {
            assert_eq!(partial.parameters[0].name, "t");
            assert_eq!(input.args, vec![Term::Const("item".into())]);
            assert_eq!(clause.consequent.args, vec![Term::Const("item".into())]);
        }
    }
    let beta_p = vec![
        rule(&module, "pAtTau").clone(),
        rule(&module, "feedbackForItem").clone(),
        rule(&module, "observeCompletion").clone(),
    ];
    let beta_q = vec![rule(&module, "qForItem").clone()];
    let beta_first = run(&result.experiment.input, &beta_p);
    let mut beta_second = run(&beta_first.known, &beta_q);
    merge_proofs(&beta_first, &mut beta_second);
    assert!(!beta_second.known.contains(&result.experiment.goal));
    let mut beta_third = run(&beta_second.known, &beta_p);
    merge_proofs(&beta_second, &mut beta_third);
    for proof in &mut beta_third.derivations {
        let correspondence = match proof.record.rule.as_str() {
            "pAtTau" => Some(("pEntry", "t", SubstitutionValue::Time(tau.clone()))),
            "qForItem" => Some((
                "qStep",
                "object",
                SubstitutionValue::Term(Term::Const("item".into())),
            )),
            "feedbackForItem" => Some((
                "pFeedback",
                "object",
                SubstitutionValue::Term(Term::Const("item".into())),
            )),
            _ => None,
        };
        if let Some((base, parameter, value)) = correspondence {
            // Compare provenance under the explicit beta correspondence only;
            // the engine records the alias and the remaining match unchanged.
            assert!(
                !proof
                    .record
                    .substitution
                    .bindings
                    .iter()
                    .any(|b| b.parameter == parameter)
            );
            proof.record.rule = base.into();
            proof
                .record
                .substitution
                .bindings
                .push(SubstitutionBinding {
                    parameter: parameter.into(),
                    value,
                });
            proof
                .record
                .substitution
                .bindings
                .sort_by(|a, b| a.parameter.cmp(&b.parameter));
        }
    }
    same_closure(&beta_third, &result.closure);
    write_report(
        "rule-program-composition-feedback.txt",
        "P={pEntry,pFeedback,observeCompletion}; Q={qStep}.\nSource temporal syntax: @ t / @ after t only; no parser extension.\nGeneric and beta-specialized composition produce:\n  start @ tau\n  intermediate @ after(tau)\n  returned @ after(after(tau))\n  completed @ after(after(after(tau)))\n  completionObserved @ tau (explicit observation rule, not a collapse of completed time).\nBeta pEntry(tau) substitutes time in both sides; qStep(item) and pFeedback(item) retain t as a remaining binder.\nRuntime matches t to after(tau) and after(after(tau)); the output substitution preserves the full nested structure.\nQ(P({start})) misses completed and the observation.\nP(Q(P({start}))) restores the joint closure in this fixture, including proofs modulo explicit beta correspondence and external derivedAt.\nComposition of monotone functions is monotone, but one pass is not generally the joint least fixpoint. Source expressibility is not a semantic bound on temporal depth.\n",
    );
}
