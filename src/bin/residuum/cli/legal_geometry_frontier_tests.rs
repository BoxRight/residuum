//! Goal frontiers, distinct from generators of a complete closed state.
use super::*;

fn ids(s: &BTreeSet<usize>) -> Vec<String> {
    s.iter().map(|i| format!("E{i:03}")).collect()
}

fn seeded_indices(m: &Module, e: &[Event]) -> BTreeSet<usize> {
    let seeded: BTreeSet<_> = m
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) if v.kind == VerbKind::Seeded => Some(v.name.as_str()),
            _ => None,
        })
        .collect();
    let result: BTreeSet<_> = e
        .iter()
        .enumerate()
        .filter(|(_, x)| seeded.contains(x.verb.as_str()))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(result.len(), 23);
    result
}

#[test]
fn minimal_frontiers_preserve_supports_witnesses_and_origins() {
    let g = module("general");
    let p = module("patrimonial");
    let reference = module("reference");
    let e = universe(&p);
    let seeded = seeded_indices(&g, &e);
    let mut certificates = String::from(
        "model\tdomain\tconclusion_id\tconclusion\tsupport_ids\tsupport\tclosure_ids\tproof_count\tproofs_with_origins\n",
    );
    let mut omissions = String::from(
        "model\tdomain\tconclusion_id\tsupport_ids\tomitted_id\tremaining_closure_ids\n",
    );
    let mut totals =
        String::from("model\tdomain\tconclusions_reachable\tminimal_supports\tdeletions_checked\n");
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let (_, full) = finite_frontiers(m, &e, &(0..e.len()).collect());
        let (_, external) = finite_frontiers(m, &e, &seeded);
        for (q, frontier) in full.iter().enumerate() {
            assert!(frontier.contains(&BTreeSet::from([q])));
            let restricted: Supports = frontier
                .iter()
                .filter(|s| s.is_subset(&seeded))
                .cloned()
                .collect();
            assert_eq!(restricted, external[q]);
        }
        for (domain, frontiers) in [("all-events", &full), ("seeded", &external)] {
            let mut count = 0;
            let mut deletions = 0;
            for (q, frontier) in frontiers.iter().enumerate() {
                for s in frontier {
                    assert!(frontier.iter().all(|t| t == s || !t.is_subset(s)));
                    let input: Vec<_> = s.iter().map(|i| e[*i].clone()).collect();
                    let c = close(m, input.clone());
                    assert!(c.known.contains(&e[q]));
                    audit_proofs(m, &c);
                    if label == "patrimonial" {
                        let cr = close(&reference, input.clone());
                        assert_eq!(full_events(&c), full_events(&cr));
                        assert_eq!(proofs(&c), proofs(&cr));
                    }
                    let closure_ids: BTreeSet<_> = e
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| c.known.contains(x))
                        .map(|(i, _)| i)
                        .collect();
                    assert_eq!(closure_ids.len(), c.known.len());
                    // Preserve all justifications, not merely one goal witness.
                    let proof_records: Vec<_> = c
                        .derivations
                        .iter()
                        .map(|d| {
                            let origin = m.origin(&d.record.rule).unwrap();
                            assert_eq!(
                                origin.module,
                                if d.record.rule == "complianceAsset" {
                                    "LegalPatrimonial"
                                } else {
                                    "LegalGeneral"
                                }
                            );
                            format!("{:?} | {:?} | {:?}", d.event, d.record, origin)
                        })
                        .collect();
                    if !s.contains(&q) {
                        assert!(c.derivations.iter().any(|d| d.event == e[q]));
                    }
                    writeln!(
                        certificates,
                        "{label}\t{domain}\tE{q:03}\t{}\t{:?}\t{:?}\t{:?}\t{}\t{:?}",
                        render_event(&e[q]),
                        ids(s),
                        input.iter().map(render_event).collect::<Vec<_>>(),
                        ids(&closure_ids),
                        c.derivations.len(),
                        proof_records
                    )
                    .unwrap();
                    for omitted in s {
                        let smaller = s
                            .iter()
                            .filter(|i| *i != omitted)
                            .map(|i| e[*i].clone())
                            .collect();
                        let reduced = close(m, smaller);
                        assert!(!reduced.known.contains(&e[q]));
                        let remaining: BTreeSet<_> = e
                            .iter()
                            .enumerate()
                            .filter(|(_, x)| reduced.known.contains(x))
                            .map(|(i, _)| i)
                            .collect();
                        writeln!(
                            omissions,
                            "{label}\t{domain}\tE{q:03}\t{:?}\tE{omitted:03}\t{:?}",
                            ids(s),
                            ids(&remaining)
                        )
                        .unwrap();
                        deletions += 1;
                    }
                    count += 1;
                }
            }
            writeln!(
                totals,
                "{label}\t{domain}\t{}\t{count}\t{deletions}",
                frontiers.iter().filter(|f| !f.is_empty()).count()
            )
            .unwrap();
            println!(
                "{label}/{domain}: {count} supports and {deletions} single-deletion certificates"
            );
        }
    }
    save("goal-frontier-certificates.tsv", &certificates);
    save("goal-frontier-omissions.tsv", &omissions);
    save("goal-frontier-totals.tsv", &totals);
}

