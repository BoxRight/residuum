//! Generated relational rule hypotheses, additive private theories, fixed reference.
use super::super::conversation::{finer, kernel, meet};
use super::*;

const HEAD: &str = "hypotheticalSuccessionImpediment";
const PREFIX: &str = "candidateImpediment";
const ENTITLEMENT: &str =
    "testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t";
const OPTIONS: [&str; 6] = [
    "intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t",
    "acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t",
    "acquisitionPurposeRecorded(act, will, beneficiary) @ t",
    "patrimonialAsset(beneficiary, object, will, authority, effective) @ t",
    "~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t",
    "~acquisitionPurposeRecorded(act, will, beneficiary) @ t",
];
const LABELS: [&str; 6] = ["K", "L", "F", "B", "notK", "notF"];

fn generated_source() -> String {
    let mut s = String::from(
        "module RiggsCandidateRules\n\nimport \"./riggs_succession_reference.res\"\n\n// Generated bounded hypotheses. Never load all candidate rules as one theory.\n// Impediment is a proposed legal conclusion, not an automatic removal of title.\nderived verb hypotheticalSuccessionImpediment(will : Fact, testator : Person, beneficiary : Person, object : Thing, authority : Authority, effective : PropositionTime)\n",
    );
    for mask in 0..64 {
        let mut body = vec![ENTITLEMENT];
        body.extend(
            OPTIONS
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, s)| *s),
        );
        let joined = body.join("\n    & ");
        let words: BTreeSet<_> = joined.split(|c: char| !c.is_alphanumeric()).collect();
        let parameters: Vec<_> = [
            "will",
            "testator",
            "beneficiary",
            "object",
            "authority",
            "effective",
            "act",
            "death",
            "actAuthority",
            "actEffective",
            "t",
        ]
        .into_iter()
        .filter(|p| words.contains(p))
        .collect();
        writeln!(s, "\nrule {PREFIX}{mask:02}({}) =\n    {joined}\n    <= {HEAD}(will, testator, beneficiary, object, authority, effective) @ t", parameters.join(", ")).unwrap();
    }
    s
}

fn output(name: &str, text: &str) {
    let dir = root().join("docs/legal-geometry/riggs-rule-learning");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

fn candidate_name(mask: usize) -> String {
    format!("{PREFIX}{mask:02}")
}

fn with_passive_head(mut m: Module, templates: &Module) -> Module {
    let d = templates
        .declarations
        .iter()
        .find(|d| d.name() == HEAD)
        .unwrap();
    m.declarations.push(d.clone());
    m.origins
        .insert(HEAD.into(), templates.origin(HEAD).unwrap().clone());
    m
}
fn label(mask: usize) -> String {
    std::iter::once("E")
        .chain(
            LABELS
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, s)| *s),
        )
        .collect::<Vec<_>>()
        .join(" & ")
}

struct Study {
    base: Module,
    templates: Module,
    baseline: Vec<Closure>,
}

impl Study {
    fn new() -> Self {
        let base = module("riggs_cases");
        let templates = module("riggs_candidate_rules");
        audit_reference(&base);
        audit_reference(&templates);
        assert_eq!(
            std::fs::read_to_string(
                root().join("examples/experiments/legal_geometry/riggs_candidate_rules.res")
            )
            .unwrap(),
            generated_source()
        );
        assert_eq!(
            templates
                .declarations
                .iter()
                .filter(|d| d.name().starts_with(PREFIX))
                .count(),
            64
        );
        let baseline = CASES
            .iter()
            .map(|n| close(&base, experiment(&base, n).input.clone()))
            .collect();
        Self {
            base,
            templates,
            baseline,
        }
    }

    // T ⊕ r = literal addition of one Derived declaration and one clause.
    // No reference rule is removed, guarded, reprioritized, or rewritten.
    fn theory(&self, mask: usize) -> Module {
        let mut m = self.base.clone();
        let name = candidate_name(mask);
        for d in &self.templates.declarations {
            if d.name() == name || d.name() == HEAD {
                m.declarations.push(d.clone());
                m.origins.insert(
                    d.name().into(),
                    self.templates.origin(d.name()).unwrap().clone(),
                );
            }
        }
        for d in &self.base.declarations {
            assert_eq!(
                m.declarations.iter().find(|p| p.name() == d.name()),
                Some(d)
            );
            assert_eq!(m.origin(d.name()), self.base.origin(d.name()));
        }
        assert_eq!(m.declarations.len(), self.base.declarations.len() + 2);
        m
    }

