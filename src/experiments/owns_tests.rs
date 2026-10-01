use super::*;
use crate::typed::{PropositionKind, SubstitutionValue, Type};
use crate::{IncrementalClosure, instantiate, parse_formal, parse_formal_to_typed};

const SOURCE: &str = include_str!("../../examples/experiments/owns_program.res");

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

fn rule<'a>(rules: &'a [Rule], name: &str) -> &'a Rule {
    rules.iter().find(|r| r.name == name).unwrap()
}

fn keys(closure: &Closure) -> BTreeSet<Event> {
    closure.known.iter().cloned().collect()
}

#[test]
fn owns_program_specialization_equals_explicit_rule_without_new_syntax() {
    assert_eq!(
        parse_formal(SOURCE).unwrap(),
        crate::formal::parse_formal_manual(SOURCE).unwrap()
    );
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let rules = rules(&module);
    let program = rule(&rules, "ownsProgram");
    assert_eq!(
        program
            .parameters
            .iter()
            .map(|p| p.ty.clone())
            .collect::<Vec<_>>(),
        [
            Type::Entity("Person".into()),
            Type::Entity("Conduct".into()),
            Type::Entity("Person".into()),
            Type::Entity("Thing".into()),
            Type::PropositionTime,
        ]
    );
    let specialized = rule(&rules, "protectionFromCharlie");
    assert_eq!(specialized.parameters, program.parameters[2..]);
    let explicit = SOURCE.replace(
        "rule protectionFromCharlie = ownsProgram(charlie, molest)",
        "rule protectionFromCharlie(owner, object, t) =\n    owns(owner, object) @ t\n    <= notDo(charlie, molest, object, owner) @ after t",
    );
    let explicit_rules = self::rules(&parse_formal_to_typed(&explicit).unwrap());
    assert_eq!(specialized, rule(&explicit_rules, "protectionFromCharlie"));
    let verb = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(verb) if verb.name == "notDo" => Some(verb),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        verb.args[2].ty,
        Type::Optional(Box::new(Type::Entity("Thing".into())))
    );
    assert!(verb.effect.is_none());
}

#[test]
fn owns_chain_reuses_derived_antecedent_and_preserves_time_and_provenance() {
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let unchanged = module.clone();
    let result = run_incremental_experiment(&module).unwrap();
    assert_eq!(module, unchanged);
    assert!(result.base.known.is_empty());
    assert!(result.logical_equal && result.provenance_equal);
    assert_eq!(
        result
            .delta
            .known
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        ["transfer", "owns", "notDo"]
    );
    assert_eq!(result.delta.derivations.len(), 2);
    let ownership = &result.delta.derivations[0];
    let norm = &result.delta.derivations[1];
    assert_eq!(ownership.record.rule, "transferToOwnership");
    assert_eq!(ownership.record.antecedents, result.additions);
    assert_eq!(
        ownership.event.args,
        [Term::Const("bob".into()), Term::Const("car".into())]
    );
    assert_eq!(ownership.event.proposition.kind, PropositionKind::Derived);
    assert_eq!(
        ownership.event.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
    );
    assert_eq!(norm.record.rule, "protectionFromCharlie");
    assert_eq!(norm.record.antecedents, [ownership.event.clone()]);
    assert_eq!(ownership.event, result.goal);
    assert_eq!(
        norm.event.args,
        [
            Term::Const("charlie".into()),
            Term::Const("molest".into()),
            Term::Const("car".into()),
            Term::Const("bob".into())
        ]
    );
    assert_eq!(
        norm.event.proposition.time,
        TimeExpr::After(Box::new(ownership.event.proposition.time.clone()))
    );
    assert_eq!(
        norm.event.polarity,
        Polarity::Positive,
        "notDo is a declared verb, not logical negation"
    );
    assert!(
        result
            .incremental
            .known
            .iter()
            .all(|e| e.transition.is_none())
    );
    assert!(
        norm.record
            .substitution
            .bindings
            .iter()
            .any(|b| b.parameter == "t"
                && b.value == SubstitutionValue::Time(ownership.event.proposition.time.clone()))
    );
    let rules = rules(&module);
    assert!(
        instantiate(rule(&rules, "ownsProgram"), &ownership.event).is_none(),
        "unbound subject/conduct cannot be invented"
    );
    let applied = instantiate(rule(&rules, "protectionFromCharlie"), &ownership.event).unwrap();
    assert_eq!(applied.event, norm.event);
    assert_eq!(applied.record, norm.record);
}

#[test]
fn owns_transformation_is_monotone_and_incremental_on_finite_knowledge_domain() {
    let module = parse_formal_to_typed(SOURCE).unwrap();
    let rules = rules(&module);
    let first = run_incremental_experiment(&module).unwrap().additions[0].clone();
    // A second owner comes from another parsed ground input, not a Rust-built event.
    let other_source = SOURCE.replace(
        "transfer(alice, car, bob) @ tau,",
        "transfer(alice, car, alice) @ tau,",
    );
    let other = run_incremental_experiment(&parse_formal_to_typed(&other_source).unwrap())
        .unwrap()
        .additions[0]
        .clone();
    let seeds = [first, other];
    let sets: Vec<Vec<Event>> = (0..4)
        .map(|mask| {
            seeds
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, e)| e.clone())
                .collect()
        })
        .collect();
    let closures: Vec<_> = sets
        .iter()
        .map(|input| close(input.clone(), &rules, &mut Clock::default()))
        .collect();
    for (i, input) in sets.iter().enumerate() {
        for (j, additions) in sets.iter().enumerate() {
            if i & j == i {
                assert!(
                    keys(&closures[i]).is_subset(&keys(&closures[j])),
                    "K order is inclusion"
                );
            }
            let mut clock = Clock::default();
            let mut state = IncrementalClosure::new(input.clone(), &rules, &mut clock);
            let before = state.closure().clone();
            let delta = state.update(additions.clone(), &mut clock);
            let mut combined = input.clone();
            combined.extend(additions.clone());
            let full = close(combined, &rules, &mut Clock::default());
            assert_eq!(keys(state.closure()), keys(&full));
            assert_eq!(
                keys(&before)
                    .union(&delta.known.iter().cloned().collect())
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                keys(&full)
            );
            assert_eq!(
                delta.known.iter().cloned().collect::<BTreeSet<_>>(),
                keys(&full).difference(&keys(&before)).cloned().collect()
            );
            assert!(
                before
                    .derivations
                    .iter()
                    .all(|d| state.closure().derivations.contains(d))
            );
            assert_eq!(state.closure().derivations.len(), full.derivations.len());
            assert!(full.derivations.iter().all(|d| {
                state
                    .closure()
                    .derivations
                    .iter()
                    .any(|e| d.event == e.event && d.record == e.record)
            }));
        }
    }
    assert_eq!(closures[3].known.len(), 6);
    let norm_beneficiaries: BTreeSet<_> = closures[3]
        .known
        .iter()
        .filter(|e| e.verb == "notDo")
        .map(|e| e.args[3].clone())
        .collect();
    assert_eq!(
        norm_beneficiaries,
        [Term::Const("alice".into()), Term::Const("bob".into())]
            .into_iter()
            .collect()
    );
}
