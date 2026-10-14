//! Argument-premise observations over an immutable, delimited succession theory.
use super::*;

#[path = "legal_geometry_riggs_conversation_tests.rs"]
mod conversation;

#[path = "legal_geometry_riggs_rule_learning_tests.rs"]
mod rule_learning;

const CASES: [&str; 6] = [
    "historicalRiggs",
    "noBeneficiaryKilling",
    "killingWithoutAcquisitionPurpose",
    "killingByThirdParty",
    "causalLinkUnestablished",
    "survivalUnestablished",
];

fn artifact(name: &str, contents: &str) {
    let dir = root().join("docs/legal-geometry/riggs");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), contents).unwrap();
}

fn rows(name: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(root().join(format!(
        "examples/experiments/legal_geometry/riggs_{name}.tsv"
    )))
    .unwrap()
    .lines()
    .filter(|s| !s.starts_with('#'))
    .skip(1)
    .map(|s| s.split('\t').map(String::from).collect())
    .collect()
}

#[derive(Debug, PartialEq, Eq)]
enum Status {
    Compatible,
    Incompatible,
    Indeterminate,
}

#[derive(Debug, PartialEq, Eq)]
struct Assessment {
    status: Status,
    reason: &'static str,
    witnesses: Vec<Event>,
    derivations: Vec<DerivationRecord>,
    legal_adequacy: &'static str,
}

fn arg<'a>(m: &Module, e: &'a Event, field: &str) -> &'a Term {
    let v = m
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == e.verb => Some(v),
            _ => None,
        })
        .unwrap();
    &e.args[v.args.iter().position(|a| a.name == field).unwrap()]
}

fn same_target(m: &Module, goal: &Event, e: &Event) -> bool {
    e.proposition.time == goal.proposition.time
        && ["will", "beneficiary", "object"]
            .iter()
            .all(|f| arg(m, goal, f) == arg(m, e, f))
}

// Only facts joined by the reference rules are evidence of the causal premise.
fn positive_relation(m: &Module, c: &Closure, goal: &Event, verb: &str) -> Vec<Event> {
    c.known
        .iter()
        .filter(|e| e.verb == verb && e.polarity == Polarity::Positive && same_target(m, goal, e))
        .cloned()
        .collect()
}

fn negative_killing(m: &Module, goal: &Event, c: &Closure) -> Vec<Event> {
    c.known
        .iter()
        .filter(|e| {
            e.verb == "intentionalKilling"
                && e.polarity == Polarity::EvidentialNot
                && e.proposition.time == goal.proposition.time
                && arg(m, e, "killer") == arg(m, goal, "beneficiary")
                && arg(m, e, "victim") == arg(m, goal, "testator")
        })
        .cloned()
        .collect()
}

fn assess(m: &Module, goal: &Event, c: &Closure, condition: &str) -> Assessment {
    let linked = positive_relation(m, c, goal, "acquisitionLinkedToWrong");
    let purpose = positive_relation(m, c, goal, "purposefulAcquisition");
    let denied_killing = negative_killing(m, goal, c);
    // Negative purpose must refer to the very act in the established causal link.
    let denied_purpose: Vec<_> = c
        .known
        .iter()
        .filter(|e| {
            e.verb == "acquisitionPurposeRecorded"
                && e.polarity == Polarity::EvidentialNot
                && e.proposition.time == goal.proposition.time
                && arg(m, e, "will") == arg(m, goal, "will")
                && arg(m, e, "beneficiary") == arg(m, goal, "beneficiary")
                && linked.iter().any(|l| arg(m, e, "act") == arg(m, l, "act"))
        })
        .cloned()
        .collect();
    let (status, reason, witnesses) = match condition {
        "literal_route" => {
            if c.known.contains(goal) {
                (
                    Status::Compatible,
                    "literal_conditions_have_a_reference_proof",
                    vec![goal.clone()],
                )
            } else {
                (
                    Status::Indeterminate,
                    "literal_conditions_not_fully_established",
                    Vec::new(),
                )
            }
        }
        "linked_wrong" => {
            if !linked.is_empty() && !denied_killing.is_empty() {
                (
                    Status::Indeterminate,
                    "opposed_evidence_about_argument_premise",
                    [linked, denied_killing].concat(),
                )
            } else if !linked.is_empty() {
                (
                    Status::Compatible,
                    "beneficiary_act_is_linked_to_triggering_death",
                    linked,
                )
            } else if !denied_killing.is_empty() {
                (
                    Status::Incompatible,
                    "explicit_counterfactual_denies_beneficiary_killing",
                    denied_killing,
                )
            } else {
                (
                    Status::Indeterminate,
                    "acquisition_or_causal_link_not_established",
                    Vec::new(),
                )
            }
        }
        "purposeful_acquisition" => {
            if !purpose.is_empty() && (!denied_purpose.is_empty() || !denied_killing.is_empty()) {
                (
                    Status::Indeterminate,
                    "opposed_evidence_about_argument_premise",
                    [purpose, denied_purpose, denied_killing].concat(),
                )
            } else if !purpose.is_empty() {
                (
                    Status::Compatible,
                    "acquisition_purpose_and_causal_link_established",
                    purpose,
                )
            } else if !denied_purpose.is_empty() {
                (
                    Status::Incompatible,
                    "linked_act_has_explicit_negative_acquisition_purpose",
                    denied_purpose,
                )
            } else if !denied_killing.is_empty() {
                (
                    Status::Incompatible,
                    "explicit_counterfactual_denies_beneficiary_killing",
                    denied_killing,
                )
            } else {
                (
                    Status::Indeterminate,
                    "acquisition_purpose_or_causal_link_not_established",
                    Vec::new(),
                )
            }
        }
        _ => panic!("unsupported declared harness condition"),
    };
    let derivations = c
        .derivations
        .iter()
        .filter(|d| witnesses.contains(&d.event))
        .map(|d| d.record.clone())
        .collect();
    Assessment {
        status,
        reason,
        witnesses,
        derivations,
        legal_adequacy: "requires_external_deliberation",
    }
}