    fn goal(&self, i: usize) -> Event {
        let mut e = experiment(&self.base, CASES[i]).goal.clone();
        e.verb = HEAD.into();
        e
    }

    fn run(&self, mask: usize, i: usize) -> Closure {
        close(
            &self.theory(mask),
            experiment(&self.base, CASES[i]).input.clone(),
        )
    }

    fn certify(&self, mask: usize, i: usize, c: &Closure) {
        let m = self.theory(mask);
        audit_proofs(&m, c);
        let mut original = c.clone();
        original.known.retain(|e| e.verb != HEAD);
        original.derivations.retain(|d| d.event.verb != HEAD);
        assert_eq!(full_events(&original), full_events(&self.baseline[i]));
        assert_eq!(proof_identity(&original), proof_identity(&self.baseline[i]));
        let fresh_reference = close(&self.base, experiment(&self.base, CASES[i]).input.clone());
        assert_eq!(
            full_events(&fresh_reference),
            full_events(&self.baseline[i])
        );
        assert_eq!(
            proof_identity(&fresh_reference),
            proof_identity(&self.baseline[i])
        );
        for d in c.derivations.iter().filter(|d| d.event.verb == HEAD) {
            assert_eq!(d.record.rule, candidate_name(mask));
            assert_eq!(
                m.origin(&d.record.rule).unwrap().module,
                "RiggsCandidateRules"
            );
            assert_eq!(d.event.args, experiment(&self.base, CASES[i]).goal.args);
            assert_eq!(
                d.event.proposition.time,
                experiment(&self.base, CASES[i]).goal.proposition.time
            );
            assert_eq!(d.record.antecedents.len(), 1 + mask.count_ones() as usize);
        }
    }
}

#[test]
fn generated_relational_rules_learn_from_historical_outcome_preserving_reference() {
    let study = Study::new();
    let observation = rows("rule_observations");
    assert_eq!(observation.len(), 1);
    assert_eq!(
        &observation[0][..4],
        [
            "historicalRiggs",
            "true",
            "documented_judgment",
            "https://www.nycourts.gov/reporter/archives/riggs_palmer.htm"
        ]
    );
    let mut matrix = String::from(
        "candidate\tsupport_schema\tcase\timpediment_derived\tliteral_entitlement\treference_projection_equal\teffective_decision\n",
    );
    let mut provenance =
        String::from("candidate\tcase\tevent\trecord\trule_origin\tverb_origin\tderived_at\n");
    let mut catalog = String::from(
        "candidate\tconditions\thistorical_compatible\telimination_reason\ttyped_rule\n",
    );
    let mut survivors = Vec::new();
    for h in 0..64 {
        let theory = study.theory(h);
        for i in 0..CASES.len() {
            let c = study.run(h, i);
            study.certify(h, i, &c);
            let yes = c.known.contains(&study.goal(i));
            if i == 0 {
                if yes {
                    survivors.push(h);
                }
                let rule = theory
                    .declarations
                    .iter()
                    .find(|d| d.name() == candidate_name(h))
                    .unwrap();
                writeln!(
                    catalog,
                    "{}\t{}\t{yes}\t{}\t{rule:?}",
                    candidate_name(h),
                    label(h),
                    if yes {
                        "retained_not_legally_approved"
                    } else {
                        "no_impediment_proof_in_delimited_historical_dossier"
                    }
                )
                .unwrap();
            }
            // A positive alternative is not an effective override of the literal title.
            assert_eq!(
                c.known.contains(&experiment(&study.base, CASES[i]).goal),
                i != 5
            );
            writeln!(
                matrix,
                "{}\t{}\t{}\t{yes}\t{}\ttrue\tnot_resolved_by_additive_Horn",
                candidate_name(h),
                label(h),
                CASES[i],
                i != 5
            )
            .unwrap();
            for d in &c.derivations {
                writeln!(
                    provenance,
                    "{}\t{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
                    candidate_name(h),
                    CASES[i],
                    d.event,
                    d.record,
                    theory.origin(&d.record.rule),
                    theory.origin(&d.event.verb),
                    d.derived_at
                )
                .unwrap();
            }
        }
    }
    assert_eq!(survivors, (0..16).collect::<Vec<_>>());
    output("candidate-rules.tsv", &catalog);
    output("case-consequences.tsv", &matrix);
    output("provenance.tsv", &provenance);
    output(
        "historical-observation.tsv",
        &std::fs::read_to_string(
            root().join("examples/experiments/legal_geometry/riggs_rule_observations.tsv"),
        )
        .unwrap(),
    );
}

