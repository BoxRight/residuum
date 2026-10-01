//! A test-only, sufficient checker for equality of typed normalized guarantees.
//! No contracts, module privacy or verification semantics are added to M2.

use super::*;
use crate::surface::{FormalDeclaration, FormalModule};
use crate::typed::{DerivationRecord, Verb};
use crate::{IncrementalClosure, elaborate_formal, parse_formal};

const SPEC: &str = include_str!("../../examples/experiments/owns_specification.res");
const IMPLEMENTATION: &str = include_str!("../../examples/experiments/owns_implementation.res");
const INLINE: &str = include_str!("../../examples/experiments/owns_implementation_inline.res");
const CONSUMER: &str = include_str!("../../examples/experiments/owns_consumer.res");

mod checker {
    use super::*;

    // Private contents, and no public unchecked constructor. No implementation
    // body is stored in the object passed to the consumer.
    pub(super) struct VerifiedInterface {
        specification: FormalModule,
    }

    impl VerifiedInterface {
        pub(super) fn elaborate_consumer(&self, source: &str) -> Result<Module, Error> {
            elaborate_composed_fixture(self.specification.clone(), parse_formal(source)?)
        }
    }

    pub(super) struct Verification {
        pub(super) interface: VerifiedInterface,
        pub(super) implementation: Module,
        pub(super) matched_rule: Rule,
    }

    pub(super) fn verify(specification: &str, implementation: &str) -> Result<Verification, Error> {
        let specification = parse_formal(specification)?;
        let expected = elaborate_formal(specification.clone())?;
        let guarantees = rules(&expected);
        if guarantees.len() != 1 {
            return Err(Error::new("experiment requires exactly one guarantee"));
        }
        let guarantee = &guarantees[0];
        let RuleBody::HornClause(clause) = &guarantee.body else {
            return Err(Error::new("experiment requires a Horn guarantee"));
        };
        let Antecedent::Event(premise) = &clause.antecedent else {
            return Err(Error::new("experiment requires one owns antecedent"));
        };
        if premise.verb != "owns" || clause.consequent.verb != "notDo" {
            return Err(Error::new("experiment requires owns to guarantee notDo"));
        }
        fn variables(event: &Event) -> BTreeSet<String> {
            fn time_name(time: &TimeExpr) -> &str {
                match time {
                    TimeExpr::At(name) => name,
                    TimeExpr::After(inner) => time_name(inner),
                }
            }
            let mut names: BTreeSet<_> = event
                .args
                .iter()
                .filter_map(|term| match term {
                    Term::Var(name) => Some(name.clone()),
                    _ => None,
                })
                .collect();
            names.insert(time_name(&event.proposition.time).into());
            names
        }
        let bound = variables(premise);
        if guarantee
            .parameters
            .iter()
            .any(|p| !bound.contains(&p.name))
            || !variables(&clause.consequent).is_subset(&bound)
        {
            return Err(Error::new("guarantee contains unresolved variables"));
        }
        let mut signatures = specification.clone();
        signatures
            .declarations
            .retain(|d| !matches!(d, FormalDeclaration::Rule(_)));
        let implementation = parse_formal(implementation)?;
        // Prevent the body from replacing the public declarations used to type it.
        if implementation
            .declarations
            .iter()
            .any(|d| !matches!(d, FormalDeclaration::Rule(_)))
        {
            return Err(Error::new(
                "experiment implementation must contain only rules",
            ));
        }
        // The guarantee is absent from this module. It cannot justify itself.
        let implementation = elaborate_composed_fixture(signatures, implementation)?;
        for signature in verbs(&expected) {
            if !verbs(&implementation).contains(&signature) {
                return Err(Error::new(
                    "implementation has incompatible public signature",
                ));
            }
        }
        let matched_rule = rules(&implementation)
            .into_iter()
            .find(|rule| rule.parameters == guarantee.parameters && rule.body == guarantee.body)
            .ok_or_else(|| Error::new("guarantee not established by normalized implementation"))?;
        Ok(Verification {
            interface: VerifiedInterface { specification },
            implementation,
            matched_rule,
        })
    }
}

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

