//! Theory/model bridge only. No new inference policy in the production engine.
use super::*;
use crate::parse_formal_to_typed;
use crate::typed::{Declaration, Experiment, ExperimentInputKind, ExperimentQueryKind};

const THEORY: &str = include_str!("../../../../../../examples/experiments/theory_bridge.res");
const TENSOR: &str =
    include_str!("../../../../../../examples/experiments/theory_bridge_tensor.res");

fn intersection(worlds: &[&Known], top: &Known) -> Known {
    worlds.iter().fold(top.clone(), |core, world| {
        core.intersection(world).cloned().collect()
    })
}

fn finite_programs(atoms: &[Event]) -> Vec<Vec<Rule>> {
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
    let mut programs = vec![vec![]];
    programs.extend(candidates.iter().map(|r| vec![r.clone()]));
    for i in 0..20 {
        for j in i + 1..20 {
            programs.push(vec![candidates[i].clone(), candidates[j].clone()]);
        }
    }
    assert_eq!(programs.len(), 251);
    programs
}

fn literals(body: &Antecedent, result: &mut Vec<Event>) {
    match body {
        Antecedent::Unit => {}
        Antecedent::Event(event) => result.push(event.clone()),
        Antecedent::And(a, b) => {
            literals(a, result);
            literals(b, result);
        }
    }
}

// Algebraic hypothesis confined to the finite model harness: a fusion truth
// inequality includes forward support and one rotated constraint per factor.
// This does NOT install contraposition or these rules in Residuum programs.
fn fusion_constraints(rules: &[Rule]) -> Vec<Rule> {
    let mut result = rules.to_vec();
    for rule in rules {
        let body = clause(rule);
        let mut factors = Vec::new();
        literals(&body.antecedent, &mut factors);
        for (i, factor) in factors.iter().enumerate() {
            let mut antecedent = Antecedent::Event(opposite(&body.consequent));
            for (j, other) in factors.iter().enumerate() {
                if i != j {
                    antecedent = Antecedent::And(
                        Box::new(antecedent),
                        Box::new(Antecedent::Event(other.clone())),
                    );
                }
            }
            result.push(ground_rule(
                &format!("{}.model-constraint.{i}", rule.name),
                antecedent,
                opposite(factor),
            ));
        }
    }
    result
}

#[test]
fn finite_theory_models_preserve_horn_saturation_and_distinguish_semantic_hulls() {
    let atoms = logical_atoms(&["P", "Q"]);
    let worlds = interpretations(&atoms);
    let top = worlds.last().unwrap();
    let mut totals = [(0usize, 0usize, 0usize); 2];
    for rules in finite_programs(&atoms) {
        let completed = fusion_constraints(&rules);
        let bases: [Vec<&Known>; 2] =
            [SEMANTICS[3], SEMANTICS[0]].map(|sem| models(sem, &worlds, &Known::new(), &rules));
        // Independent semantics: equality of model classes, not closure values.
        assert_eq!(
            models(SEMANTICS[3], &worlds, &Known::new(), &completed),
            bases[1]
        );
        for (index, sem) in [SEMANTICS[3], SEMANTICS[0]].into_iter().enumerate() {
            let base = &bases[index];
            assert!(
                base.contains(&top),
                "both evidence for all atoms is a model"
            );
            for a in base {
                for b in base {
                    let meet = a.intersection(b).cloned().collect::<Known>();
                    assert!(
                        base.iter().any(|world| **world == meet),
                        "knowledge intersections must stay models in this family"
                    );
                }
            }
            for facts in &worlds {
                let original = models(sem, &worlds, facts, &rules);
                let closed = horn_closure(facts, &rules);
                let saturated = models(sem, &worlds, &closed, &rules);
                assert_eq!(original, saturated, "Mod(S,R) = Mod(close(S),R)");
                let core = intersection(&original, top);
                assert_eq!(original, models(sem, &worlds, &core, &rules));
                assert!(
                    original.iter().any(|world| **world == core),
                    "core is an actual least model"
                );
                assert!(
                    closed.is_subset(&core),
                    "explicit Horn is sound for signed atomic entailment"
                );
                totals[index].0 += 1;
                totals[index].1 += usize::from(closed != core);
                totals[index].2 += core.difference(&closed).count();
                if index == 0 {
                    assert_eq!(closed, core, "support-model least model equals Horn");
                } else {
                    assert_eq!(
                        horn_closure(facts, &completed),
                        core,
                        "test-only fusion constraints describe this semantic hull"
                    );
                }
            }
            // Adding evidence restricts models, even though adding evidence to
            // an individual model need not preserve satisfaction of its rules.
            for a in &worlds {
                for b in &worlds {
                    if a.is_subset(b) {
                        let ma = models(sem, &worlds, a, &rules);
                        let mb = models(sem, &worlds, b, &rules);
                        assert!(mb.iter().all(|world| ma.contains(world)));
                    }
                }
            }
        }
    }
    println!(
        "| Semantics | Theories | Model-class changes after Horn | Horn != least model | Extra entailed literals |\n|---|---:|---:|---:|---:|"
    );
    for (name, total) in ["support", "FOUR fusion / truth / I-filter"]
        .into_iter()
        .zip(totals)
    {
        println!("| {name} | {} | 0 | {} | {} |", total.0, total.1, total.2);
    }
    assert_eq!(totals[0], (4016, 0, 0));
    assert_eq!(totals[1], (4016, 640, 680));
}

