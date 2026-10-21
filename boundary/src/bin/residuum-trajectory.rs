//! Pilot 004-R trajectory process: runs `generative::run_trajectory` with a
//! learner reached over line-delimited JSON. Each turn it writes one
//! `{"view": …}` line to stdout and reads one `{"action": …, "usage": …}` line
//! from stdin; at the end it writes `{"report": …}` and the same report to
//! `<ledger>/trajectory-report.json`. The reserved valuations are read here
//! and only appear in the final report, after the learner has finished.
use residuum::experiments::generative::{
    Condition, DossierInput, FeedbackView, HypothesisRecord, Learner, LearnerAction, LearnerView,
    Phase, ReservedValuations, Scoring, TrajectoryBudgets, TrajectoryManifest, TrajectoryReport,
    Usage, parse_event, profile_004r, run_trajectory,
};
use residuum::experiments::worker::{Request, Response};
use residuum::load_formal_module;
use residuum::typed::{Declaration, Event, Polarity, SubstitutionValue, Term, TimeExpr};
use residuum_boundary::wire::{WireEvent, WireRequest, encode_response};
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

fn term(t: &Term) -> String {
    match t {
        Term::Const(c) | Term::Var(c) => c.clone(),
        Term::None => "none".into(),
        Term::Constructor { argument, .. } => format!("after {}", term(argument)),
        other => format!("{other:?}"),
    }
}

fn time(t: &TimeExpr) -> String {
    match t {
        TimeExpr::At(c) | TimeExpr::Var(c) => c.clone(),
        TimeExpr::After(inner) => format!("after {}", time(inner)),
    }
}

fn event(e: &Event) -> String {
    let neg = if e.polarity == Polarity::EvidentialNot {
        "~"
    } else {
        ""
    };
    let args: Vec<String> = e.args.iter().map(term).collect();
    format!(
        "{neg}{}({}) @ {}",
        e.verb,
        args.join(", "),
        time(&e.proposition.time)
    )
}

fn events(es: &[Event]) -> Value {
    json!(es.iter().map(event).collect::<Vec<_>>())
}

fn phase(p: Phase) -> &'static str {
    match p {
        Phase::Explore => "explore",
        Phase::Revise => "revise",
    }
}

fn position(p: &residuum::experiments::generative::Position) -> String {
    format!(
        "{}({}, {}, {}, {})",
        p.head, p.subject, p.conduct, p.object, p.beneficiary
    )
}

