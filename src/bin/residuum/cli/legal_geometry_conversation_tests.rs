//! Conversational replay inside the existing, finite monotone hypothesis language.
//! This harness never changes the norm, uses no residual API, and invents no model
//! when observations exhaust the declared version space.
use super::*;

const DOMAIN: usize = 16;

fn write_artifact(name: &str, text: &str) {
    save(&format!("conversation-{name}"), text);
    let dir = root().join("docs/legal-geometry/conversation");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

fn rows(path: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(root().join(path))
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| l.split('\t').map(String::from).collect())
        .collect()
}

fn support(text: &str) -> usize {
    if text == "empty" {
        return 0;
    }
    text.chars().fold(0, |mask, c| {
        mask | (1
            << "ORAP"
                .chars()
                .position(|x| x == c)
                .expect("declared feature"))
    })
}

fn symbol(mask: usize) -> String {
    let s: String = "ORAP"
        .chars()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, c)| c)
        .collect();
    if s.is_empty() { "empty".into() } else { s }
}

#[derive(Clone)]
struct Hypothesis {
    id: String,
    theta: Vec<usize>,
    previous_rejection: String,
}
impl Hypothesis {
    fn predict(&self, s: usize) -> bool {
        self.theta.iter().any(|&b| b & s == b)
    }
    fn family(&self) -> String {
        if self.theta.is_empty() {
            "false".into()
        } else {
            self.theta
                .iter()
                .map(|&s| symbol(s))
                .collect::<Vec<_>>()
                .join("|")
        }
    }
}

#[derive(Clone)]
struct Case {
    name: String,
    input: Vec<Event>,
    goal: Event,
    origin: DeclarationOrigin,
}
impl Case {
    fn from_module(m: &Module, name: &str) -> Self {
        let e = experiment(m, name);
        assert_eq!(e.input_kind, ExperimentInputKind::Seeds);
        Self {
            name: name.into(),
            input: e.input.clone(),
            goal: e.goal.clone(),
            origin: m.origin(name).unwrap().clone(),
        }
    }
    fn snapshot(&self) -> (BTreeSet<String>, String) {
        (
            self.input.iter().map(|e| format!("{e:?}")).collect(),
            format!("{:?}", self.goal),
        )
    }
}

#[derive(Clone)]
struct Valuation {
    id: String,
    case: Case,
    value: bool,
    source: String,
    rationale: String,
}

struct Question {
    case: Case,
    description: String,
    legal_adequacy: String,
    independent_valuation: String,
}
struct Data {
    base: Module,
    questions_module: Module,
    seeds: Vec<Event>,
    goal: Event,
    pool: Vec<Hypothesis>,
    six: Vec<Valuation>,
    responses: BTreeMap<String, Valuation>,
    questions: Vec<Question>,
}

