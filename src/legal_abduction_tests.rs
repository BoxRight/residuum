//! A bounded, test-only harness for the two-rule LegalAbduction example.
//! Abducibles are configured externally; no new surface syntax or L-order policy.

use crate::elab::Error;
use crate::typed::{
    Antecedent, Declaration, DerivationRecord, DerivationTime, Event, Module, Polarity,
    PropositionKind, PropositionType, ResidualRequirement, Rule, StateTime, StateTransitionType,
    Term, TimeExpr, VerbKind,
};
use crate::{
    DerivationClock, close, instantiate_from_known, parse_formal_to_typed, query_residual,
};

const PROGRAM: &str = include_str!("../examples/legal/sales_delivery_formal.res");

#[derive(Debug)]
struct Explanation {
    required: Antecedent,
    queries: Vec<ResidualRequirement>,
    supported: Vec<DerivationRecord>,
}

impl Explanation {
    fn unit() -> Self {
        Self {
            required: Antecedent::Unit,
            queries: vec![],
            supported: vec![],
        }
    }
}

struct Harness<'a> {
    known: &'a [Event],
    rules: &'a [Rule],
    abducibles: &'a [&'a str],
    rule_queries: usize,
}

impl Harness<'_> {
    fn goal(&mut self, goal: &Event, depth: usize) -> Result<Option<Explanation>, Error> {
        if self.known.contains(goal) {
            return Ok(Some(Explanation::unit()));
        }
        if goal.proposition.kind == PropositionKind::Seeded {
            return Ok((goal.polarity == Polarity::Positive
                && self.abducibles.contains(&goal.verb.as_str()))
            .then(|| Explanation {
                required: Antecedent::Event(goal.clone()),
                queries: vec![],
                supported: vec![],
            }));
        }
        // This harness covers delivery -> sales only. An unsupported deeper
        // search fails explicitly, rather than being mistaken for no explanation.
        if depth >= 2 {
            return Err(Error::new("LegalAbduction harness depth exceeded"));
        }
        let mut solutions = Vec::new();
        let rules = self.rules;
        for rule in rules {
            self.rule_queries += 1;
            let pending = query_residual(rule, goal, self.known)?;
            if pending.is_empty() {
                // The query's empty result can mean either a nonmatching head
                // or a fully evidenced body. Distinguish these without stamping.
                if let Some(proof) = instantiate_from_known(rule, self.known)
                    .into_iter()
                    .find(|proof| proof.event == *goal)
                {
                    let mut solution = Explanation::unit();
                    solution.supported.push(proof.record);
                    solutions.push(solution);
                }
            } else {
                for query in pending {
                    if !query.parameters.is_empty() {
                        return Err(Error::new(
                            "harness requires ground instantiated requirements",
                        ));
                    }
                    if let Some(mut solution) = self.requirement(&query.required, depth + 1)? {
                        solution.queries.insert(0, query);
                        solutions.push(solution);
                    }
                }
            }
        }
        if solutions.len() > 1 {
            return Err(Error::new(
                "alternative explanations are outside this harness",
            ));
        }
        Ok(solutions.pop())
    }

    fn requirement(
        &mut self,
        required: &Antecedent,
        depth: usize,
    ) -> Result<Option<Explanation>, Error> {
        match required {
            Antecedent::Unit => Ok(Some(Explanation::unit())),
            Antecedent::Event(event) => self.goal(event, depth),
            Antecedent::And(left, right) => {
                let Some(mut left) = self.requirement(left, depth)? else {
                    return Ok(None);
                };
                let Some(right) = self.requirement(right, depth)? else {
                    return Ok(None);
                };
                left.required = match (left.required, right.required) {
                    (Antecedent::Unit, right) => right,
                    (left, Antecedent::Unit) => left,
                    (left, right) => Antecedent::And(Box::new(left), Box::new(right)),
                };
                left.queries.extend(right.queries);
                left.supported.extend(right.supported);
                Ok(Some(left))
            }
        }
    }
}

fn program() -> (Module, Vec<Rule>) {
    let module = parse_formal_to_typed(PROGRAM).unwrap();
    let rules = ["delivery", "sales"]
        .into_iter()
        .map(|name| {
            module
                .declarations
                .iter()
                .find_map(|declaration| match declaration {
                    Declaration::Rule(rule) if rule.name == name => Some(rule.clone()),
                    _ => None,
                })
                .unwrap()
        })
        .collect();
    for name in ["agreesPrice", "agreesObject"] {
        assert!(
            module
                .declarations
                .iter()
                .any(|declaration| matches!(declaration,
            Declaration::Verb(verb) if verb.name == name && verb.kind == VerbKind::Seeded))
        );
    }
    (module, rules)
}

fn agreement(verb: &str, args: [&str; 3]) -> Event {
    Event {
        polarity: Polarity::Positive,
        verb: verb.into(),
        args: args.map(|name| Term::Var(name.into())).to_vec(),
        proposition: PropositionType {
            kind: PropositionKind::Seeded,
            time: TimeExpr::At("tau".into()),
        },
        transition: None,
    }
}

fn goal() -> Event {
    let input = TimeExpr::After(Box::new(TimeExpr::At("tau".into())));
    let output = TimeExpr::After(Box::new(input.clone()));
    Event {
        polarity: Polarity::Positive,
        verb: "do".into(),
        args: vec![
            Term::Var("alice".into()),
            Term::Const("deliver".into()),
            Term::Var("car".into()),
            Term::Var("bob".into()),
        ],
        proposition: PropositionType {
            kind: PropositionKind::Effect,
            time: output.clone(),
        },
        transition: Some(StateTransitionType {
            input: StateTime::At(input),
            output: StateTime::At(output),
        }),
    }
}