fn predictions(study: &Study) -> Vec<Vec<bool>> {
    (0..64)
        .map(|h| {
            (0..6)
                .map(|i| study.run(h, i).known.contains(&study.goal(i)))
                .collect()
        })
        .collect()
}

#[test]
fn learned_rule_partitions_and_counterfactual_questions_preserve_indetermination() {
    let study = Study::new();
    let p = predictions(&study);
    let initial = vec![0; 64];
    let historical = kernel(p.iter().map(|v| vec![v[0]]).collect());
    assert_eq!(meet(&initial, &historical), historical);
    let retained: Vec<_> = (0..64).filter(|&h| p[h][0]).collect();
    assert_eq!(retained.len(), 16);
    let mut versions =
        String::from("history\tcandidate\tretained\treason\tvaluation_source\tvaluation_status\n");
    let mut questions = String::from(
        "case\tpredict_impediment\tno_proof\tworst_branch\tpremise_status_causal\tpremise_status_purpose\texternal_valuation\n",
    );
    let mut partition = String::from("history\tkernel\n");
    writeln!(partition, "historical\t{historical:?}").unwrap();
    let mut scores = Vec::new();
    for i in 1..6 {
        let positives: Vec<_> = retained.iter().copied().filter(|&h| p[h][i]).collect();
        let negatives: Vec<_> = retained.iter().copied().filter(|&h| !p[h][i]).collect();
        scores.push((i, positives.len().max(negatives.len())));
        let ex = experiment(&study.base, CASES[i]);
        let causal = assess(&study.base, &ex.goal, &study.baseline[i], "linked_wrong");
        let purpose = assess(
            &study.base,
            &ex.goal,
            &study.baseline[i],
            "purposeful_acquisition",
        );
        writeln!(
            questions,
            "{}\t{positives:?}\t{negatives:?}\t{}\t{:?}\t{:?}\tpending_no_historical_judgment",
            CASES[i],
            positives.len().max(negatives.len()),
            causal.status,
            purpose.status
        )
        .unwrap();
        let one = kernel(p.iter().map(|v| vec![v[i]]).collect());
        let whole = kernel(p.iter().map(|v| vec![v[0], v[i]]).collect());
        assert_eq!(meet(&historical, &one), whole);
        assert!(finer(&whole, &historical));
    }
    let best = scores.iter().map(|(_, w)| *w).min().unwrap();
    assert_eq!(best, 8);
    assert_eq!(
        scores
            .iter()
            .filter(|(_, w)| *w == best)
            .map(|(i, _)| *i)
            .collect::<Vec<_>>(),
        [2]
    );
    // Independent hypothetical answers, not invented historical decisions.
    let followups = rows("rule_followup_valuations");
    assert_eq!(followups.len(), 3);
    for v in &followups {
        assert_eq!(v[1], CASES[2]);
        assert_eq!(v[4], "hypothetical_external_assumption");
        let answer = match v[2].as_str() {
            "true" => Some(true),
            "false" => Some(false),
            "pending" => None,
            _ => panic!("invalid answer"),
        };
        let keep: Vec<_> = retained
            .iter()
            .copied()
            .filter(|&h| answer.is_none_or(|y| p[h][2] == y))
            .collect();
        assert!(!keep.is_empty());
        if answer.is_none() {
            assert_eq!(keep, retained);
        } else {
            assert_eq!(keep.len(), 8);
        }
        writeln!(
            partition,
            "historical_then_no_purpose_{answer:?}\t{:?}",
            if answer.is_none() {
                historical.clone()
            } else {
                meet(&historical, &kernel(p.iter().map(|v| vec![v[2]]).collect()))
            }
        )
        .unwrap();
        for &h in &retained {
            writeln!(
                versions,
                "historical_then_no_purpose_{answer:?}\t{}\t{}\t{}\t{}\t{}",
                candidate_name(h),
                keep.contains(&h),
                if keep.contains(&h) {
                    "compatible_or_pending"
                } else {
                    "hypothetical_external_response_mismatch"
                },
                v[3],
                v[4]
            )
            .unwrap();
        }
    }
    // Catalog-bound observational equivalence is different from rule identity.
    let case_partition = kernel(retained.iter().map(|&h| p[h].clone()).collect());
    writeln!(partition, "six_dossiers_on_survivors\t{case_partition:?}").unwrap();
    let mut pair_minima =
        String::from("left\tright\tseed_mask\tevidence\tleft_result\tright_result\n");
    let seed = experiment(&study.base, CASES[0]).input.clone();
    let tables: Vec<Vec<bool>> = retained
        .iter()
        .map(|&h| {
            let theory = study.theory(h);
            (0..64)
                .map(|s| {
                    close(&theory, subset(&seed, s))
                        .known
                        .contains(&study.goal(0))
                })
                .collect()
        })
        .collect();
    for (i, &h) in retained.iter().enumerate() {
        for (j, &k) in retained.iter().enumerate().skip(i + 1) {
            for s in 0..64 {
                if tables[i][s] != tables[j][s]
                    && (0..64).all(|t| t == s || t & s != t || tables[i][t] == tables[j][t])
                {
                    writeln!(
                        pair_minima,
                        "{}\t{}\t{s}\t{:?}\t{}\t{}",
                        candidate_name(h),
                        candidate_name(k),
                        subset(&seed, s),
                        tables[i][s],
                        tables[j][s]
                    )
                    .unwrap();
                }
            }
        }
    }
    assert!(pair_minima.lines().count() > 1);
    output("questions.tsv", &questions);
    output("versions.tsv", &versions);
    output("partitions.tsv", &partition);
    output("pair-minimal-discriminants.tsv", &pair_minima);
    output(
        "followup-valuations.tsv",
        &std::fs::read_to_string(
            root().join("examples/experiments/legal_geometry/riggs_rule_followup_valuations.tsv"),
        )
        .unwrap(),
    );
}