fn proof_identity(c: &Closure) -> Vec<String> {
    let mut p: Vec<_> = c
        .derivations
        .iter()
        .map(|d| format!("{:?}\t{:?}", d.event, d.record))
        .collect();
    p.sort();
    p
}

fn reference() -> Module {
    module("riggs_succession_reference")
}

fn audit_reference(m: &Module) {
    let reference = reference();
    for d in &reference.declarations {
        assert_eq!(
            m.declarations.iter().find(|x| x.name() == d.name()),
            Some(d)
        );
        assert_eq!(m.origin(d.name()), reference.origin(d.name()));
    }
    let general = residuum::load_formal_module(
        root().join("examples/experiments/legal_geometry/ontology_reference/legal_general.res"),
    )
    .unwrap();
    for d in &general.declarations {
        assert_eq!(
            reference.declarations.iter().find(|x| x.name() == d.name()),
            Some(d)
        );
        assert_eq!(reference.origin(d.name()), general.origin(d.name()));
    }
    assert_eq!(
        reference
            .declarations
            .iter()
            .filter(|d| matches!(d, Declaration::Rule(_)))
            .count(),
        31
    );
}

#[test]
fn literal_reference_and_typed_counterfactuals_preserve_complete_provenance() {
    let m = module("riggs_cases");
    audit_reference(&m);
    let sources = rows("case_sources");
    assert_eq!(sources.len(), CASES.len());
    for (i, name) in CASES.iter().enumerate() {
        let source = sources.iter().filter(|r| r[0] == *name).collect::<Vec<_>>();
        assert_eq!(source.len(), 1);
        assert_eq!(source[0].len(), 5);
        if i == 0 {
            assert_eq!(source[0][1], "historical_abstraction");
            assert!(source[0][3].starts_with("https://www.nycourts.gov/"));
        } else {
            assert!(source[0][1].ends_with("stipulation"));
            assert!(source[0][3].starts_with("stipulation:riggs/"));
        }
    }
    artifact(
        "case-sources.tsv",
        &std::fs::read_to_string(
            root().join("examples/experiments/legal_geometry/riggs_case_sources.tsv"),
        )
        .unwrap(),
    );
    let mut summary = String::from("case\tinput\tgoal\tgoal_present\tknown\tjustifications\n");
    let mut known = String::from("case\tevent\tverb_origin\n");
    let mut proof = String::from("case\tevent\trecord\trule_origin\tderived_at\n");
    for (i, name) in CASES.iter().enumerate() {
        let ex = experiment(&m, name);
        let c = close(&m, ex.input.clone());
        audit_proofs(&m, &c);
        assert_eq!(c.known.contains(&ex.goal), i != 5);
        assert_eq!(
            positive_relation(&m, &c, &ex.goal, "acquisitionLinkedToWrong").len(),
            usize::from(i == 0 || i == 2)
        );
        assert_eq!(
            positive_relation(&m, &c, &ex.goal, "purposefulAcquisition").len(),
            usize::from(i == 0)
        );
        assert_eq!(
            c.derivations.iter().filter(|d| d.event == ex.goal).count(),
            usize::from(i != 5)
        );
        for verb in ["patrimonialAsset", "owns", "capacity"] {
            assert_eq!(
                c.known.iter().filter(|e| e.verb == verb).count(),
                usize::from(i != 5)
            );
        }
        writeln!(
            summary,
            "{name}\t{:?}\t{:?}\t{}\t{}\t{}",
            ex.input,
            ex.goal,
            c.known.contains(&ex.goal),
            c.known.len(),
            c.derivations.len()
        )
        .unwrap();
        for e in &c.known {
            writeln!(known, "{name}\t{e:?}\t{:?}", m.origin(&e.verb).unwrap()).unwrap();
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
        assert!(
            c.derivations
                .iter()
                .all(|d| d.event.proposition.time == ex.goal.proposition.time)
        );
        assert!(
            !c.known
                .iter()
                .any(|e| e.verb == "annulled" || e.verb == "give")
        );
    }
    artifact("cases.tsv", &summary);
    artifact("known.tsv", &known);
    artifact("provenance.tsv", &proof);
    // Minimal support of the literal goal, independently checked by all 64 subsets.
    let ex = experiment(&m, CASES[0]);
    let mut supports = Vec::new();
    for mask in 0..(1usize << ex.input.len()) {
        let subset: Vec<_> = ex
            .input
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| e.clone())
            .collect();
        if close(&m, subset).known.contains(&ex.goal) {
            supports.push(mask);
        }
    }
    let minimal: Vec<_> = supports
        .iter()
        .copied()
        .filter(|s| !supports.iter().any(|t| t != s && t & s == *t))
        .collect();
    assert_eq!(minimal, vec![7], "operative disposition, death, survival");
    artifact(
        "literal-frontier.tsv",
        "support\tclaim\noperativeDisposition,deathRecorded,survivorRecorded\tunique minimal support within historical six-seed domain\n",
    );
}

