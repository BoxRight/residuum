//! Exhaustive model entailment, independent of close and the matcher.
//! Only finite ground theories; signed atomic Horn queries plus logical I.

use super::*;

mod theory_bridge;

#[derive(Clone, Copy, Debug)]
enum Designation {
    UnitFilter,
    PositiveSupport,
}

#[derive(Clone, Copy, Debug)]
enum RuleMeaning {
    TruthInequality,
    SupportImplication,
}

#[derive(Clone, Copy, Debug)]
struct Semantics {
    name: &'static str,
    model: Model,
    designation: Designation,
    rule: RuleMeaning,
}

const SEMANTICS: [Semantics; 5] = [
    Semantics {
        name: "Fusion / unit / truth",
        model: Model::Fusion,
        designation: Designation::UnitFilter,
        rule: RuleMeaning::TruthInequality,
    },
    Semantics {
        name: "Meet / unit / truth",
        model: Model::Meet,
        designation: Designation::UnitFilter,
        rule: RuleMeaning::TruthInequality,
    },
    Semantics {
        name: "Meet / positive / truth",
        model: Model::Meet,
        designation: Designation::PositiveSupport,
        rule: RuleMeaning::TruthInequality,
    },
    Semantics {
        name: "Fusion / positive / support",
        model: Model::Fusion,
        designation: Designation::PositiveSupport,
        rule: RuleMeaning::SupportImplication,
    },
    Semantics {
        name: "Meet / positive / support",
        model: Model::Meet,
        designation: Designation::PositiveSupport,
        rule: RuleMeaning::SupportImplication,
    },
];

impl Semantics {
    fn designated(self, value: Evidence) -> bool {
        match self.designation {
            Designation::UnitFilter => self.model.unit().le(&value),
            Designation::PositiveSupport => value.p,
        }
    }
    fn satisfies(self, world: &Known, query: &Formula) -> bool {
        self.designated(evaluate(self.model, world, query))
    }
    fn satisfies_rule(self, world: &Known, rule: &Rule) -> bool {
        let clause = clause(rule);
        let body = evaluate(self.model, world, &formula(&clause.antecedent));
        let head = atomic_value(world, &clause.consequent);
        match self.rule {
            RuleMeaning::TruthInequality => body.le(&head),
            RuleMeaning::SupportImplication => !body.p || head.p,
        }
    }
    fn satisfies_theory(self, world: &Known, facts: &Known, rules: &[Rule]) -> bool {
        facts
            .iter()
            .all(|fact| self.satisfies(world, &Formula::Atom(fact.clone())))
            && rules.iter().all(|rule| self.satisfies_rule(world, rule))
    }
}

fn clause(rule: &Rule) -> &HornClause {
    assert!(rule.parameters.is_empty(), "ground theories only");
    let RuleBody::HornClause(clause) = &rule.body else {
        panic!("Horn clause expected")
    };
    clause
}

fn interpretations(atoms: &[Event]) -> Vec<Known> {
    assert!(atoms.len() <= 4, "explicit finite bound");
    // Support sets encode ALL interpretations, not just the actual input K
    // or the Horn closure. Nothing here invokes forward derivation.
    (0..1 << (2 * atoms.len()))
        .map(|mask| known_from_mask(mask, atoms))
        .collect()
}

fn models<'a>(
    semantics: Semantics,
    worlds: &'a [Known],
    facts: &Known,
    rules: &[Rule],
) -> Vec<&'a Known> {
    worlds
        .iter()
        .filter(|world| semantics.satisfies_theory(world, facts, rules))
        .collect()
}

fn entails(semantics: Semantics, models: &[&Known], query: &Formula) -> bool {
    // Universal quantification, including vacuity. Model counts are reported
    // separately so an empty class is never mistaken for ordinary evidence.
    models.iter().all(|world| semantics.satisfies(world, query))
}

fn entails_inequality(semantics: Semantics, models: &[&Known], a: &Formula, c: &Formula) -> bool {
    models
        .iter()
        .all(|world| truth_inequality(semantics.model, world, a, c))
}

fn horn_closure(facts: &Known, rules: &[Rule]) -> Known {
    close(
        facts.iter().cloned().collect(),
        rules,
        &mut Clock::default(),
    )
    .known
    .into_iter()
    .collect()
}

