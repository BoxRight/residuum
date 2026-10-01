//! Source-defined finite experiments only. No order or composition operation
//! is installed on Module, Verb, Rule, or the production inference engines.
use super::*;
use crate::algebra::Preorder;
use crate::algebra::tests::evidence_pairs::{FUSION_VALUES, Fusion};
use crate::surface::FormalDeclaration;
use crate::typed::PropositionKind;
use crate::{IncrementalClosure, close_program, elaborate_formal, parse_formal};

const DOMAIN: &str = include_str!("../../examples/experiments/composition_domain.res");
const ID: &str = include_str!("../../examples/experiments/composition_identity.res");
const P: &str = include_str!("../../examples/experiments/composition_p.res");
const PE: &str = include_str!("../../examples/experiments/composition_p_extra.res");
const Q: &str = include_str!("../../examples/experiments/composition_q.res");
const QE: &str = include_str!("../../examples/experiments/composition_q_extra.res");
const INLINE: &str = include_str!("../../examples/experiments/composition_inline.res");

// Fixture linking, not imports or a language feature. A fragment may annotate
// an identical catalog signature with its source-written program reference.
fn module(fragments: &[&str]) -> Module {
    let mut catalog = parse_formal(DOMAIN).unwrap();
    for source in fragments {
        for declaration in parse_formal(source).unwrap().declarations {
            if let FormalDeclaration::Verb(incoming) = &declaration {
                let existing = catalog
                    .declarations
                    .iter_mut()
                    .find_map(|d| match d {
                        FormalDeclaration::Verb(v) if v.name == incoming.name => Some(v),
                        _ => None,
                    })
                    .expect("association must annotate a catalog verb");
                let mut signature = incoming.clone();
                signature.program = existing.program.clone();
                assert_eq!(*existing, signature, "fragments cannot change signatures");
                assert!(existing.program.is_none(), "cannot replace an association");
                existing.program = incoming.program.clone();
            } else {
                assert!(matches!(declaration, FormalDeclaration::Rule(_)));
                catalog.declarations.push(declaration);
            }
        }
    }
    let typed = elaborate_formal(catalog).unwrap();
    require_acyclic(&rules(&typed)).unwrap();
    // The incremental Horn adapter is deliberately restricted to programs
    // whose body is their own fully parameterized positive trigger.
    for declaration in &typed.declarations {
        let Declaration::Verb(v) = declaration else {
            continue;
        };
        let Some(reference) = &v.program else {
            continue;
        };
        let target = typed
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Rule(r) if r.name == reference.name => Some(r),
                _ => None,
            })
            .unwrap();
        let clause = horn(target);
        let Antecedent::Event(trigger) = clause.antecedent else {
            panic!("self-anchored programs only")
        };
        assert_eq!(trigger.verb, v.name);
        assert_eq!(trigger.polarity, Polarity::Positive);
        assert_eq!(
            trigger.args,
            target.parameters[..v.args.len()]
                .iter()
                .map(|p| Term::Var(p.name.clone()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            trigger.proposition.time,
            TimeExpr::At(target.parameters.last().unwrap().name.clone())
        );
    }
    typed
}

