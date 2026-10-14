//! Independent finite closure oracles for the integrated active library API.
use super::*;
use residuum::{SearchLimits, query_active_program_residual};

fn hypotheses(a: &Antecedent) -> BTreeSet<Event> {
    let mut out = Vec::new();
    atoms(a, &mut out);
    out.into_iter().cloned().collect()
}

// This oracle never calls a residual routine, reads its proof tree or projects
// its guards. It enumerates at most 2^3 candidate evidence subsets per query.
fn exhaustive_completions(m: &Module, domain: &[Event], known: &[Event], goal: &Event) -> Supports {
    assert!(domain.len() <= 3);
    let mut frontier = Supports::new();
    for mask in 0..1 << domain.len() {
        let h = subset(domain, mask);
        if h.iter().any(|e| known.contains(e)) {
            continue;
        }
        let mut input = known.to_vec();
        input.extend(h);
        let c = residuum::derive::close_program_reference(input, m, &mut Clock(1000)).unwrap();
        if c.known.contains(goal) {
            insert_minimal(
                &mut frontier,
                (0..domain.len()).filter(|i| mask & (1 << i) != 0).collect(),
            );
        }
    }
    frontier
}

#[test]
fn active_residual_matches_exhaustive_legal_frontiers_and_activation_requirements() {
    let guarded = module("active_evidence_check");
    let ex = experiment(&guarded, "exactEvidence");
    let g = module("general");
    let p = module("patrimonial");
    let source = |m: &Module| {
        m.declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Rule(r) if r.name == "conflictingActEvidence" => Some(r.clone()),
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(source(&guarded), source(&g));
    assert_eq!(source(&p), source(&g));
    let mut report = String::from(
        "model\tknown_mask\toracle_minimal_supports\tquery_minimal_supports\tbranches\tactivations\tforward_records\n",
    );
    for (name, m, domain) in [
        ("general", &g, ex.input[..2].to_vec()),
        ("patrimonial", &p, ex.input[..2].to_vec()),
        ("activated", &guarded, ex.input.clone()),
    ] {
        let original = m.clone();
        for mask in 0..1 << domain.len() {
            let known = subset(&domain, mask);
            let before = known.clone();
            let oracle = exhaustive_completions(m, &domain, &known, &ex.goal);
            let q = query_active_program_residual(
                m,
                &known,
                &ex.goal,
                SearchLimits::default(),
                &mut Clock(0),
            )
            .unwrap();
            assert!(q.failure.is_none());
            let mut actual = Supports::new();
            for a in &q.explanations {
                let h = hypotheses(&a.candidate.required);
                let indices: BTreeSet<_> = h
                    .iter()
                    .map(|e| {
                        domain
                            .iter()
                            .position(|v| v == e)
                            .expect("outside declared domain")
                    })
                    .collect();
                insert_minimal(&mut actual, indices);
                let mut input = known.clone();
                input.extend(h);
                let independent =
                    residuum::derive::close_program_reference(input, m, &mut Clock(2000)).unwrap();
                assert!(independent.known.contains(&ex.goal));
                assert_eq!(full_events(&a.hypothetical), full_events(&independent));
                assert_eq!(proofs(&a.hypothetical), proofs(&independent));
                assert_eq!(a.activations.len(), if name == "activated" { 1 } else { 0 });
                if name == "activated" {
                    let activation = &a.activations[0];
                    assert_eq!(activation.trigger, ex.input[2]);
                    let origin = activation.program_origin.as_ref().unwrap();
                    assert_eq!(origin.program, "EvidenceCheck");
                    assert_eq!(origin.defining_module, "GeometryActiveEvidenceCheck");
                    assert_eq!(origin.rule_origin.module, "LegalGeneral");
                    let d = a
                        .hypothetical
                        .derivations
                        .iter()
                        .find(|d| d.event == ex.goal)
                        .unwrap();
                    assert_eq!(d.record.rule, "EvidenceCheck::0::conflictingActEvidence");
                    assert_eq!(
                        d.record.antecedents,
                        vec![
                            ex.input[2].clone(),
                            ex.input[0].clone(),
                            ex.input[1].clone()
                        ]
                    );
                }
            }
            assert_eq!(actual, oracle, "{name}/{mask}");
            assert_eq!(known, before);
            writeln!(
                report,
                "{name}\t{mask}\t{oracle:?}\t{actual:?}\t{}\t{:?}\t{:?}",
                q.explanations.len(),
                q.explanations
                    .iter()
                    .map(|a| &a.activations)
                    .collect::<Vec<_>>(),
                q.explanations
                    .iter()
                    .map(|a| proofs(&a.hypothetical))
                    .collect::<Vec<_>>()
            )
            .unwrap();
        }
        assert_eq!(*m, original);
    }
    // The guarded program is deliberately not the original ontology operator:
    // the same Acts need an invocation here, not a new legal premise globally.
    assert!(close(&g, ex.input[..2].to_vec()).known.contains(&ex.goal));
    assert!(
        !close(&guarded, ex.input[..2].to_vec())
            .known
            .contains(&ex.goal)
    );
    let historical = residuum::query_program_residual(
        &guarded,
        &ex.input[..2],
        &ex.goal,
        SearchLimits::default(),
    )
    .unwrap();
    assert_eq!(
        historical.required,
        Some(residuum::RequirementFormula::Unit)
    );
    save("active-legal-exhaustive-frontiers.tsv", &report);
    println!(
        "16 active queries: 8 unchanged legal and 8 explicitly guarded, exhaustive minimal completions agree"
    );
}

#[test]
fn active_residual_classifies_real_ontology_coverage_without_existential_expansion() {
    let g = module("general");
    let p = module("patrimonial");
    let e = universe(&p);
    let seeded: BTreeSet<_> = e
        .iter()
        .enumerate()
        .filter(|(_, event)| event.proposition.kind == PropositionKind::Seeded)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(e.len(), 73);
    assert_eq!(seeded.len(), 23);
    let mut report = String::from(
        "model\tgoal\tgeometric_supports\tstatus\tbranches\tminimal_in_domain\tmissing\toutside\tdetail\n",
    );
    for (name, m) in [("general", &g), ("patrimonial", &p)] {
        let original = m.clone();
        let (_, frontier) = finite_frontiers(m, &e, &seeded);
        for (i, goal) in e.iter().enumerate() {
            let q =
                query_active_program_residual(m, &[], goal, SearchLimits::default(), &mut Clock(0));
            match q {
                Err(error) => {
                    assert!(
                        error.to_string().contains("not determined by the goal"),
                        "{error}"
                    );
                    writeln!(
                        report,
                        "{name}\tE{i:03}\t{}\tunsupported-unbound-parameter\t0\t0\t{}\t0\t{error}",
                        frontier[i].len(),
                        frontier[i].len()
                    )
                    .unwrap();
                }
                Ok(q) => {
                    let mut actual = Supports::new();
                    let mut outside = BTreeSet::new();
                    for a in &q.explanations {
                        assert!(a.activations.is_empty());
                        assert!(a.hypothetical.known.contains(goal));
                        audit_proofs(m, &a.hypothetical);
                        let h = hypotheses(&a.candidate.required);
                        let indices: Option<BTreeSet<_>> = h
                            .iter()
                            .map(|event| e.iter().position(|v| v == event))
                            .collect();
                        match indices {
                            Some(s) if s.is_subset(&seeded) => insert_minimal(&mut actual, s),
                            _ => {
                                outside.insert(h);
                            }
                        }
                    }
                    assert_eq!(actual, frontier[i], "{name}/E{i:03}");
                    writeln!(
                        report,
                        "{name}\tE{i:03}\t{}\t{}\t{}\t{}\t0\t{}\t{:?}",
                        frontier[i].len(),
                        if outside.is_empty() {
                            "equal-minimal-frontier"
                        } else {
                            "outside-finite-source-domain"
                        },
                        q.explanations.len(),
                        actual.len(),
                        outside.len(),
                        q.metrics
                    )
                    .unwrap();
                }
            }
        }
        assert_eq!(*m, original);
    }
    save("active-legal-ontology-coverage.tsv", &report);
    println!(
        "146 active API queries classified against unchanged legal frontiers, without resolving existentials"
    );
}