fn verbs(module: &Module) -> Vec<Verb> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(verb) => Some(verb.clone()),
            _ => None,
        })
        .collect()
}

fn keys(closure: &Closure) -> BTreeSet<Event> {
    closure.known.iter().cloned().collect()
}

fn proofs(closure: &Closure) -> Vec<(Event, DerivationRecord)> {
    closure
        .derivations
        .iter()
        .map(|derived| {
            let mut record = derived.record.clone();
            if matches!(
                record.rule.as_str(),
                "implementedProtection" | "inlineProtection"
            ) {
                record.rule = "ownsGuarantee".into();
            }
            (derived.event.clone(), record)
        })
        .collect()
}

fn equivalent(a: &Closure, b: &Closure) {
    assert_eq!(keys(a), keys(b));
    let (a, b) = (proofs(a), proofs(b));
    assert_eq!(a.len(), b.len());
    assert!(
        a.iter().all(|proof| b.contains(proof)),
        "substitutions and witnesses stay exact; only the guarantee's rule name changes"
    );
}

fn implementation_view(consumer: &Module, verification: &checker::Verification) -> Vec<Rule> {
    let mut result: Vec<_> = rules(consumer)
        .into_iter()
        .filter(|rule| rule.name != "ownsGuarantee")
        .collect();
    result.extend(rules(&verification.implementation));
    result
}

#[test]
fn interface_checker_uses_implementation_not_the_written_guarantee() {
    for source in [SPEC, IMPLEMENTATION, INLINE, CONSUMER] {
        assert_eq!(
            parse_formal(source).unwrap(),
            crate::formal::parse_formal_manual(source).unwrap()
        );
    }
    let encapsulated = checker::verify(SPEC, IMPLEMENTATION).unwrap();
    let inline = checker::verify(SPEC, INLINE).unwrap();
    assert_eq!(
        encapsulated.matched_rule.parameters,
        inline.matched_rule.parameters
    );
    assert_eq!(encapsulated.matched_rule.body, inline.matched_rule.body);
    assert!(
        rules(&encapsulated.implementation)
            .iter()
            .all(|rule| rule.name != "ownsGuarantee")
    );
    for wrong in [
        "module EmptyImplementation".to_owned(),
        IMPLEMENTATION.replace(
            "rule implementedProtection = ownsProgram(charlie, molest)",
            "",
        ),
        INLINE.replace("@ after t", "@ t"),
        INLINE.replace("object, owner)", "object, charlie)"),
    ] {
        assert!(
            checker::verify(SPEC, &wrong).is_err(),
            "writing the unchanged guarantee does not make this implementation valid"
        );
    }
    let incompatible = format!("{INLINE}\nderived verb owns(owner : Person)\n");
    assert!(checker::verify(SPEC, &incompatible).is_err());
    assert!(
        checker::verify(&SPEC.replace("@ after t", "@ t"), IMPLEMENTATION).is_err(),
        "changing the requirement must trigger fresh verification"
    );
}

