//! Dependency audit before any inference of unknown normative justifications.
use super::*;

#[path = "legal_geometry_identifiability_tests.rs"]
mod identifiability;

fn ground_clauses(module: &Module, domain: &[Event]) -> Vec<(usize, usize)> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(instantiate_from_known(r, domain)),
            _ => None,
        })
        .flatten()
        .map(|d| {
            (
                mask_of(&d.record.antecedents, domain),
                mask_of(&[d.event], domain),
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

// The graph is only a projection of conjunctive hyperedges. It does NOT erase
// guards from the evaluator, nor decide anti-exchange from SCCs alone.
fn components(clauses: &[(usize, usize)], n: usize) -> Vec<Vec<usize>> {
    let mut reachable = vec![vec![false; n]; n];
    for &(body, head) in clauses {
        for (i, row) in reachable.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                if body & (1 << i) != 0 && head & (1 << j) != 0 {
                    *value = true;
                }
            }
        }
    }
    for (i, row) in reachable.iter_mut().enumerate() {
        row[i] = true;
    }
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                reachable[i][j] |= reachable[i][k] && reachable[k][j];
            }
        }
    }
    let mut pending: BTreeSet<_> = (0..n).collect();
    let mut out = Vec::new();
    while let Some(&i) = pending.first() {
        let component: Vec<_> = pending
            .iter()
            .copied()
            .filter(|&j| reachable[i][j] && reachable[j][i])
            .collect();
        for j in &component {
            pending.remove(j);
        }
        out.push(component);
    }
    out
}

fn reverse_ground_conjunctions(module: &Module) -> Module {
    let mut result = module.clone();
    for d in &mut result.declarations {
        if let Declaration::Rule(r) = d {
            if r.name.starts_with("hypotheticalCompliance") {
                let RuleBody::HornClause(clause) = &mut r.body else {
                    unreachable!()
                };
                let mut leaves = Vec::new();
                atoms(&clause.antecedent, &mut leaves);
                let owned: Vec<_> = leaves.into_iter().cloned().collect();
                let mut ordered = owned.into_iter().map(Antecedent::Event);
                let first = ordered.next().unwrap();
                // [A,P,D] becomes D & (P & A): reverse order AND association.
                clause.antecedent = ordered.fold(first, |tail, next| {
                    Antecedent::And(Box::new(next), Box::new(tail))
                });
            }
        }
    }
    result
}

fn export_proof(report: &mut String, package: &[usize], side: &str, module: &Module, c: &Closure) {
    audit_proofs(module, c);
    for d in &c.derivations {
        writeln!(
            report,
            "{}\t{side}\t{:?}\t{:?}\t{:?}",
            package_name(package),
            d.event,
            d.record,
            module.origin(&d.record.rule).unwrap()
        )
        .unwrap();
    }
}