#[test]
fn finite_evidence_model_class_polarity_forms_an_adjunction_with_reverse_inclusion() {
    let atoms = logical_atoms(&["P", "Q"]);
    let worlds = interpretations(&atoms);
    let implication = ground_rule(
        "implication",
        Antecedent::Event(atoms[0].clone()),
        atoms[1].clone(),
    );
    let mut pairs = 0;
    for rules in [vec![], vec![implication]] {
        for sem in [SEMANTICS[3], SEMANTICS[0]] {
            let admissible: Vec<usize> = worlds
                .iter()
                .enumerate()
                .filter_map(|(i, world)| {
                    sem.satisfies_theory(world, &Known::new(), &rules)
                        .then_some(i)
                })
                .collect();
            assert!(admissible.len() <= 16);
            let mod_masks: Vec<u64> = worlds
                .iter()
                .map(|facts| {
                    admissible
                        .iter()
                        .filter(|i| sem.satisfies_theory(&worlds[**i], facts, &rules))
                        .fold(0, |mask, i| mask | (1u64 << i))
                })
                .collect();
            for class in 0usize..1 << admissible.len() {
                let mut class_mask = 0u64;
                let mut common_evidence = 15usize; // intersection of empty class is top_K
                for (position, world) in admissible.iter().enumerate() {
                    if class & (1 << position) != 0 {
                        class_mask |= 1 << world;
                        common_evidence &= world;
                    }
                }
                for (facts, model_mask) in mod_masks.iter().enumerate() {
                    assert_eq!(
                        class_mask & model_mask == class_mask,
                        facts & common_evidence == facts,
                        "C subset Mod_R(S) iff S subset Common(C)"
                    );
                    pairs += 1;
                }
            }
            let hulls: Vec<usize> = mod_masks
                .iter()
                .map(|mask| {
                    admissible
                        .iter()
                        .filter(|i| mask & (1 << **i) != 0)
                        .fold(15, |core, world| core & world)
                })
                .collect();
            for a in 0..16 {
                assert_eq!(a & hulls[a], a);
                assert_eq!(hulls[hulls[a]], hulls[a]);
                for b in 0..16 {
                    if a & b == a {
                        assert_eq!(hulls[a] & hulls[b], hulls[a]);
                    }
                }
            }
            println!(
                "POLARITY | {} | rules={} | admissible worlds={} | model classes={}",
                sem.name,
                rules.len(),
                admissible.len(),
                1usize << admissible.len()
            );
        }
    }
    println!(
        "ADJUNCTION | Mod_R : K -> P(W_R)^op; Common is right adjoint | {pairs} evidence/class pairs checked"
    );
}

fn source_theory(source: &str) -> (Vec<Event>, Vec<Rule>, Vec<Experiment>) {
    let module = parse_formal_to_typed(source).unwrap();
    let mut atoms = Vec::new();
    let mut rules = Vec::new();
    let mut experiments = Vec::new();
    for declaration in module.declarations {
        match declaration {
            Declaration::Verb(verb) => atoms.push(atom(&verb.name)),
            Declaration::Rule(rule) => {
                assert!(rule.parameters.is_empty());
                rules.push(rule);
            }
            Declaration::Experiment(experiment) => {
                assert_eq!(experiment.input_kind, ExperimentInputKind::Premises);
                assert_eq!(experiment.query_kind, ExperimentQueryKind::FourFusion);
                experiments.push(experiment);
            }
            _ => {}
        }
    }
    // atom's historical helper sets kinds by name; source signatures are Derived.
    for event in &mut atoms {
        event.proposition.kind = PropositionKind::Derived;
    }
    (atoms, rules, experiments)
}