#[test]
fn sourced_arguments_expose_tension_and_a_discriminating_question() {
    let m = module("riggs_cases");
    let arguments = rows("arguments");
    assert_eq!(arguments.len(), 3);
    let mut report = String::from("case\targument\trole\tassessment\tsource\tpinpoint\tclaim\n");
    use Status::*;
    let expected = [
        [Compatible, Compatible, Compatible],
        [Incompatible, Incompatible, Compatible],
        [Compatible, Incompatible, Compatible],
        [Incompatible, Incompatible, Compatible],
        [Indeterminate, Indeterminate, Compatible],
        [Indeterminate, Indeterminate, Indeterminate],
    ];
    for (i, name) in CASES.iter().enumerate() {
        let ex = experiment(&m, name);
        let c = close(&m, ex.input.clone());
        let before = c.clone();
        for (j, a) in arguments.iter().enumerate() {
            assert_eq!(a.len(), 6);
            assert!(a[4].starts_with("https://www.nycourts.gov/"));
            let result = assess(&m, &ex.goal, &c, &a[1]);
            assert_eq!(result.status, expected[i][j]);
            writeln!(
                report,
                "{name}\t{}\t{}\t{result:?}\t{}\t{}\t{}",
                a[0], a[2], a[4], a[5], a[3]
            )
            .unwrap();
        }
        assert_eq!(c, before);
    }
    let questions = rows("objections");
    assert_eq!(questions.len(), 2);
    assert!(
        questions
            .iter()
            .all(|r| r.len() == 7 && r[1] == CASES[0] && r[4] == "historical_argument")
    );
    let ex = experiment(&m, CASES[2]);
    let c = close(&m, ex.input.clone());
    assert_eq!(
        assess(&m, &ex.goal, &c, &arguments[0][1]).status,
        Compatible
    );
    assert_eq!(
        assess(&m, &ex.goal, &c, &arguments[1][1]).status,
        Incompatible
    );
    let question = format!(
        "case\tquestion\tcompared_arguments\tbasis\tstatus\n{}\tWhen the beneficiary intentionally kills the testator but explicitly lacks acquisition purpose, is the causal benefit sufficient for the objection, and what authorizes restricting testamentary efficacy?\t{},{}; {}\t{:?}\tproposed_for_external_deliberation\n",
        CASES[2], arguments[0][0], arguments[1][0], arguments[2][0], ex.input
    );
    artifact("assessments.tsv", &report);
    artifact("question.tsv", &question);
    artifact(
        "objections.tsv",
        &std::fs::read_to_string(
            root().join("examples/experiments/legal_geometry/riggs_objections.tsv"),
        )
        .unwrap(),
    );
}

#[test]
fn argument_assessment_leaves_theory_known_and_justification_multiplicity_fixed() {
    let m = module("riggs_cases");
    let before = m.clone();
    let arguments = rows("arguments");
    for name in CASES {
        let ex = experiment(&m, name);
        let c = close(&m, ex.input.clone());
        let original = c.clone();
        for a in &arguments {
            let _ = assess(&m, &ex.goal, &c, &a[1]);
        }
        assert_eq!(c, original);
        let mut reversed = ex.input.clone();
        reversed.reverse();
        let other = close(&m, reversed);
        assert_eq!(full_events(&c), full_events(&other));
        assert_eq!(proof_identity(&c), proof_identity(&other));
        assert_eq!(m, before);
    }
    // Explicit opposition remains observable; the harness suspends its premise
    // assessment rather than resolving the evidence or changing the legal route.
    let ex = experiment(&m, CASES[0]);
    let negative = experiment(&m, CASES[2]).input.last().unwrap().clone();
    let mut opposed = ex.input.clone();
    opposed.push(negative);
    let c = close(&m, opposed);
    audit_proofs(&m, &c);
    assert!(c.known.contains(&ex.goal));
    assert_eq!(
        assess(&m, &ex.goal, &c, "purposeful_acquisition").status,
        Status::Indeterminate
    );
    audit_reference(&m);
    artifact(
        "invariance.tsv",
        "check\tresult\nreference_declarations_and_origins\tunchanged\nassessment_known_and_theory\tunchanged\nreversed_input_proof_multisets\tequal_except_derivedAt\nopposed_purpose\tretained_with_indeterminate_assessment\n",
    );
}
