//! External forward-only mode. Uses existing experiment inputs without running
//! their query algorithms or extending the Formal grammar.

use super::{Clock, render_event, require_acyclic};
use crate::elab::Error;
use crate::typed::{Closure, Declaration, Event, ExperimentInputKind, Module, RuleBody, VerbKind};
use crate::{ClosureDelta, IncrementalClosure, close};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalExperimentResult {
    pub seeds: Vec<Event>,
    pub additions: Vec<Event>,
    pub goal: Event,
    pub base: Closure,
    pub delta: ClosureDelta,
    pub incremental: Closure,
    pub recomputed: Closure,
    pub logical_equal: bool,
    pub provenance_equal: bool,
}

pub fn run_incremental_experiment(module: &Module) -> Result<IncrementalExperimentResult, Error> {
    let mut rules = Vec::new();
    let mut inputs = Vec::new();
    for declaration in &module.declarations {
        match declaration {
            Declaration::Default(_) => {
                return Err(Error::new("incremental experiment excludes defaults"));
            }
            Declaration::Verb(verb) if verb.kind == VerbKind::Effect => {
                return Err(Error::new("incremental experiment excludes effects"));
            }
            Declaration::Rule(rule) => {
                if !matches!(rule.body, RuleBody::HornClause(_)) {
                    return Err(Error::new("incremental experiment requires Horn rules"));
                }
                rules.push(rule.clone());
            }
            Declaration::Experiment(experiment) => inputs.push(experiment),
            _ => {}
        }
    }
    if inputs.len() != 2
        || inputs
            .iter()
            .any(|e| e.input_kind != ExperimentInputKind::Seeds)
    {
        return Err(Error::new(
            "incremental experiment requires exactly two seed blocks: base and delta",
        ));
    }
    let base = inputs
        .iter()
        .find(|e| e.name == "base")
        .ok_or_else(|| Error::new("missing base seed block"))?;
    let additions = inputs
        .iter()
        .find(|e| e.name == "delta")
        .ok_or_else(|| Error::new("missing delta seed block"))?;
    if base.goal != additions.goal {
        return Err(Error::new("base and delta must observe the same goal"));
    }
    // Conservative finite termination guard, checked before either closure.
    require_acyclic(&rules)?;
    let mut clock = Clock::default();
    let mut state = IncrementalClosure::new(base.input.clone(), &rules, &mut clock);
    let before = state.closure().clone();
    let delta = state.update(additions.input.clone(), &mut clock);
    let mut combined = base.input.clone();
    combined.extend(additions.input.clone());
    let recomputed = close(combined, &rules, &mut Clock::default());
    let incremental = state.closure().clone();
    let logical_equal = keys(&incremental) == keys(&recomputed);
    let provenance_equal = same_justifications(&incremental, &recomputed);
    if !logical_equal || !provenance_equal {
        return Err(Error::new(
            "incremental closure disagrees with full Horn recomputation",
        ));
    }
    Ok(IncrementalExperimentResult {
        seeds: base.input.clone(),
        additions: additions.input.clone(),
        goal: base.goal.clone(),
        base: before,
        delta,
        incremental,
        recomputed,
        logical_equal,
        provenance_equal,
    })
}

fn keys(closure: &Closure) -> BTreeSet<Event> {
    closure.known.iter().cloned().collect()
}

// A derivation clock and discovery order do not identify a justification.
fn same_justifications(a: &Closure, b: &Closure) -> bool {
    fn included(a: &Closure, b: &Closure) -> bool {
        a.derivations.iter().all(|d| {
            b.derivations
                .iter()
                .any(|other| d.event == other.event && d.record == other.record)
        })
    }
    included(a, b) && included(b, a)
}

pub fn render_incremental(result: &IncrementalExperimentResult) -> String {
    fn events(events: &[Event]) -> String {
        events
            .iter()
            .map(render_event)
            .collect::<Vec<_>>()
            .join(", ")
    }
    format!(
        "forward incremental\n  S: {{{}}}\n  delta S: {{{}}}\n  close(S): {{{}}}\n  Dclose: {{{}}}\n  incremental: {{{}}}\n  recomputed: {{{}}}\n  logical equality: {}\n  justification equality: {}\n  goal: {}\n  Horn before: {}\n  Horn after: {}\n  new justifications: {}\n  rounds: {}, rules examined: {}, delta pivots: {}, full matches: {}\n",
        events(&result.seeds),
        events(&result.additions),
        events(&result.base.known),
        events(&result.delta.known),
        events(&result.incremental.known),
        events(&result.recomputed.known),
        result.logical_equal,
        result.provenance_equal,
        render_event(&result.goal),
        result.base.known.contains(&result.goal),
        result.incremental.known.contains(&result.goal),
        result.delta.derivations.len(),
        result.delta.stats.rounds,
        result.delta.stats.rules_examined,
        result.delta.stats.delta_pivot_matches,
        result.delta.stats.full_matches,
    )
}

#[cfg(test)]
mod tests;
