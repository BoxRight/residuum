//! Equivalence of the parametrised analysis layer with the archived harness
//! certificates. Each test is meant to run individually through
//! `scripts/test_safe.py`; none runs a full suite.
use super::exploration::*;
use crate::typed::*;
use std::collections::BTreeSet;
use std::path::Path;

const HEAD: &str = "hypotheticalSuccessionImpediment";
const PREFIX: &str = "candidateImpediment";
const CASES: [&str; 6] = [
    "historicalRiggs",
    "noBeneficiaryKilling",
    "killingWithoutAcquisitionPurpose",
    "killingByThirdParty",
    "causalLinkUnestablished",
    "survivalUnestablished",
];

fn module(name: &str) -> Module {
    crate::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("examples/experiments/legal_geometry/{name}.res")),
    )
    .unwrap()
}

fn experiment<'a>(m: &'a Module, name: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}

fn seeds_union(m: &Module) -> Vec<Event> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) if e.input_kind == ExperimentInputKind::Seeds => {
                Some(e.input.clone())
            }
            _ => None,
        })
        .flatten()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn verb(m: &Module, name: &str) -> Verb {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == name => Some(v.clone()),
            _ => None,
        })
        .unwrap()
}

fn catalog() -> (Module, Verb, Vec<Rule>) {
    let base = module("riggs_cases");
    let templates = module("riggs_candidate_rules");
    let head = verb(&templates, HEAD);
    let mut rules: Vec<Rule> = templates
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) if r.name.starts_with(PREFIX) => Some(r.clone()),
            _ => None,
        })
        .collect();
    rules.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(rules.len(), 64);
    (base, head, rules)
}

fn riggs_decision(base: &Module) -> Decision {
    let attr = experiment(base, CASES[0]).goal.clone();
    let mut imp = attr.clone();
    imp.verb = HEAD.into();
    Decision {
        components: vec![attr, imp],
        criterion: Criterion::Component(1),
    }
}

#[test]
fn exploration_frontiers_reproduce_certified_general_and_patrimonial_totals() {
    // Oracle: docs/legal-geometry/goal-frontier-results.json, computed by the
    // harness helper that asserted a universe of exactly 73 events.
    let g = module("general");
    let p = module("patrimonial");
    let e = universe(&p, &seeds_union(&p)).unwrap();
    assert_eq!(e.len(), 73);
    let all: BTreeSet<usize> = (0..e.len()).collect();
    let seeded = seeded_indices(&g, &e);
    assert_eq!(seeded.len(), 23);
    for (m, full, external) in [(&g, (73, 284), (61, 99)), (&p, (73, 332), (73, 114))] {
        let f = frontiers(m, &e, &all).unwrap();
        assert_eq!((f.reachable(), f.total_supports()), full);
        let s = frontiers(m, &e, &seeded).unwrap();
        assert_eq!((s.reachable(), s.total_supports()), external);
        for (q, frontier) in f.supports.iter().enumerate() {
            assert!(frontier.contains(&Support::from([q])));
            let restricted: Supports = frontier
                .iter()
                .filter(|x| x.is_subset(&seeded))
                .cloned()
                .collect();
            assert_eq!(restricted, s.supports[q]);
        }
    }
}