#[test]
fn minimal_revision_feedback_is_conditional_and_invariant_under_equal_closures() {
    let study = Study::new();
    let original = study.module.clone();
    let n = study.universe.len();
    let c = study
        .universe
        .iter()
        .position(|e| e == &study.goal)
        .unwrap();
    let d = study
        .universe
        .iter()
        .position(|e| e == &study.catalog[5])
        .unwrap();
    let violation = experiment(&study.module, "theoryViolation").input[0].clone();
    let mut report = String::from("package\tnew_rule\tconjunctive_body\thead\torigin\n");
    let mut geometry = String::from(
        "package\tcyclic_components\tanti_exchange\treordered_full_closure_equal\treordered_provenance_equal\n",
    );
    let mut proofs_report = String::from("package\tside\tevent\trecord\torigin\n");
    for p in minimal_packages(&study) {
        let m = study.hypothetical(&p);
        let rewritten = reverse_ground_conjunctions(&m);
        let clauses = ground_clauses(&m, &study.universe);
        assert_eq!(
            clauses,
            study
                .model_clauses(&p)
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        );
        let table = closure_table(&clauses, n);
        let rewritten_table = closure_table(&ground_clauses(&rewritten, &study.universe), n);
        assert_eq!(table, rewritten_table);
        assert_eq!(
            anti_exchange_witness(&table, n),
            anti_exchange_witness(&rewritten_table, n)
        );
        let cycles: Vec<_> = components(&clauses, n)
            .into_iter()
            .filter(|s| s.len() > 1)
            .collect();
        let feedback_supports: Vec<_> = p.iter().copied().filter(|h| h & 32 != 0).collect();
        assert_eq!(cycles.is_empty(), feedback_supports.is_empty());
        if !cycles.is_empty() {
            assert_eq!(cycles.len(), 1);
            assert_eq!(
                cycles[0].iter().copied().collect::<BTreeSet<_>>(),
                [c, d].into_iter().collect()
            );
        }
        for r in m.declarations.iter().filter_map(|dec| match dec {
            Declaration::Rule(r) if r.name.starts_with("hypotheticalCompliance") => Some(r),
            _ => None,
        }) {
            let RuleBody::HornClause(clause) = &r.body else {
                unreachable!()
            };
            let mut body = Vec::new();
            atoms(&clause.antecedent, &mut body);
            writeln!(
                report,
                "{}\t{}\t{:?}\t{:?}\t{:?}",
                package_name(&p),
                r.name,
                body,
                clause.consequent,
                m.origin(&r.name).unwrap()
            )
            .unwrap();
        }
        // No witness at all on E' for AP. For every failing package reproduce
        // the PARTICULAR guarded cycle, rather than infer failure from a graph.
        for h in feedback_supports {
            let mut input = subset(&study.catalog, h & !32);
            input.push(violation.clone());
            let f = close(&study.module, input);
            assert!(!f.known.contains(&study.goal));
            assert!(!f.known.contains(&study.catalog[5]));
            assert_eq!(events(&close(&m, f.known.clone())), events(&f));
            let mut fc = f.known.clone();
            fc.push(study.goal.clone());
            let mut fd = f.known.clone();
            fd.push(study.catalog[5].clone());
            let cx = close(&m, fc);
            let dy = close(&m, fd);
            assert!(cx.known.contains(&study.catalog[5]));
            assert!(dy.known.contains(&study.goal));
            assert_eq!(events(&cx), events(&dy));
            assert!(
                cx.derivations
                    .iter()
                    .any(|r| r.event == study.catalog[5] && r.record.rule == "performanceDispute")
            );
            assert!(dy.derivations.iter().any(|r| r.event == study.goal
                && r.record.rule == format!("hypotheticalCompliance{}", label(h))));
            export_proof(&mut proofs_report, &p, "F_plus_C", &m, &cx);
            export_proof(&mut proofs_report, &p, "F_plus_D", &m, &dy);
        }
        let all = study.catalog.clone();
        let before = close(&m, all.clone());
        let after = close(&rewritten, all);
        assert_eq!(full_events(&before), full_events(&after));
        assert_ne!(
            proofs(&before),
            proofs(&after),
            "reordered witnesses retain their order in provenance"
        );
        audit_proofs(&rewritten, &after);
        writeln!(
            geometry,
            "{}\t{:?}\t{}\ttrue\tfalse",
            package_name(&p),
            cycles,
            anti_exchange_witness(&table, n).is_none()
        )
        .unwrap();
    }
    assert_eq!(study.module, original);
    save("revision-feedback-hyperedges.tsv", &report);
    save("revision-feedback-presentations.tsv", &geometry);
    save("revision-feedback-provenance.tsv", &proofs_report);
    println!(
        "5 minimal packages: AP acyclic, four guarded C/D cycles witnessed; 4096 closures each invariant under equivalent presentation, ordered provenance differs"
    );
}

