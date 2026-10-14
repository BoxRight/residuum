//! Explanations of fixed reference derivations, not replacements for legal rules.
//! Compatible means compatible with this observed computation, never legal approval.
use super::*;

const CASES: [&str; 4] = [
    "owedAndPerformed",
    "permittedAndPerformed",
    "claimedAndPerformed",
    "owedUnobserved",
];

fn artifact(name: &str, text: &str) {
    save(&format!("typed-explanations-{name}"), text);
    let dir = root().join("docs/legal-geometry/typed-explanations");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

fn rows(file: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(root().join(file))
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with('#'))
        .skip(1)
        .map(|line| line.split('\t').map(String::from).collect())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Compatible,
    Incompatible,
    Indeterminate,
}

#[derive(Clone)]
enum Condition {
    ProducerFamily(Vec<RuleRef>),
    RealizationSufficiency,
    UnresolvedPurpose,
}

struct Explanation {
    id: String,
    condition: Condition,
    claim: String,
    source: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Assessment {
    status: Status,
    reason: &'static str,
    witnesses: Vec<Event>,
    records: Vec<DerivationRecord>,
    unresolved: Vec<&'static str>,
    // Even a compatible mechanism does not establish normative justification.
    legal_adequacy: &'static str,
}

fn explanations(m: &Module) -> Vec<Explanation> {
    rows("examples/experiments/legal_geometry/typed_explanatory_hypotheses.tsv")
        .into_iter()
        .map(|r| {
            assert_eq!(r.len(), 5);
            let condition =
                match r[1].as_str() {
                    "producer_family" => {
                        let refs: Vec<_> =
                            r[2].split(',')
                                .map(|name| {
                                    let rule=m.declarations.iter().find_map(|d| match d {
                            Declaration::Rule(rule) if rule.name==name=>Some(rule), _=>None,
                        }).expect("rule reference resolves in unchanged reference ontology");
                                    let RuleBody::HornClause(c) = &rule.body else {
                                        panic!("reference Horn rule")
                                    };
                                    assert_eq!(
                                        c.consequent.verb,
                                        experiment(m, CASES[0]).goal.verb
                                    );
                                    assert_eq!(m.origin(name).unwrap().module, "LegalGeneral");
                                    RuleRef { name: name.into() }
                                })
                                .collect();
                        Condition::ProducerFamily(refs)
                    }
                    "realization_sufficiency" => Condition::RealizationSufficiency,
                    "unresolved_purpose" => Condition::UnresolvedPurpose,
                    _ => panic!("declared harness condition"),
                };
            Explanation {
                id: r[0].clone(),
                condition,
                claim: r[3].clone(),
                source: r[4].clone(),
            }
        })
        .collect()
}

fn arg<'a>(m: &Module, event: &'a Event, name: &str) -> &'a Term {
    let verb = m
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == event.verb => Some(v),
            _ => None,
        })
        .unwrap();
    let index = verb.args.iter().position(|a| a.name == name).unwrap();
    &event.args[index]
}

// A typed ground Act identifying the goal's person, object, recipient and Act id.
// Conduct is fixed to presentarInforme by this fixture, not invented by Rust.
fn realization(m: &Module, goal: &Event, c: &Closure) -> Vec<Event> {
    c.known
        .iter()
        .filter(|e| {
            e.verb == "Act"
                && e.polarity == Polarity::Positive
                && e.proposition.time == goal.proposition.time
                && arg(m, e, "id") == arg(m, goal, "act")
                && ["subject", "object", "beneficiary"]
                    .iter()
                    .all(|name| arg(m, e, name) == arg(m, goal, name))
        })
        .cloned()
        .collect()
}

fn assess(m: &Module, goal: &Event, c: &Closure, h: &Explanation) -> Assessment {
    let records: Vec<_> = c
        .derivations
        .iter()
        .filter(|d| d.event == *goal)
        .map(|d| d.record.clone())
        .collect();
    let acts = realization(m, goal, c);
    let (status, reason, unresolved) = match &h.condition {
        Condition::ProducerFamily(refs) => {
            if records.is_empty() {
                (
                    Status::Indeterminate,
                    "no_target_derivation_to_explain",
                    vec!["target_proof_unavailable"],
                )
            } else if records.iter().all(|r| {
                refs.iter().any(|reference| reference.name == r.rule)
                    && r.antecedents.iter().any(|e| acts.contains(e))
            }) {
                (
                    Status::Compatible,
                    "reference_proof_matches_proposed_family",
                    Vec::new(),
                )
            } else {
                (
                    Status::Incompatible,
                    "observed_target_proof_uses_a_different_family",
                    Vec::new(),
                )
            }
        }
        Condition::RealizationSufficiency => {
            if acts.is_empty() {
                (
                    Status::Indeterminate,
                    "positive_realization_premise_not_established",
                    vec!["matching_positive_Act"],
                )
            } else if c.known.contains(goal) {
                (
                    Status::Compatible,
                    "sample_is_compatible_with_sufficiency_not_a_causal_proof",
                    Vec::new(),
                )
            } else {
                // A counterexample to a claim about this completed computation.
                // Not a ~compliance event or a conclusion about historical nonperformance.
                (
                    Status::Incompatible,
                    "matching_Act_present_but_reference_target_not_derivable",
                    Vec::new(),
                )
            }
        }
        Condition::UnresolvedPurpose => (
            Status::Indeterminate,
            "purpose_claim_not_interpretable_as_existing_typed_relation",
            vec!["meaning_of_protection", "independent_reason_for_the_norm"],
        ),
    };
    Assessment {
        status,
        reason,
        witnesses: acts,
        records,
        unresolved,
        legal_adequacy: "not_established_by_reference_derivation",
    }
}

