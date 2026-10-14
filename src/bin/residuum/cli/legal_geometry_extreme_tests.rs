//! Extreme points of complete closed theories, distinct from supports of one goal.
use super::*;

type State = BTreeSet<Event>;

fn ids(state: &State, index: &BTreeMap<Event, usize>) -> Vec<String> {
    state.iter().map(|e| format!("E{:03}", index[e])).collect()
}

fn extreme_points(m: &Module, f: &State) -> State {
    assert_eq!(events(&close(m, f.iter().cloned().collect())), *f);
    f.iter()
        .filter(|x| {
            let without = f.iter().filter(|y| *y != *x).cloned().collect();
            !close(m, without).known.contains(*x)
        })
        .cloned()
        .collect()
}

fn reverse_delete_basis(m: &Module, f: &State, reverse: bool) -> State {
    let mut result = f.clone();
    let mut order: Vec<_> = f.iter().cloned().collect();
    if reverse {
        order.reverse();
    }
    for x in order {
        let mut smaller = result.clone();
        smaller.remove(&x);
        if events(&close(m, smaller.iter().cloned().collect())) == *f {
            result = smaller;
        }
    }
    result
}

fn verify_basis(m: &Module, f: &State, basis: &State) {
    assert_eq!(events(&close(m, basis.iter().cloned().collect())), *f);
    for x in basis {
        let smaller = basis.iter().filter(|y| *y != x).cloned().collect();
        assert_ne!(events(&close(m, smaller)), *f);
    }
    assert_eq!(reverse_delete_basis(m, f, false), *basis);
    assert_eq!(reverse_delete_basis(m, f, true), *basis);
}

fn theory_inputs(g: &Module) -> Vec<Event> {
    vec![
        experiment(g, "performance").input[0].clone(),
        experiment(g, "performance").input[5].clone(),
        experiment(g, "theoryCompliance").input[0].clone(),
        experiment(g, "theoryAsset").input[0].clone(),
    ]
}

fn index(p: &Module) -> BTreeMap<Event, usize> {
    universe(p)
        .into_iter()
        .enumerate()
        .map(|(i, e)| (e, i))
        .collect()
}

fn write_state(
    out: &mut String,
    name: &str,
    label: &str,
    f: &State,
    basis: &State,
    index: &BTreeMap<Event, usize>,
) {
    let free = f.len() - basis.len();
    assert!(
        free < 128,
        "generator count exceeds bounded integer representation"
    );
    // Counts the interval [basis,F]; does not enumerate a powerset.
    writeln!(
        out,
        "{name}\t{label}\t{}\t{}\t{:?}\t{:?}\t{}\t{:?}\t{:?}",
        f.len(),
        basis.len(),
        ids(f, index),
        ids(basis, index),
        1u128 << free,
        f.iter().map(render_event).collect::<Vec<_>>(),
        basis.iter().map(render_event).collect::<Vec<_>>()
    )
    .unwrap();
}

fn state_header() -> String {
    "state\tmodel\tclosed_count\tbasis_count\tclosed_ids\textreme_ids\tgenerator_count\tclosed_events\textreme_events\n".into()
}

fn certificates(
    m: &Module,
    name: &str,
    label: &str,
    f: &State,
    basis: &State,
    index: &BTreeMap<Event, usize>,
    out: &mut String,
    loss: &mut String,
) {
    for x in f {
        let without = f.iter().filter(|y| *y != x).cloned().collect();
        let c = close(m, without);
        let is_extreme = basis.contains(x);
        assert_eq!(!c.known.contains(x), is_extreme);
        let witnesses: Vec<_> = c
            .derivations
            .iter()
            .filter(|d| &d.event == x)
            .map(|d| {
                let origin = m.origin(&d.record.rule).unwrap();
                format!(
                    "{} | {} | {:?}",
                    origin.module,
                    origin.file.as_ref().unwrap().display(),
                    d.record
                )
            })
            .collect();
        assert_eq!(witnesses.is_empty(), is_extreme);
        let verb = m
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Verb(v) if v.name == x.verb => Some(v),
                _ => None,
            })
            .unwrap();
        writeln!(
            out,
            "{name}\t{label}\tE{:03}\t{}\t{is_extreme}\t{:?}\t{:?}",
            index[x],
            render_event(x),
            verb.kind,
            witnesses
        )
        .unwrap();
        if is_extreme {
            let smaller: Vec<_> = basis.iter().filter(|y| *y != x).cloned().collect();
            let c = events(&close(m, smaller));
            let missing: State = f.difference(&c).cloned().collect();
            writeln!(
                loss,
                "{name}\t{label}\tE{:03}\t{}\t{:?}\t{:?}",
                index[x],
                render_event(x),
                ids(&missing, index),
                missing.iter().map(render_event).collect::<Vec<_>>()
            )
            .unwrap();
        }
    }
}

