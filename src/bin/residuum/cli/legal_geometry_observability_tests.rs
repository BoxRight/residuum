//! Fixed hypotheses, broader Seeded inputs, and an explicit observation boundary.
use super::*;

const SEED_NAMES: [&str; 7] = ["O", "R", "B", "M", "P", "A", "N"];

fn seed_name(mask: usize) -> String {
    let parts: Vec<_> = SEED_NAMES
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, name)| *name)
        .collect();
    if parts.is_empty() {
        "empty".into()
    } else {
        parts.join("+")
    }
}

fn seeds() -> Vec<Event> {
    // Imported experiments stay in their declaring module.
    let input = experiment(&module("general"), "performance").input.clone();
    assert_eq!(input.len(), 7);
    assert_eq!(
        input.iter().map(|e| e.verb.as_str()).collect::<Vec<_>>(),
        [
            "obligationFiled",
            "right",
            "prohibition",
            "omissionProhibition",
            "permissionFiled",
            "Act",
            "Act"
        ]
    );
    assert!(input[..6].iter().all(|e| e.polarity == Polarity::Positive));
    assert!(
        input
            .iter()
            .all(|e| e.proposition.kind == PropositionKind::Seeded)
    );
    assert_eq!(input[6].polarity, Polarity::EvidentialNot);
    let mut negative_act = input[5].clone();
    negative_act.polarity = Polarity::EvidentialNot;
    assert_eq!(format!("{negative_act:?}"), format!("{:?}", input[6]));
    input
}

fn write_proofs(report: &mut String, context: &str, m: &Module, c: &Closure) {
    audit_proofs(m, c);
    for d in &c.derivations {
        writeln!(
            report,
            "{context}\t{:?}\t{:?}\t{:?}",
            d.event,
            d.record,
            m.origin(&d.record.rule).unwrap()
        )
        .unwrap();
    }
}

fn without_dispute_feedback(m: &Module) -> Module {
    let mut result = m.clone();
    result.declarations.retain(|d| {
        let Declaration::Rule(r) = d else { return true };
        if !r.name.starts_with("hypotheticalCompliance") {
            return true;
        }
        let RuleBody::HornClause(c) = &r.body else {
            unreachable!()
        };
        let mut body = Vec::new();
        atoms(&c.antecedent, &mut body);
        !body.iter().any(|e| e.verb == "disputedPerformance")
    });
    result
}

// Check the AST premises of the conditional, general redundancy argument.
// D cannot be Seeded; its unique producer requires exactly the corresponding C.
fn assert_dispute_requires_existing_compliance(study: &Study, hypotheses: &[Hypothesis]) {
    assert_eq!(
        study
            .module
            .declarations
            .iter()
            .filter(|d| matches!(d, Declaration::Rule(_)))
            .count(),
        27
    );
    assert!(
        !study
            .module
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Program(_) | Declaration::Default(_)))
    );
    for d in &study.module.declarations {
        if let Declaration::Verb(v) = d {
            assert!(v.program.is_none(), "no hidden dynamic producer");
            if v.name == "disputedPerformance" {
                assert_eq!(v.kind, VerbKind::Derived);
            }
        }
    }
    let producers: Vec<_> = study
        .module
        .declarations
        .iter()
        .filter_map(|d| {
            let Declaration::Rule(r) = d else { return None };
            let RuleBody::HornClause(c) = &r.body else {
                panic!("inspect other rule bodies")
            };
            (c.consequent.verb == "disputedPerformance").then_some((r, c))
        })
        .collect();
    assert_eq!(producers.len(), 1);
    let (r, c) = producers[0];
    assert_eq!(r.name, "performanceDispute");
    let mut body = Vec::new();
    atoms(&c.antecedent, &mut body);
    let required = body.iter().find(|e| e.verb == "compliance").unwrap();
    let mut same = (*required).clone();
    same.verb = "disputedPerformance".into();
    assert_eq!(format!("{same:?}"), format!("{:?}", c.consequent));
    for h in hypotheses {
        for d in &h.module.declarations {
            if let Declaration::Rule(r) = d {
                if study.module.declarations.iter().any(|d| d.name() == r.name) {
                    continue;
                }
                let RuleBody::HornClause(c) = &r.body else {
                    unreachable!()
                };
                assert_eq!(c.consequent, study.goal);
                let mut body = Vec::new();
                atoms(&c.antecedent, &mut body);
                for d in body.iter().filter(|e| e.verb == "disputedPerformance") {
                    let mut corresponding = (*d).clone();
                    corresponding.verb = "compliance".into();
                    assert_eq!(corresponding, study.goal);
                }
            }
        }
    }
}