fn hypotheses(required: &Antecedent) -> Vec<Event> {
    match required {
        Antecedent::Unit => vec![],
        Antecedent::Event(event) => vec![event.clone()],
        Antecedent::And(left, right) => {
            let mut events = hypotheses(left);
            events.extend(hypotheses(right));
            events
        }
    }
}

#[derive(Default)]
struct HypotheticalClock(usize);

impl DerivationClock for HypotheticalClock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("hypothetical-{}", self.0),
        }
    }
}

fn check_scenario(known: Vec<Event>, expected: Antecedent) {
    let (module, rules) = program();
    let original_module = module.clone();
    let original_known = known.clone();
    let target = goal();
    let mut harness = Harness {
        known: &known,
        rules: &rules,
        abducibles: &["agreesPrice", "agreesObject"],
        rule_queries: 0,
    };
    let explanation = harness.goal(&target, 0).unwrap().expect("one explanation");
    assert_eq!(explanation.required, expected);
    assert_eq!(harness.rule_queries, 4);
    assert_eq!(explanation.queries[0].rule.name, "delivery");
    assert_eq!(explanation.queries[0].goal, target);
    let Antecedent::Event(intermediate) = &explanation.queries[0].required else {
        panic!("delivery requires give");
    };
    assert_eq!(intermediate.verb, "give");
    assert_eq!(
        intermediate.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
    );
    assert_eq!(
        intermediate.transition.as_ref().unwrap().input,
        StateTime::Unresolved,
        "the harness must not repair transition metadata to make logical matching work"
    );
    if expected == Antecedent::Unit {
        assert_eq!(explanation.queries.len(), 1);
        assert_eq!(explanation.supported.len(), 1);
        assert_eq!(explanation.supported[0].rule, "sales");
        assert_eq!(explanation.supported[0].antecedents, known);
    } else {
        assert_eq!(explanation.queries.len(), 2);
        assert_eq!(explanation.queries[1].rule.name, "sales");
        assert_eq!(explanation.queries[1].evidence, known);
        assert!(explanation.supported.is_empty());
    }
    let proposed = hypotheses(&explanation.required);
    assert!(
        proposed
            .iter()
            .all(|event| event.proposition.kind == PropositionKind::Seeded
                && ["agreesPrice", "agreesObject"].contains(&event.verb.as_str()))
    );
    assert!(proposed.iter().all(|event| !known.contains(event)));
    assert_eq!(known, original_known);
    assert_eq!(module, original_module);
    assert!(!known.contains(&target));
    assert!(
        known
            .iter()
            .all(|event| event.proposition.kind == PropositionKind::Seeded)
    );

    // Validate only on a separate hypothetical closure. This does not execute
    // effects or adopt hypotheses into the actual knowledge passed to the query.
    let mut hypothetical_facts = known.clone();
    hypothetical_facts.extend(proposed.clone());
    let verified = close(
        hypothetical_facts,
        &rules,
        &mut HypotheticalClock::default(),
    );
    assert!(verified.known.contains(&target));
    assert_eq!(verified.derivations.len(), 2);
    let derived_goal = verified
        .derivations
        .iter()
        .find(|derived| derived.event == target)
        .unwrap();
    assert_eq!(derived_goal.event.transition, target.transition);
    for omitted in 0..proposed.len() {
        let mut smaller = known.clone();
        smaller.extend(
            proposed
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != omitted)
                .map(|(_, event)| event.clone()),
        );
        let closure = close(smaller, &rules, &mut HypotheticalClock::default());
        assert!(
            !closure.known.contains(&target),
            "each proposed agreement is necessary in this example"
        );
    }
    assert_eq!(known, original_known);
}

#[test]
fn empty_evidence_requires_both_agreements() {
    check_scenario(
        vec![],
        Antecedent::And(
            Box::new(Antecedent::Event(agreement(
                "agreesPrice",
                ["alice", "car", "bob"],
            ))),
            Box::new(Antecedent::Event(agreement(
                "agreesObject",
                ["bob", "car", "alice"],
            ))),
        ),
    );
}

#[test]
fn price_evidence_requires_only_object_agreement() {
    check_scenario(
        vec![agreement("agreesPrice", ["alice", "car", "bob"])],
        Antecedent::Event(agreement("agreesObject", ["bob", "car", "alice"])),
    );
}

#[test]
fn complete_evidence_requires_unit() {
    check_scenario(
        vec![
            agreement("agreesPrice", ["alice", "car", "bob"]),
            agreement("agreesObject", ["bob", "car", "alice"]),
        ],
        Antecedent::Unit,
    );
}

#[test]
fn disallowed_agreement_and_wrong_goal_time_have_no_explanation() {
    let (_, rules) = program();
    let mut harness = Harness {
        known: &[],
        rules: &rules,
        abducibles: &["agreesPrice"],
        rule_queries: 0,
    };
    assert!(harness.goal(&goal(), 0).unwrap().is_none());
    harness.abducibles = &["agreesPrice", "agreesObject"];
    let mut earlier = goal();
    earlier.proposition.time = TimeExpr::After(Box::new(TimeExpr::At("tau".into())));
    assert!(
        harness.goal(&earlier, 0).unwrap().is_none(),
        "after(after(tau)) cannot collapse to after(tau)"
    );
}