fn view_json(v: &LearnerView, draft_contract: bool, navigation: bool) -> Value {
    let mut view = json!({
        "phase": phase(v.phase),
        "dossier": v.dossier,
        "dossiers": v.dossiers.iter().map(|d| json!({
            "id": d.id,
            "seeds": events(&d.seeds),
            "closure_of_O_plus_committed": events(&d.known),
            "justifications": d.justifications.iter().map(|j| json!({
                "event": event(&j.event),
                "rule": j.rule,
                "origin": j.origin,
                "antecedents": events(&j.antecedents),
                "substitution": j.substitution.bindings.iter().map(|b| format!("{}={}", b.parameter, match &b.value {
                    SubstitutionValue::Term(t) => term(t),
                    SubstitutionValue::Time(t) => time(t),
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "hypotheses": v.hypotheses.iter().map(|h| json!({
            "id": h.id,
            "parent": h.parent,
            "text": h.text,
            "rules": h.rules,
            "admission": h.admission,
            "committed": h.committed,
            "evaluations": h.evaluations.iter().map(|e| json!({
                "dossier": e.dossier,
                "status": e.status,
                "certified": e.certified,
                "new_events": events(&e.new_events),
                "activated_base_rules": e.activated_base_rules,
                "positions_contributed": e.positions.iter().map(position).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "committed": v.committed,
        "feedback": v.feedback.iter().map(|f| match f {
            FeedbackView::Discrepancies { dossier, missed, excess, valued_total, discrepant } => json!({
                "dossier": dossier,
                "documented_positions_not_derived": missed.iter().map(position).collect::<Vec<_>>(),
                "excluded_positions_derived": excess.iter().map(|(p, owners)| json!({
                    "position": position(p), "by_hypotheses": owners })).collect::<Vec<_>>(),
                "valued_positions": valued_total,
                "discrepancies": discrepant,
            }),
            FeedbackView::Evaluated { dossier } => json!({ "dossier": dossier, "evaluated": true }),
        }).collect::<Vec<_>>(),
        "verified_evaluations_left": v.verified_evaluations_left,
        "proposals_left": v.proposals_left,
        "turns_left_in_phase": v.turns_left,
        "last_result": v.last_result,
        "navigation_actions_left": v.navigation_actions_left,
        "navigation_steps_left": v.navigation_steps_left,
        "last_navigation": v.last_navigation.as_ref().map(navigation_json),
    });
    // Navigation fields only where navigation exists (run-004 onwards), so
    // earlier manifests show exactly their original views.
    if !navigation {
        if let Some(map) = view.as_object_mut() {
            for key in [
                "navigation_actions_left",
                "navigation_steps_left",
                "last_navigation",
            ] {
                map.remove(key);
            }
        }
    }
    // Draft contract (run-006): each hypothesis is shown as a draft, committed
    // or discarded; earlier manifests keep their views unchanged.
    if draft_contract {
        for (h, rendered) in v
            .hypotheses
            .iter()
            .zip(view["hypotheses"].as_array_mut().into_iter().flatten())
        {
            let status = if h.committed {
                "committed"
            } else if h.discarded.is_some() {
                "discarded"
            } else {
                "draft"
            };
            rendered["status"] = json!(status);
            rendered["discard_reason"] = json!(h.discarded);
        }
    }
    view
}

/// Largest rendered navigation response passed to the learner; the full
/// response stays in the harness ledger's view of the driver.
const NAVIGATION_CHARS: usize = 40_000;

/// The worker's wire encoding with every event rendered as Formal text.
fn navigation_json(r: &Response) -> Value {
    fn render(v: Value) -> Value {
        match v {
            Value::Object(map) => {
                let keys: Vec<&str> = map.keys().map(String::as_str).collect();
                let mut sorted = keys.clone();
                sorted.sort_unstable();
                if sorted == ["args", "kind", "polarity", "time", "transition", "verb"] {
                    let value = Value::Object(map.clone());
                    if let Ok(w) = serde_json::from_value::<WireEvent>(value) {
                        if let Ok(e) = Event::try_from(w) {
                            return Value::String(event(&e));
                        }
                    }
                }
                Value::Object(map.into_iter().map(|(k, v)| (k, render(v))).collect())
            }
            Value::Array(items) => Value::Array(items.into_iter().map(render).collect()),
            other => other,
        }
    }
    let encoded = match encode_response(r) {
        Ok(text) => text,
        Err(e) => return json!({ "error": format!("cannot encode the response: {e:?}") }),
    };
    let rendered = render(serde_json::from_str(&encoded).unwrap_or(Value::Null));
    let text = rendered.to_string();
    if text.len() <= NAVIGATION_CHARS {
        return rendered;
    }
    let mut cut = NAVIGATION_CHARS;
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    json!({
        "truncated": true,
        "characters": text.len(),
        "head": &text[..cut],
    })
}

struct LineLearner {
    input: std::io::StdinLock<'static>,
    eof: bool,
    imports: Vec<PathBuf>,
    workdir: PathBuf,
    goals: usize,
    draft_contract: bool,
    navigation: bool,
}

/// A navigation tool call as the wire request of the existing worker, with
/// the goal written in Formal and hypothesis identifiers as theories.
fn navigation(
    learner: &mut LineLearner,
    tool: &str,
    a: &Value,
) -> Result<Option<LearnerAction>, String> {
    let s = |k: &str| a[k].as_str().map(str::to_string);
    let mut goal = || -> Result<Value, String> {
        let text = s("goal").ok_or(format!("{tool} needs goal"))?;
        learner.goals += 1;
        let e = parse_event(&learner.imports, &learner.workdir, learner.goals, &text)
            .map_err(|e| format!("goal `{text}`: {e}"))?;
        serde_json::to_value(WireEvent::from(&e)).map_err(|e| e.to_string())
    };
    let optional = |k: &str| match &a[k] {
        Value::Null => Value::Null,
        v => v.clone(),
    };
    let wire = match tool {
        "inspect_theory" => json!("inspect_theory"),
        "begin_forward" => json!({ "begin_forward": {
            "dossier": s("dossier").ok_or("begin_forward needs dossier")?,
            "theory": s("hypothesis"),
        }}),
        "begin_residual" | "begin_active_residual" => {
            let query = json!({
                "dossier": s("dossier").ok_or(format!("{tool} needs dossier"))?,
                "theory": s("hypothesis"),
                "goal": goal()?,
                "scope": optional("scope"),
                "bindings": match &a["bindings"] { Value::Null => json!([]), v => v.clone() },
                "allowed": optional("allowed"),
                "limits": optional("limits"),
            });
            let mut wrapper = serde_json::Map::new();
            wrapper.insert(tool.to_string(), query);
            Value::Object(wrapper)
        }
        "advance_session" => json!({ "advance_session": {
            "session": s("session").ok_or("advance_session needs session")?,
            "max_steps": a["max_steps"].as_u64().ok_or("advance_session needs max_steps")?,
        }}),
        "minimal_supports" => json!({ "minimal_supports": {
            "candidate": s("hypothesis").ok_or("minimal_supports needs hypothesis")?,
            "goal": goal()?,
        }}),
        "compare_partition" => json!({ "compare_partition": {
            "candidates": a["hypotheses"].clone(),
            "goal": goal()?,
        }}),
        _ => return Ok(None),
    };
    let wire: WireRequest = serde_json::from_value(wire).map_err(|e| e.to_string())?;
    let request = Request::try_from(wire).map_err(|e| format!("{e:?}"))?;
    Ok(Some(LearnerAction::Navigate { request }))
}

fn parse_action(learner: &mut LineLearner, v: &Value) -> Result<LearnerAction, String> {
    let tool = v["tool"].as_str().ok_or("action without tool")?;
    let a = &v["arguments"];
    if let Some(action) = navigation(learner, tool, a)? {
        return Ok(action);
    }
    let s = |k: &str| a[k].as_str().map(str::to_string);
    Ok(match tool {
        "propose_rule" => LearnerAction::Propose {
            text: s("text").ok_or("propose_rule needs text")?,
            parent: s("parent"),
            rationale: s("rationale").unwrap_or_default(),
        },
        "evaluate_rule" => LearnerAction::Evaluate {
            hypothesis: s("hypothesis").ok_or("evaluate_rule needs hypothesis")?,
            dossier: s("dossier").ok_or("evaluate_rule needs dossier")?,
        },
        "commit" => LearnerAction::Commit {
            hypotheses: a["hypotheses"]
                .as_array()
                .ok_or("commit needs hypotheses")?
                .iter()
                .filter_map(|h| h.as_str().map(str::to_string))
                .collect(),
            reason: s("reason"),
        },
        "discard" => LearnerAction::Discard {
            hypothesis: s("hypothesis").ok_or("discard needs hypothesis")?,
            reason: s("reason").ok_or("discard needs reason")?,
        },
        "pose_question" => LearnerAction::Question {
            question: s("question").ok_or("pose_question needs question")?,
            tool: s("tool").ok_or("pose_question needs tool")?,
        },
        "end_phase" => LearnerAction::EndPhase {
            abstention: s("abstention"),
        },
        other => return Err(format!("unknown tool `{other}`")),
    })
}

impl Learner for LineLearner {
    fn act(&mut self, view: &LearnerView) -> (LearnerAction, Usage) {
        if self.eof {
            return (
                LearnerAction::EndPhase { abstention: None },
                Usage::default(),
            );
        }
        let mut out = std::io::stdout().lock();
        writeln!(
            out,
            "{}",
            json!({ "view": view_json(view, self.draft_contract, self.navigation) })
        )
        .unwrap();
        out.flush().unwrap();
        let mut line = String::new();
        if self.input.read_line(&mut line).unwrap_or(0) == 0 {
            self.eof = true;
            return (
                LearnerAction::EndPhase { abstention: None },
                Usage::default(),
            );
        }
        let msg: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
        let u = &msg["usage"];
        let usage = Usage {
            calls: u["calls"].as_u64().unwrap_or(0),
            input_tokens: u["input_tokens"].as_u64().unwrap_or(0),
            output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
            cost_microusd: u["cost_microusd"].as_u64().unwrap_or(0),
        };
        let action = match parse_action(self, &msg["action"]) {
            Ok(a) => a,
            Err(message) => LearnerAction::Malformed { message },
        };
        (action, usage)
    }
}

fn scoring_json(s: &Scoring) -> Value {
    json!({
        "dossier": s.dossier,
        "committed": s.committed,
        "loss": s.loss,
        "valued": s.valued,
        "unvalued_derived": s.unvalued_derived,
        "missed": s.missed.iter().map(|v| v.id.clone()).collect::<Vec<_>>(),
        "excess": s.excess.iter().map(|(v, o)| json!({"valuation": v.id, "by": o})).collect::<Vec<_>>(),
        "exposures": s.exposures.iter().map(|(h, e)| json!({
            "hypothesis": h, "valuation": e.valuation, "prequential": e.prequential, "correct": e.correct
        })).collect::<Vec<_>>(),
    })
}

fn hypothesis_json(h: &HypothesisRecord, r: &TrajectoryReport) -> Value {
    json!({
        "id": h.id,
        "parent": h.parent,
        "text": h.text,
        "rationale": h.rationale,
        "origin": {"dossier": r.order[h.origin.0], "phase": phase(h.origin.1)},
        "admission": h.admission,
        "case_specific": h.case_specific,
        "certified_evaluations": h.certified_evaluations,
        "committed_ok": h.committed_ok,
        "retired_at": h.retired_at.map(|i| r.order[i].clone()),
        "discarded": h.discarded,
        "statuses": r.statuses.get(&h.id).map(|s| s.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>()),
        "evaluations": h.evaluations.iter().map(|e| json!({
            "dossier": e.dossier, "status": e.status, "certified": e.certified,
            "positions_contributed": e.positions.iter().map(position).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

fn report_json(r: &TrajectoryReport) -> Value {
    let k = &r.counters;
    json!({
        "trajectory_id": r.trajectory_id,
        "condition": format!("{:?}", r.condition),
        "order": r.order,
        "prequential_loss": r.prequential_loss(),
        "prequential": r.prequential.iter().map(scoring_json).collect::<Vec<_>>(),
        "compatibility_after_each_revision": r.compatibility.iter()
            .map(|c| c.iter().map(scoring_json).collect::<Vec<_>>()).collect::<Vec<_>>(),
        "hypotheses": r.hypotheses.iter().map(|h| hypothesis_json(h, r)).collect::<Vec<_>>(),
        "counters": {
            "turns": k.turns, "attempts": k.attempts, "admitted": k.admitted,
            "verified_evaluations": k.verified_evaluations, "refused_actions": k.refused_actions,
            "abstentions": k.abstentions, "questions": k.questions,
            "discards": k.discards,
            "navigation_actions": k.navigation_actions, "navigation_steps": k.navigation_steps,
            "commits": k.commits, "commit_rejections": k.commit_rejections,
            "calls": k.usage.calls, "input_tokens": k.usage.input_tokens,
            "output_tokens": k.usage.output_tokens, "cost_microusd": k.usage.cost_microusd,
        },
        "ledger": r.ledger.iter().map(|e| json!({
            "seq": e.seq, "dossier": e.dossier, "phase": e.phase.map(phase),
            "kind": e.kind, "detail": e.detail,
        })).collect::<Vec<_>>(),
    })
}

fn seeds(module: &residuum::typed::Module, name: &str) -> Vec<Event> {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e.input.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("experiment {name}"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| panic!("{flag} is required"))
    };
    let manifest_path = PathBuf::from(get("--manifest"));
    let ledger = PathBuf::from(get("--ledger"));
    let base = manifest_path.parent().unwrap().to_path_buf();
    let doc: Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let t = &doc["trajectory"];
    let rel = |v: &Value| -> PathBuf { base.join(v.as_str().unwrap()) };
    let dossiers = load_formal_module(rel(&t["dossiers_module"])).unwrap();
    let order = t["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| DossierInput {
            id: d["id"].as_str().unwrap().into(),
            seeds: match d["experiment"].as_str() {
                Some(name) => seeds(&dossiers, name),
                None => Vec::new(),
            },
        })
        .collect();
    let b = &t["budgets"];
    let n = |k: &str| b[k].as_u64().unwrap() as usize;
    let manifest = TrajectoryManifest {
        trajectory_id: t["trajectory_id"].as_str().unwrap().into(),
        condition: match t["condition"].as_str().unwrap() {
            "feedback" => Condition::Feedback,
            "no_feedback" => Condition::NoFeedback,
            other => panic!("condition {other}"),
        },
        imports: t["imports"].as_array().unwrap().iter().map(rel).collect(),
        workdir: rel(&t["workdir"]),
        profile: profile_004r(),
        order,
        budgets: TrajectoryBudgets {
            max_verified_evaluations: n("max_verified_evaluations"),
            max_proposals: n("max_proposals"),
            max_turns_per_phase: n("max_turns_per_phase"),
            max_clauses: n("max_clauses"),
            // Absent in run-001 to run-003: navigation disabled.
            max_navigation_actions: b["max_navigation_actions"].as_u64().unwrap_or(0) as usize,
            max_navigation_steps: b["max_navigation_steps"].as_u64().unwrap_or(0) as usize,
        },
        base_module: t["base_module"].as_str().unwrap().into(),
        // Absent before run-005: the run-001 to run-004 explore protocol.
        question_first: t["question_first"].as_bool().unwrap_or(false),
    };
    let reserved = ReservedValuations::load_tsv(Path::new(&rel(&t["reserved"]))).unwrap();
    let mut learner = LineLearner {
        input: std::io::stdin().lock(),
        eof: false,
        imports: manifest.imports.clone(),
        workdir: manifest.workdir.clone(),
        goals: 0,
        draft_contract: t["draft_contract"].as_bool().unwrap_or(false),
        navigation: manifest.budgets.max_navigation_actions > 0,
    };
    let report = run_trajectory(&manifest, &reserved, &mut learner).unwrap();
    let value = report_json(&report);
    std::fs::write(
        ledger.join("trajectory-report.json"),
        serde_json::to_string_pretty(&value).unwrap() + "\n",
    )
    .unwrap();
    let mut out = std::io::stdout().lock();
    writeln!(out, "{}", json!({ "report": value })).unwrap();
    out.flush().unwrap();
}
