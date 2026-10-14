//! Compare existing queries with independent geometric support frontiers.
use super::*;
use residuum::{ResidualProof, SearchLimits, query_program_residual, query_residual};

fn hypotheses(a: &Antecedent) -> BTreeSet<Event> {
    let mut values = Vec::new();
    atoms(a, &mut values);
    values.into_iter().cloned().collect()
}

fn proof_origins(m: &Module, p: &ResidualProof, out: &mut Vec<String>) {
    match p {
        ResidualProof::Rule {
            rule, antecedent, ..
        } => {
            out.push(format!(
                "{} | {:?}",
                rule.name,
                m.origin(&rule.name).unwrap()
            ));
            proof_origins(m, antecedent, out);
        }
        ResidualProof::Tensor(a, b) => {
            proof_origins(m, a, out);
            proof_origins(m, b, out);
        }
        _ => (),
    }
}

fn source_indices(m: &Module, e: &[Event]) -> BTreeSet<usize> {
    e.iter()
        .enumerate()
        .filter(|(_, x)| {
            m.declarations.iter().any(|d| {
                matches!(d, Declaration::Verb(v)
            if v.name == x.verb && v.kind == VerbKind::Seeded)
            })
        })
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn actual_program_query_reports_its_boundary_against_ground_frontiers() {
    let g = module("general");
    let p = module("patrimonial");
    let e = universe(&p);
    let sources = source_indices(&g, &e);
    assert_eq!(sources.len(), 23);
    let mut report = String::from(
        "model\tgoal_id\tgoal\tgeometric_supports\tstatus\treturned_branches\treturned_minimal_supports\tin_domain_supports\tmissing_supports\toutside_domain_supports\tdetail\n",
    );
    let mut witnesses = String::from(
        "model\tgoal_id\tbranch\thypotheses\tquery_proof\tquery_rule_origins\tforward_records\n",
    );
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let (_, frontier) = finite_frontiers(m, &e, &sources);
        for (q, goal) in e.iter().enumerate() {
            let original: Vec<Event> = Vec::new();
            let known = original.clone();
            let result = query_program_residual(m, &known, goal, SearchLimits::default());
            assert_eq!(original, known);
            match result {
                Err(error) => {
                    let detail = error.to_string();
                    assert!(
                        detail.contains("not determined by the goal"),
                        "unexpected failure: {detail}"
                    );
                    writeln!(report, "{label}\tE{q:03}\t{}\t{}\tunsupported-unbound-parameter\t0\t0\t0\t{}\t0\t{detail}", render_event(goal), frontier[q].len(), frontier[q].len()).unwrap();
                }
                Ok(query) => {
                    let mut canonical = Supports::new();
                    let mut outside = BTreeSet::new();
                    for (branch, explanation) in query.explanations.iter().enumerate() {
                        let h = hypotheses(&explanation.required);
                        assert!(h.iter().all(|x| x.args.iter().all(Term::is_ground)
                            && x.proposition.time.to_term().is_ground()));
                        let forward = close(m, h.iter().cloned().collect());
                        assert!(forward.known.contains(goal));
                        audit_proofs(m, &forward);
                        let mut origins = Vec::new();
                        proof_origins(m, &explanation.proof, &mut origins);
                        writeln!(
                            witnesses,
                            "{label}\tE{q:03}\t{branch}\t{:?}\t{:?}\t{origins:?}\t{:?}",
                            h.iter().map(render_event).collect::<Vec<_>>(),
                            explanation.proof,
                            proofs(&forward)
                        )
                        .unwrap();
                        let indexed: Option<BTreeSet<_>> =
                            h.iter().map(|x| e.iter().position(|y| y == x)).collect();
                        match indexed {
                            Some(s) if s.is_subset(&sources) => insert_minimal(&mut canonical, s),
                            _ => {
                                outside.insert(h);
                            }
                        }
                    }
                    assert_eq!(
                        canonical,
                        frontier[q],
                        "completed in-domain query misses a minimal support for {}",
                        render_event(goal)
                    );
                    let status = if outside.is_empty() {
                        "equal-minimal-frontier"
                    } else {
                        "outside-finite-source-domain"
                    };
                    let detail = format!(
                        "failure={:?}; metrics={:?}; outside={:?}",
                        query.failure, query.metrics, outside
                    );
                    writeln!(
                        report,
                        "{label}\tE{q:03}\t{}\t{}\t{status}\t{}\t{}\t{}\t0\t{}\t{detail}",
                        render_event(goal),
                        frontier[q].len(),
                        query.explanations.len(),
                        canonical.len() + outside.len(),
                        canonical.len(),
                        outside.len()
                    )
                    .unwrap();
                }
            }
        }
    }
    save("residual-frontier-comparison.tsv", &report);
    save("residual-frontier-witnesses.tsv", &witnesses);
    println!(
        "146 unchanged program queries classified; returned hypotheses forward-certified without modifying Known"
    );
}