#[test]
fn discriminating_cases_classify_seeded_and_indirect_observability() {
    let study = Study::new();
    let before = study.module.clone();
    let input = seeds();
    let hypotheses = existing_hypotheses(&study);
    assert_dispute_requires_existing_compliance(&study, &hypotheses);
    let paths = module("observability_cases");
    let v = experiment(&study.module, "theoryViolation").input[0].clone();
    let cases = [
        (
            "AP",
            "directAP",
            vec![study.catalog[2].clone(), study.catalog[3].clone()],
            true,
            true,
        ),
        (
            "AM",
            "directAM",
            vec![study.catalog[2].clone(), study.catalog[4].clone()],
            true,
            true,
        ),
        (
            "AD",
            "reachableAD",
            vec![study.catalog[2].clone(), study.catalog[5].clone()],
            false,
            false,
        ),
        (
            "AV",
            "reachableAV",
            vec![study.catalog[2].clone(), v.clone()],
            false,
            true,
        ),
        (
            "APV",
            "reachableAPV",
            vec![study.catalog[2].clone(), study.catalog[3].clone(), v],
            false,
            true,
        ),
    ];
    let baseline: Vec<_> = (0..128)
        .map(|s| events(&close(&study.module, subset(&input, s))))
        .collect();
    let mut report = String::from(
        "case\tdirect_seed_input\tminimal_seed_supports_for_reachability\tminimal_seed_supports_without_prior_C\tfixture\n",
    );
    let mut provenance = String::from("context\tevent\trecord\torigin\n");
    for (name, fixture, target, direct, clean) in cases {
        assert_eq!(
            target
                .iter()
                .all(|e| e.proposition.kind == PropositionKind::Seeded),
            direct
        );
        let reachable: BTreeSet<_> = baseline
            .iter()
            .enumerate()
            .filter(|(_, k)| target.iter().all(|e| k.contains(e)))
            .map(|(s, _)| s)
            .collect();
        let clean_reachable: BTreeSet<_> = reachable
            .iter()
            .copied()
            .filter(|&s| !baseline[s].contains(&study.goal))
            .collect();
        assert!(!reachable.is_empty());
        assert_eq!(!clean_reachable.is_empty(), clean);
        let ex = experiment(&paths, fixture);
        assert_eq!(ex.input_kind, ExperimentInputKind::Seeds);
        assert!(
            ex.input
                .iter()
                .all(|e| e.proposition.kind == PropositionKind::Seeded)
        );
        let saved = ex.input.clone();
        let current = close(&paths, saved.clone());
        assert!(target.iter().all(|e| current.known.contains(e)));
        assert_eq!(!current.known.contains(&study.goal), clean);
        assert_eq!(saved, ex.input);
        write_proofs(&mut provenance, name, &paths, &current);
        writeln!(
            report,
            "{name}\t{direct}\t{}\t{}\t{fixture}",
            frontier(&reachable)
                .iter()
                .map(|&s| seed_name(s))
                .collect::<Vec<_>>()
                .join(";"),
            frontier(&clean_reachable)
                .iter()
                .map(|&s| seed_name(s))
                .collect::<Vec<_>>()
                .join(";")
        )
        .unwrap();
    }
    assert_eq!(before, study.module);
    save("observability-paths.tsv", &report);
    save("observability-path-provenance.tsv", &provenance);
    println!(
        "5 cases classified; 128 baseline Seeded closures; D is reachable only after corresponding C; no external answer supplied"
    );
}

