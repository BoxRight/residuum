//! Informative cases for a fixed version space; external answers stay pending.
use super::*;

#[path = "legal_geometry_observability_tests.rs"]
mod observability;

struct Hypothesis {
    name: String,
    module: Module,
    table: Vec<usize>,
}

fn existing_hypotheses(study: &Study) -> Vec<Hypothesis> {
    let controls = module("feedback_completions");
    let mut out = Vec::new();
    for p in minimal_packages(study) {
        let m = study.hypothetical(&p);
        let table = closure_table(&ground_clauses(&m, &study.universe), study.universe.len());
        out.push(Hypothesis {
            name: package_name(&p),
            module: m.clone(),
            table,
        });
        if let Some(&h) = p.iter().find(|h| *h & 32 != 0) {
            let name = if h & 8 != 0 {
                "hypotheticalGuardCompletionAPV"
            } else {
                "hypotheticalGuardCompletionAV"
            };
            let mut completed = m;
            completed.declarations.push(
                controls
                    .declarations
                    .iter()
                    .find(|d| d.name() == name)
                    .unwrap()
                    .clone(),
            );
            completed
                .origins
                .insert(name.into(), controls.origin(name).unwrap().clone());
            let table = closure_table(
                &ground_clauses(&completed, &study.universe),
                study.universe.len(),
            );
            out.push(Hypothesis {
                name: format!(
                    "{}+{}",
                    package_name(&p),
                    if h & 8 != 0 { "APV" } else { "AV" }
                ),
                module: completed,
                table,
            });
        }
    }
    assert_eq!(out.len(), 9);
    for h in &out {
        for (_, s, positive) in &study.labels {
            assert_eq!(study.has_goal(h.table[study.projected(*s)]), *positive);
        }
    }
    out
}

// Difference of two monotone closures is NOT generally an upward region.
// Propagate existence of ANY witness below a set, then exclude all witnesses
// with a proper witness subset (not merely one-step raw differing parents).
fn inclusion_minimal_witnesses(difference: &[bool], n: usize) -> Vec<usize> {
    assert_eq!(difference.len(), 1 << n);
    let mut below = difference.to_vec();
    for i in 0..n {
        for s in 0..difference.len() {
            if s & (1 << i) != 0 {
                below[s] |= below[s & !(1 << i)];
            }
        }
    }
    (0..difference.len())
        .filter(|&s| difference[s] && (0..n).all(|i| s & (1 << i) == 0 || !below[s & !(1 << i)]))
        .collect()
}

fn case_name(s: usize) -> String {
    format!("case_{s:04}")
}

fn short_case(study: &Study, s: usize) -> String {
    subset(&study.universe, s)
        .iter()
        .map(|e| {
            study
                .catalog
                .iter()
                .position(|p| p == e)
                .map(|i| SYMBOLS[i].to_owned())
                .unwrap_or_else(|| e.verb.clone())
        })
        .collect::<Vec<_>>()
        .join("+")
}

