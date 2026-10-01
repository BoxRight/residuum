//! Two independent relations on the same source programs and finite domain.
//! The embedding is explicit and experimental, not an order on runtime Module.

use super::*;
use crate::algebra::tests::evidence_pairs::{FUSION_VALUES, Fusion};
use crate::algebra::{CommutativeResiduatedLattice, Lattice, Monoid, Preorder};

// Exhaust the subsets of this acyclic vocabulary, not all possible programs.
const EDGES: [&str; 3] = [
    "rule protect(owner, object, t) = owns(owner, object) @ t <= protected(owner, object) @ after t",
    "rule insure(owner, object, t) = owns(owner, object) @ t <= insured(owner, object) @ after t",
    "rule finish(owner, object, t) = protected(owner, object) @ t <= insured(owner, object) @ t",
];

fn implementation(mask: usize) -> String {
    let mut source = String::from("module FiniteComparison\n");
    for (i, edge) in EDGES.iter().enumerate() {
        if mask & (1 << i) != 0 {
            source.push_str(edge);
            source.push('\n');
        }
    }
    source
}

fn specification(mask: usize) -> String {
    assert!((1..4).contains(&mask));
    let mut source = SPEC
        .split("rule protectionGuarantee")
        .next()
        .unwrap()
        .to_owned();
    for (i, edge) in EDGES[..2].iter().enumerate() {
        if mask & (1 << i) != 0 {
            source.push_str(edge);
            source.push('\n');
        }
    }
    source
}

#[derive(Clone, Copy, Debug)]
enum Guarantee {
    Protected,
    BothRules,
    Insured,
    Alternative,
    Product,
}

const GUARANTEES: [Guarantee; 5] = [
    Guarantee::Protected,
    Guarantee::BothRules,
    Guarantee::Insured,
    Guarantee::Alternative,
    Guarantee::Product,
];

impl Guarantee {
    fn head_value(self, world: &[Fusion; 3]) -> Fusion {
        let [_, p, q] = *world;
        match self {
            Self::Protected => p,
            Self::Insured => q,
            Self::BothRules => p.meet(&q),
            Self::Alternative => p.join(&q),
            Self::Product => p.tensor(&q),
        }
    }

    fn operational(self, supported: [bool; 2]) -> bool {
        let [p, q] = supported;
        match self {
            Self::Protected => p,
            Self::Insured => q,
            Self::Alternative => p || q,
            Self::BothRules | Self::Product => p && q,
        }
    }

    fn value(self, world: &[Fusion; 3]) -> Fusion {
        let [owns, p, q] = *world;
        match self {
            Self::Protected => owns.residual(&p),
            Self::Insured => owns.residual(&q),
            Self::BothRules => owns.residual(&p).meet(&owns.residual(&q)),
            Self::Alternative => owns.residual(&p.join(&q)),
            Self::Product => owns.residual(&p.tensor(&q)),
        }
    }
}

type GroundTheory = Vec<(usize, usize)>;

// Use the real instantiator to ground the actual parsed rules, then project
// their Events into THREE local atoms. No private atom is eliminated silently.
fn ground(rules: &[Rule], atoms: &[Event; 3]) -> GroundTheory {
    rules
        .iter()
        .map(|rule| {
            let instances: Vec<_> = atoms
                .iter()
                .enumerate()
                .filter_map(|(a, event)| instantiate(rule, event).map(|d| (a, d)))
                .collect();
            assert_eq!(instances.len(), 1);
            let (a, instance) = &instances[0];
            let b = atoms.iter().position(|e| *e == instance.event).unwrap();
            assert_eq!(instance.record.antecedents, vec![atoms[*a].clone()]);
            (*a, b)
        })
        .collect()
}

fn top() -> Fusion {
    *FUSION_VALUES
        .iter()
        .find(|candidate| FUSION_VALUES.iter().all(|v| v.le(candidate)))
        .unwrap()
}