#[test]
fn local_residual_and_partial_evidence_make_the_missing_parameter_boundary_explicit() {
    let g = module("general");
    let p = module("patrimonial");
    let e = universe(&p);
    let mut report = String::from("model\tcase\texpected_frontier\tactual\tquery_proof\n");
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let performance = &experiment(m, "performance").input;
        let compliance = &experiment(m, "theoryCompliance").input[0];
        let (_, source_frontiers) = finite_frontiers(m, &e, &source_indices(m, &e));
        let qc = e.iter().position(|x| x == compliance).unwrap();
        for evidence in [
            Vec::new(),
            vec![performance[5].clone()],
            vec![performance[0].clone(), performance[5].clone()],
        ] {
            let before = evidence.clone();
            let existing: BTreeSet<_> = evidence
                .iter()
                .map(|x| e.iter().position(|y| y == x).unwrap())
                .collect();
            let mut completions = Supports::new();
            for s in &source_frontiers[qc] {
                insert_minimal(&mut completions, s.difference(&existing).copied().collect());
            }
            let expected: Vec<Vec<String>> = completions
                .iter()
                .map(|s| s.iter().map(|i| format!("E{i:03}")).collect())
                .collect();
            let error = query_program_residual(m, &evidence, compliance, SearchLimits::default())
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("rule `obligationCompliance` has parameter `conduct` not determined")
            );
            assert_eq!(before, evidence);
            writeln!(
                report,
                "{label}\tcompliance-raw-evidence-{}\t{expected:?}\t{}\tno complete query result",
                evidence.len(),
                error
            )
            .unwrap();
        }
        let closed = close(m, vec![performance[0].clone(), performance[5].clone()]);
        let before = proofs(&closed);
        let query =
            query_program_residual(m, &closed.known, compliance, SearchLimits::default()).unwrap();
        assert_eq!(query.explanations.len(), 1);
        assert_eq!(query.explanations[0].required, Antecedent::Unit);
        assert_eq!(proofs(&closed), before);
        writeln!(
            report,
            "{label}\tcompliance-already-known\tUnit\tUnit\t{:?}",
            query.explanations[0].proof
        )
        .unwrap();
        let rule = |name: &str| {
            m.declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Rule(r) if r.name == name => Some(r),
                    _ => None,
                })
                .unwrap()
        };
        let symbolic = query_residual(rule("rightCompliance"), compliance, &[]).unwrap();
        assert_eq!(symbolic.len(), 1);
        assert!(!symbolic[0].parameters.is_empty());
        writeln!(report, "{label}\tlocal-symbolic-rightCompliance\tscoped pending parameters, not ground supports\t{:?}\t{:?}", symbolic[0].parameters, symbolic[0]).unwrap();
        let local = query_residual(
            rule("rightCompliance"),
            compliance,
            &[performance[5].clone()],
        )
        .unwrap();
        assert_eq!(local.len(), 1);
        assert!(local[0].parameters.is_empty());
        assert_eq!(
            hypotheses(&local[0].required),
            BTreeSet::from([performance[1].clone()])
        );
        writeln!(
            report,
            "{label}\tlocal-rightCompliance-with-Act\tright\tright\t{:?}",
            local[0]
        )
        .unwrap();
        let inconsistent = e.iter().find(|x| x.verb == "inconsistent").unwrap();
        let (_, full_frontier) = finite_frontiers(m, &e, &source_indices(m, &e));
        let q = e.iter().position(|x| x == inconsistent).unwrap();
        for mask in 0..4 {
            let evidence = subset(&[performance[5].clone(), performance[6].clone()], mask);
            let before = evidence.clone();
            let existing: BTreeSet<_> = evidence.iter().cloned().collect();
            let mut expected = BTreeSet::new();
            for s in &full_frontier[q] {
                expected.insert(
                    s.iter()
                        .map(|i| e[*i].clone())
                        .filter(|x| !existing.contains(x))
                        .collect::<BTreeSet<_>>(),
                );
            }
            let query = query_program_residual(m, &evidence, inconsistent, SearchLimits::default())
                .unwrap();
            let actual: BTreeSet<_> = query
                .explanations
                .iter()
                .map(|x| hypotheses(&x.required))
                .collect();
            assert_eq!(actual, expected);
            assert_eq!(before, evidence);
            for h in &actual {
                let mut input = evidence.clone();
                input.extend(h.iter().cloned());
                assert!(close(m, input).known.contains(inconsistent));
            }
            writeln!(
                report,
                "{label}\tinconsistent-evidence-{mask}\t{expected:?}\t{actual:?}\t{:?}",
                query.explanations
            )
            .unwrap();
        }
    }
    save("residual-frontier-boundaries.tsv", &report);
    println!(
        "Local queries preserve free scope; the global query rejects hidden parameters before evidence matching; head-ground inconsistency queries match all four completion frontiers"
    );
}