fn mask_of(input: &[Event], domain: &[Event]) -> usize {
    input.iter().fold(0, |m, e| {
        m | (1
            << domain
                .iter()
                .position(|x| x == e)
                .expect("closed finite ground universe"))
    })
}
fn saturation(mut s: usize, clauses: &[(usize, usize)]) -> usize {
    loop {
        let old = s;
        for &(b, h) in clauses {
            if b & s == b {
                s |= h;
            }
        }
        if old == s {
            return s;
        }
    }
}
fn grounding(m: &Module, domain: &[Event]) -> Vec<(usize, usize)> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r),
            _ => None,
        })
        .flat_map(|r| instantiate_from_known(r, domain))
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
fn minima(region: &[usize]) -> Vec<usize> {
    region
        .iter()
        .copied()
        .filter(|&s| !region.iter().any(|&t| t != s && t & s == t))
        .collect()
}

#[test]
fn candidate_theories_check_geometry_extreme_bases_and_seed_frontiers() {
    let study = Study::new();
    let seeds = experiment(&study.base, CASES[0]).input.clone();
    let mut domain = study.baseline[0].known.clone();
    domain.push(study.goal(0));
    domain.sort();
    domain.dedup();
    assert_eq!(domain.len(), 13);
    let n = domain.len();
    let goal_bit = mask_of(&[study.goal(0)], &domain);
    let reference_clauses = grounding(
        &with_passive_head(study.base.clone(), &study.templates),
        &domain,
    );
    let reference_table: Vec<_> = (0..1 << n)
        .map(|s| saturation(s, &reference_clauses))
        .collect();
    let mut mergers = String::from(
        "candidate\twithout_impediment\twith_impediment\treference_closures_different\tcandidate_closures_equal\n",
    );
    let mut geometry = String::from(
        "theory\tpoints\tstates\tclosed\textensive\tmonotone\tidempotent\tempty_closed\tanti_exchange\textreme_bases_verified\tacyclic\n",
    );
    let mut bases = String::from("theory\tclosed_state\textreme_basis\n");
    let mut supports = String::from("candidate\tseed_mask\tminimal_seed_evidence\n");
    let mut frontier_proofs = String::from("candidate\tseed_mask\tevent\trecord\torigin\n");
    let mut dependencies = String::from("theory\trule\torigin\tantecedents\tconsequent\n");
    for h in 0..=64 {
        let m = if h == 64 {
            with_passive_head(study.base.clone(), &study.templates)
        } else {
            study.theory(h)
        };
        let name = if h == 64 {
            "reference".into()
        } else {
            candidate_name(h)
        };
        let clauses = grounding(&m, &domain);
        let table: Vec<_> = (0..1 << n).map(|s| saturation(s, &clauses)).collect();
        assert_eq!(table[0], 0);
        for (s, &f) in table.iter().enumerate() {
            assert_eq!(f & !goal_bit, reference_table[s] & !goal_bit);
            assert_eq!(s & f, s);
            assert_eq!(table[f], f);
            for i in 0..n {
                assert_eq!(f & table[s | (1 << i)], f);
            }
        }
        let mut closed = 0;
        for f in (0..table.len()).filter(|&f| table[f] == f) {
            closed += 1;
            let ex = (0..n)
                .filter(|&i| f & (1 << i) != 0 && table[f & !(1 << i)] & (1 << i) == 0)
                .fold(0, |s, i| s | (1 << i));
            assert_eq!(table[ex], f, "extreme set generates {name}/{f}");
            for i in 0..n {
                if ex & (1 << i) != 0 {
                    assert_ne!(table[ex & !(1 << i)], f);
                }
                for j in i + 1..n {
                    let bits = (1 << i) | (1 << j);
                    if f & bits == 0 {
                        assert!(
                            !(table[f | (1 << i)] & (1 << j) != 0
                                && table[f | (1 << j)] & (1 << i) != 0),
                            "anti-exchange counterexample {name}/{f}/{i}/{j}"
                        );
                    }
                }
            }
            writeln!(bases, "{name}\t{f}\t{ex}").unwrap();
        }
        let mut reachable = vec![vec![false; n]; n];
        for &(b, head) in &clauses {
            for (i, row) in reachable.iter_mut().enumerate() {
                for (j, value) in row.iter_mut().enumerate() {
                    *value |= b & (1 << i) != 0 && head & (1 << j) != 0;
                }
            }
        }
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    reachable[i][j] |= reachable[i][k] && reachable[k][j];
                }
            }
        }
        assert!((0..n).all(|i| !reachable[i][i]));
        writeln!(
            geometry,
            "{name}\t{n}\t{}\t{closed}\ttrue\ttrue\ttrue\ttrue\ttrue\t{closed}\ttrue",
            table.len()
        )
        .unwrap();
        for d in m
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) => Some(r),
                _ => None,
            })
            .flat_map(|r| instantiate_from_known(r, &domain))
        {
            writeln!(
                dependencies,
                "{name}\t{}\t{:?}\t{:?}\t{:?}",
                d.record.rule,
                m.origin(&d.record.rule),
                d.record.antecedents,
                d.event
            )
            .unwrap();
        }
        if h < 16 {
            let region: Vec<_> = (0..64)
                .filter(|&s| table[mask_of(&subset(&seeds, s), &domain)] & goal_bit != 0)
                .collect();
            for s in minima(&region) {
                writeln!(supports, "{name}\t{s}\t{:?}", subset(&seeds, s)).unwrap();
                let c = close(&m, subset(&seeds, s));
                audit_proofs(&m, &c);
                assert!(c.known.contains(&study.goal(0)));
                for d in &c.derivations {
                    writeln!(
                        frontier_proofs,
                        "{name}\t{s}\t{:?}\t{:?}\t{:?}",
                        d.event,
                        d.record,
                        m.origin(&d.record.rule)
                    )
                    .unwrap();
                }
            }
            let f = mask_of(&study.baseline[0].known, &domain);
            assert_ne!(reference_table[f], reference_table[f | goal_bit]);
            assert_eq!(table[f], table[f | goal_bit]);
            writeln!(mergers, "{name}\t{f}\t{}\ttrue\ttrue", f | goal_bit).unwrap();
        }
        // All admitted Seeded subsets are checked against the independent real evaluator.
        // Check all 64 for retained rules; rejected negative-body rules use two controls.
        let checked: Vec<_> = if h < 16 || h == 64 {
            (0..64).collect()
        } else {
            vec![0, 63]
        };
        for s in checked {
            let c = close(&m, subset(&seeds, s));
            assert_eq!(
                mask_of(&c.known, &domain),
                table[mask_of(&subset(&seeds, s), &domain)]
            );
        }
        // Also certify inputs containing Derived points (finite theories, not seeds).
        for s in (0..n).map(|i| 1 << i).chain(std::iter::once((1 << n) - 1)) {
            let c = close(&m, subset(&domain, s));
            audit_proofs(&m, &c);
            assert_eq!(mask_of(&c.known, &domain), table[s]);
        }
    }
    let mut inventory = String::from("point\tevent\tverb_origin\n");
    for (i, e) in domain.iter().enumerate() {
        writeln!(
            inventory,
            "{i}\t{e:?}\t{:?}",
            study.theory(0).origin(&e.verb)
        )
        .unwrap();
    }
    output("geometry.tsv", &geometry);
    output("extreme-bases.tsv", &bases);
    output("minimal-seed-supports.tsv", &supports);
    output("seed-frontier-provenance.tsv", &frontier_proofs);
    output("dependencies.tsv", &dependencies);
    output("geometry-universe.tsv", &inventory);
    output("closure-class-mergers.tsv", &mergers);
}

