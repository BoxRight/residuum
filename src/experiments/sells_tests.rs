//! All declarations, rules, seeds and goals come from the Formal fixture.
//! Rust verifies two uses of the same program; it defines no sells semantics.

use super::*;
use crate::residual::{AbductionFailure, ResidualProof};
use crate::typed::{DerivationRecord, PropositionKind, StateTime, SubstitutionValue};
use crate::{IncrementalClosure, parse_formal, parse_formal_to_typed};

const SOURCE: &str = include_str!("../../examples/experiments/sells_program.res");

fn rules(module: &Module) -> Vec<Rule> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(rule) => Some(rule.clone()),
            _ => None,
        })
        .collect()
}

fn result<'a>(results: &'a [ExperimentResult], name: &str) -> &'a ExperimentResult {
    results.iter().find(|r| r.experiment.name == name).unwrap()
}

fn events(antecedent: &Antecedent) -> Vec<Event> {
    match antecedent {
        Antecedent::Unit => vec![],
        Antecedent::Event(event) => vec![event.clone()],
        Antecedent::And(a, b) => {
            let mut result = events(a);
            result.extend(events(b));
            result
        }
    }
}

fn keys(closure: &Closure) -> BTreeSet<Event> {
    closure.known.iter().cloned().collect()
}

fn exact_event(a: &Event, b: &Event) {
    assert_eq!(a, b);
    // Logical Event equality intentionally ignores operational metadata.
    assert_eq!(a.proposition, b.proposition);
    assert_eq!(a.transition, b.transition);
}

fn equivalent(a: &Closure, b: &Closure) {
    assert_eq!(keys(a), keys(b));
    for event in &a.known {
        exact_event(event, b.known.iter().find(|e| *e == event).unwrap());
    }
    assert_eq!(a.derivations.len(), b.derivations.len());
    for proof in &a.derivations {
        let other = b
            .derivations
            .iter()
            .find(|p| p.event == proof.event && p.record == proof.record)
            .unwrap();
        exact_event(&proof.event, &other.event);
        assert_eq!(proof.record.substitution, other.record.substitution);
        for (x, y) in proof
            .record
            .antecedents
            .iter()
            .zip(&other.record.antecedents)
        {
            exact_event(x, y);
        }
        // Clock stamps are external. Provenance identity is the exact record,
        // with no rule renaming or alias map required between these two paths.
    }
}

struct HypotheticalClock(usize);
impl DerivationClock for HypotheticalClock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("hypothetical-{}", self.0),
        }
    }
}