fn p(rank: usize) -> Vec<&'static str> {
    match rank {
        0 => vec![ID],
        1 => vec![P],
        2 => vec![P, PE],
        _ => unreachable!(),
    }
}
fn q(rank: usize) -> Vec<&'static str> {
    match rank {
        0 => vec![ID],
        1 => vec![Q],
        2 => vec![Q, QE],
        _ => unreachable!(),
    }
}
fn rules(module: &Module) -> Vec<Rule> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect()
}
fn input(module: &Module, name: &str) -> Vec<Event> {
    experiment(module, name).input.clone()
}
fn experiment<'a>(module: &'a Module, name: &str) -> &'a Experiment {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}
fn powerset(basis: &[Event]) -> Vec<Vec<Event>> {
    assert!(basis.len() <= 8, "finite harness bound");
    (0..1 << basis.len())
        .map(|mask| {
            basis
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, e)| e.clone())
                .collect()
        })
        .collect()
}
fn keys(closure: &Closure) -> BTreeSet<Event> {
    closure.known.iter().cloned().collect()
}
fn run(module: &Module, input: &[Event]) -> Closure {
    close_program(input.to_vec(), module, &mut Clock::default()).unwrap()
}
fn staged(p: &Module, q: &Module, input: &[Event]) -> Closure {
    let mut clock = Clock::default();
    let first = close_program(input.to_vec(), p, &mut clock).unwrap();
    let mut second = close_program(first.known, q, &mut clock).unwrap();
    second.derivations.splice(0..0, first.derivations);
    second
}
fn exact_event(a: &Event, b: &Event) {
    assert_eq!(a, b);
    assert_eq!(a.proposition, b.proposition);
    assert_eq!(a.transition, b.transition);
}
fn same(a: &Closure, b: &Closure) {
    assert_eq!(keys(a), keys(b));
    for e in &a.known {
        exact_event(e, b.known.iter().find(|other| *other == e).unwrap());
    }
    assert_eq!(a.derivations.len(), b.derivations.len());
    for proof in &a.derivations {
        let compared = b
            .derivations
            .iter()
            .find(|d| d.event == proof.event && d.record == proof.record)
            .unwrap();
        exact_event(&proof.event, &compared.event);
        for (a, b) in proof
            .record
            .antecedents
            .iter()
            .zip(&compared.record.antecedents)
        {
            exact_event(a, b);
        }
        assert!(!proof.derived_at.name.is_empty());
    }
    assert_eq!(
        a.derivations
            .iter()
            .map(|d| &d.derived_at.name)
            .collect::<BTreeSet<_>>()
            .len(),
        a.derivations.len()
    );
}
fn merge_incremental(p: &IncrementalClosure, q: &IncrementalClosure) -> Closure {
    let mut closure = q.closure().clone();
    closure
        .derivations
        .splice(0..0, p.closure().derivations.clone());
    closure
}
fn difference(after: &Closure, before: &Closure) -> Closure {
    Closure {
        known: after
            .known
            .iter()
            .filter(|e| !before.known.contains(e))
            .cloned()
            .collect(),
        derivations: after
            .derivations
            .iter()
            .filter(|d| {
                !before
                    .derivations
                    .iter()
                    .any(|old| old.event == d.event && old.record == d.record)
            })
            .cloned()
            .collect(),
    }
}

#[test]
fn composition_staged_associated_inline_and_incremental_deltas_agree_exhaustively() {
    for rank in [1, 2] {
        let pm = module(&p(rank));
        let qm = module(&q(rank));
        let mut both = p(rank);
        both.extend(q(rank));
        let associated = module(&both);
        let inline = module(if rank == 1 {
            &[INLINE]
        } else {
            &[INLINE, PE, QE]
        });
        let inputs = powerset(&input(&associated, "seedDomain"));
        let (rp, rq, ri) = (rules(&pm), rules(&qm), rules(&inline));
        let reduced = rules(&qm)
            .into_iter()
            .find(|r| r.name == "protectionFromCharlie")
            .unwrap();
        assert_eq!(
            reduced,
            ri.iter().find(|r| r.name == reduced.name).unwrap().clone()
        );
        for base in &inputs {
            let a = run(&associated, base);
            same(&a, &staged(&pm, &qm, base));
            same(&a, &run(&inline, base));
            for additions in &inputs {
                // This restricted, self-anchored Horn view is verified against
                // active semantics above and after each update below. It does
                // not extend IncrementalClosure's production API to programs.
                let mut staged_clock = Clock::default();
                let mut ps = IncrementalClosure::new(base.clone(), &rp, &mut staged_clock);
                let mut qs =
                    IncrementalClosure::new(ps.closure().known.clone(), &rq, &mut staged_clock);
                let mut inline_clock = Clock::default();
                let mut is = IncrementalClosure::new(base.clone(), &ri, &mut inline_clock);
                same(&a, &merge_incremental(&ps, &qs));
                same(&a, is.closure());
                let dp = ps.update(additions.clone(), &mut staged_clock);
                let dq = qs.update(dp.known, &mut staged_clock);
                let di = is.update(additions.clone(), &mut inline_clock);
                let mut combined = base.clone();
                combined.extend(additions.clone());
                let full = run(&associated, &combined);
                same(&full, &merge_incremental(&ps, &qs));
                same(&full, is.closure());
                same(&full, &staged(&pm, &qm, &combined));
                let staged_delta = Closure {
                    known: dq.known,
                    derivations: dp.derivations.into_iter().chain(dq.derivations).collect(),
                };
                same(
                    &staged_delta,
                    &Closure {
                        known: di.known,
                        derivations: di.derivations,
                    },
                );
                same(&staged_delta, &difference(&full, &a));
                // Repeated inputs create neither events nor new proof records.
                let repeated_p = ps.update(additions.clone(), &mut staged_clock);
                assert!(repeated_p.known.is_empty() && repeated_p.derivations.is_empty());
                let repeated_q = qs.update(repeated_p.known, &mut staged_clock);
                let repeated_i = is.update(additions.clone(), &mut inline_clock);
                assert!(repeated_q.known.is_empty() && repeated_q.derivations.is_empty());
                assert!(repeated_i.known.is_empty() && repeated_i.derivations.is_empty());
            }
        }
    }
    println!(
        "composition/deltas: 2 families x 16 base states x 16 additions = 512 updates; exact events, metadata, substitutions and provenance agree; duplicate updates empty"
    );
}