fn theory_value(theory: &GroundTheory, world: &[Fusion; 3]) -> Fusion {
    // A set of axioms is a logical meet, NOT tensor. Empty theory = truth top,
    // not the fusion unit B and not empty knowledge U.
    theory.iter().fold(top(), |acc, (a, b)| {
        acc.meet(&world[*a].residual(&world[*b]))
    })
}

fn worlds() -> Vec<[Fusion; 3]> {
    let mut result = Vec::new();
    for a in FUSION_VALUES {
        for b in FUSION_VALUES {
            for c in FUSION_VALUES {
                result.push([a, b, c]);
            }
        }
    }
    // Minimal witness = least total support bits, then deterministic labels.
    result.sort_by_key(|world| {
        (
            world
                .iter()
                .map(|v| match v.label() {
                    "U" => 0,
                    "B" => 2,
                    _ => 1,
                })
                .sum::<usize>(),
            world.map(Fusion::label),
        )
    });
    assert_eq!(result.len(), 64);
    result
}

struct Probe {
    atoms: [Event; 3],
    theory: GroundTheory,
    supported: [bool; 2],
}

struct Candidate {
    mask: usize,
    probes: Vec<Probe>,
    checker: [bool; 3],
}

fn candidates() -> Vec<Candidate> {
    let domain = ownership_domain();
    let specifications = [specification(1), specification(3), specification(2)];
    // The named fixtures really are the same programs, modulo rule names.
    for (source, expected) in [(SPEC, 1), (STRONG, 3)] {
        let goals: Vec<_> = rules(&elaborate_formal(parse_formal(source).unwrap()).unwrap())
            .iter()
            .map(|r| instantiate(r, &domain[0]).unwrap().event)
            .collect();
        let generated: Vec<_> =
            rules(&elaborate_formal(parse_formal(&specification(expected)).unwrap()).unwrap())
                .iter()
                .map(|r| instantiate(r, &domain[0]).unwrap().event)
                .collect();
        assert_eq!(goals, generated);
    }
    (0..8)
        .map(|mask| {
            let source = implementation(mask);
            let checks: Vec<_> = specifications
                .iter()
                .map(|s| refine(s, &source, &domain).unwrap())
                .collect();
            let typed_rules = rules(&checks[0].implementation);
            let probes = domain
                .iter()
                .map(|owns| {
                    let p = instantiate(
                        &rules(&elaborate_formal(parse_formal(SPEC).unwrap()).unwrap())[0],
                        owns,
                    )
                    .unwrap()
                    .event;
                    let q = instantiate(
                        &rules(
                            &elaborate_formal(parse_formal(&specifications[2]).unwrap()).unwrap(),
                        )[0],
                        owns,
                    )
                    .unwrap()
                    .event;
                    let atoms = [owns.clone(), p, q];
                    let closure = close(vec![owns.clone()], &typed_rules, &mut Clock::default());
                    Probe {
                        theory: ground(&typed_rules, &atoms),
                        supported: [
                            closure.known.contains(&atoms[1]),
                            closure.known.contains(&atoms[2]),
                        ],
                        atoms,
                    }
                })
                .collect();
            let result = Candidate {
                mask,
                probes,
                checker: std::array::from_fn(|i| checks[i].accepted()),
            };
            for (i, guarantee) in GUARANTEES[..3].iter().enumerate() {
                assert_eq!(
                    result.checker[i],
                    result
                        .probes
                        .iter()
                        .all(|p| guarantee.operational(p.supported))
                );
            }
            result
        })
        .collect()
}

#[derive(Debug)]
struct Comparison {
    horn: bool,
    algebra: bool,
    models: bool,
    contextual_models: bool,
    witness: Option<([Fusion; 3], Fusion, Fusion)>,
}