#[test]
fn hypothetical_guard_completion_preserves_observations_but_changes_global_geometry() {
    let study = Study::new();
    let controls = module("feedback_completions");
    let original = study.module.clone();
    let n = study.universe.len();
    let violation = experiment(&study.module, "theoryViolation").input[0].clone();
    let mut report = String::from(
        "package\tcontrol\tcatalog_closures_equal\tcatalog_provenance_equal\tglobal_changed_states\told_anti_exchange\tnew_anti_exchange\tcyclic_component_preserved\tdiscriminating_premises\n",
    );
    let mut proof_report = String::from("package\tside\tevent\trecord\torigin\n");
    let mut checked = 0;
    for p in minimal_packages(&study) {
        let Some(&h) = p.iter().find(|h| *h & 32 != 0) else {
            continue;
        };
        let name = if h & 8 != 0 {
            "hypotheticalGuardCompletionAPV"
        } else {
            "hypotheticalGuardCompletionAV"
        };
        let mut extended = study.hypothetical(&p);
        let uncompleted = extended.clone();
        let declaration = controls
            .declarations
            .iter()
            .find(|d| d.name() == name)
            .unwrap()
            .clone();
        extended.declarations.push(declaration);
        extended
            .origins
            .insert(name.into(), controls.origin(name).unwrap().clone());
        let old_clauses = ground_clauses(&uncompleted, &study.universe);
        let new_clauses = ground_clauses(&extended, &study.universe);
        let old_table = closure_table(&old_clauses, n);
        let new_table = closure_table(&new_clauses, n);
        assert!(anti_exchange_witness(&old_table, n).is_some());
        assert!(anti_exchange_witness(&new_table, n).is_none());
        assert_eq!(
            components(&old_clauses, n),
            components(&new_clauses, n),
            "same projected SCC despite different geometry"
        );
        for s in 0..64 {
            assert_eq!(old_table[study.projected(s)], new_table[study.projected(s)]);
            let input = subset(&study.catalog, s);
            let old = close(&uncompleted, input.clone());
            let new = close(&extended, input.clone());
            let reference =
                residuum::derive::close_program_reference(input, &extended, &mut Clock(1000))
                    .unwrap();
            assert_eq!(full_events(&old), full_events(&new));
            assert_eq!(proofs(&old), proofs(&new));
            assert_eq!(full_events(&new), full_events(&reference));
            assert_eq!(proofs(&new), proofs(&reference));
            assert_eq!(
                mask_of(&new.known, &study.universe),
                new_table[study.projected(s)]
            );
            audit_proofs(&extended, &new);
        }
        let mut distinguishing = subset(&study.catalog, h & !32);
        distinguishing.push(violation.clone());
        let old = close(&uncompleted, distinguishing.clone());
        let new = close(&extended, distinguishing.clone());
        assert!(!old.known.contains(&study.goal));
        assert!(new.known.contains(&study.goal));
        assert!(new.derivations.iter().any(|r| r.record.rule == name));
        export_proof(
            &mut proof_report,
            &p,
            "old_control_case",
            &uncompleted,
            &old,
        );
        export_proof(
            &mut proof_report,
            &p,
            "completed_control_case",
            &extended,
            &new,
        );
        let differences = old_table
            .iter()
            .zip(&new_table)
            .filter(|(a, b)| a != b)
            .count();
        assert!(differences > 0);
        for (s, &f) in new_table.iter().enumerate() {
            assert_eq!(s & f, s);
            assert_eq!(new_table[f], f);
            for i in 0..n {
                assert_eq!(f & new_table[s | (1 << i)], f);
            }
        }
        assert_eq!(new_table[0], 0);
        writeln!(
            report,
            "{}\t{name}\ttrue\ttrue\t{differences}\tfalse\ttrue\ttrue\t{:?}",
            package_name(&p),
            distinguishing
        )
        .unwrap();
        checked += 1;
    }
    assert_eq!(checked, 4);
    assert_eq!(study.module, original);
    save("revision-feedback-completion.tsv", &report);
    save("revision-feedback-completion-provenance.tsv", &proof_report);
    println!(
        "4 controls: all 64 catalog closures/provenance unchanged; anti-exchange restored over 4096 expanded states despite same SCC; operator changes outside catalog, not a normative justification"
    );
}