fn remaining(m: &Module, hypotheses: &[Explanation], cases: &[&str]) -> Vec<String> {
    hypotheses
        .iter()
        .filter(|h| {
            cases.iter().all(|name| {
                let ex = experiment(m, name);
                assess(m, &ex.goal, &close(m, ex.input.clone()), h).status != Status::Incompatible
            })
        })
        .map(|h| h.id.clone())
        .collect()
}

fn reference() -> Module {
    residuum::load_formal_module(
        root().join("examples/experiments/legal_geometry/ontology_reference/legal_general.res"),
    )
    .unwrap()
}

fn audit_reference(m: &Module) {
    let original = reference();
    for d in &original.declarations {
        let imported = m
            .declarations
            .iter()
            .find(|i| i.name() == d.name())
            .unwrap();
        assert_eq!(
            imported,
            d,
            "literal typed declaration unchanged: {}",
            d.name()
        );
        assert_eq!(m.origin(d.name()), original.origin(d.name()));
    }
    assert_eq!(
        m.declarations
            .iter()
            .filter(|d| matches!(d, Declaration::Rule(_)))
            .count(),
        27
    );
    assert!(
        !m.declarations
            .iter()
            .any(|d| matches!(d, Declaration::Program(_) | Declaration::Default(_)))
    );
}

#[test]
fn concrete_dossiers_use_complete_unchanged_reference_closures_and_provenance() {
    let m = module("typed_explanatory_cases");
    audit_reference(&m);
    let before = m.clone();
    let mut cases =
        String::from("case\tinput\tgoal\tknown_count\tjustification_count\tgoal_derived\torigin\n");
    let mut known = String::from("case\tevent\tverb_origin\n");
    let mut proof = String::from("case\tevent\trecord\trule_origin\tderived_at\n");
    for (name, count, goal_present) in [
        (CASES[0], 5, true),
        (CASES[1], 3, false),
        (CASES[2], 3, true),
        (CASES[3], 3, false),
    ] {
        let ex = experiment(&m, name);
        assert_eq!(ex.input_kind, ExperimentInputKind::Seeds);
        let c = close(&m, ex.input.clone());
        audit_proofs(&m, &c);
        assert_eq!(c.known.len(), count);
        assert_eq!(c.known.contains(&ex.goal), goal_present);
        writeln!(
            cases,
            "{name}\t{:?}\t{:?}\t{}\t{}\t{}\t{:?}",
            ex.input,
            ex.goal,
            c.known.len(),
            c.derivations.len(),
            goal_present,
            m.origin(name).unwrap()
        )
        .unwrap();
        for e in &c.known {
            writeln!(known, "{name}\t{e:?}\t{:?}", m.origin(&e.verb).unwrap()).unwrap();
            assert_eq!(m.origin(&e.verb).unwrap().module, "LegalGeneral");
            assert!(
                e.polarity == Polarity::Positive,
                "no missing fact becomes evidential negation"
            );
        }
        for d in &c.derivations {
            writeln!(
                proof,
                "{name}\t{:?}\t{:?}\t{:?}\t{:?}",
                d.event,
                d.record,
                m.origin(&d.record.rule).unwrap(),
                d.derived_at
            )
            .unwrap();
        }
    }
    let owed = experiment(&m, CASES[0]);
    let permitted = experiment(&m, CASES[1]);
    assert_eq!(owed.goal, permitted.goal);
    assert_eq!(
        owed.input[1], permitted.input[1],
        "same full historical event"
    );
    assert_eq!(owed.input[0].args, permitted.input[0].args);
    assert_eq!(
        owed.input[0].proposition.time,
        permitted.input[0].proposition.time
    );
    let incomplete = close(&m, experiment(&m, CASES[3]).input.clone());
    assert!(incomplete.known.iter().any(|e| e.verb == "do"));
    assert!(
        !incomplete
            .known
            .iter()
            .any(|e| e.verb == "Act" || e.verb == "violation")
    );
    assert_eq!(m, before);
    artifact("cases.tsv", &cases);
    artifact("known.tsv", &known);
    artifact("provenance.tsv", &proof);
}