#[test]
fn complete_theory_state_bases_are_unique_and_generate_their_closure() {
    let g = module("general");
    let p = module("patrimonial");
    let index = index(&p);
    let input = theory_inputs(&g);
    let mut report = state_header();
    let mut counts = String::from(
        "model\tdistinct_closed_states\tcandidate_subsets\tgenerators\tunique_bases\n",
    );
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let mut distinct = BTreeMap::new();
        for mask in 0..16 {
            distinct
                .entry(events(&close(m, subset(&input, mask))))
                .or_insert(mask);
        }
        let mut candidates = 0;
        let mut generators = 0;
        for (f, mask) in &distinct {
            // Explicit ceiling before enumeration, independent of observed size.
            assert!(
                f.len() <= 8,
                "theory window grew: refuse exponential enumeration"
            );
            let basis = extreme_points(m, f);
            verify_basis(m, f, &basis);
            let atoms: Vec<_> = f.iter().cloned().collect();
            let mut minima: Vec<State> = Vec::new();
            let mut state_generators = 0;
            for bits in 0..1usize << atoms.len() {
                candidates += 1;
                let s: State = subset(&atoms, bits).into_iter().collect();
                let generates = events(&close(m, s.iter().cloned().collect())) == *f;
                assert_eq!(generates, basis.is_subset(&s));
                if generates {
                    state_generators += 1;
                    if !minima.iter().any(|b| b.is_subset(&s)) {
                        minima.push(s);
                    }
                }
            }
            assert_eq!(minima, vec![basis.clone()]);
            assert_eq!(state_generators, 1usize << (f.len() - basis.len()));
            generators += state_generators;
            write_state(
                &mut report,
                &format!("theory-{mask}"),
                label,
                f,
                &basis,
                &index,
            );
        }
        writeln!(
            counts,
            "{label}\t{}\t{candidates}\t{generators}\t{}",
            distinct.len(),
            distinct.len()
        )
        .unwrap();
        println!(
            "{label}: {} closed states, {candidates} subsets, one intrinsic basis per state",
            distinct.len()
        );
    }
    save("extreme-theory-states.tsv", &report);
    save("extreme-theory-counts.tsv", &counts);
}

#[test]
fn seeded_performance_states_preserve_their_indispensable_evidence() {
    let g = module("general");
    let p = module("patrimonial");
    let index = index(&p);
    let input = &experiment(&g, "performance").input;
    let mut report = state_header();
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        for mask in 0..128 {
            let s: State = subset(input, mask).into_iter().collect();
            let f = events(&close(m, s.iter().cloned().collect()));
            let basis = extreme_points(m, &f);
            assert_eq!(basis, s);
            assert_eq!(events(&close(m, basis.iter().cloned().collect())), f);
            write_state(
                &mut report,
                &format!("performance-{mask}"),
                label,
                &f,
                &basis,
                &index,
            );
        }
    }
    // Extreme for the complete state does not mean necessary for a particular goal.
    for m in [&g, &p] {
        let s = subset(input, 35); // obligationFiled + right + Act
        let f = events(&close(m, s.clone()));
        assert_eq!(extreme_points(m, &f), s.iter().cloned().collect());
        for norm in [0, 1] {
            let reduced = s.iter().filter(|e| *e != &input[norm]).cloned().collect();
            assert!(
                close(m, reduced)
                    .known
                    .contains(&experiment(m, "theoryCompliance").input[0])
            );
        }
        // The extreme-point map is not monotone when the state grows.
        let little = events(&close(m, experiment(m, "theoryCompliance").input.clone()));
        assert!(little.is_subset(&f));
        assert!(!extreme_points(m, &little).is_subset(&extreme_points(m, &f)));
    }
    save("extreme-seeded-states.tsv", &report);
    println!(
        "256 seeded closures: basis equals input; goal-support and monotonicity counterexamples verified"
    );
}