#[test]
fn expanded_seed_inputs_distinguish_seven_event_classes_and_preserve_proofs() {
    let study = Study::new();
    let before = study.module.clone();
    let input = seeds();
    let hypotheses = existing_hypotheses(&study);
    assert_dispute_requires_existing_compliance(&study, &hypotheses);
    let domain: Vec<_> = events(&close(&study.module, input.clone()))
        .into_iter()
        .collect();
    assert_eq!(domain.len(), 17);
    let mut inventory = String::from("index\tevent\n");
    for (i, e) in domain.iter().enumerate() {
        writeln!(inventory, "{i}\t{e:?}").unwrap();
    }
    let mut tables = Vec::new();
    let mut provenance = String::from("context\tevent\trecord\torigin\n");
    let mut execution = String::from(
        "hypothesis\tseed_mask\tinput\tclosed_mask\tcompliance\tevents\tjustifications\n",
    );
    for h in &hypotheses {
        let kept = h.module.clone();
        let clauses = ground_clauses(&h.module, &domain);
        let mut table = Vec::new();
        for s in 0..128 {
            let evidence = subset(&input, s);
            let saved = evidence.clone();
            let current = close(&h.module, evidence.clone());
            let closed = mask_of(&current.known, &domain);
            assert_eq!(closed, saturate(mask_of(&evidence, &domain), &clauses));
            assert_eq!(
                full_events(&current),
                subset(&domain, closed)
                    .iter()
                    .map(|e| format!("{e:?}"))
                    .collect()
            );
            assert_eq!(evidence, saved);
            audit_proofs(&h.module, &current);
            if s == 127 {
                let reference =
                    residuum::close_program_reference(evidence, &h.module, &mut Clock(2000))
                        .unwrap();
                assert_eq!(full_events(&current), full_events(&reference));
                assert_eq!(proofs(&current), proofs(&reference));
                write_proofs(&mut provenance, &h.name, &h.module, &current);
            }
            writeln!(
                execution,
                "{}\t{s}\t{}\t{closed}\t{}\t{}\t{}",
                h.name,
                seed_name(s),
                current.known.contains(&study.goal),
                current.known.len(),
                current.derivations.len()
            )
            .unwrap();
            table.push(closed);
        }
        assert_eq!(h.module, kept);
        tables.push(table);
    }
    let mut classes: BTreeMap<Vec<usize>, Vec<String>> = BTreeMap::new();
    for (h, table) in hypotheses.iter().zip(&tables) {
        classes
            .entry(table.clone())
            .or_default()
            .push(h.name.clone());
    }
    assert_eq!(classes.len(), 7);
    let mut class_report = String::from("class\thypotheses\tobservation\n");
    for (i, members) in classes.values().enumerate() {
        writeln!(
            class_report,
            "{i}\t{}\tall_17_ground_events_over_128_Seeded_inputs",
            members.join(";")
        )
        .unwrap();
    }
    let mut pairs =
        String::from("left\tright\tdiffering_inputs\tminimal_discriminating_seed_supports\n");
    let mut equal_pairs = 0;
    for (i, left) in hypotheses.iter().enumerate() {
        for (j, right) in hypotheses.iter().enumerate().skip(i + 1) {
            let different: Vec<_> = tables[i]
                .iter()
                .zip(&tables[j])
                .map(|(a, b)| a != b)
                .collect();
            let minimal = inclusion_minimal_witnesses(&different, 7);
            if minimal.is_empty() {
                equal_pairs += 1;
                assert!(matches!(
                    (left.name.as_str(), right.name.as_str()),
                    ("AM+AD", "AM+APD") | ("APM+AD", "APM+APD")
                ));
                assert_ne!(
                    proofs(&close(&left.module, input.clone())),
                    proofs(&close(&right.module, input.clone())),
                    "event equivalence does not identify proof records"
                );
                let a = without_dispute_feedback(&left.module);
                let b = without_dispute_feedback(&right.module);
                assert_eq!(a.declarations, b.declarations);
                for d in &a.declarations {
                    assert_eq!(a.origin(d.name()), b.origin(d.name()));
                }
            }
            writeln!(
                pairs,
                "{}\t{}\t{}\t{}",
                left.name,
                right.name,
                different.iter().filter(|d| **d).count(),
                minimal
                    .iter()
                    .map(|&s| seed_name(s))
                    .collect::<Vec<_>>()
                    .join(";")
            )
            .unwrap();
        }
    }
    assert_eq!(equal_pairs, 2);
    // Four legal-input probes distinguish the seven event classes in this
    // fixed family. They are predictions, NOT newly supplied legal labels.
    let probe_inputs = [16 | 32, 8 | 32, 4 | 32, 4 | 16 | 32]; // AP, AM, AB, ABP
    let goal_bit = mask_of(std::slice::from_ref(&study.goal), &domain);
    let signatures: BTreeSet<_> = tables
        .iter()
        .map(|table| {
            probe_inputs
                .iter()
                .map(|&s| table[s] & goal_bit != 0)
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(signatures.len(), 7);
    let mut probes = String::from("hypothesis\tAP\tAM\tAB\tABP\texternal_answer\n");
    for (h, table) in hypotheses.iter().zip(&tables) {
        writeln!(
            probes,
            "{}\t{}\t{}\t{}\t{}\tpending",
            h.name,
            table[probe_inputs[0]] & goal_bit != 0,
            table[probe_inputs[1]] & goal_bit != 0,
            table[probe_inputs[2]] & goal_bit != 0,
            table[probe_inputs[3]] & goal_bit != 0
        )
        .unwrap();
    }
    // Finite corroboration of removing only feedback clauses whose D already
    // entails the exact head C in this Seeded protocol. Never edit the sources.
    for (h, table) in hypotheses.iter().zip(&tables) {
        let without_feedback = without_dispute_feedback(&h.module);
        let clauses = ground_clauses(&without_feedback, &domain);
        for (s, &closed) in table.iter().enumerate() {
            assert_eq!(
                closed,
                saturate(mask_of(&subset(&input, s), &domain), &clauses)
            );
        }
    }
    // Recheck the two equivalent pairs against all original Seeded scenarios,
    // including authority, legislation, transfer and holdings.
    let mut background_report = String::from("left\tright\texperiment\tevents\tverdict\n");
    for (left, right) in [("AM+AD", "AM+APD"), ("APM+AD", "APM+APD")] {
        let a = &hypotheses.iter().find(|h| h.name == left).unwrap().module;
        let b = &hypotheses.iter().find(|h| h.name == right).unwrap().module;
        for d in &module("general").declarations {
            if let Declaration::Experiment(ex) = d {
                if ex.input_kind != ExperimentInputKind::Seeds {
                    continue;
                }
                let mut evidence = input.clone();
                evidence.extend(ex.input.clone());
                let ca = close(a, evidence.clone());
                let cb = close(b, evidence);
                assert_eq!(full_events(&ca), full_events(&cb));
                audit_proofs(a, &ca);
                audit_proofs(b, &cb);
                write_proofs(&mut provenance, &format!("{left}/{}", ex.name), a, &ca);
                write_proofs(&mut provenance, &format!("{right}/{}", ex.name), b, &cb);
                writeln!(
                    background_report,
                    "{left}\t{right}\t{}\t{}\tequal_all_events",
                    ex.name,
                    ca.known.len()
                )
                .unwrap();
            }
        }
    }
    assert_eq!(study.module, before);
    save("observability-ground-universe.tsv", &inventory);
    save("observability-engine-closures.tsv", &execution);
    save("observability-seeded-classes.tsv", &class_report);
    save("observability-discriminating-pairs.tsv", &pairs);
    save("observability-seeded-probes.tsv", &probes);
    save(
        "observability-background-observations.tsv",
        &background_report,
    );
    save("observability-hypothesis-provenance.tsv", &provenance);
    println!(
        "1152 engine/model comparisons; 7 event classes; 2 equal pairs with DIFFERENT provenance; all 9 hypotheses retained"
    );
}