fn compare(candidate: &Candidate, guarantee: Guarantee) -> Comparison {
    let valuations = worlds();
    let mut result = Comparison {
        horn: true,
        algebra: true,
        models: true,
        contextual_models: true,
        witness: None,
    };
    for probe in &candidate.probes {
        result.horn &= guarantee.operational(probe.supported);
        for world in &valuations {
            let implementation = theory_value(&probe.theory, world);
            let specification = guarantee.value(world);
            let ordered = implementation.le(&specification);
            result.algebra &= ordered;
            result.models &=
                !Fusion::unit().le(&implementation) || Fusion::unit().le(&specification);
            // A distinct CONTROL: model consequence after assuming the owns
            // premise as a supported fact. This is not the pointwise order.
            result.contextual_models &= !Fusion::unit().le(&world[0])
                || !Fusion::unit().le(&implementation)
                || Fusion::unit().le(&guarantee.head_value(world));
            if !ordered && result.witness.is_none() {
                result.witness = Some((*world, implementation, specification));
            }
        }
    }
    result
}

#[test]
fn finite_refinement_and_four_order_compare_same_three_programs() {
    let candidates = candidates();
    let expected = [
        [true, false, false],
        [true, true, true],
        [false, false, true],
    ];
    println!("implementation | Horn Spec0/1/2 | FOUR pointwise Spec0/1/2 | FOUR models Spec0/1/2");
    for (i, mask) in [1, 3, 2].into_iter().enumerate() {
        let comparisons: Vec<_> = GUARANTEES[..3]
            .iter()
            .map(|g| compare(&candidates[mask], *g))
            .collect();
        assert_eq!(candidates[mask].checker, expected[i]);
        println!(
            "Impl{i} | {:?} | {:?} | {:?}",
            candidates[mask].checker,
            comparisons.iter().map(|c| c.algebra).collect::<Vec<_>>(),
            comparisons.iter().map(|c| c.models).collect::<Vec<_>>()
        );
        // Exact one-step tables coincide. This is NOT the general theorem.
        assert_eq!(
            comparisons.iter().map(|c| c.algebra).collect::<Vec<_>>(),
            expected[i]
        );
        assert_eq!(
            comparisons.iter().map(|c| c.models).collect::<Vec<_>>(),
            expected[i]
        );
    }
}

#[test]
fn finite_refinement_four_join_and_tensor_are_checked_independently() {
    let candidates = candidates();
    println!(
        "implementation | guarantee | Horn | FOUR pointwise | FOUR models | FOUR with owns premise | countervaluation"
    );
    for mask in [1, 3, 2] {
        for guarantee in GUARANTEES {
            let c = compare(&candidates[mask], guarantee);
            println!(
                "{mask:03b} | {guarantee:?} | {} | {} | {} | {} | {:?}",
                c.horn,
                c.algebra,
                c.models,
                c.contextual_models,
                c.witness
                    .map(|(w, i, s)| (w.map(Fusion::label), i.label(), s.label()))
            );
        }
    }
    assert!(compare(&candidates[1], Guarantee::Alternative).algebra);
    assert!(compare(&candidates[2], Guarantee::Alternative).algebra);
    let product = compare(&candidates[3], Guarantee::Product);
    assert!(product.horn && product.contextual_models);
    assert!(!product.algebra && !product.models);
    let (world, implementation, specification) = product.witness.unwrap();
    assert_eq!(world.map(Fusion::label), ["U", "U", "U"]);
    assert_eq!((implementation.label(), specification.label()), ("T", "U"));
    for world in worlds() {
        let [o, p, q] = world;
        assert_eq!(o.residual(&p.meet(&q)), Guarantee::BothRules.value(&world));
        assert_eq!(
            Fusion::unit().le(&Guarantee::Product.value(&world)),
            o.tensor(&Fusion::unit()).le(&p.tensor(&q))
        );
    }
    assert!(
        worlds()
            .iter()
            .any(|w| Guarantee::BothRules.value(w) != Guarantee::Product.value(w))
    );
    assert!(worlds().iter().any(|w| {
        Guarantee::Alternative.value(w) != w[0].residual(&w[1]).join(&w[0].residual(&w[2]))
    }));
}