#[test]
fn patrimonial_extension_shrinks_extremes_of_common_closed_states() {
    let g = module("general");
    let p = module("patrimonial");
    let index = index(&p);
    let input = theory_inputs(&g);
    let mut report = state_header();
    let mut changes = String::from(
        "state\tcomparison\tgained_closed_ids\tlost_extreme_ids\tretained_extreme_ids\tlost_extreme_events\n",
    );
    let mut evidence = String::from(
        "state\tmodel\tid\tevent\textreme\tcategory\tremoval_rederivation_provenance\n",
    );
    let mut loss = String::from(
        "state\tmodel\tremoved_basis_id\tremoved_basis_event\tlost_closure_ids\tlost_closure_events\n",
    );
    for mask in 0..16 {
        let s = subset(&input, mask);
        let fg = events(&close(&g, s.clone()));
        let fp = events(&close(&p, s));
        let bg = extreme_points(&g, &fg);
        let bp = extreme_points(&p, &fp);
        assert!(fg.is_subset(&fp));
        assert!(bp.is_subset(&bg));
        let gained = fp.difference(&fg).cloned().collect();
        let removed = bg.difference(&bp).cloned().collect();
        writeln!(
            changes,
            "theory-{mask}\tsame-input\t{:?}\t{:?}\t{:?}\t{:?}",
            ids(&gained, &index),
            ids(&removed, &index),
            ids(&bp, &index),
            removed.iter().map(render_event).collect::<Vec<_>>()
        )
        .unwrap();
    }
    let all: State = universe(&p).into_iter().collect();
    let cases = [
        (
            "known-compliance",
            events(&close(&p, vec![input[2].clone()])),
        ),
        (
            "fulfilled-duty",
            events(&close(&p, vec![input[0].clone(), input[1].clone()])),
        ),
        ("whole-universe", all),
    ];
    for (name, f) in cases {
        assert_eq!(events(&close(&g, f.iter().cloned().collect())), f);
        assert_eq!(events(&close(&p, f.iter().cloned().collect())), f);
        let bg = extreme_points(&g, &f);
        let bp = extreme_points(&p, &f);
        assert!(bp.is_subset(&bg));
        let removed: State = bg.difference(&bp).cloned().collect();
        assert!(removed.iter().all(|e| e.verb == "patrimonialAsset"));
        if name == "whole-universe" {
            assert_eq!(bg.len(), 27);
            assert_eq!(bp.len(), 23);
            assert_eq!(removed.len(), 4);
        }
        if name == "known-compliance" {
            assert_eq!(bg.len(), 2);
            assert_eq!(bp.len(), 1);
        }
        if name == "fulfilled-duty" {
            assert_eq!(bg.len(), 3);
            assert_eq!(bp.len(), 2);
        }
        writeln!(
            changes,
            "{name}\tsame-closed-state\t[]\t{:?}\t{:?}\t{:?}",
            ids(&removed, &index),
            ids(&bp, &index),
            removed.iter().map(render_event).collect::<Vec<_>>()
        )
        .unwrap();
        for (label, m, basis) in [("general", &g, &bg), ("patrimonial", &p, &bp)] {
            verify_basis(m, &f, basis);
            write_state(&mut report, name, label, &f, basis, &index);
            certificates(m, name, label, &f, basis, &index, &mut evidence, &mut loss);
        }
    }
    save("extreme-common-states.tsv", &report);
    save("extreme-extension-comparison.tsv", &changes);
    save("extreme-certificates.tsv", &evidence);
    save("extreme-basis-removal.tsv", &loss);
    println!(
        "common E: general has 27 extreme points, patrimonial has 23; 4 asset snapshots become derivable"
    );
}