#[test]
fn sells_real_and_abduced_seed_produce_identical_program_consequences() {
    assert_eq!(
        parse_formal(SOURCE).unwrap(),
        crate::formal::parse_formal_manual(SOURCE).unwrap()
    );
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let original = module.clone();
    let results = run_experiments(&module).unwrap();
    let real = result(&results, "real");
    let absent = result(&results, "hypothetical");
    assert!(real.horn_goal);
    assert!(!absent.horn_goal);
    assert_eq!(real.experiment.goal, absent.experiment.goal);
    assert_eq!(real.experiment.input.len(), 1);
    assert!(absent.experiment.input.is_empty());
    let seed = &real.experiment.input[0];
    assert_eq!(seed.verb, "sells");
    assert_eq!(seed.proposition.kind, PropositionKind::Seeded);
    assert!(module.declarations.iter().any(|d| matches!(d,
        Declaration::Verb(v) if v.name == seed.verb && v.is_abducible())));
    let query = absent.residual.as_ref().unwrap();
    assert_eq!(
        query.required,
        Some(RequirementFormula::Event(seed.clone()))
    );
    assert_eq!(query.failure, None);
    assert_eq!(query.explanations.len(), 2);
    assert_eq!(
        real.residual.as_ref().unwrap().required,
        Some(RequirementFormula::Unit)
    );
    // The query runner closes only the supplied evidence, not its hypotheses.
    assert!(absent.closure.known.is_empty());
    assert!(absent.closure.derivations.is_empty());

    for explanation in &query.explanations {
        let hypothesis = events(&explanation.required);
        assert_eq!(hypothesis, real.experiment.input);
        let ResidualProof::Rule {
            rule,
            goal,
            substitution,
            antecedent,
        } = &explanation.proof
        else {
            panic!("a rule-backed residual proof is required");
        };
        exact_event(goal, &real.experiment.goal);
        let ResidualProof::Hypothesis(h) = antecedent.as_ref() else {
            panic!("missing sells is a hypothesis, never evidence");
        };
        exact_event(h, seed);
        let mut clock = HypotheticalClock(0);
        let hypothetical = close(hypothesis, &rules(&module), &mut clock);
        equivalent(&real.closure, &hypothetical);
        let forward = hypothetical
            .derivations
            .iter()
            .find(|d| d.record.rule == rule.name && d.event == *goal)
            .unwrap();
        assert_eq!(*substitution, forward.record.substitution);
        assert_eq!(forward.record.antecedents, [seed.clone()]);
        assert!(
            hypothetical
                .derivations
                .iter()
                .all(|d| d.derived_at.name.starts_with("hypothetical-"))
        );
    }
    assert_eq!(
        real.closure
            .known
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        ["sells", "give", "owns", "notDo"]
    );
    assert_eq!(real.closure.derivations.len(), 4);
    let transfer = real
        .closure
        .known
        .iter()
        .find(|e| e.verb == "give")
        .unwrap();
    let ownership = real
        .closure
        .known
        .iter()
        .find(|e| e.verb == "owns")
        .unwrap();
    let protection = real
        .closure
        .known
        .iter()
        .find(|e| e.verb == "notDo")
        .unwrap();
    let after_sale = TimeExpr::After(Box::new(seed.proposition.time.clone()));
    assert_eq!(transfer.proposition.time, after_sale);
    assert_eq!(ownership.proposition.time, after_sale);
    assert_eq!(
        protection.proposition.time,
        TimeExpr::After(Box::new(after_sale.clone()))
    );
    assert_eq!(transfer.proposition.kind, PropositionKind::Effect);
    let transition = transfer.transition.as_ref().unwrap();
    assert_eq!(
        transition.input,
        StateTime::At(seed.proposition.time.clone())
    );
    assert_eq!(transition.output, StateTime::At(after_sale));
    assert_eq!(ownership.args, [seed.args[2].clone(), seed.args[1].clone()]);
    assert_eq!(
        protection.args[2..],
        [seed.args[1].clone(), seed.args[2].clone()]
    );
    println!(
        "{} -> {}",
        render_event(seed),
        render_requirement(query.required.as_ref().unwrap())
    );
    for proof in &real.closure.derivations {
        println!(
            "{} via {}: {:?}",
            render_event(&proof.event),
            proof.record.rule,
            proof.record.substitution
        );
    }
    assert_eq!(module, original);
    assert!(absent.closure.known.is_empty());
}

#[test]
fn sells_beta_specialization_and_reuse_depend_on_fixture_rules_only() {
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let typed_rules = rules(&module);
    let generic = typed_rules
        .iter()
        .find(|r| r.name == "sellsProgram")
        .unwrap();
    let applied = typed_rules.iter().find(|r| r.name == "juanSale").unwrap();
    assert_eq!(applied.parameters, generic.parameters[1..]);
    let inline = SOURCE.replace("rule juanSale = sellsProgram(juan)",
        "rule juanSale(object, buyer, t) = sells(juan, object, buyer) @ t <= give(juan, object, buyer) @ after t");
    let direct = parse_formal_to_typed(&inline).unwrap();
    assert_eq!(
        applied,
        rules(&direct)
            .iter()
            .find(|r| r.name == "juanSale")
            .unwrap()
    );
    let results = run_experiments(&module).unwrap();
    let inline_results = run_experiments(&direct).unwrap();
    for (actual, inline) in results.iter().zip(&inline_results) {
        equivalent(&actual.closure, &inline.closure);
        assert_eq!(actual.residual, inline.residual);
    }
    let reused = result(&results, "reuse");
    assert!(reused.horn_goal);
    assert_eq!(reused.closure.known.len(), 4);
    assert_eq!(reused.closure.derivations.len(), 3);
    assert!(
        !reused
            .closure
            .derivations
            .iter()
            .any(|d| d.record.rule == "juanSale")
    );
    assert!(
        reused
            .closure
            .derivations
            .iter()
            .any(|d| d.record.rule == "sellsProgram")
    );

    // Removing source rules removes the behavior. No seeded name implies any
    // engine-specific consequence, and Derived/Effect goals aren't abducibles.
    let mut bare = module.clone();
    bare.declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    let bare_results = run_experiments(&bare).unwrap();
    assert_eq!(
        result(&bare_results, "real").closure.known,
        result(&results, "real").experiment.input
    );
    let blocked = result(&bare_results, "hypothetical")
        .residual
        .as_ref()
        .unwrap();
    assert_eq!(blocked.required, None);
    assert_eq!(blocked.failure, Some(AbductionFailure::NotAbducible));
    let renamed = parse_formal_to_typed(&SOURCE.replace("sells", "offers")).unwrap();
    let renamed_results = run_experiments(&renamed).unwrap();
    let renamed_real = result(&renamed_results, "real");
    assert!(renamed_real.horn_goal);
    assert_eq!(renamed_real.experiment.input[0].verb, "offers");
    assert_eq!(
        result(&renamed_results, "hypothetical")
            .residual
            .as_ref()
            .unwrap()
            .required,
        Some(RequirementFormula::Event(
            renamed_real.experiment.input[0].clone()
        ))
    );
}

