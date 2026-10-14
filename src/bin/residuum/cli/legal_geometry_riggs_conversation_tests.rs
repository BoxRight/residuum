//! Conversation over the existing three Riggs arguments, never over legal rules.
use super::super::conversation::{finer, kernel, meet};
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Answer {
    Sustain,
    Reject,
    Pending,
}
impl Answer {
    fn parse(s: &str) -> Self {
        match s {
            "sustain" => Self::Sustain,
            "reject" => Self::Reject,
            "pending" => Self::Pending,
            _ => panic!("declared conversational answer"),
        }
    }
    fn bits(self) -> [bool; 2] {
        match self {
            Self::Sustain => [true, false],
            Self::Reject => [false, true],
            Self::Pending => [true, true],
        }
    }
    fn admits(self, valuation: Self) -> bool {
        valuation == Self::Pending || self == Self::Pending || self == valuation
    }
}

struct Hypothesis {
    id: String,
    condition: String,
    role: String,
    claim: String,
    source: String,
    pinpoint: String,
    compatible: Answer,
    incompatible: Answer,
    commitment_source: String,
}

#[derive(Clone, Debug)]
struct Valuation {
    id: String,
    case: String,
    answer: Answer,
    source: String,
    rationale: String,
}

struct Data {
    m: Module,
    hypotheses: Vec<Hypothesis>,
    closures: BTreeMap<String, Closure>,
    valuations: BTreeMap<String, Valuation>,
    questions: BTreeMap<String, Vec<String>>,
    case_sources: BTreeMap<String, Vec<String>>,
    objections: Vec<Vec<String>>,
}
impl Data {
    fn new() -> Self {
        let m = module("riggs_cases");
        audit_reference(&m);
        let arguments = rows("arguments");
        let contracts = rows("conversation_contract");
        assert_eq!(arguments.len(), 3);
        assert_eq!(contracts.len(), arguments.len());
        let hypotheses = arguments
            .iter()
            .map(|a| {
                let c = contracts
                    .iter()
                    .filter(|c| c[0] == a[0])
                    .collect::<Vec<_>>();
                assert_eq!(c.len(), 1);
                let c = c[0];
                assert_eq!(c.len(), 6);
                assert_eq!(
                    c[3], "pending",
                    "indeterminate premises must admit both answers"
                );
                assert_eq!(c[4], "hypothetical_reading_commitment");
                assert!(c[5].starts_with("stipulation:riggs-conversation/"));
                Hypothesis {
                    id: a[0].clone(),
                    condition: a[1].clone(),
                    role: a[2].clone(),
                    claim: a[3].clone(),
                    source: a[4].clone(),
                    pinpoint: a[5].clone(),
                    compatible: Answer::parse(&c[1]),
                    incompatible: Answer::parse(&c[2]),
                    commitment_source: c[5].clone(),
                }
            })
            .collect();
        let closures = CASES
            .iter()
            .map(|name| {
                let ex = experiment(&m, name);
                assert_eq!(ex.input_kind, ExperimentInputKind::Seeds);
                let c = close(&m, ex.input.clone());
                audit_proofs(&m, &c);
                ((*name).into(), c)
            })
            .collect();
        let valuations = rows("conversation_valuations")
            .into_iter()
            .map(|v| {
                assert_eq!(v.len(), 6);
                assert!(CASES.contains(&v[1].as_str()));
                assert_eq!(v[4], "hypothetical_external_assumption");
                assert!(v[3].starts_with("stipulation:riggs-conversation/"));
                (
                    v[0].clone(),
                    Valuation {
                        id: v[0].clone(),
                        case: v[1].clone(),
                        answer: Answer::parse(&v[2]),
                        source: v[3].clone(),
                        rationale: v[5].clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(valuations.len(), 7);
        let questions = rows("conversation_questions")
            .into_iter()
            .map(|q| {
                assert_eq!(q.len(), 4);
                assert!(CASES.contains(&q[0].as_str()));
                assert_eq!(q[2], "pending_external_deliberation");
                assert_eq!(q[3], "hypothetical_only");
                (q[0].clone(), q)
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(questions.len(), CASES.len());
        assert_eq!(
            questions
                .values()
                .map(|q| q[1].clone())
                .collect::<BTreeSet<_>>()
                .len(),
            1,
            "all answers concern the same legal question, in different dossiers"
        );
        let case_sources = rows("case_sources")
            .into_iter()
            .map(|r| (r[0].clone(), r))
            .collect();
        let objections = rows("objections");
        Self {
            m,
            hypotheses,
            closures,
            valuations,
            questions,
            case_sources,
            objections,
        }
    }
    fn assessment(&self, h: usize, case: &str) -> Assessment {
        assess(
            &self.m,
            &experiment(&self.m, case).goal,
            &self.closures[case],
            &self.hypotheses[h].condition,
        )
    }
    fn prediction(&self, h: usize, case: &str) -> Answer {
        match self.assessment(h, case).status {
            Status::Compatible => self.hypotheses[h].compatible,
            Status::Incompatible => self.hypotheses[h].incompatible,
            Status::Indeterminate => Answer::Pending,
        }
    }
    fn replay(&self, history: &[Valuation]) -> Vec<usize> {
        (0..self.hypotheses.len())
            .filter(|&h| {
                history
                    .iter()
                    .all(|v| self.prediction(h, &v.case).admits(v.answer))
            })
            .collect()
    }
    fn partition(&self, history: &[Valuation]) -> Vec<usize> {
        kernel(
            (0..self.hypotheses.len())
                .map(|h| {
                    history
                        .iter()
                        .flat_map(|v| self.prediction(h, &v.case).bits())
                        .collect()
                })
                .collect(),
        )
    }
    fn feedback_partition(&self, history: &[Valuation]) -> Vec<usize> {
        kernel(
            (0..self.hypotheses.len())
                .map(|h| {
                    history
                        .iter()
                        .map(|v| self.prediction(h, &v.case).admits(v.answer))
                        .collect()
                })
                .collect(),
        )
    }
    fn names(&self, ids: &[usize]) -> Vec<String> {
        ids.iter().map(|&h| self.hypotheses[h].id.clone()).collect()
    }
}

#[derive(Clone)]
struct State {
    active: Vec<usize>,
    history: Vec<Valuation>,
}
impl State {
    fn initial(data: &Data) -> Self {
        Self {
            active: (0..data.hypotheses.len()).collect(),
            history: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct Decision {
    hypothesis: String,
    premise: Assessment,
    prediction: Answer,
    external: Answer,
    retained: bool,
    reason: &'static str,
    valuation: String,
    valuation_source: String,
    rationale: String,
    argument_source: String,
    pinpoint: String,
    commitment_source: String,
}

fn submit(data: &Data, state: &State, v: &Valuation) -> (State, Vec<Decision>) {
    assert!(data.questions.contains_key(&v.case));
    let mut history = state.history.clone();
    history.push(v.clone());
    let mut active = Vec::new();
    let mut decisions = Vec::new();
    for &h in &state.active {
        let premise = data.assessment(h, &v.case);
        let prediction = data.prediction(h, &v.case);
        let retained = prediction.admits(v.answer);
        let reason = if v.answer == Answer::Pending {
            "external_judgment_suspended"
        } else if premise.status == Status::Indeterminate {
            "premise_indeterminate_no_refutation"
        } else if prediction == Answer::Pending {
            "interpretive_commitment_does_not_determine_an_answer"
        } else if retained {
            "declared_answer_agrees_with_external_valuation"
        } else {
            "declared_answer_disagrees_with_external_valuation"
        };
        if retained {
            active.push(h);
        }
        let hypothesis = &data.hypotheses[h];
        decisions.push(Decision {
            hypothesis: hypothesis.id.clone(),
            premise,
            prediction,
            external: v.answer,
            retained,
            reason,
            valuation: v.id.clone(),
            valuation_source: v.source.clone(),
            rationale: v.rationale.clone(),
            argument_source: hypothesis.source.clone(),
            pinpoint: hypothesis.pinpoint.clone(),
            commitment_source: hypothesis.commitment_source.clone(),
        });
    }
    assert_eq!(
        active,
        data.replay(&history),
        "iterative filtering equals replay of the fixed catalog"
    );
    (State { active, history }, decisions)
}

#[derive(Debug)]
struct Score {
    case: String,
    sustain: Vec<usize>,
    reject: Vec<usize>,
    pending: Vec<usize>,
    worst: usize,
    resolved: bool,
    informative: bool,
}
fn scores(data: &Data, state: &State) -> Vec<Score> {
    data.questions
        .keys()
        .map(|name| {
            let possible = |answer| {
                state
                    .active
                    .iter()
                    .copied()
                    .filter(|&h| data.prediction(h, name).admits(answer))
                    .collect::<Vec<_>>()
            };
            let sustain = possible(Answer::Sustain);
            let reject = possible(Answer::Reject);
            let pending = state
                .active
                .iter()
                .copied()
                .filter(|&h| data.prediction(h, name) == Answer::Pending)
                .collect::<Vec<_>>();
            let worst = sustain.len().max(reject.len());
            let resolved = state
                .history
                .iter()
                .any(|v| v.case == *name && v.answer != Answer::Pending);
            let informative = !state.active.is_empty() && worst < state.active.len();
            Score {
                case: name.clone(),
                sustain,
                reject,
                pending,
                worst,
                resolved,
                informative,
            }
        })
        .collect()
}
fn best(data: &Data, state: &State) -> Vec<String> {
    let scores = scores(data, state);
    let threshold = scores
        .iter()
        .filter(|s| s.informative && !s.resolved)
        .map(|s| s.worst)
        .min();
    scores
        .iter()
        .filter(|s| s.informative && !s.resolved && Some(s.worst) == threshold)
        .map(|s| s.case.clone())
        .collect()
}

fn artifact(name: &str, contents: &str) {
    let dir = root().join("docs/legal-geometry/riggs-conversation");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), contents).unwrap();
}
fn export_stage(
    data: &Data,
    stage: &str,
    state: &State,
    versions: &mut String,
    partitions: &mut String,
    questions: &mut String,
) {
    writeln!(
        versions,
        "{stage}\t{:?}\t{:?}",
        data.names(&state.active),
        state.history
    )
    .unwrap();
    writeln!(
        partitions,
        "{stage}\t{:?}\t{:?}\t{:?}",
        data.partition(&state.history),
        data.feedback_partition(&state.history),
        data.names(&(0..data.hypotheses.len()).collect::<Vec<_>>())
    )
    .unwrap();
    let winners = best(data, state);
    for score in scores(data, state) {
        writeln!(
            questions,
            "{stage}\t{}\t{:?}\t{:?}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}",
            score.case,
            data.names(&score.sustain),
            data.names(&score.reject),
            data.names(&score.pending),
            score.worst,
            score.resolved,
            score.informative,
            winners.contains(&score.case),
            data.questions[&score.case][1],
            data.questions[&score.case][2]
        )
        .unwrap();
    }
}

#[test]
fn external_answers_update_argument_versions_without_conflating_premises() {
    let data = Data::new();
    let initial = State::initial(&data);
    let mut report = String::from("stage\tdecision\n");
    let mut predictions = String::from(
        "case\targument\trole\tclaim\tpremise\tdeclared_answer\targument_source\tpinpoint\tcommitment_source\n",
    );
    for name in CASES {
        for (h, arg) in data.hypotheses.iter().enumerate() {
            writeln!(
                predictions,
                "{name}\t{}\t{}\t{}\t{:?}\t{:?}\t{}\t{}\t{}",
                arg.id,
                arg.role,
                arg.claim,
                data.assessment(h, name),
                data.prediction(h, name),
                arg.source,
                arg.pinpoint,
                arg.commitment_source
            )
            .unwrap();
        }
    }
    let focal = "killingWithoutAcquisitionPurpose";
    assert_eq!(data.assessment(0, focal).status, Status::Compatible);
    assert_eq!(data.assessment(1, focal).status, Status::Incompatible);
    assert_eq!(data.assessment(2, focal).status, Status::Compatible);
    assert_eq!(
        initial.active,
        vec![0, 1, 2],
        "premise incompatibility alone never filters the catalog"
    );
    for (name, expected) in [
        ("firstSustain", vec![0]),
        ("firstReject", vec![1, 2]),
        ("firstPending", vec![0, 1, 2]),
    ] {
        let (next, decisions) = submit(&data, &initial, &data.valuations[name]);
        assert_eq!(next.active, expected);
        for d in &decisions {
            assert_eq!(
                d.retained,
                expected.contains(
                    &data
                        .hypotheses
                        .iter()
                        .position(|h| h.id == d.hypothesis)
                        .unwrap()
                )
            );
            assert_eq!(d.valuation, name);
            assert!(d.valuation_source.starts_with("stipulation:"));
            assert!(d.argument_source.starts_with("https://www.nycourts.gov/"));
            assert!(!d.pinpoint.is_empty() && !d.rationale.is_empty());
            assert!(d.commitment_source.starts_with("stipulation:"));
            assert_eq!(d.external, data.valuations[name].answer);
            assert!(d.prediction.admits(d.external) == d.retained);
            assert!(!d.reason.is_empty());
            writeln!(report, "{name}\t{d:?}").unwrap();
        }
        if name == "firstReject" {
            assert_eq!(decisions[1].premise.status, Status::Incompatible);
            assert!(
                decisions[1].retained,
                "a hypothesis can correctly reject a case with a refuted premise"
            );
            assert_eq!(decisions[0].premise.status, Status::Compatible);
            assert!(
                !decisions[0].retained,
                "supported facts do not compel external endorsement"
            );
        }
    }
    for (id, expected) in [
        ("unknownSustain", vec![0, 1]),
        ("unknownReject", vec![0, 1, 2]),
    ] {
        let (next, decisions) = submit(&data, &initial, &data.valuations[id]);
        assert_eq!(next.active, expected);
        for d in &decisions[..2] {
            assert_eq!(d.premise.status, Status::Indeterminate);
            assert!(d.retained);
            assert_eq!(d.reason, "premise_indeterminate_no_refutation");
        }
        for d in decisions {
            writeln!(report, "{id}\t{d:?}").unwrap();
        }
    }
    artifact("predictions.tsv", &predictions);
    artifact("decisions.tsv", &report);
}

#[test]
fn observation_partitions_meet_and_minimax_questions_respect_pending_answers() {
    let data = Data::new();
    let initial = State::initial(&data);
    let mut versions = String::from("stage\tactive\thistory\n");
    let mut partitions =
        String::from("stage\tprediction_partition\tfeedback_partition\tcatalog_order\n");
    let mut questions = String::from(
        "stage\tcase\tsustain_survivors\treject_survivors\tpending_hypotheses\tworst\tresolved\tinformative\tminimax\tquestion\tlegal_adequacy\n",
    );
    export_stage(
        &data,
        "initial",
        &initial,
        &mut versions,
        &mut partitions,
        &mut questions,
    );
    assert_eq!(
        best(&data, &initial),
        vec!["historicalRiggs", "killingWithoutAcquisitionPurpose"]
    );
    assert_eq!(data.partition(&[]), vec![0, 0, 0]);
    for id in ["firstSustain", "firstReject", "firstPending"] {
        let (state, _) = submit(&data, &initial, &data.valuations[id]);
        let one = data.partition(&state.history);
        assert_eq!(one, vec![0, 1, 1]);
        assert_eq!(one, meet(&data.partition(&initial.history), &one));
        assert!(finer(&one, &data.partition(&initial.history)));
        let feedback = data.feedback_partition(&state.history);
        assert_eq!(
            feedback,
            if id == "firstPending" {
                vec![0, 0, 0]
            } else {
                vec![0, 1, 1]
            }
        );
        assert_eq!(
            feedback,
            meet(&data.feedback_partition(&initial.history), &feedback)
        );
        export_stage(
            &data,
            id,
            &state,
            &mut versions,
            &mut partitions,
            &mut questions,
        );
        match id {
            "firstSustain" => assert!(best(&data, &state).is_empty()),
            "firstReject" => {
                assert_eq!(best(&data, &state), vec!["historicalRiggs"]);
                let unknown = scores(&data, &state)
                    .into_iter()
                    .find(|s| s.case == "causalLinkUnestablished")
                    .unwrap();
                assert_eq!(unknown.pending, vec![1]);
                assert_eq!(unknown.worst, 2);
                assert!(
                    !unknown.informative,
                    "pending predictions cannot be counted as a disjoint reject group"
                );
                for (follow, survivors) in [("followSustain", vec![1]), ("followReject", vec![2])] {
                    let v = &data.valuations[follow];
                    let (next, decisions) = submit(&data, &state, v);
                    assert_eq!(next.active, survivors);
                    assert_eq!(data.partition(&next.history), vec![0, 1, 2]);
                    assert_eq!(
                        data.partition(&next.history),
                        meet(&one, &data.partition(&[v.clone()]))
                    );
                    assert_eq!(data.feedback_partition(&next.history), vec![0, 1, 2]);
                    assert_eq!(
                        data.feedback_partition(&next.history),
                        meet(&feedback, &data.feedback_partition(&[v.clone()]))
                    );
                    assert!(best(&data, &next).is_empty());
                    export_stage(
                        &data,
                        follow,
                        &next,
                        &mut versions,
                        &mut partitions,
                        &mut questions,
                    );
                    artifact(
                        &format!("{follow}-decisions.tsv"),
                        &format!(
                            "decision\n{}\n",
                            decisions
                                .iter()
                                .map(|d| format!("{d:?}"))
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                    );
                }
            }
            "firstPending" => {
                assert_eq!(state.active, initial.active);
                assert_eq!(best(&data, &state), best(&data, &initial));
            }
            _ => unreachable!(),
        }
    }
    for id in ["unknownSustain", "unknownReject"] {
        let (state, _) = submit(&data, &initial, &data.valuations[id]);
        let p = data.partition(&state.history);
        let feedback = data.feedback_partition(&state.history);
        assert_eq!(p, vec![0, 0, 1]);
        assert_eq!(p, meet(&data.partition(&[]), &p));
        assert_eq!(
            feedback,
            if id == "unknownSustain" {
                vec![0, 0, 1]
            } else {
                vec![0, 0, 0]
            }
        );
        assert_eq!(feedback, meet(&data.feedback_partition(&[]), &feedback));
        assert_eq!(
            best(&data, &state),
            if id == "unknownSustain" {
                vec!["killingWithoutAcquisitionPurpose"]
            } else {
                vec!["historicalRiggs", "killingWithoutAcquisitionPurpose"]
            }
        );
        export_stage(
            &data,
            id,
            &state,
            &mut versions,
            &mut partitions,
            &mut questions,
        );
    }
    artifact("versions.tsv", &versions);
    artifact("partitions.tsv", &partitions);
    artifact("questions.tsv", &questions);
}

#[test]
fn conversations_preserve_complete_legal_closures_and_sourced_objections() {
    let data = Data::new();
    let original_module = data.m.clone();
    let initial = State::initial(&data);
    let mut invariance =
        String::from("valuation\tcase\tfull_events\tordered_proof_multiset\torigins\n");
    let mut context = String::from("case\texperiment\torigin\tinput_source\n");
    let mut proof = String::from("case\tevent\trecord\trule_origin\tderived_at\n");
    for name in CASES {
        writeln!(
            context,
            "{name}\t{:?}\t{:?}\t{:?}",
            experiment(&data.m, name),
            data.m.origin(name).unwrap(),
            data.case_sources[name]
        )
        .unwrap();
        for d in &data.closures[name].derivations {
            writeln!(
                proof,
                "{name}\t{:?}\t{:?}\t{:?}\t{:?}",
                d.event,
                d.record,
                data.m.origin(&d.record.rule).unwrap(),
                d.derived_at
            )
            .unwrap();
        }
    }
    for v in data.valuations.values() {
        let (_, _) = submit(&data, &initial, v);
        for name in CASES {
            let ex = experiment(&data.m, name);
            let before = &data.closures[name];
            let after = close(&data.m, ex.input.clone());
            assert_eq!(full_events(before), full_events(&after));
            assert_eq!(proof_identity(before), proof_identity(&after));
            audit_proofs(&data.m, &after);
            writeln!(
                invariance,
                "{}\t{name}\tequal\tequal_except_derivedAt\tequal",
                v.id
            )
            .unwrap();
        }
        assert_eq!(data.m, original_module);
    }
    assert_eq!(data.objections, rows("objections"));
    assert_eq!(data.closures.len(), CASES.len());
    audit_reference(&data.m);
    for name in [
        "arguments",
        "objections",
        "case_sources",
        "conversation_contract",
        "conversation_valuations",
    ] {
        artifact(
            &format!("source-{name}.tsv"),
            &std::fs::read_to_string(root().join(format!(
                "examples/experiments/legal_geometry/riggs_{name}.tsv"
            )))
            .unwrap(),
        );
    }
    artifact("legal-contexts.tsv", &context);
    artifact("legal-provenance.tsv", &proof);
    artifact("legal-invariance.tsv", &invariance);
}