#[test]
fn consumer_uses_verified_specification_without_implementation_body() {
    let verification = checker::verify(SPEC, IMPLEMENTATION).unwrap();
    // Consumer compilation takes the verified interface only.
    let consumer = verification.interface.elaborate_consumer(CONSUMER).unwrap();
    assert_eq!(
        rules(&consumer)
            .iter()
            .map(|rule| rule.name.as_str())
            .collect::<Vec<_>>(),
        [
            "ownsGuarantee",
            "transferToOwnership",
            "protectionRecognized"
        ]
    );
    assert!(rules(&consumer).iter().all(|rule| !matches!(
        rule.name.as_str(),
        "ownsProgram" | "implementedProtection" | "inlineProtection"
    )));
    let result = run_incremental_experiment(&consumer).unwrap();
    assert_eq!(
        result
            .incremental
            .known
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        ["transfer", "owns", "notDo", "protected"]
    );
    let norm = result
        .incremental
        .derivations
        .iter()
        .find(|d| d.event.verb == "notDo")
        .unwrap();
    assert_eq!(norm.record.rule, "ownsGuarantee");
    let protected = result
        .incremental
        .derivations
        .iter()
        .find(|d| d.event.verb == "protected")
        .unwrap();
    assert_eq!(protected.record.antecedents, [norm.event.clone()]);
    assert_eq!(
        protected.event.proposition.time,
        TimeExpr::After(Box::new(norm.event.proposition.time.clone()))
    );
    let actual_rules = implementation_view(&consumer, &verification);
    assert!(actual_rules.iter().all(|rule| rule.name != "ownsGuarantee"));
    let actual = close(
        result.additions.clone(),
        &actual_rules,
        &mut Clock::default(),
    );
    equivalent(&result.incremental, &actual);
    let without_guarantee: Vec<_> = rules(&consumer)
        .into_iter()
        .filter(|r| r.name != "ownsGuarantee")
        .collect();
    let unsupported = close(result.additions, &without_guarantee, &mut Clock::default());
    assert!(
        unsupported
            .known
            .iter()
            .all(|e| e.verb != "notDo" && e.verb != "protected")
    );
}

#[test]
fn interface_inline_and_encapsulated_paths_have_equal_incremental_behavior() {
    let encapsulated = checker::verify(SPEC, IMPLEMENTATION).unwrap();
    let inline = checker::verify(SPEC, INLINE).unwrap();
    let consumer = encapsulated.interface.elaborate_consumer(CONSUMER).unwrap();
    let other_consumer = inline.interface.elaborate_consumer(CONSUMER).unwrap();
    assert_eq!(
        consumer, other_consumer,
        "consumer sees the same interface regardless of body"
    );
    let first = run_incremental_experiment(&consumer).unwrap().additions[0].clone();
    let other_source = CONSUMER.replace(
        "transfer(alice, car, bob) @ tau,",
        "transfer(alice, car, alice) @ tau,",
    );
    let other = run_incremental_experiment(
        &encapsulated
            .interface
            .elaborate_consumer(&other_source)
            .unwrap(),
    )
    .unwrap()
    .additions[0]
        .clone();
    let basis = [first, other];
    let inputs: Vec<Vec<Event>> = (0..4)
        .map(|mask| {
            basis
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, event)| event.clone())
                .collect()
        })
        .collect();
    let paths = [
        rules(&consumer),
        implementation_view(&consumer, &encapsulated),
        implementation_view(&consumer, &inline),
    ];
    for input in &inputs {
        for additions in &inputs {
            let mut updated = Vec::new();
            let mut deltas = Vec::new();
            for rules in &paths {
                let mut clock = Clock::default();
                let mut state = IncrementalClosure::new(input.clone(), rules, &mut clock);
                let before = state.closure().clone();
                let delta = state.update(additions.clone(), &mut clock);
                let mut combined = input.clone();
                combined.extend(additions.clone());
                let recomputed = close(combined, rules, &mut Clock::default());
                equivalent(state.closure(), &recomputed);
                assert_eq!(
                    delta.known.iter().cloned().collect::<BTreeSet<_>>(),
                    keys(&recomputed)
                        .difference(&keys(&before))
                        .cloned()
                        .collect()
                );
                assert!(
                    before
                        .derivations
                        .iter()
                        .all(|d| state.closure().derivations.contains(d))
                );
                updated.push(state.closure().clone());
                deltas.push(Closure {
                    known: delta.known,
                    derivations: delta.derivations,
                });
            }
            for path in 1..3 {
                equivalent(&updated[0], &updated[path]);
                equivalent(&deltas[0], &deltas[path]);
            }
        }
    }
}