#[test]
fn finite_refinement_four_order_exhaustive_search_records_counterexamples() {
    let candidates = candidates();
    let mut disagreements = [0; 2];
    let mut models_disagreements = [0; 2];
    let mut minimal: [Option<(usize, Guarantee, Comparison)>; 2] = [None, None];
    // Sorted by number of rules, so the first retained witness is globally
    // minimal in rule count WITHIN this declared eight-program universe.
    let mut ordered: Vec<_> = candidates.iter().collect();
    ordered.sort_by_key(|c| (c.mask.count_ones(), c.mask));
    for candidate in ordered {
        for guarantee in GUARANTEES {
            let c = compare(candidate, guarantee);
            assert_eq!(c.horn, c.contextual_models);
            if c.horn != c.models {
                models_disagreements[usize::from(!c.horn)] += 1;
            }
            if c.horn != c.algebra {
                println!(
                    "discrepancy: mask={:03b}, guarantee={guarantee:?}, Horn={}, order={}, models={}, contextual_models={}",
                    candidate.mask, c.horn, c.algebra, c.models, c.contextual_models
                );
                let direction = usize::from(!c.horn);
                disagreements[direction] += 1;
                if minimal[direction].is_none() {
                    minimal[direction] = Some((candidate.mask, guarantee, c));
                }
            }
        }
    }
    println!(
        "Universe: 8 programs x 5 specs x 8 owns inputs x 64 FOUR valuations = 20480 comparisons"
    );
    println!(
        "Horn true / order false: {}; Horn false / order true: {}",
        disagreements[0], disagreements[1]
    );
    println!("Horn/model disagreements, respectively: {models_disagreements:?}");
    assert_eq!(disagreements, [5, 0]);
    assert_eq!(models_disagreements, [3, 0]);
    for (direction, counter) in minimal.iter().enumerate() {
        if let Some((mask, guarantee, c)) = counter {
            println!("minimal direction {direction}: mask={mask:03b}, spec={guarantee:?}, {c:?}");
            println!(
                "input: {}",
                render_event(&candidates[*mask].probes[0].atoms[0])
            );
            for (i, edge) in EDGES.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    println!("  {edge}");
                }
            }
            if let Some((world, i, s)) = c.witness {
                println!(
                    "  [owns,protected,insured]={:?}, Impl={}, Spec={}",
                    world.map(Fusion::label),
                    i.label(),
                    s.label()
                );
            }
        } else {
            println!("direction {direction}: no counterexample in this finite universe");
        }
    }
    assert!(
        minimal[0].is_some(),
        "do not turn finite Horn refinement into pointwise algebra order"
    );
    // A separate minimal counterexample using ONLY ordinary Horn specs.
    let chain = compare(&candidates[5], Guarantee::Insured);
    assert!(chain.horn && chain.models && chain.contextual_models);
    assert!(!chain.algebra);
    let (world, implementation, specification) = chain.witness.unwrap();
    println!(
        "minimal ordinary-Horn counterexample: owns -> protected -> insured; spec owns -> insured"
    );
    println!(
        "  [owns,protected,insured]={:?}, Impl={}, Spec={}",
        world.map(Fusion::label),
        implementation.label(),
        specification.label()
    );
    // A local witness extends to the entire eight-input universe: the seven
    // other components may all be T, making every other axiom/head formula T.
    let all_true = [top(); 3];
    assert_eq!(
        theory_value(&candidates[5].probes[0].theory, &all_true),
        top()
    );
    assert_eq!(Guarantee::Insured.value(&all_true), top());
    assert!(!implementation.meet(&top()).le(&specification.meet(&top())));
    // A rule's designated residual really uses the existing adjunction.
    for world in worlds() {
        assert_eq!(
            Fusion::unit().le(&world[0].residual(&world[1])),
            world[0].le(&world[1])
        );
    }
}