impl Data {
    fn new() -> Self {
        let base = module("recalcitrance_cases");
        let questions_module = module("conversation_cases");
        let seeds = experiment(&base, "catalog").input.clone();
        let goal = experiment(&base, "catalog").goal.clone();
        assert_eq!(seeds.len(), 4);
        assert!(
            seeds
                .iter()
                .all(|e| e.proposition.kind == PropositionKind::Seeded)
        );
        let pool: Vec<_> = rows("docs/legal-geometry/recalcitrance/functions.tsv")
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                assert_eq!(r.len(), 4);
                assert_eq!(r[0], format!("H{i:03}"));
                let theta: Vec<_> = if r[1] == "false" {
                    Vec::new()
                } else {
                    r[1].split('|').map(support).collect()
                };
                assert!(
                    theta
                        .iter()
                        .enumerate()
                        .all(|(i, &a)| theta[i + 1..].iter().all(|&b| a & b != a && a & b != b))
                );
                let h = Hypothesis {
                    id: r[0].clone(),
                    theta,
                    previous_rejection: r[3].clone(),
                };
                assert_eq!(h.family(), r[1]);
                assert_eq!(
                    (0..DOMAIN)
                        .filter(|&s| h.predict(s))
                        .map(symbol)
                        .collect::<Vec<_>>()
                        .join(","),
                    r[2]
                );
                h
            })
            .collect();
        assert_eq!(pool.len(), 168);
        assert_eq!(
            pool.iter()
                .map(|h| (0..DOMAIN).map(|s| h.predict(s)).collect::<Vec<_>>())
                .collect::<BTreeSet<_>>()
                .len(),
            168
        );
        let six: Vec<_> = rows("examples/experiments/legal_geometry/recalcitrance_labels.tsv")
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                assert_eq!(r.len(), 6);
                assert_eq!(r[0], (i + 1).to_string());
                assert_eq!(r[4], "hypothetical_external_assumption");
                Valuation {
                    id: format!("prior{}", i + 1),
                    case: Case::from_module(&base, &r[1]),
                    value: r[2] == "1",
                    source: r[3].clone(),
                    rationale: r[5].clone(),
                }
            })
            .collect();
        assert_eq!(six.len(), 6);
        let responses = rows("examples/experiments/legal_geometry/conversation_valuations.tsv")
            .into_iter()
            .map(|r| {
                assert_eq!(r.len(), 6);
                assert_eq!(r[4], "hypothetical_external_assumption");
                assert!(["0", "1"].contains(&r[2].as_str()));
                let v = Valuation {
                    id: r[0].clone(),
                    case: Case::from_module(&questions_module, &r[1]),
                    value: r[2] == "1",
                    source: r[3].clone(),
                    rationale: r[5].clone(),
                };
                (r[0].clone(), v)
            })
            .collect();
        let questions = rows("examples/experiments/legal_geometry/conversation_questions.tsv")
            .into_iter()
            .map(|r| {
                assert_eq!(r.len(), 4);
                assert_eq!(r[2], "pending_external_deliberation");
                assert_eq!(r[3], "simulated_only");
                Question {
                    case: Case::from_module(&questions_module, &r[0]),
                    description: r[1].clone(),
                    legal_adequacy: r[2].clone(),
                    independent_valuation: r[3].clone(),
                }
            })
            .collect();
        let data = Self {
            base,
            questions_module,
            seeds,
            goal,
            pool,
            six,
            responses,
            questions,
        };
        for v in &data.six {
            assert!(data.mask(&v.case).is_ok());
        }
        let all: Vec<_> = (0..data.pool.len()).collect();
        assert_eq!(data.filter(&all, &data.six).len(), 11);
        for (h, p) in data.pool.iter().enumerate() {
            let rejection = data
                .six
                .iter()
                .position(|v| p.predict(data.mask(&v.case).unwrap()) != v.value)
                .map(|i| (i + 1).to_string())
                .unwrap_or_else(|| "survives".into());
            assert_eq!(
                rejection, p.previous_rejection,
                "same six-case provenance for {}",
                data.pool[h].id
            );
        }
        data
    }

    fn mask(&self, case: &Case) -> Result<usize, Diagnosis> {
        if format!("{:?}", case.goal) != format!("{:?}", self.goal) {
            return Err(Diagnosis::GoalOutsideScope);
        }
        let mut mask = 0;
        for e in &case.input {
            if e.proposition.kind != PropositionKind::Seeded {
                return Err(Diagnosis::NonSeededInput);
            }
            let Some(i) = self
                .seeds
                .iter()
                .position(|s| format!("{s:?}") == format!("{e:?}"))
            else {
                return Err(Diagnosis::FactsOutsideScope);
            };
            mask |= 1 << i;
        }
        Ok(mask)
    }

    fn filter(&self, ids: &[usize], history: &[Valuation]) -> Vec<usize> {
        ids.iter()
            .copied()
            .filter(|&h| {
                history.iter().all(|v| {
                    self.pool[h].predict(self.mask(&v.case).expect("validated observation"))
                        == v.value
                })
            })
            .collect()
    }

    fn initial(&self, catalog: Vec<usize>) -> State {
        let active = self.filter(&catalog, &self.six);
        State {
            catalog,
            active,
            history: self.six.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Diagnosis {
    OpposedValuations { positive: String, negative: String },
    MonotonicityIncompatible { positive: String, negative: String },
    CatalogIncomplete { full_replay_survivors: usize },
    FactsOutsideScope,
    GoalOutsideScope,
    NonSeededInput,
    UnclassifiedEmpty,
}

#[derive(Clone)]
struct State {
    catalog: Vec<usize>,
    active: Vec<usize>,
    history: Vec<Valuation>,
}
struct Update {
    state: State,
    diagnosis: Option<Diagnosis>,
    accepted: bool,
    full_replay: Option<Vec<usize>>,
}

fn submit(data: &Data, before: &State, answer: &Valuation) -> Update {
    if let Err(diagnosis) = data.mask(&answer.case) {
        return Update {
            state: before.clone(),
            diagnosis: Some(diagnosis),
            accepted: false,
            full_replay: None,
        };
    }
    let mut history = before.history.clone();
    history.push(answer.clone());
    let active = data.filter(&before.active, &history);
    assert_eq!(
        active,
        data.filter(&before.catalog, &history),
        "iterative filtering equals replay"
    );
    let all: Vec<_> = (0..data.pool.len()).collect();
    let full_replay = data.filter(&all, &history);
    let diagnosis = if !active.is_empty() {
        None
    } else {
        let exact = history.iter().filter(|v| v.value).find_map(|p| {
            history
                .iter()
                .find(|n| !n.value && p.case.snapshot() == n.case.snapshot())
                .map(|n| Diagnosis::OpposedValuations {
                    positive: p.id.clone(),
                    negative: n.id.clone(),
                })
        });
        let monotone = history.iter().filter(|v| v.value).find_map(|p| {
            history
                .iter()
                .find(|n| {
                    !n.value && {
                        let a = data.mask(&p.case).unwrap();
                        let b = data.mask(&n.case).unwrap();
                        a != b && a & b == a
                    }
                })
                .map(|n| Diagnosis::MonotonicityIncompatible {
                    positive: p.id.clone(),
                    negative: n.id.clone(),
                })
        });
        exact
            .or(monotone)
            .or_else(|| {
                (!full_replay.is_empty()).then_some(Diagnosis::CatalogIncomplete {
                    full_replay_survivors: full_replay.len(),
                })
            })
            .or(Some(Diagnosis::UnclassifiedEmpty))
    };
    Update {
        state: State {
            catalog: before.catalog.clone(),
            active,
            history,
        },
        diagnosis,
        accepted: true,
        full_replay: Some(full_replay),
    }
}

pub(super) fn kernel(signatures: Vec<Vec<bool>>) -> Vec<usize> {
    let mut ids = BTreeMap::new();
    signatures
        .into_iter()
        .map(|s| {
            let n = ids.len();
            *ids.entry(s).or_insert(n)
        })
        .collect()
}
fn observation_kernel(data: &Data, history: &[Valuation]) -> Vec<usize> {
    kernel(
        data.pool
            .iter()
            .map(|h| {
                history
                    .iter()
                    .map(|v| h.predict(data.mask(&v.case).unwrap()))
                    .collect()
            })
            .collect(),
    )
}
pub(super) fn meet(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut ids = BTreeMap::new();
    a.iter()
        .zip(b)
        .map(|(&x, &y)| {
            let n = ids.len();
            *ids.entry((x, y)).or_insert(n)
        })
        .collect()
}
pub(super) fn finer(a: &[usize], b: &[usize]) -> bool {
    (0..a.len()).all(|i| (0..a.len()).all(|j| a[i] != a[j] || b[i] == b[j]))
}

struct Score {
    question: usize,
    positive: Vec<usize>,
    negative: Vec<usize>,
    answered: bool,
    worst: usize,
}
impl Score {
    fn informative(&self) -> bool {
        !self.positive.is_empty() && !self.negative.is_empty()
    }
}
fn scores(data: &Data, state: &State) -> Vec<Score> {
    if state.active.is_empty() {
        return Vec::new();
    }
    data.questions
        .iter()
        .enumerate()
        .map(|(question, q)| {
            let mask = data.mask(&q.case).expect("typed, admitted question");
            let (positive, negative): (Vec<_>, Vec<_>) = state
                .active
                .iter()
                .copied()
                .partition(|&h| data.pool[h].predict(mask));
            let answered = state
                .history
                .iter()
                .any(|v| v.case.snapshot() == q.case.snapshot());
            let worst = positive.len().max(negative.len());
            Score {
                question,
                positive,
                negative,
                answered,
                worst,
            }
        })
        .collect()
}
fn best(data: &Data, state: &State) -> Vec<usize> {
    let s = scores(data, state);
    let threshold = s
        .iter()
        .filter(|s| s.informative() && !s.answered)
        .map(|s| s.worst)
        .min();
    s.iter()
        .filter(|s| s.informative() && !s.answered && Some(s.worst) == threshold)
        .map(|s| s.question)
        .collect()
}
fn names(data: &Data, ids: &[usize]) -> String {
    ids.iter()
        .map(|&h| data.pool[h].id.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn export_scores(out: &mut String, stage: &str, data: &Data, state: &State) {
    let winners = best(data, state);
    for s in scores(data, state) {
        let q = &data.questions[s.question];
        writeln!(
            out,
            "{stage}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\ttrue\t{}\t{}\t{}",
            q.case.name,
            symbol(data.mask(&q.case).unwrap()),
            s.positive.len(),
            s.negative.len(),
            s.worst,
            s.informative(),
            s.answered,
            winners.contains(&s.question),
            q.legal_adequacy,
            q.independent_valuation,
            q.description
        )
        .unwrap();
    }
}

fn export_space(out: &mut String, stage: &str, data: &Data, state: &State) {
    for &h in &state.active {
        writeln!(
            out,
            "{stage}\t{}\t{}",
            data.pool[h].id,
            data.pool[h].family()
        )
        .unwrap();
    }
}

fn export_shared(out: &mut String, stage: &str, data: &Data, state: &State) {
    if state.active.is_empty() {
        writeln!(out, "{stage}\tempty_space\t-\t-\t-").unwrap();
        return;
    }
    let shared: Vec<_> = data.pool[state.active[0]]
        .theta
        .iter()
        .copied()
        .filter(|s| state.active.iter().all(|&h| data.pool[h].theta.contains(s)))
        .collect();
    writeln!(
        out,
        "{stage}\t{}\t{}\t{}\t{}",
        state.active.len(),
        shared.into_iter().map(symbol).collect::<Vec<_>>().join(","),
        (0..DOMAIN)
            .filter(|&s| state.active.iter().all(|&h| data.pool[h].predict(s)))
            .map(symbol)
            .collect::<Vec<_>>()
            .join(","),
        (0..DOMAIN)
            .filter(|&s| state.active.iter().all(|&h| !data.pool[h].predict(s)))
            .map(symbol)
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
}

fn export_minima(out: &mut String, stage: &str, data: &Data, state: &State) {
    for (i, &a) in state.active.iter().enumerate() {
        for &b in &state.active[i + 1..] {
            let diff: Vec<_> = (0..DOMAIN)
                .filter(|&s| data.pool[a].predict(s) != data.pool[b].predict(s))
                .collect();
            for &s in &diff {
                if !diff.iter().any(|&t| s != t && t & s == t) {
                    writeln!(
                        out,
                        "{stage}\t{}\t{}\t{}\t{}\t{}\tpending",
                        data.pool[a].id,
                        data.pool[b].id,
                        symbol(s),
                        data.pool[a].predict(s),
                        data.pool[b].predict(s)
                    )
                    .unwrap();
                }
            }
        }
    }
}

fn export_lineage(out: &mut String, stage: &str, data: &Data, state: &State) {
    for (h, p) in data.pool.iter().enumerate() {
        if !state.catalog.contains(&h) {
            writeln!(out, "{stage}\t{}\toutside_catalog\t-\t-\t-", p.id).unwrap();
            continue;
        }
        if let Some(v) = state
            .history
            .iter()
            .find(|v| p.predict(data.mask(&v.case).unwrap()) != v.value)
        {
            writeln!(
                out,
                "{stage}\t{}\texcluded\t{}\t{}\t{}",
                p.id, v.id, v.source, v.case.name
            )
            .unwrap();
        } else {
            writeln!(out, "{stage}\t{}\tsurvives\t-\t-\t-", p.id).unwrap();
        }
    }
}

#[test]
fn seventh_external_valuation_updates_versions_meet_and_minimax_questions() {
    let data = Data::new();
    let baseline_debug = format!("{:?}", data.base);
    let initial = data.initial((0..data.pool.len()).collect());
    assert_eq!(initial.active.len(), 11);
    let seventh = &data.responses["seventh"];
    let optimal = best(&data, &initial);
    assert_eq!(
        optimal
            .iter()
            .map(|&q| data.questions[q].case.name.as_str())
            .collect::<Vec<_>>(),
        ["candidateOR", "candidateOP", "candidateRP"]
    );
    assert!(
        optimal
            .iter()
            .any(|&q| data.questions[q].case.snapshot() == seventh.case.snapshot()),
        "the stipulated question belongs to the complete minimax tie"
    );
    let update = submit(&data, &initial, seventh);
    assert!(update.accepted);
    assert!(update.diagnosis.is_none());
    assert_eq!(
        update
            .state
            .active
            .iter()
            .map(|&h| data.pool[h].id.as_str())
            .collect::<Vec<_>>(),
        ["H032", "H065", "H066", "H120", "H121"]
    );
    assert_eq!(update.state.active.len(), 5);
    assert_eq!(
        initial
            .active
            .iter()
            .filter(|h| !update.state.active.contains(h))
            .count(),
        6
    );
    let old = observation_kernel(&data, &initial.history);
    let next = observation_kernel(&data, &update.state.history);
    let one = observation_kernel(&data, std::slice::from_ref(seventh));
    assert_eq!(next, meet(&old, &one));
    assert!(finer(&next, &old));
    let restricted = kernel(
        initial
            .active
            .iter()
            .map(|&h| vec![data.pool[h].predict(data.mask(&seventh.case).unwrap())])
            .collect(),
    );
    assert_eq!(restricted.iter().collect::<BTreeSet<_>>().len(), 2);
    assert_eq!(
        best(&data, &update.state)
            .iter()
            .map(|&q| data.questions[q].case.name.as_str())
            .collect::<Vec<_>>(),
        ["candidateOR", "candidateRP"]
    );
    let mut questions = String::from(
        "stage\tcase\tinput\tpositive\tnegative\tworst\tinformative\talready_answered\tminimax\trepresentable_seeded\tlegal_adequacy\tindependent_valuation\tdescription\n",
    );
    let mut versions = String::from("stage\thypothesis\tantichain\n");
    let mut common = String::from(
        "stage\tcount\tshared_minimal_supports\tunanimous_positive\tunanimous_negative\n",
    );
    let mut minima =
        String::from("stage\th1\th2\tminimal_case\tvalue_h1\tvalue_h2\texternal_answer\n");
    let mut lineage =
        String::from("stage\thypothesis\tstatus\tfirst_rejecting_observation\tsource\tcase\n");
    let mut partitions = String::from("stage\thypothesis\tclass\n");
    for (stage, state) in [("six", &initial), ("seven", &update.state)] {
        export_scores(&mut questions, stage, &data, state);
        export_space(&mut versions, stage, &data, state);
        export_shared(&mut common, stage, &data, state);
        export_minima(&mut minima, stage, &data, state);
        export_lineage(&mut lineage, stage, &data, state);
        for (h, class) in observation_kernel(&data, &state.history).iter().enumerate() {
            writeln!(partitions, "{stage}\t{}\t{class}", data.pool[h].id).unwrap();
        }
    }
    // Counterfactual answer, not an additional real valuation or history change.
    let mut counterfactual = seventh.clone();
    counterfactual.value = !seventh.value;
    counterfactual.id = "counterfactual_seventh".into();
    let opposite = submit(&data, &initial, &counterfactual);
    assert_eq!(opposite.state.active.len(), 6);
    assert!(opposite.diagnosis.is_none());
    export_space(
        &mut versions,
        "counterfactual_negative",
        &data,
        &opposite.state,
    );
    export_scores(
        &mut questions,
        "counterfactual_negative",
        &data,
        &opposite.state,
    );
    export_shared(
        &mut common,
        "counterfactual_negative",
        &data,
        &opposite.state,
    );
    for (file, text) in [
        ("questions.tsv", questions),
        ("versions.tsv", versions),
        ("shared-conditions.tsv", common),
        ("minimal-discriminants.tsv", minima),
        ("elimination-provenance.tsv", lineage),
        ("hypothesis-partitions.tsv", partitions),
    ] {
        write_artifact(file, &text);
    }
    write_artifact(
        "turn.tsv",
        &format!(
            "observation\tcase\tvalue\tsource\tstatus\trationale\tbefore\tafter\tall_minimax_ties\tnext_minimax_ties\n{}\t{}\t{}\t{}\thypothetical_external_assumption\t{}\t11\t5\tcandidateOR,candidateOP,candidateRP\tcandidateOR,candidateRP\n",
            seventh.id, seventh.case.name, seventh.value, seventh.source, seventh.rationale
        ),
    );
    assert_eq!(baseline_debug, format!("{:?}", data.base));
}

#[test]
fn empty_version_spaces_have_distinct_diagnostics_without_inventing_functions() {
    let data = Data::new();
    let initial = data.initial((0..data.pool.len()).collect());
    let seven = submit(&data, &initial, &data.responses["seventh"]).state;
    let mut report = String::from(
        "scenario\taccepted_in_scope\tbefore\tafter\tfull_replay_survivors\tdiagnosis\tproposed_observation\tsource\tavailable_functions_after\n",
    );
    let mut witnesses = String::from(
        "scenario\tpositive_observation\tnegative_observation\tpositive_input\tnegative_input\tstrict_subset\tsame_snapshot\tpositive_source\tnegative_source\n",
    );
    let mut provenance =
        String::from("scenario\thypothesis\tstatus\tfirst_rejecting_observation\tsource\tcase\n");
    let narrow = data.initial(vec![data.pool.iter().position(|h| h.id == "H033").unwrap()]);
    assert_eq!(narrow.active.len(), 1);
    for (scenario, before) in [
        ("strict_monotonicity", &seven),
        ("same_snapshot", &seven),
        ("catalog_gap", &narrow),
        ("outside_features", &initial),
        ("outside_time", &initial),
    ] {
        let answer = &data.responses[scenario];
        let update = submit(&data, before, answer);
        match scenario {
            "strict_monotonicity" => {
                assert!(update.state.active.is_empty());
                assert_eq!(update.full_replay.as_ref().unwrap().len(), 0);
                assert_eq!(
                    update.diagnosis,
                    Some(Diagnosis::MonotonicityIncompatible {
                        positive: "strict_monotonicity".into(),
                        negative: "prior3".into()
                    })
                );
            }
            "same_snapshot" => {
                assert!(update.state.active.is_empty());
                assert_eq!(update.full_replay.as_ref().unwrap().len(), 0);
                assert_eq!(
                    update.diagnosis,
                    Some(Diagnosis::OpposedValuations {
                        positive: "seventh".into(),
                        negative: "same_snapshot".into()
                    })
                );
            }
            "catalog_gap" => {
                assert!(update.state.active.is_empty());
                assert_eq!(
                    update.diagnosis,
                    Some(Diagnosis::CatalogIncomplete {
                        full_replay_survivors: 5
                    })
                );
                assert_eq!(
                    update.state.catalog, before.catalog,
                    "diagnosis does not enlarge the catalog"
                );
            }
            "outside_features" => assert_eq!(update.diagnosis, Some(Diagnosis::FactsOutsideScope)),
            "outside_time" => assert_eq!(update.diagnosis, Some(Diagnosis::GoalOutsideScope)),
            _ => unreachable!(),
        }
        if !update.accepted {
            assert_eq!(update.state.active, before.active);
            assert_eq!(update.state.history.len(), before.history.len());
            assert!(update.full_replay.is_none());
        }
        if update.state.active.is_empty() {
            assert!(best(&data, &update.state).is_empty());
        }
        assert!(
            update
                .state
                .active
                .iter()
                .all(|h| before.active.contains(h)),
            "never invent or resurrect a hypothesis"
        );
        if let Some(
            Diagnosis::OpposedValuations { positive, negative }
            | Diagnosis::MonotonicityIncompatible { positive, negative },
        ) = &update.diagnosis
        {
            let p = update
                .state
                .history
                .iter()
                .find(|v| &v.id == positive)
                .unwrap();
            let n = update
                .state
                .history
                .iter()
                .find(|v| &v.id == negative)
                .unwrap();
            let a = data.mask(&p.case).unwrap();
            let b = data.mask(&n.case).unwrap();
            assert!(p.value && !n.value && a & b == a);
            writeln!(
                witnesses,
                "{scenario}\t{positive}\t{negative}\t{}\t{}\t{}\t{}\t{}\t{}",
                symbol(a),
                symbol(b),
                a != b,
                p.case.snapshot() == n.case.snapshot(),
                p.source,
                n.source
            )
            .unwrap();
        }
        writeln!(
            report,
            "{scenario}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}",
            update.accepted,
            before.active.len(),
            update.state.active.len(),
            update
                .full_replay
                .as_ref()
                .map(|ids| ids.len().to_string())
                .unwrap_or_else(|| "not_evaluated".into()),
            update.diagnosis,
            answer.id,
            answer.source,
            names(&data, &update.state.active)
        )
        .unwrap();
        export_lineage(&mut provenance, scenario, &data, &update.state);
    }
    write_artifact("diagnostics.tsv", &report);
    write_artifact("diagnostic-witnesses.tsv", &witnesses);
    write_artifact("diagnostic-elimination-provenance.tsv", &provenance);
}

#[test]
fn conversational_learning_keeps_norm_known_times_and_legal_provenance_intact() {
    let data = Data::new();
    let original = data.base.clone();
    let initial = data.initial((0..data.pool.len()).collect());
    let snapshots: Vec<_> = (0..DOMAIN)
        .map(|s| {
            let input = subset(&data.seeds, s);
            let c = close(&data.base, input);
            audit_proofs(&data.base, &c);
            c
        })
        .collect();
    let seven = submit(&data, &initial, &data.responses["seventh"]).state;
    let empty = submit(&data, &seven, &data.responses["strict_monotonicity"]);
    assert!(empty.state.active.is_empty());
    assert_eq!(data.base, original);
    let mut normative =
        String::from("case\tnorm\tknown_equal\tproof_multiset_equal\tderived_at_used\n");
    let mut legal_proofs = String::from("case\tevent\trecord\torigin\n");
    for (s, before) in snapshots.iter().enumerate() {
        let after = close(&data.base, subset(&data.seeds, s));
        audit_proofs(&data.base, &after);
        assert_eq!(full_events(before), full_events(&after));
        assert_eq!(proofs(before), proofs(&after));
        assert_eq!(before.known.contains(&data.goal), s & 5 == 5 || s & 6 == 6);
        writeln!(
            normative,
            "{}\t{}\ttrue\ttrue\tfalse",
            symbol(s),
            before.known.contains(&data.goal)
        )
        .unwrap();
        for d in &after.derivations {
            writeln!(
                legal_proofs,
                "{}\t{:?}\t{:?}\t{:?}",
                symbol(s),
                d.event,
                d.record,
                data.base.origin(&d.record.rule).unwrap()
            )
            .unwrap();
        }
    }
    let mut observations = String::from(
        "observation\tcase\tvalue\tsource\trationale\tdeclaring_origin\tinput\tgoal\n",
    );
    for v in seven
        .history
        .iter()
        .chain(std::iter::once(&data.responses["strict_monotonicity"]))
    {
        writeln!(
            observations,
            "{}\t{}\t{}\t{}\t{}\t{:?}\t{:?}\t{:?}",
            v.id,
            v.case.name,
            v.value,
            v.source,
            v.rationale,
            v.case.origin,
            v.case.input,
            v.case.goal
        )
        .unwrap();
    }
    // Imported experiments are not executed. The norm in this fixture still
    // has exactly its original rules; no learning rule/Program has been added.
    assert_eq!(
        data.questions_module
            .declarations
            .iter()
            .filter(|d| matches!(d, Declaration::Rule(_)))
            .count(),
        27
    );
    assert_eq!(data.base, original);
    write_artifact("normative-invariance.tsv", &normative);
    write_artifact("legal-provenance.tsv", &legal_proofs);
    write_artifact("observation-provenance.tsv", &observations);
}