fn preorder(table: &[Vec<BTreeSet<Event>>]) -> Vec<Vec<bool>> {
    table
        .iter()
        .map(|p| {
            table
                .iter()
                .map(|q| p.iter().zip(q).all(|(p, q)| p.is_subset(q)))
                .collect()
        })
        .collect()
}
fn print_order(name: &str, order: &[Vec<bool>]) {
    println!("{name}: rows/columns identity, basic, extra; order is observable inclusion");
    for row in order {
        println!("  {row:?}");
    }
}

#[test]
fn finite_program_order_and_composition_are_monotone_in_each_argument() {
    let ps: Vec<_> = (0..3).map(|rank| module(&p(rank))).collect();
    let qs: Vec<_> = (0..3).map(|rank| module(&q(rank))).collect();
    let mut basis = input(&ps[0], "seedDomain");
    basis.extend(input(&ps[0], "knowledgeDomain"));
    let omega = powerset(&basis);
    assert_eq!(omega.len(), 256);
    let pt: Vec<Vec<_>> = ps
        .iter()
        .map(|p| omega.iter().map(|x| keys(&run(p, x))).collect())
        .collect();
    // Q's observational domain must include ALL actual intermediate inputs,
    // not merely the original sale inputs on which Q could appear vacuous.
    let mut q_domain: BTreeSet<BTreeSet<Event>> =
        omega.iter().map(|x| x.iter().cloned().collect()).collect();
    q_domain.extend(pt.iter().flatten().cloned());
    let q_inputs: Vec<Vec<_>> = q_domain
        .iter()
        .map(|x| x.iter().cloned().collect())
        .collect();
    let qt: Vec<Vec<_>> = qs
        .iter()
        .map(|q| q_inputs.iter().map(|x| keys(&run(q, x))).collect())
        .collect();
    let po = preorder(&pt);
    let qo = preorder(&qt);
    let expected = vec![
        vec![true, true, true],
        vec![false, true, true],
        vec![false, false, true],
    ];
    assert_eq!(po, expected);
    assert_eq!(qo, expected);
    // Check knowledge monotonicity independently of program comparison.
    let q_on_omega: Vec<Vec<_>> = qs
        .iter()
        .map(|q| omega.iter().map(|x| keys(&run(q, x))).collect())
        .collect();
    for table in pt.iter().chain(&q_on_omega) {
        for a in 0..256 {
            for b in 0..256 {
                if a & b == a {
                    assert!(table[a].is_subset(&table[b]));
                }
            }
        }
    }
    let composed: Vec<Vec<Vec<_>>> = ps
        .iter()
        .map(|p| {
            qs.iter()
                .map(|q| omega.iter().map(|x| keys(&staged(p, q, x))).collect())
                .collect()
        })
        .collect();
    let mut checks = [0, 0];
    for a in 0..3 {
        for b in 0..3 {
            for fixed in 0..3 {
                for x in 0..256 {
                    if po[a][b] {
                        assert!(composed[a][fixed][x].is_subset(&composed[b][fixed][x]));
                        checks[0] += 1;
                    }
                    if qo[a][b] {
                        assert!(composed[fixed][a][x].is_subset(&composed[fixed][b][x]));
                        checks[1] += 1;
                    }
                }
            }
        }
    }
    print_order("P", &po);
    print_order("Q", &qo);
    println!(
        "Omega: 256 knowledge inputs; Q extended domain: {}; composition monotonicity: {} first-argument and {} second-argument comparisons; not <=_L",
        q_inputs.len(),
        checks[0],
        checks[1]
    );
}

fn fusion(mask: usize, pair: usize) -> Fusion {
    let label = match (
        (mask >> (2 * pair)) & 1 != 0,
        (mask >> (2 * pair + 1)) & 1 != 0,
    ) {
        (false, false) => "U",
        (true, false) => "T",
        (false, true) => "F",
        (true, true) => "B",
    };
    *FUSION_VALUES
        .iter()
        .find(|value| value.label() == label)
        .unwrap()
}