fn proof_records(closure: &Closure) -> Vec<(Event, DerivationRecord)> {
    closure
        .derivations
        .iter()
        .map(|d| (d.event.clone(), d.record.clone()))
        .collect()
}

#[test]
fn sells_real_and_hypothetical_updates_have_identical_incremental_deltas() {
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let results = run_experiments(&module).unwrap();
    let real = result(&results, "real");
    let absent = result(&results, "hypothetical");
    let typed_rules = rules(&module);
    // Every input, including an optional nonempty base, comes from the fixture.
    for base in [
        &absent.experiment.input,
        &result(&results, "reuse").experiment.input,
    ] {
        for explanation in &absent.residual.as_ref().unwrap().explanations {
            let hypothesized = events(&explanation.required);
            let mut actual_clock = Clock::default();
            let mut hypothetical_clock = HypotheticalClock(0);
            let mut actual = IncrementalClosure::new(base.clone(), &typed_rules, &mut actual_clock);
            let mut hypothetical =
                IncrementalClosure::new(base.clone(), &typed_rules, &mut hypothetical_clock);
            let before = actual.closure().clone();
            let preserved = hypothetical.closure().derivations.clone();
            let a = actual.update(real.experiment.input.clone(), &mut actual_clock);
            let b = hypothetical.update(hypothesized.clone(), &mut hypothetical_clock);
            equivalent(actual.closure(), hypothetical.closure());
            equivalent(
                &Closure {
                    known: a.known.clone(),
                    derivations: a.derivations.clone(),
                },
                &Closure {
                    known: b.known.clone(),
                    derivations: b.derivations.clone(),
                },
            );
            assert_eq!(a.stats, b.stats);
            let mut union = base.clone();
            union.extend(real.experiment.input.clone());
            let full = close(union, &typed_rules, &mut Clock::default());
            equivalent(actual.closure(), &full);
            assert_eq!(
                a.known.iter().cloned().collect::<BTreeSet<_>>(),
                keys(&full).difference(&keys(&before)).cloned().collect()
            );
            let records = proof_records(&before);
            let expected_new: Vec<_> = proof_records(&full)
                .into_iter()
                .filter(|r| !records.contains(r))
                .collect();
            assert_eq!(a.derivations.len(), expected_new.len());
            assert!(
                a.derivations
                    .iter()
                    .all(|d| expected_new.contains(&(d.event.clone(), d.record.clone())))
            );
            assert!(
                preserved
                    .iter()
                    .all(|d| hypothetical.closure().derivations.contains(d))
            );
            let duplicate = hypothetical.update(hypothesized, &mut hypothetical_clock);
            assert!(duplicate.known.is_empty() && duplicate.derivations.is_empty());
            assert_eq!(duplicate.stats, Default::default());
        }
    }
    // The direct logical engine accepts Effect events; no State is executed.
    assert_eq!(
        absent.closure,
        Closure {
            known: vec![],
            derivations: vec![]
        }
    );
    assert!(
        real.closure
            .derivations
            .iter()
            .any(|d| d
                .record
                .substitution
                .bindings
                .iter()
                .any(|b| b.parameter == "t"
                    && b.value
                        == SubstitutionValue::Time(
                            real.experiment.input[0].proposition.time.clone()
                        )))
    );
}