#[test]
fn exploration_riggs_catalog_partition_and_discriminants_match_certificates() {
    // Oracles: docs/legal-geometry/riggs-rule-learning/{partitions,questions}.tsv.
    let (base, head, rules) = catalog();
    let decision = riggs_decision(&base);
    let historical = experiment(&base, CASES[0]);
    let reference = crate::close_program(
        historical.input.clone(),
        &base,
        &mut CountingClock::default(),
    )
    .unwrap();
    let declared = BTreeSet::from([HEAD.to_string()]);
    let mut retained = Vec::new();
    let mut historical_signature = Vec::new();
    for (i, r) in rules.iter().enumerate() {
        let m = install(
            &base,
            std::slice::from_ref(&head),
            std::slice::from_ref(r),
            "RiggsCandidateRules",
        )
        .unwrap();
        assert_eq!(m.declarations.len(), base.declarations.len() + 2);
        assert_eq!(m.origin(&r.name).unwrap().module, "RiggsCandidateRules");
        let ev = evaluate(&m, &historical.input, &decision).unwrap();
        assert!(ev.components[0], "literal attribution is monotone");
        assert_eq!(ev.compatible, ev.components[1]);
        assert!(theory_preserved(&base, &m).is_ok());
        let cert = certify_addition(&reference, &ev.closure, &declared).unwrap();
        assert!(cert.holds(), "{}: {cert:?}", r.name);
        assert_eq!(cert.added_heads.is_empty(), !ev.components[1]);
        historical_signature.push(vec![ev.compatible]);
        if ev.compatible {
            retained.push(i);
        }
    }
    assert_eq!(retained.len(), 16);
    let historical_kernel = kernel(historical_signature);
    assert!(
        historical_kernel
            .iter()
            .enumerate()
            .all(|(i, k)| (*k == 0) == (i < 16))
    );

    // Joint universe and frontier-based regions.
    let sigma = seeds_union(&base);
    let (joint, e) = joint_universe(&base, std::slice::from_ref(&head), &rules, &sigma).unwrap();
    let allowed = seeded_indices(&joint, &e);
    let q = goal_index(&e, &decision.components[1]).unwrap();
    let fronts: Vec<Supports> = rules
        .iter()
        .map(|r| {
            let m = install(
                &base,
                std::slice::from_ref(&head),
                std::slice::from_ref(r),
                "RiggsCandidateRules",
            )
            .unwrap();
            frontiers(&m, &e, &allowed).unwrap().supports[q].clone()
        })
        .collect();
    // Every retained candidate reaches the goal; rejected ones have no
    // seeded support inside this positive universe with its negations.
    for (i, f) in fronts.iter().enumerate() {
        let hist = state_index(&e, &historical.input).unwrap();
        assert_eq!(
            in_region(f, &hist),
            retained.contains(&i),
            "{}",
            rules[i].name
        );
    }

    // Case predictions via frontiers equal the certified predictions and the
    // real closure.
    let certified: [&[usize]; 5] = [
        &[0, 8],
        &[0, 1, 2, 3, 8, 9, 10, 11],
        &[0, 8],
        &[0, 1, 8, 9],
        &[],
    ];
    for (c, expected) in CASES[1..].iter().zip(certified) {
        let ex = experiment(&base, c);
        let state = state_index(&e, &ex.input).unwrap();
        let predicted: Vec<usize> = retained
            .iter()
            .enumerate()
            .filter(|(_, h)| in_region(&fronts[**h], &state))
            .map(|(pos, _)| pos)
            .collect();
        assert_eq!(predicted, expected, "{c}");
        for &h in &retained {
            let m = install(
                &base,
                std::slice::from_ref(&head),
                std::slice::from_ref(&rules[h]),
                "RiggsCandidateRules",
            )
            .unwrap();
            let ev = evaluate(&m, &ex.input, &decision).unwrap();
            assert_eq!(
                ev.components[1],
                in_region(&fronts[h], &state),
                "{c} {}",
                rules[h].name
            );
        }
    }

    // Separation pool: the hypothesis partition over P refines every
    // case-based partition and separates the retained candidates into the
    // classes the catalog's six options induce.
    let pool = region_pool(retained.iter().map(|h| &fronts[*h]));
    let signatures: Vec<Vec<bool>> = retained
        .iter()
        .map(|h| signature(&fronts[*h], &pool))
        .collect();
    let pool_kernel = kernel(signatures.clone());
    let case_kernel = kernel(retained.iter().map(|h| {
        CASES
            .iter()
            .map(|c| {
                in_region(
                    &fronts[*h],
                    &state_index(&e, &experiment(&base, c).input).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    }));
    assert!(finer(&pool_kernel, &case_kernel).unwrap());
    let ids: Vec<String> = retained.iter().map(|h| rules[*h].name.clone()).collect();
    let by_pool = Partition::new(ids.clone(), signatures.clone()).unwrap();
    let by_cases = Partition::new(
        ids.clone(),
        retained
            .iter()
            .map(|h| {
                CASES
                    .iter()
                    .map(|c| {
                        in_region(
                            &fronts[*h],
                            &state_index(&e, &experiment(&base, c).input).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect(),
    )
    .unwrap();
    assert!(by_pool.finer(&by_cases).unwrap());
    assert_eq!(by_pool.meet(&by_cases).unwrap().kernel, pool_kernel);
    // Controls that turn tool misuse into errors, never into verdicts.
    assert!(meet(&pool_kernel, &pool_kernel[..pool_kernel.len() - 1]).is_err());
    assert!(finer(&pool_kernel[..1], &pool_kernel).is_err());
    let mut shifted = ids.clone();
    shifted.rotate_left(1);
    let rotated = Partition::new(shifted, signatures.clone()).unwrap();
    assert!(by_pool.meet(&rotated).is_err());
    assert!(Partition::new(ids[..3].to_vec(), signatures.clone()).is_err());
    let out_of_range = Decision {
        criterion: Criterion::Component(5),
        ..decision.clone()
    };
    let m0 = install(
        &base,
        std::slice::from_ref(&head),
        std::slice::from_ref(&rules[0]),
        "X",
    )
    .unwrap();
    assert!(evaluate(&m0, &historical.input, &out_of_range).is_err());
    let wrong_length = Decision {
        criterion: Criterion::Exact(vec![true]),
        ..decision.clone()
    };
    assert!(evaluate(&m0, &historical.input, &wrong_length).is_err());
    // Certificate scope: a head the reference already produces is refused.
    let ev0 = evaluate(&m0, &historical.input, &decision).unwrap();
    assert!(
        certify_addition(
            &reference,
            &ev0.closure,
            &BTreeSet::from([HEAD.to_string(), "testamentaryEntitlement".to_string()])
        )
        .is_err()
    );
    assert!(class_count(&pool_kernel) >= class_count(&case_kernel));
    let split = discriminants(&signatures);
    assert!(!split.is_empty());
    // Every pool state is realised by the real closure for retained candidates.
    for (pos, &h) in retained.iter().enumerate() {
        let m = install(
            &base,
            std::slice::from_ref(&head),
            std::slice::from_ref(&rules[h]),
            "RiggsCandidateRules",
        )
        .unwrap();
        for (j, s) in pool.iter().enumerate() {
            let ev = evaluate(&m, &state_events(&e, s), &decision).unwrap();
            assert_eq!(
                ev.components[1], signatures[pos][j],
                "{} on pool state {j}",
                rules[h].name
            );
        }
    }
    // Generality: the unconditional candidate covers every other retained one.
    assert!(retained.iter().all(|h| covers(&fronts[0], &fronts[*h])));

    // Diagnostics for the contract; not an oracle.
    let mut report = String::from("quantity\tvalue\n");
    for (k, v) in [
        ("joint_universe_events", e.len()),
        ("allowed_seeded_leaves", allowed.len()),
        ("retained_candidates", retained.len()),
        ("separation_pool_states", pool.len()),
        ("pool_partition_classes", class_count(&pool_kernel)),
        ("six_case_partition_classes", class_count(&case_kernel)),
        ("pool_discriminant_states", split.len()),
        ("historical_frontier_elements_candidate00", fronts[0].len()),
    ] {
        report.push_str(&format!("{k}\t{v}\n"));
    }
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/exploration");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("riggs-separation-pool.tsv"), report).unwrap();
}

#[test]
fn exploration_profile_admits_riggs_catalog_and_reports_bounded_rejections() {
    let (base, head, rules) = catalog();
    let joint = install(
        &base,
        std::slice::from_ref(&head),
        &rules,
        "RiggsCandidateRules",
    )
    .unwrap();
    let verbs: BTreeSet<String> = joint
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) => Some(v.name.clone()),
            _ => None,
        })
        .collect();
    let seeded: BTreeSet<String> = joint
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) if v.kind == VerbKind::Seeded => Some(v.name.clone()),
            _ => None,
        })
        .collect();
    let riggs = Profile {
        heads: BTreeSet::from([HEAD.to_string()]),
        body_verbs: verbs,
        evidential_not_verbs: seeded,
        max_atoms: 7,
        max_binders: 16,
        max_after_depth: 2,
        max_unit_nodes: 1,
        require_connected: true,
    };
    for r in &rules {
        assert_eq!(classify(&joint, r, &riggs), BTreeSet::new(), "{}", r.name);
    }
    // With the P1 bound of four atoms exactly the masks with at most three
    // options survive: C(6,0)+C(6,1)+C(6,2)+C(6,3) = 42.
    let p1 = Profile {
        max_atoms: 4,
        ..riggs.clone()
    };
    let admitted = rules
        .iter()
        .filter(|r| classify(&joint, r, &p1).is_empty())
        .count();
    assert_eq!(admitted, 42);
    for (i, r) in rules.iter().enumerate() {
        let reasons = classify(&joint, r, &p1);
        assert_eq!(
            reasons.contains("B_ATOMS"),
            i.count_ones() > 3,
            "{}",
            r.name
        );
    }

    let RuleBody::HornClause(c0) = &rules[0].body else {
        unreachable!()
    };
    // Head outside the interface.
    let mut wrong_head = rules[0].clone();
    let mut clause = c0.clone();
    clause.consequent.verb = "testamentaryEntitlement".into();
    wrong_head.body = RuleBody::HornClause(clause);
    assert!(classify(&joint, &wrong_head, &riggs).contains("P_HEAD_INTERFACE"));
    // Unbound head variables.
    let mut unbound = rules[0].clone();
    let mut clause = c0.clone();
    clause.antecedent = Antecedent::Unit;
    unbound.body = RuleBody::HornClause(clause);
    let reasons = classify(&joint, &unbound, &riggs);
    assert!(reasons.contains("S_UNBOUND_HEAD"), "{reasons:?}");
    assert!(!reasons.contains("B_UNIT_NODES"));
    // A unit node inside a conjunction.
    let mut padded = rules[0].clone();
    let mut clause = c0.clone();
    clause.antecedent =
        Antecedent::And(Box::new(Antecedent::Unit), Box::new(c0.antecedent.clone()));
    padded.body = RuleBody::HornClause(clause);
    let reasons = classify(&joint, &padded, &riggs);
    assert!(
        reasons.contains("B_UNIT_NODES") && reasons.contains("B_AND_NODES"),
        "{reasons:?}"
    );
    // Body vocabulary and legacy profile.
    let narrow = Profile {
        body_verbs: BTreeSet::new(),
        ..riggs.clone()
    };
    assert!(classify(&joint, &rules[0], &narrow).contains("P_BODY_VOCABULARY"));
    let legacy = classify(&joint, &rules[0], &Profile::legacy_compliance());
    assert!(legacy.contains("P_HEAD_INTERFACE") && legacy.contains("P_BODY_VOCABULARY"));
    // Polarity: evidential negation on a head or a non-admitted verb.
    let strict = Profile {
        evidential_not_verbs: BTreeSet::new(),
        ..riggs.clone()
    };
    let negated = rules.iter().find(|r| {
        let RuleBody::HornClause(c) = &r.body else {
            return false;
        };
        let mut atoms = Vec::new();
        fn collect<'a>(a: &'a Antecedent, out: &mut Vec<&'a Event>) {
            match a {
                Antecedent::Unit => (),
                Antecedent::Event(e) => out.push(e),
                Antecedent::And(l, r) => {
                    collect(l, out);
                    collect(r, out);
                }
            }
        }
        collect(&c.antecedent, &mut atoms);
        atoms.iter().any(|e| e.polarity == Polarity::EvidentialNot)
    });
    let negated = negated.expect("catalog has evidentially negated options");
    assert!(classify(&joint, negated, &riggs).is_empty());
    assert!(classify(&joint, negated, &strict).contains("P_POLARITY"));

    // P1 plus the Riggs problem manifest reproduces the four-atom admission.
    let problem_body = [
        "operativeDisposition",
        "deathRecorded",
        "survivorRecorded",
        "intentionalKilling",
        "deathCausedBy",
        "acquisitionPurposeRecorded",
        "testamentaryEntitlement",
        "acquisitionLinkedToWrong",
        "purposefulAcquisition",
        "patrimonialAsset",
    ];
    let p1_riggs = Profile::p1().with_problem(
        [HEAD],
        problem_body,
        ["intentionalKilling", "acquisitionPurposeRecorded"],
    );
    assert_eq!(p1_riggs.max_atoms, 4);
    let admitted_p1 = rules
        .iter()
        .filter(|r| classify(&joint, r, &p1_riggs).is_empty())
        .count();
    assert_eq!(admitted_p1, 42);
    assert!(classify(&joint, &rules[0], &Profile::p1()).contains("P_HEAD_INTERFACE"));

    // Candidate-level admission: clause count, own intermediate heads as
    // body vocabulary, and no chains longer than intermediate → final.
    assert!(classify_candidate(&joint, &rules[0..1], &p1_riggs, 2).is_empty());
    assert!(classify_candidate(&joint, &rules[0..2], &p1_riggs, 2).is_empty());
    assert!(classify_candidate(&joint, &rules[0..3], &p1_riggs, 2).contains("B_CLAUSES"));
    assert!(classify_candidate(&joint, &[], &p1_riggs, 2).contains("B_CLAUSES"));
    let mut feeder = rules[0].clone();
    feeder.name = "feeder".into();
    let mut clause = c0.clone();
    clause.antecedent = Antecedent::Event(c0.consequent.clone());
    clause.consequent = match &c0.antecedent {
        Antecedent::Event(e) => e.clone(),
        _ => unreachable!(),
    };
    feeder.body = RuleBody::HornClause(clause);
    // Alone, `feeder` uses the experimental head in its body without it being
    // problem vocabulary unless declared; paired with a clause that produces
    // it, the own-head exemption applies but the pair is a cycle.
    let without_head_literal = Profile::p1().with_problem(
        [HEAD, "testamentaryEntitlement"],
        problem_body,
        ["intentionalKilling", "acquisitionPurposeRecorded"],
    );
    assert!(
        classify_candidate(
            &joint,
            std::slice::from_ref(&feeder),
            &without_head_literal,
            2
        )
        .contains("P_BODY_VOCABULARY")
    );
    let pair = classify_candidate(
        &joint,
        &[rules[0].clone(), feeder],
        &without_head_literal,
        2,
    );
    assert!(!pair.contains("P_BODY_VOCABULARY"), "{pair:?}");
    assert!(pair.contains("B_CLAUSES"), "{pair:?}");
}

#[test]
fn exploration_install_rejects_collisions_and_non_derived_heads() {
    let (base, head, rules) = catalog();
    let dup = install(
        &base,
        &[head.clone()],
        &[rules[0].clone(), rules[0].clone()],
        "X",
    );
    assert!(dup.unwrap_err().to_string().contains("S_INSTALL"));
    let mut seeded_head = head.clone();
    seeded_head.kind = VerbKind::Seeded;
    let bad = install(&base, &[seeded_head], &[rules[0].clone()], "X");
    assert!(bad.unwrap_err().to_string().contains("S_INSTALL"));
    let mut existing = rules[0].clone();
    existing.name = "literalTestamentaryEntitlement".into();
    let clash = install(&base, &[head.clone()], &[existing], "X");
    assert!(clash.unwrap_err().to_string().contains("S_INSTALL"));
    // Re-installing an identical verb is idempotent; the theory is untouched.
    let once = install(&base, &[head.clone()], &[rules[0].clone()], "X").unwrap();
    let twice = install(&once, &[head.clone()], &[rules[1].clone()], "X").unwrap();
    assert_eq!(twice.declarations.len(), base.declarations.len() + 3);
    for d in &base.declarations {
        assert_eq!(
            twice.declarations.iter().find(|p| p.name() == d.name()),
            Some(d)
        );
        assert_eq!(twice.origin(d.name()), base.origin(d.name()));
    }
    assert!(theory_preserved(&base, &twice).is_ok());
    // Tampering with a theory declaration or claiming a theory origin fails.
    let mut altered = twice.clone();
    let idx = altered
        .declarations
        .iter()
        .position(|d| d.name() == "literalTestamentaryEntitlement")
        .unwrap();
    altered.declarations.remove(idx);
    assert!(theory_preserved(&base, &altered).is_err());
    let mut claimed = twice.clone();
    let theory_origin = base
        .origin("literalTestamentaryEntitlement")
        .unwrap()
        .clone();
    claimed.origins.insert(rules[0].name.clone(), theory_origin);
    assert!(theory_preserved(&base, &claimed).is_err());
}

#[derive(Default)]
struct CountingClock(usize);
impl crate::DerivationClock for CountingClock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}