#[test]
fn finite_activation_observation_adjunction_is_scoped_and_requirements_have_alternatives() {
    let mut both = p(2);
    both.extend(q(2));
    let program = module(&both);
    let mut basis = input(&program, "seedDomain");
    basis.extend(input(&program, "competingSource"));
    assert_eq!(basis.len(), 6);
    for pair in basis.chunks_exact(2) {
        assert_eq!(pair[0].polarity, Polarity::Positive);
        assert_eq!(pair[1].polarity, Polarity::EvidentialNot);
        let mut opposite = pair[0].clone();
        opposite.polarity = Polarity::EvidentialNot;
        assert_eq!(opposite, pair[1]);
    }
    let inputs = powerset(&basis);
    let observations: Vec<_> = inputs
        .iter()
        .map(|x| {
            keys(&run(&program, x))
                .into_iter()
                .filter(|e| e.proposition.kind == PropositionKind::Derived)
                .collect::<BTreeSet<_>>()
        })
        .collect();
    let output_basis: Vec<_> = observations
        .iter()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(output_basis.len(), 8);
    let f: Vec<usize> = observations
        .iter()
        .map(|events| {
            output_basis
                .iter()
                .enumerate()
                .filter(|(_, e)| events.contains(*e))
                .map(|(i, _)| 1 << i)
                .sum()
        })
        .collect();
    let goals = 1 << output_basis.len();
    let target = &experiment(&program, "seedDomain").goal;
    let target_mask = 1 << output_basis.iter().position(|e| e == target).unwrap();
    for truth in [false, true] {
        let source_order: Vec<Vec<_>> = (0..64)
            .map(|a| {
                (0..64)
                    .map(|b| {
                        if truth {
                            (0..3).all(|pair| fusion(a, pair).le(&fusion(b, pair)))
                        } else {
                            a & b == a
                        }
                    })
                    .collect()
            })
            .collect();
        let mut upper = Vec::new();
        let mut missing_lower = Vec::new();
        for y in 0..goals {
            let admissible: Vec<_> = (0..64).filter(|&x| f[x] & y == f[x]).collect();
            let greatest = admissible
                .iter()
                .copied()
                .find(|&candidate| admissible.iter().all(|&x| source_order[x][candidate]));
            let g = greatest.expect("this finite activation map has a right adjoint");
            upper.push(g);
            for x in 0..64 {
                assert_eq!(f[x] & y == f[x], source_order[x][g]);
            }
            let sufficient: Vec<_> = (0..64).filter(|&x| y & f[x] == y).collect();
            let least = sufficient
                .iter()
                .copied()
                .find(|&candidate| sufficient.iter().all(|&x| source_order[candidate][x]));
            if let Some(left) = least {
                for x in 0..64 {
                    assert_eq!(source_order[left][x], y & f[x] == y);
                }
            } else {
                missing_lower.push(y);
            }
        }
        assert!(missing_lower.contains(&target_mask));
        // A right adjoint here bounds permitted effects; it is not abduction.
        assert_eq!(f[upper[0]], 0);
        println!(
            "activation evidence ({}) -> program observations (inclusion): F -| G holds for all 64 x 256 pairs; no L -| F: {} goals lack a least sufficient input",
            if truth { "FOUR truth" } else { "knowledge" },
            missing_lower.len()
        );
    }
    let sufficient: Vec<usize> = (0..64)
        .filter(|&x| target_mask & f[x] == target_mask)
        .collect();
    let minimal: Vec<_> = sufficient
        .iter()
        .copied()
        .filter(|&x| !sufficient.iter().any(|&y| y != x && y & x == y))
        .collect();
    assert_eq!(minimal.len(), 2);
    assert!(minimal.iter().all(|x| x.count_ones() == 1));
    assert_eq!(f[minimal[0] & minimal[1]], 0);
    println!(
        "no unique least requirement for {}: {} OR {}; common lower knowledge bound derives no goal",
        render_event(target),
        render_event(&basis[minimal[0].trailing_zeros() as usize]),
        render_event(&basis[minimal[1].trailing_zeros() as usize])
    );
    // Even knowledge meet preservation fails: two independent sources produce
    // the same observable. No abstract syntax or forced adjunction is added.
    assert_ne!(f[minimal[0]] & f[minimal[1]], f[minimal[0] & minimal[1]]);
}