// The universe contains derived atoms, so either sign can be a rule head.
// Initial facts are logical premises, not new seeded declarations in M2.
fn logical_atoms(names: &[&str]) -> Vec<Event> {
    names
        .iter()
        .map(|name| {
            let mut result = atom(name);
            result.proposition.kind = PropositionKind::Derived;
            result
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Classification {
    HornSoundnessFailure,
    HornCompletenessFailure,
    RuleFormulaMismatch,
    SupportEntailmentDifference,
}

#[derive(Clone, Debug)]
struct Counterexample {
    facts: Known,
    rules: Vec<Rule>,
    query: Event,
    model_count: usize,
    countermodel: Option<Known>,
}

impl Counterexample {
    fn size(&self) -> usize {
        self.facts.len() + self.rules.len()
    }
}

#[derive(Default)]
struct Comparison {
    theories: usize,
    queries: usize,
    soundness_failures: usize,
    completeness_failures: usize,
    nonvacuous_completeness_failures: usize,
    empty_model_theories: usize,
    closure_not_model: usize,
    minimal_soundness: Option<Counterexample>,
    minimal_completeness: Option<Counterexample>,
}

fn retain_smaller(slot: &mut Option<Counterexample>, candidate: Counterexample) {
    if slot
        .as_ref()
        .is_none_or(|old| candidate.size() < old.size())
    {
        *slot = Some(candidate);
    }
}

impl Comparison {
    fn examine(
        &mut self,
        sem: Semantics,
        worlds: &[Known],
        facts: &Known,
        rules: &[Rule],
        closed: &Known,
        atoms: &[Event],
    ) {
        self.theories += 1;
        let interpretations = models(sem, worlds, facts, rules);
        if interpretations.is_empty() {
            self.empty_model_theories += 1;
        }
        if !sem.satisfies_theory(closed, facts, rules) {
            self.closure_not_model += 1;
        }
        // I is a logical query, not a member of the event closure.
        assert!(entails(sem, &interpretations, &Formula::Unit));
        self.queries += 1;
        for atom in atoms {
            for query in [atom.clone(), opposite(atom)] {
                self.queries += 1;
                let semantic = entails(sem, &interpretations, &Formula::Atom(query.clone()));
                let horn = closed.contains(&query);
                match (horn, semantic) {
                    (true, false) => {
                        self.soundness_failures += 1;
                        let countermodel = interpretations
                            .iter()
                            .find(|world| !sem.satisfies(world, &Formula::Atom(query.clone())))
                            .map(|world| (*world).clone());
                        retain_smaller(
                            &mut self.minimal_soundness,
                            Counterexample {
                                facts: facts.clone(),
                                rules: rules.to_vec(),
                                query,
                                model_count: interpretations.len(),
                                countermodel,
                            },
                        );
                    }
                    (false, true) => {
                        self.completeness_failures += 1;
                        if !interpretations.is_empty() {
                            self.nonvacuous_completeness_failures += 1;
                            retain_smaller(
                                &mut self.minimal_completeness,
                                Counterexample {
                                    facts: facts.clone(),
                                    rules: rules.to_vec(),
                                    query,
                                    model_count: interpretations.len(),
                                    countermodel: None,
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    fn print(&self, family: &str, sem: Semantics) {
        println!(
            "| {family} | {} | {} | {} | {} | {} | {} | {} | {} |",
            sem.name,
            self.theories,
            self.queries,
            self.soundness_failures,
            self.completeness_failures,
            self.nonvacuous_completeness_failures,
            self.empty_model_theories,
            self.closure_not_model
        );
        if let Some(counterexample) = &self.minimal_completeness {
            println!(
                "COUNTEREXAMPLE {:?} / {} / {}: premises={}, facts={}, rules={}, goal={}, models={}",
                Classification::HornCompletenessFailure,
                family,
                sem.name,
                counterexample.size(),
                facts_text(&counterexample.facts),
                rules_text(&counterexample.rules),
                event_text(&counterexample.query),
                counterexample.model_count
            );
        }
        if let Some(counterexample) = &self.minimal_soundness {
            println!(
                "COUNTEREXAMPLE {:?}: {counterexample:?}",
                Classification::HornSoundnessFailure
            );
            assert!(counterexample.countermodel.is_some());
        }
    }
}

fn event_text(event: &Event) -> String {
    format!(
        "{}{}",
        if event.polarity == Polarity::EvidentialNot {
            "~"
        } else {
            ""
        },
        event.verb
    )
}
fn antecedent_text(antecedent: &Antecedent) -> String {
    match antecedent {
        Antecedent::Unit => "I".into(),
        Antecedent::Event(event) => event_text(event),
        Antecedent::And(a, b) => format!("({} & {})", antecedent_text(a), antecedent_text(b)),
    }
}
fn facts_text(facts: &Known) -> String {
    facts.iter().map(event_text).collect::<Vec<_>>().join(",")
}
fn rules_text(rules: &[Rule]) -> String {
    rules
        .iter()
        .map(|rule| {
            let c = clause(rule);
            format!(
                "{} <= {}",
                antecedent_text(&c.antecedent),
                event_text(&c.consequent)
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}
fn table_header() {
    println!(
        "| Family | Semantics | Theories | Queries | Unsound | Incomplete | Incomplete nonempty | No models | Closure not model |"
    );
    println!("|---|---|---:|---:|---:|---:|---:|---:|---:|");
}

#[test]
fn facts_negation_and_empty_model_classes_are_explicit() {
    let atoms = logical_atoms(&["P"]);
    let worlds = interpretations(&atoms);
    for sem in SEMANTICS {
        let with_unit: Vec<_> = worlds
            .iter()
            .filter(|world| sem.satisfies(world, &Formula::Unit))
            .collect();
        assert_eq!(with_unit.len(), worlds.len(), "Gamma={{I}} has all models");
        let conjunction = Formula::Tensor(
            Box::new(Formula::Atom(atoms[0].clone())),
            Box::new(Formula::Not(Box::new(Formula::Atom(atoms[0].clone())))),
        );
        let conjunction_models: Vec<_> = worlds
            .iter()
            .filter(|world| sem.satisfies(world, &conjunction))
            .collect();
        assert_eq!(
            conjunction_models,
            models(sem, &worlds, &known_from_mask(3, &atoms), &[]),
            "Gamma={{P & ~P}} and Gamma={{P,~P}} have the same models here"
        );
        let expected = match sem.designation {
            Designation::UnitFilter if matches!(sem.model, Model::Meet) => [4, 1, 1, 0],
            _ => [4, 2, 2, 1],
        };
        for (mask, count) in expected.into_iter().enumerate() {
            let facts = known_from_mask(mask, &atoms);
            let admissible = models(sem, &worlds, &facts, &[]);
            assert_eq!(admissible.len(), count);
            let positive = entails(sem, &admissible, &Formula::Atom(atoms[0].clone()));
            let negative = entails(
                sem,
                &admissible,
                &Formula::Not(Box::new(Formula::Atom(atoms[0].clone()))),
            );
            assert_eq!(positive, mask & 1 != 0);
            assert_eq!(negative, mask & 2 != 0);
            assert!(entails(sem, &admissible, &Formula::Unit));
            println!(
                "FACTS | {} | {{{}}} | models={} | P={} | ~P={}",
                sem.name,
                facts_text(&facts),
                count,
                positive,
                negative
            );
        }
    }
}

#[test]
fn exhaustive_small_programs_report_soundness_and_completeness() {
    let atoms = logical_atoms(&["P", "Q"]);
    let worlds = interpretations(&atoms);
    let signed: Vec<_> = atoms
        .iter()
        .flat_map(|a| [a.clone(), opposite(a)])
        .collect();
    let mut bodies = vec![Antecedent::Unit];
    bodies.extend(signed.iter().cloned().map(Antecedent::Event));
    for i in 0..signed.len() {
        for j in i..signed.len() {
            bodies.push(Antecedent::And(
                Box::new(Antecedent::Event(signed[i].clone())),
                Box::new(Antecedent::Event(signed[j].clone())),
            ));
        }
    }
    let mut candidates = Vec::new();
    for body in bodies {
        for head in &signed {
            candidates.push(ground_rule(
                &format!("r{}", candidates.len()),
                body.clone(),
                head.clone(),
            ));
        }
    }
    assert_eq!(candidates.len(), 60);
    let mut programs = vec![vec![]];
    programs.extend(candidates.iter().map(|rule| vec![rule.clone()]));
    // All distinct pairs of the 20 unit/unary rules, including cycles and
    // conflicting heads. All 60 unit/unary/binary rules occur singly.
    for i in 0..20 {
        for j in i + 1..20 {
            programs.push(vec![candidates[i].clone(), candidates[j].clone()]);
        }
    }
    assert_eq!(programs.len(), 251);
    let mut comparisons: [Comparison; 5] = std::array::from_fn(|_| Comparison::default());
    for rules in programs {
        for facts in &worlds {
            let closed = horn_closure(facts, &rules);
            for (i, sem) in SEMANTICS.into_iter().enumerate() {
                comparisons[i].examine(sem, &worlds, facts, &rules, &closed, &atoms);
            }
        }
    }
    table_header();
    for (i, sem) in SEMANTICS.into_iter().enumerate() {
        let result = &comparisons[i];
        result.print("two atoms", sem);
        assert_eq!(result.theories, 4016);
        assert_eq!(result.queries, 20080);
        assert_eq!(result.soundness_failures, 0);
        if matches!(sem.rule, RuleMeaning::SupportImplication) {
            assert_eq!(result.completeness_failures, 0);
            assert_eq!(result.closure_not_model, 0);
            assert_eq!(result.empty_model_theories, 0);
        } else {
            assert!(result.nonvacuous_completeness_failures > 0);
        }
    }
}

#[test]
fn tensor_chains_cycles_and_inconsistent_heads_report_all_model_comparisons() {
    table_header();
    for family in ["tensor", "chain", "cycle", "inconsistent heads"] {
        let names: &[&str] = match family {
            "tensor" => &["A", "B", "C"],
            "chain" => &["A", "B", "C", "D"],
            _ => &["P", "Q"],
        };
        let atoms = logical_atoms(names);
        let a = Antecedent::Event(atoms[0].clone());
        let b = Antecedent::Event(atoms[1].clone());
        let rules = match family {
            "tensor" | "chain" => {
                let mut rules = vec![ground_rule(
                    "combine",
                    Antecedent::And(Box::new(a), Box::new(b)),
                    atoms[2].clone(),
                )];
                if family == "chain" {
                    rules.push(ground_rule(
                        "continue",
                        Antecedent::Event(atoms[2].clone()),
                        atoms[3].clone(),
                    ));
                }
                rules
            }
            "cycle" => vec![
                ground_rule("pq", a, atoms[1].clone()),
                ground_rule("qp", b, atoms[0].clone()),
            ],
            _ => vec![
                ground_rule("positive", a.clone(), atoms[1].clone()),
                ground_rule("negative", a, opposite(&atoms[1])),
            ],
        };
        let worlds = interpretations(&atoms);
        assert!(
            horn_closure(&Known::new(), &rules).is_empty(),
            "cycles do not bootstrap evidence"
        );
        for sem in SEMANTICS {
            let mut comparison = Comparison::default();
            for facts in &worlds {
                let closed = horn_closure(facts, &rules);
                comparison.examine(sem, &worlds, facts, &rules, &closed, &atoms);
            }
            comparison.print(family, sem);
            assert_eq!(comparison.soundness_failures, 0);
            if matches!(sem.rule, RuleMeaning::SupportImplication) {
                assert_eq!(comparison.completeness_failures, 0);
            }
        }
    }
}

#[test]
fn minimal_counterexamples_and_closed_both_head_are_classified() {
    let atoms = logical_atoms(&["P", "Q"]);
    let worlds = interpretations(&atoms);
    let rule = ground_rule("pq", Antecedent::Event(atoms[0].clone()), atoms[1].clone());
    let facts = Known::from([opposite(&atoms[1])]);
    let query = Formula::Atom(opposite(&atoms[0]));
    for sem in SEMANTICS[..3].iter().copied() {
        let admissible = models(sem, &worlds, &facts, std::slice::from_ref(&rule));
        assert!(!admissible.is_empty());
        assert!(entails(sem, &admissible, &query));
        assert!(!horn_closure(&facts, std::slice::from_ref(&rule)).contains(&opposite(&atoms[0])));
        assert!(!entails(
            sem,
            &models(sem, &worlds, &Known::new(), std::slice::from_ref(&rule)),
            &query
        ));
        assert!(!entails(sem, &models(sem, &worlds, &facts, &[]), &query));
    }
    println!(
        "{:?}: {{~Q, P <= Q}} entails ~P, not Horn; removing either premise destroys entailment",
        Classification::HornCompletenessFailure
    );
    let contradictory = Known::from([atoms[0].clone(), opposite(&atoms[0])]);
    assert!(models(SEMANTICS[1], &worlds, &contradictory, &[]).is_empty());
    assert!(entails(SEMANTICS[1], &[], &Formula::Atom(atoms[1].clone())));
    assert!(!horn_closure(&contradictory, &[]).contains(&atoms[1]));
    for sem in [SEMANTICS[0], SEMANTICS[2]] {
        assert!(!entails(
            sem,
            &models(sem, &worlds, &contradictory, &[]),
            &Formula::Atom(atoms[1].clone())
        ));
    }
    println!(
        "{:?}: Meet/unit has no models of {{P,~P}}; entailment of Q is vacuous, not evidence",
        Classification::HornCompletenessFailure
    );

    let abc = logical_atoms(&["A", "B", "C"]);
    let rules = [ground_rule(
        "combine",
        Antecedent::And(
            Box::new(Antecedent::Event(abc[0].clone())),
            Box::new(Antecedent::Event(abc[1].clone())),
        ),
        abc[2].clone(),
    )];
    let facts = Known::from([abc[0].clone(), abc[1].clone(), opposite(&abc[2])]);
    let closed = horn_closure(&facts, &rules);
    assert_eq!(atomic_value(&closed, &abc[2]), BOTH);
    let worlds = interpretations(&abc);
    for sem in [SEMANTICS[0], SEMANTICS[2]] {
        let admissible = models(sem, &worlds, &facts, &rules);
        assert!(!admissible.is_empty());
        assert!(!sem.satisfies_theory(&closed, &facts, &rules));
        assert!(entails(sem, &admissible, &Formula::Atom(abc[2].clone())));
        assert!(closed.contains(&abc[2]));
    }
    println!(
        "{:?}: A=T,B=T,C=B is not a truth-rule model, but C is Horn-derived AND model-entailed; not unsoundness",
        Classification::SupportEntailmentDifference
    );
}

#[test]
fn residual_formula_and_rule_entailment_are_compared_over_the_same_models() {
    let atoms = logical_atoms(&["A", "B", "C"]);
    let worlds = interpretations(&atoms);
    let [a, b, c]: [Formula; 3] = std::array::from_fn(|i| Formula::Atom(atoms[i].clone()));
    let tensor = Formula::Tensor(Box::new(a.clone()), Box::new(b.clone()));
    let implication = Formula::Residual(Box::new(tensor.clone()), Box::new(c.clone()));
    let curried = Formula::Residual(Box::new(b), Box::new(c.clone()));
    let rules = [ground_rule(
        "combine",
        Antecedent::And(
            Box::new(Antecedent::Event(atoms[0].clone())),
            Box::new(Antecedent::Event(atoms[1].clone())),
        ),
        atoms[2].clone(),
    )];
    for sem in SEMANTICS {
        let mut mismatches = 0;
        let encoding_mismatches = worlds
            .iter()
            .filter(|world| {
                sem.satisfies_rule(world, &rules[0]) != sem.satisfies(world, &implication)
            })
            .count();
        println!(
            "ENCODING | {} | worlds=64 | rule/implication mismatches={encoding_mismatches}",
            sem.name
        );
        if matches!(sem.designation, Designation::UnitFilter)
            || (matches!(sem.model, Model::Meet)
                && matches!(sem.rule, RuleMeaning::SupportImplication))
        {
            assert_eq!(encoding_mismatches, 0);
        } else {
            assert!(encoding_mismatches > 0);
        }
        for include_rule in [false, true] {
            for facts in &worlds {
                let program = if include_rule { &rules[..] } else { &[] };
                let admissible = models(sem, &worlds, facts, program);
                let formula_entailment = entails(sem, &admissible, &implication);
                let rule_entailment = entails_inequality(sem, &admissible, &tensor, &c);
                assert_eq!(
                    rule_entailment,
                    entails_inequality(sem, &admissible, &a, &curried),
                    "residuation in the SAME model class"
                );
                if formula_entailment != rule_entailment {
                    mismatches += 1;
                }
            }
        }
        println!(
            "RESIDUAL | {} | theories=128 | formula/order mismatches={mismatches}",
            sem.name
        );
        if matches!(sem.designation, Designation::UnitFilter) || matches!(sem.model, Model::Fusion)
        {
            assert_eq!(mismatches, 0);
        } else {
            assert!(mismatches > 0);
        }
    }
    let facts = Known::from([
        atoms[0].clone(),
        atoms[1].clone(),
        atoms[2].clone(),
        opposite(&atoms[2]),
    ]);
    let admissible = models(SEMANTICS[2], &worlds, &facts, &[]);
    assert!(entails(SEMANTICS[2], &admissible, &implication));
    assert!(!entails_inequality(SEMANTICS[2], &admissible, &tensor, &c));
    println!(
        "{:?}: Meet/positive designates residual B although I=T is not <= B; {{A,B,C,~C}} entails implication but not its inequality",
        Classification::RuleFormulaMismatch
    );
}