#[test]
fn source_theories_interpret_formulas_over_models_and_preserve_residuation() {
    let mut triples = 0;
    for source in [THEORY, TENSOR] {
        let (atoms, rules, experiments) = source_theory(source);
        let worlds = interpretations(&atoms);
        let mut formulas = vec![Formula::Unit, Formula::Bottom];
        formulas.extend(
            atoms
                .iter()
                .flat_map(|event| [Formula::Atom(event.clone()), Formula::Atom(opposite(event))]),
        );
        for experiment in experiments {
            let facts: Known = experiment.input.into_iter().collect();
            let snapshot = facts.clone();
            let interpretations = models(SEMANTICS[0], &worlds, &facts, &rules);
            let closed = horn_closure(&facts, &rules);
            let core = intersection(&interpretations, worlds.last().unwrap());
            assert_eq!(
                interpretations,
                models(SEMANTICS[0], &worlds, &closed, &rules)
            );
            assert!(interpretations.iter().any(|world| **world == core));
            for a in &formulas {
                for b in &formulas {
                    for c in &formulas {
                        let product = Formula::Tensor(Box::new(a.clone()), Box::new(b.clone()));
                        let residual = Formula::Residual(Box::new(b.clone()), Box::new(c.clone()));
                        let implication =
                            Formula::Residual(Box::new(product.clone()), Box::new(c.clone()));
                        let direct =
                            entails_inequality(SEMANTICS[0], &interpretations, &product, c);
                        assert_eq!(
                            direct,
                            entails_inequality(SEMANTICS[0], &interpretations, a, &residual)
                        );
                        assert_eq!(
                            direct,
                            entails(SEMANTICS[0], &interpretations, &implication)
                        );
                        let join = Formula::Join(Box::new(a.clone()), Box::new(b.clone()));
                        let meet = Formula::Meet(Box::new(a.clone()), Box::new(b.clone()));
                        assert_eq!(
                            entails_inequality(SEMANTICS[0], &interpretations, &join, c),
                            entails_inequality(SEMANTICS[0], &interpretations, a, c)
                                && entails_inequality(SEMANTICS[0], &interpretations, b, c)
                        );
                        assert_eq!(
                            entails_inequality(SEMANTICS[0], &interpretations, c, &meet),
                            entails_inequality(SEMANTICS[0], &interpretations, c, a)
                                && entails_inequality(SEMANTICS[0], &interpretations, c, b)
                        );
                        triples += 1;
                    }
                }
            }
            if experiment.name == "negative" {
                assert!(!closed.contains(&experiment.goal));
                assert!(core.contains(&experiment.goal));
            }
            if experiment.name == "empty" {
                // Even the least FOUR model cannot replace the whole theory
                // when interpreting non-knowledge-monotone formulas.
                let reverse = Formula::Residual(
                    Box::new(Formula::Atom(atoms[1].clone())),
                    Box::new(Formula::Atom(atoms[0].clone())),
                );
                assert!(SEMANTICS[0].satisfies(&core, &reverse));
                assert!(!entails(SEMANTICS[0], &interpretations, &reverse));
                println!(
                    "CORE_NOT_THEORY | empty / P <= Q | least-model valuation designates Q -o P, but the theory does not entail it"
                );
            }
            assert_eq!(
                core.contains(&experiment.goal),
                entails(
                    SEMANTICS[0],
                    &interpretations,
                    &Formula::Atom(experiment.goal.clone())
                )
            );
            if experiment.name == "closedButNotModel" || experiment.name == "inconsistentConclusion"
            {
                assert!(!SEMANTICS[0].satisfies_theory(&closed, &facts, &rules));
            }
            println!(
                "SOURCE | {} | facts={} | Horn={} | least FOUR model={} | models={} | Horn goal={} | FOUR goal={} | Horn valuation is model={}",
                experiment.name,
                facts_text(&facts),
                facts_text(&closed),
                facts_text(&core),
                interpretations.len(),
                closed.contains(&experiment.goal),
                core.contains(&experiment.goal),
                SEMANTICS[0].satisfies_theory(&closed, &facts, &rules)
            );
            assert_eq!(
                facts, snapshot,
                "model reasoning does not insert its conclusions into evidence"
            );
        }
    }
    println!(
        "RESIDUATION | {triples} theory-relative formula triples | formula/inequality/curried entailment agree"
    );
    assert_eq!(triples, 1592);
}