#[test]
fn frontier_upsets_reconstruct_all_controlled_derivability_regions() {
    let g = module("general");
    let p = module("patrimonial");
    let e = universe(&p);
    let mut report = String::from(
        "model\twindow\tconclusion_id\tstates\tderivable_states\twindow_frontier_ids\n",
    );
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let (_, frontiers) = finite_frontiers(m, &e, &seeded_indices(m, &e));
        let compliance = &experiment(m, "theoryCompliance").input[0];
        let goal = |verb: &str| {
            e.iter()
                .position(|x| {
                    x.verb == verb
                        && x.args == compliance.args
                        && x.proposition.time == compliance.proposition.time
                })
                .unwrap()
        };
        let qc = goal("compliance");
        let qv = goal("violation");
        let qd = goal("disputedPerformance");
        let product_frontier = |a: &Supports, b: &Supports| {
            let mut result = Supports::new();
            for s in a {
                for t in b {
                    insert_minimal(&mut result, s.union(t).copied().collect());
                }
            }
            result
        };
        assert_eq!(
            frontiers[qd],
            product_frontier(&frontiers[qc], &frontiers[qv])
        );
        let (_, full) = finite_frontiers(m, &e, &(0..e.len()).collect());
        let mut full_dispute = product_frontier(&full[qc], &full[qv]);
        insert_minimal(&mut full_dispute, BTreeSet::from([qd]));
        assert_eq!(full[qd], full_dispute);
        // A dispute can follow from a right plus a prohibition and a positive Act,
        // without any negative historical evidence or inferred inconsistent fact.
        let performance = &experiment(m, "performance").input;
        let c = close(
            m,
            vec![
                performance[1].clone(),
                performance[2].clone(),
                performance[5].clone(),
            ],
        );
        assert!(c.known.contains(&e[qd]));
        assert!(!c.known.iter().any(|x| x.verb == "inconsistent"));
        for name in [
            "performance",
            "authority",
            "legislation",
            "transferCase",
            "holding",
        ] {
            let input = &experiment(m, name).input;
            assert!(input.len() <= 8, "refuse unbounded powerset");
            let window: BTreeSet<_> = input
                .iter()
                .map(|x| e.iter().position(|y| y == x).unwrap())
                .collect();
            let mut regions = vec![0; e.len()];
            for mask in 0..1usize << input.len() {
                let s = subset(input, mask);
                let chosen: BTreeSet<_> = s
                    .iter()
                    .map(|x| e.iter().position(|y| y == x).unwrap())
                    .collect();
                let c = close(m, s);
                for (q, frontier) in frontiers.iter().enumerate() {
                    let predicted = frontier.iter().any(|b| b.is_subset(&chosen));
                    assert_eq!(predicted, c.known.contains(&e[q]));
                    regions[q] += usize::from(predicted);
                }
            }
            for (q, frontier) in frontiers.iter().enumerate() {
                let local: Vec<_> = frontier
                    .iter()
                    .filter(|s| s.is_subset(&window))
                    .map(ids)
                    .collect();
                writeln!(
                    report,
                    "{label}\t{name}\tE{q:03}\t{}\t{}\t{local:?}",
                    1usize << input.len(),
                    regions[q]
                )
                .unwrap();
            }
        }
    }
    save("goal-frontier-regions.tsv", &report);
    println!(
        "Frontier upsets agree with real closure for all 73 goals in 792 controlled evaluations"
    );
}