#[test]
fn explanations_remain_three_way_and_only_counterexamples_exclude_candidates() {
    let m = module("typed_explanatory_cases");
    let hs = explanations(&m);
    assert_eq!(hs.len(), 4);
    let expected = [
        [
            Status::Compatible,
            Status::Compatible,
            Status::Compatible,
            Status::Indeterminate,
        ],
        [
            Status::Indeterminate,
            Status::Indeterminate,
            Status::Incompatible,
            Status::Indeterminate,
        ],
        [
            Status::Compatible,
            Status::Incompatible,
            Status::Compatible,
            Status::Indeterminate,
        ],
        [
            Status::Indeterminate,
            Status::Indeterminate,
            Status::Indeterminate,
            Status::Indeterminate,
        ],
    ];
    let mut out = String::from(
        "case\thypothesis\tstatus\treason\tunresolved\tlegal_adequacy\tclaim\thypothesis_source\twitnesses\ttarget_records\n",
    );
    for (i, name) in CASES.iter().enumerate() {
        let ex = experiment(&m, name);
        let c = close(&m, ex.input.clone());
        for (j, h) in hs.iter().enumerate() {
            let result = assess(&m, &ex.goal, &c, h);
            assert_eq!(result.status, expected[i][j]);
            assert!(result.witnesses.iter().all(|e| c.known.contains(e)));
            assert!(result.records.iter().all(|r| {
                c.derivations
                    .iter()
                    .any(|d| d.record == *r && d.event == ex.goal)
            }));
            writeln!(
                out,
                "{name}\t{}\t{:?}\t{}\t{:?}\t{}\t{}\t{}\t{:?}\t{:?}",
                h.id,
                result.status,
                result.reason,
                result.unresolved,
                result.legal_adequacy,
                h.claim,
                h.source,
                result.witnesses,
                result.records
            )
            .unwrap();
        }
    }
    let after_pair = remaining(&m, &hs, &CASES[..2]);
    assert_eq!(
        after_pair,
        ["HRelational", "HFiledDuty", "HRecipientPurpose"]
    );
    let after_question = remaining(&m, &hs, &CASES[..3]);
    assert_eq!(after_question, ["HRelational", "HRecipientPurpose"]);
    // Incomplete evidence must not eliminate any explanation.
    assert_eq!(
        remaining(&m, &hs, &[CASES[3]]),
        hs.iter().map(|h| h.id.clone()).collect::<Vec<_>>()
    );
    let question = experiment(&m, CASES[2]);
    for mask in [0, 1, 2] {
        let c = close(&m, subset(&question.input, mask));
        assert!(!c.known.contains(&question.goal));
        assert_eq!(
            assess(&m, &question.goal, &c, &hs[0]).status,
            Status::Indeterminate
        );
        assert_eq!(
            assess(&m, &question.goal, &c, &hs[1]).status,
            Status::Indeterminate
        );
    }
    artifact("assessments.tsv", &out);
    artifact(
        "candidate-stages.tsv",
        &format!(
            "stage\tcandidates\ninitial\t{}\nafter_pair\t{}\nafter_proposed_discriminant\t{}\n",
            hs.iter()
                .map(|h| h.id.clone())
                .collect::<Vec<_>>()
                .join(","),
            after_pair.join(","),
            after_question.join(",")
        ),
    );
    // The question is not converted into a legal valuation, nor approved here.
    let objections = rows("examples/experiments/legal_geometry/typed_explanatory_objections.tsv");
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0][4], "open_external_question");
    artifact(
        "objection.tsv",
        &std::fs::read_to_string(
            root().join("examples/experiments/legal_geometry/typed_explanatory_objections.tsv"),
        )
        .unwrap(),
    );
}

#[test]
fn explanatory_assessments_do_not_change_known_theory_or_proof_multiplicity() {
    let m = module("typed_explanatory_cases");
    let before = m.clone();
    let hs = explanations(&m);
    let mut out = String::from(
        "case\tknown_equal\tproof_multiset_equal\tinput_unchanged\toriginal_theory_unchanged\n",
    );
    for name in CASES {
        let ex = experiment(&m, name);
        let input = ex.input.clone();
        let c = close(&m, input.clone());
        let snapshot = c.clone();
        for h in &hs {
            let _ = assess(&m, &ex.goal, &c, h);
        }
        assert_eq!(
            c, snapshot,
            "assessments do not even reseal existing stamps"
        );
        assert_eq!(ex.input, input);
        let mut reversed = input.clone();
        reversed.reverse();
        let again = close(&m, reversed);
        audit_proofs(&m, &again);
        assert_eq!(full_events(&c), full_events(&again));
        assert_eq!(
            proofs(&c),
            proofs(&again),
            "identity and multiplicity, ignoring only derivedAt"
        );
        assert_eq!(m, before);
        audit_reference(&m);
        writeln!(out, "{name}\ttrue\ttrue\ttrue\ttrue").unwrap();
    }
    artifact("invariance.tsv", &out);
}