#[test]
fn learned_rules_generalize_bindings_and_preserve_distinct_justifications() {
    let study = Study::new();
    let mut input = experiment(&study.base, CASES[0]).input.clone();
    let source = input.clone();
    // Rename all domain constants simultaneously, retaining the elaborated types.
    fn rename(t: &mut Term) {
        if let Term::Const(name) = t {
            *name = format!("renamed{name}");
        }
    }
    fn rename_time(t: &mut TimeExpr) {
        match t {
            TimeExpr::At(name) => *name = format!("renamed{name}"),
            TimeExpr::After(t) => rename_time(t),
            _ => panic!("ground fixture"),
        }
    }
    for e in &mut input {
        for arg in &mut e.args {
            rename(arg);
        }
        rename_time(&mut e.proposition.time);
    }
    let mut target = study.goal(0);
    for arg in &mut target.args {
        rename(arg);
    }
    rename_time(&mut target.proposition.time);
    let mut m = study.theory(7); // E & beneficiary killing & causal link & purpose.
    let renamed: Vec<_> = study
        .base
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Const(c) => Some(Constant {
                name: format!("renamed{}", c.name),
                ty: c.ty.clone(),
            }),
            _ => None,
        })
        .collect();
    for c in renamed.into_iter().chain(std::iter::once(Constant {
        name: "secondKillingEvidence".into(),
        ty: Type::Entity("Fact".into()),
    })) {
        m.origins.insert(
            c.name.clone(),
            DeclarationOrigin {
                module: "RiggsRuleGeneralization".into(),
                file: None,
            },
        );
        m.declarations.push(Declaration::Const(c));
    }
    let c = close(&m, input.clone());
    audit_proofs(&m, &c);
    assert!(c.known.contains(&target));
    assert!(!c.known.contains(&study.goal(0)));
    let mut two = source.clone();
    two.extend(input);
    let concurrent = close(&m, two);
    audit_proofs(&m, &concurrent);
    assert!(concurrent.known.contains(&target));
    assert!(concurrent.known.contains(&study.goal(0)));
    assert_eq!(
        concurrent
            .derivations
            .iter()
            .filter(|d| d.event.verb == HEAD)
            .count(),
        2
    );
    // An additional matching act produces a second real proof of the SAME head.
    let mut alternate = source.clone();
    for e in &mut alternate {
        for arg in &mut e.args {
            if let Term::Const(name) = arg {
                if name == "killingEvidence" {
                    *name = "secondKillingEvidence".into();
                }
            }
        }
    }
    let mut two_acts = source;
    two_acts.extend(alternate);
    let multiple = close(&m, two_acts);
    audit_proofs(&m, &multiple);
    let proof: Vec<_> = multiple
        .derivations
        .iter()
        .filter(|d| d.event == study.goal(0))
        .collect();
    assert_eq!(proof.len(), 2);
    assert_ne!(proof[0].record, proof[1].record);
    let inverted = close(
        &m,
        experiment(&study.base, CASES[0])
            .input
            .iter()
            .rev()
            .cloned()
            .collect(),
    );
    assert_eq!(full_events(&inverted), full_events(&study.run(7, 0)));
    assert_eq!(proof_identity(&inverted), proof_identity(&study.run(7, 0)));
    output(
        "relational-generalization.tsv",
        &format!(
            "case\tnew_impediments\tprovenance\nrenamed_dossier\t1\t{:?}\nconcurrent_dossiers\t2\t{:?}\nsame_head_two_acts\t2_justifications\t{:?}\n",
            c.derivations, concurrent.derivations, proof
        ),
    );
}