#[test]
fn fixed_hypotheses_have_all_inclusion_minimal_discriminating_cases() {
    let study = Study::new();
    let original = study.module.clone();
    let hypotheses = existing_hypotheses(&study);
    let n = study.universe.len();
    let mut pairs =
        String::from("left\tright\tdiffering_states\tminimal_cases\tminimal_goal_cases\n");
    let mut cases = String::from(
        "left\tright\tcase\tevidence\tleft_compliance\tright_compliance\tleft_only_events\tright_only_events\n",
    );
    let mut pool = BTreeSet::new();
    let mut pair_count = 0;
    let mut mixed_size_antichain = false;
    for (i, left) in hypotheses.iter().enumerate() {
        for right in &hypotheses[i + 1..] {
            let difference: Vec<_> = left
                .table
                .iter()
                .zip(&right.table)
                .map(|(a, b)| a != b)
                .collect();
            let goal_difference: Vec<_> = left
                .table
                .iter()
                .zip(&right.table)
                .map(|(&a, &b)| study.has_goal(a) != study.has_goal(b))
                .collect();
            // This family changes only rules whose head is C. Verify rather than
            // presume that C-observability distinguishes full logical closures.
            assert_eq!(difference, goal_difference);
            let minimal = inclusion_minimal_witnesses(&difference, n);
            let exact: BTreeSet<_> = difference
                .iter()
                .enumerate()
                .filter(|(_, d)| **d)
                .map(|(s, _)| s)
                .collect();
            assert_eq!(minimal, frontier(&exact)); // independent all-proper-subset oracle
            assert!(
                !minimal.is_empty(),
                "inequivalent operators in this fixed universe"
            );
            mixed_size_antichain |= minimal
                .iter()
                .any(|a| minimal.iter().any(|b| a.count_ones() != b.count_ones()));
            for s in 0..difference.len() {
                if difference[s] {
                    assert!(
                        minimal.iter().any(|&m| m & s == m),
                        "all differences have a minimal witness below"
                    );
                }
            }
            let goal_bit = mask_of(std::slice::from_ref(&study.goal), &study.universe);
            assert!(
                minimal.iter().any(|&s| !difference[s | goal_bit]),
                "adding C can erase the disagreement, so do not infer an upward cone"
            );
            writeln!(
                pairs,
                "{}\t{}\t{}\t{:?}\t{:?}",
                left.name,
                right.name,
                exact.len(),
                minimal,
                inclusion_minimal_witnesses(&goal_difference, n)
            )
            .unwrap();
            for &s in &minimal {
                assert!(
                    !study
                        .labels
                        .iter()
                        .any(|(_, seen, _)| study.projected(*seen) == s)
                );
                pool.insert(s);
                let a = left.table[s];
                let b = right.table[s];
                writeln!(
                    cases,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{:?}",
                    left.name,
                    right.name,
                    case_name(s),
                    short_case(&study, s),
                    study.has_goal(a),
                    study.has_goal(b),
                    subset(&study.universe, a & !b),
                    subset(&study.universe, b & !a)
                )
                .unwrap();
            }
            pair_count += 1;
        }
    }
    assert_eq!(pair_count, 36);
    assert!(
        mixed_size_antichain,
        "inclusion minima can have different cardinalities"
    );
    let expected: BTreeSet<_> = ["A+P", "A+M", "A+D", "A+violation", "A+P+violation"]
        .into_iter()
        .map(String::from)
        .collect();
    // Display order is the existing ground-event order, not a preference.
    let actual: BTreeSet<_> = pool
        .iter()
        .map(|&s| {
            let values: BTreeSet<_> = subset(&study.universe, s)
                .iter()
                .map(|e| e.verb.clone())
                .collect();
            if values.contains("violation") {
                if values.contains("permissionFiled") {
                    "A+P+violation"
                } else {
                    "A+violation"
                }
            } else if values.contains("disputedPerformance") {
                "A+D"
            } else if values.contains("omissionProhibition") {
                "A+M"
            } else {
                "A+P"
            }
            .to_owned()
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(pool.len(), 5);
    let mut predictions =
        String::from("case\tevidence\tpositive_candidates\tnegative_candidates\texternal_answer\n");
    let mut fixture = String::from(
        "module PendingDiscriminatingCases\nimport \"../../examples/experiments/legal_geometry/region_revision.res\"\n\n// Predictions are not external labels. No answer is supplied here.\n// Existing premises/fourFusion syntax is used for typed input only;\n// the harness compares Horn closures, not FOUR entailment.\n",
    );
    let mut signatures = BTreeSet::new();
    for h in &hypotheses {
        assert!(
            signatures.insert(
                pool.iter()
                    .map(|&s| study.has_goal(h.table[s]))
                    .collect::<Vec<_>>()
            )
        );
    }
    let mut hypothetical_answers = String::from("answer_pattern\tcompatible_candidates\tstatus\n");
    let mut nonempty_patterns = 0;
    for bits in 0..1 << pool.len() {
        let pattern: Vec<_> = (0..pool.len()).map(|i| bits & (1 << i) != 0).collect();
        let survivors: Vec<_> = hypotheses
            .iter()
            .filter(|h| {
                pool.iter()
                    .enumerate()
                    .all(|(i, &s)| study.has_goal(h.table[s]) == pattern[i])
            })
            .map(|h| h.name.clone())
            .collect();
        assert!(survivors.len() <= 1);
        if !survivors.is_empty() {
            nonempty_patterns += 1;
        }
        writeln!(
            hypothetical_answers,
            "{:?}\t{}\t{}",
            pattern,
            survivors.join(";"),
            if survivors.is_empty() {
                "counterfactual_version_space_empty"
            } else {
                "counterfactual_candidate_remains"
            }
        )
        .unwrap();
    }
    assert_eq!(nonempty_patterns, 9);
    // Restricting observable inputs to Seeded facts is a DIFFERENT question.
    // Derived D/V cannot be silently assumed observable in that protocol.
    let seeded: Vec<_> = study
        .universe
        .iter()
        .enumerate()
        .filter(|(_, e)| e.proposition.kind == PropositionKind::Seeded)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(seeded.len(), 5);
    let seed_masks: Vec<_> = (0..1 << seeded.len())
        .map(|s| {
            seeded
                .iter()
                .enumerate()
                .filter(|(i, _)| s & (1 << i) != 0)
                .fold(0, |mask, (_, i)| mask | (1 << i))
        })
        .collect();
    let mut seed_classes: BTreeMap<Vec<usize>, Vec<String>> = BTreeMap::new();
    for h in &hypotheses {
        seed_classes
            .entry(seed_masks.iter().map(|&s| h.table[s]).collect())
            .or_default()
            .push(h.name.clone());
    }
    assert_eq!(seed_classes.len(), 3);
    let mut restricted = String::from("observation_domain\tclass\thypotheses\n");
    for (i, members) in seed_classes.values().enumerate() {
        writeln!(
            restricted,
            "32_states_over_5_seeded_facts\t{i}\t{}",
            members.join(";")
        )
        .unwrap();
    }
    for &s in &pool {
        let positive: Vec<_> = hypotheses
            .iter()
            .filter(|h| study.has_goal(h.table[s]))
            .map(|h| h.name.clone())
            .collect();
        let negative: Vec<_> = hypotheses
            .iter()
            .filter(|h| !study.has_goal(h.table[s]))
            .map(|h| h.name.clone())
            .collect();
        assert!(!positive.is_empty() && !negative.is_empty());
        assert_eq!(positive.len() + negative.len(), hypotheses.len());
        writeln!(
            predictions,
            "{}\t{}\t{}\t{}\tpending",
            case_name(s),
            short_case(&study, s),
            positive.join(";"),
            negative.join(";")
        )
        .unwrap();
        writeln!(
            fixture,
            "\nexperiment {} {{\n    premises {{\n{}\n    }}\n    query fourFusion {}\n}}",
            case_name(s),
            subset(&study.universe, s)
                .iter()
                .map(|e| format!("        {},", render_event(e)))
                .collect::<Vec<_>>()
                .join("\n"),
            render_event(&study.goal)
        )
        .unwrap();
    }
    // Known alternative presentation from the previous study. No new hypothesis:
    // APD is redundant under AP. Its justification can differ, but there is no
    // evidence case discriminating its logical closure from AP on E'.
    let ap = study.hypothetical(&[4 | 8]);
    let redundant = study.hypothetical(&[4 | 8, 4 | 8 | 32]);
    let a = closure_table(&ground_clauses(&ap, &study.universe), n);
    let b = closure_table(&ground_clauses(&redundant, &study.universe), n);
    let diff: Vec<_> = a.iter().zip(&b).map(|(a, b)| a != b).collect();
    assert!(inclusion_minimal_witnesses(&diff, n).is_empty());
    let evidence = study.catalog.clone();
    assert_ne!(
        proofs(&close(&ap, evidence.clone())),
        proofs(&close(&redundant, evidence))
    );
    assert_eq!(study.module, original);
    save("identifiability-pairs.tsv", &pairs);
    save("identifiability-minimal-cases.tsv", &cases);
    save("identifiability-predictions.tsv", &predictions);
    save(
        "identifiability-counterfactual-answer-patterns.tsv",
        &hypothetical_answers,
    );
    save(
        "identifiability-seeded-observation-classes.tsv",
        &restricted,
    );
    save("pending-discriminating-cases.res", &fixture);
    let parsed = residuum::load_formal_module(
        root().join("target/legal-geometry/pending-discriminating-cases.res"),
    )
    .unwrap();
    for &s in &pool {
        let ex = experiment(&parsed, &case_name(s));
        assert_eq!(ex.input, subset(&study.universe, s));
        assert_eq!(ex.goal, study.goal);
    }
    let published = module("identifiability_cases");
    let declared: BTreeSet<_> = published
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) => Some(e.name.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(declared, pool.iter().map(|&s| case_name(s)).collect());
    for &s in &pool {
        assert_eq!(
            experiment(&published, &case_name(s)),
            experiment(&parsed, &case_name(s))
        );
    }
    println!(
        "9 fixed compatible hypotheses, 36 pairs, 4096 states each; 5 unlabelled cases separate all 9 behavioral signatures; equivalent AP/AP+APD have no logical discriminant"
    );
}

#[test]
fn informative_cases_replay_closures_and_preserve_unanswered_external_labels() {
    let study = Study::new();
    let original = study.module.clone();
    let hypotheses = existing_hypotheses(&study);
    let n = study.universe.len();
    let mut pool = BTreeSet::new();
    for (i, a) in hypotheses.iter().enumerate() {
        for b in &hypotheses[i + 1..] {
            let diff: Vec<_> = a.table.iter().zip(&b.table).map(|(x, y)| x != y).collect();
            pool.extend(inclusion_minimal_witnesses(&diff, n));
        }
    }
    let mut report = String::from("hypothesis\tcase\tcompliance\tknown_events\tjustifications\n");
    let mut provenance = String::from("hypothesis\tcase\tevent\trecord\torigin\n");
    for h in &hypotheses {
        let before = h.module.clone();
        for &s in &pool {
            let input = subset(&study.universe, s);
            let kept = input.clone();
            let current = close(&h.module, input.clone());
            let reference = residuum::derive::close_program_reference(
                input.clone(),
                &h.module,
                &mut Clock(1000),
            )
            .unwrap();
            assert_eq!(input, kept);
            assert_eq!(full_events(&current), full_events(&reference));
            assert_eq!(proofs(&current), proofs(&reference));
            assert_eq!(mask_of(&current.known, &study.universe), h.table[s]);
            audit_proofs(&h.module, &current);
            for d in &current.derivations {
                writeln!(
                    provenance,
                    "{}\t{}\t{:?}\t{:?}\t{:?}",
                    h.name,
                    case_name(s),
                    d.event,
                    d.record,
                    h.module.origin(&d.record.rule).unwrap()
                )
                .unwrap();
            }
            writeln!(
                report,
                "{}\t{}\t{}\t{}\t{}",
                h.name,
                case_name(s),
                current.known.contains(&study.goal),
                current.known.len(),
                current.derivations.len()
            )
            .unwrap();
        }
        assert_eq!(h.module, before);
    }
    // Both branches are CONDITIONAL version spaces. Neither is an actual legal
    // answer and no candidate is dropped absent that independent answer.
    for &s in &pool {
        let yes: BTreeSet<_> = hypotheses
            .iter()
            .filter(|h| study.has_goal(h.table[s]))
            .map(|h| h.name.clone())
            .collect();
        let no: BTreeSet<_> = hypotheses
            .iter()
            .filter(|h| !study.has_goal(h.table[s]))
            .map(|h| h.name.clone())
            .collect();
        assert!(yes.is_disjoint(&no));
        assert_eq!(yes.len() + no.len(), 9);
        assert!(!yes.is_empty() && !no.is_empty());
    }
    assert_eq!(study.module, original);
    assert_eq!(study.labels.len(), 13);
    save("identifiability-engine-closures.tsv", &report);
    save("identifiability-provenance.tsv", &provenance);
    println!(
        "45 informative executions: indexed/reference/model agree with complete provenance; all external answers pending, 9 hypotheses retained"
    );
}
