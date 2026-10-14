//! Enumerate ground residual proof branches over a typed monotonic program.
//! Hypotheses, proof witnesses, and the logical join stay outside Known.

use std::collections::{BTreeMap, BTreeSet};

use super::engine;
use crate::derive::instantiate_from_witness;
use crate::elab::Error;
use crate::typed::{
    Antecedent, Declaration, Event, Module, Polarity, PropositionKind, Rule, RuleRef, Substitution,
    Verb, VerbKind,
};

/// A declared admissible kind of seed hypothesis, including explicit polarity.
/// Missing evidence never establishes an evidentially negative fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Abducible {
    pub verb: String,
    pub polarity: Polarity,
}

/// Logical result syntax. It is not an Event, a closure, or a preference order.
/// Join summarizes alternatives; Tensor combines the requirements of a branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementFormula {
    Unit,
    Event(Event),
    Tensor(Box<Self>, Box<Self>),
    Join(Box<Self>, Box<Self>),
}

impl From<&Antecedent> for RequirementFormula {
    fn from(antecedent: &Antecedent) -> Self {
        match antecedent {
            Antecedent::Unit => Self::Unit,
            Antecedent::Event(event) => Self::Event(event.clone()),
            Antecedent::And(left, right) => Self::Tensor(
                Box::new(Self::from(left.as_ref())),
                Box::new(Self::from(right.as_ref())),
            ),
        }
    }
}

/// A query witness, not a stamped derivation or an instruction to execute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResidualProof {
    Unit,
    Evidence(Event),
    Hypothesis(Event),
    Tensor(Box<Self>, Box<Self>),
    Rule {
        rule: RuleRef,
        goal: Event,
        substitution: Substitution,
        antecedent: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualExplanation {
    pub required: Antecedent,
    pub proof: ResidualProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchLimits {
    /// Includes recursion through antecedent trees; at most 64 is supported.
    pub max_depth: usize,
    pub max_nodes: usize,
    /// Total intermediate and final branch allocations, not just final results.
    pub max_branches: usize,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_nodes: 4096,
            max_branches: 4096,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchMetrics {
    pub nodes: usize,
    pub rules_examined: usize,
    pub head_matches: usize,
    pub branches_generated: usize,
    pub rejected_not_forward_supported: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbductionFailure {
    NotAbducible,
    NotForwardSupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramResidualQuery {
    pub goal: Event,
    /// None means no explanation, not Unit and not an invented bottom Event.
    /// Join can retain Unit alongside pending branches; inspect explanations
    /// to find a branch needing no hypotheses without discarding alternatives.
    pub required: Option<RequirementFormula>,
    /// Separate proof branches are retained even when their requirements agree.
    pub explanations: Vec<ResidualExplanation>,
    pub metrics: SearchMetrics,
    /// A failed query proposes no hypotheses and is distinct from Unit.
    pub failure: Option<AbductionFailure>,
}

/// Search proof branches against immutable evidence, with Seeded abducible by
/// default in both explicit polarities. Returned branches are forward-certified.
/// Certification is for the explicit Horn clauses, not dynamic Verb.program
/// activation. Use query_active_program_residual for executable module behavior.
///
/// The supported fragment has ground goals, with every matched rule parameter
/// determined by its head. Already evidenced atoms stop expansion. Missing seeds
/// are leaves only when admitted by the seeded policy. Other atoms expand through every
/// matching rule, and conjunctions enumerate the Cartesian product of branches.
/// Cycles, unbound parameters, and exhausted limits return Err, never partial
/// explanations. This is not general abduction or an implementation of <=_L.
pub fn query_program_residual(
    program: &Module,
    evidence: &[Event],
    goal: &Event,
    limits: SearchLimits,
) -> Result<ProgramResidualQuery, Error> {
    let abducibles = engine::default_abducibles(program);
    query_program_residual_with_abducibles(program, evidence, goal, &abducibles, limits)
}

/// Optional explicit restriction of the default seeded hypotheses. This is an
/// admissibility policy, never a preference or a second mathematical order.
/// Like query_program_residual, certification is Horn-only; this does not apply
/// dynamic Verb.program guards or run close_program.
pub fn query_program_residual_with_abducibles(
    program: &Module,
    evidence: &[Event],
    goal: &Event,
    abducibles: &[Abducible],
    limits: SearchLimits,
) -> Result<ProgramResidualQuery, Error> {
    query_with_constraints(program, evidence, goal, abducibles, limits, None)
}

pub(super) fn query_with_constraints(
    program: &Module,
    evidence: &[Event],
    goal: &Event,
    abducibles: &[Abducible],
    limits: SearchLimits,
    constraints: Option<&BTreeMap<String, super::query::Bindings>>,
) -> Result<ProgramResidualQuery, Error> {
    let rules = engine::validate_search(program, evidence, goal, limits, abducibles)?;
    let mut search = Search {
        rules,
        evidence,
        abducibles: abducibles.iter().cloned().collect(),
        limits,
        metrics: SearchMetrics::default(),
        active: Vec::new(),
        constraints,
    };
    let candidates = search.event(goal, 0)?;
    certify_candidates(
        &search.rules,
        evidence,
        goal,
        &search.abducibles,
        search.metrics,
        candidates,
    )
}

pub(super) fn certify_candidates(
    rules: &[&Rule],
    evidence: &[Event],
    goal: &Event,
    allowed: &BTreeSet<Abducible>,
    mut metrics: SearchMetrics,
    candidates: Vec<ResidualExplanation>,
) -> Result<ProgramResidualQuery, Error> {
    let mut explanations = Vec::new();
    let mut rejected = 0;
    for candidate in candidates {
        // The final boundary checks admissibility and replays the actual program
        // rules using forward semantics. No hypothetical facts enter Known.
        if !admissible(&candidate.required, allowed) {
            continue;
        }
        if let Some(events) = replay_proof(&candidate.proof, rules, evidence, allowed) {
            if events.as_slice() == std::slice::from_ref(goal) {
                explanations.push(candidate);
                continue;
            }
        }
        rejected += 1;
    }
    metrics.rejected_not_forward_supported = rejected;
    let failure = explanations.is_empty().then_some(if rejected > 0 {
        AbductionFailure::NotForwardSupported
    } else {
        AbductionFailure::NotAbducible
    });
    // Join is idempotent, but proof provenance is not: consolidate only identical
    // formula trees, preserving every rule/evidence branch in explanations.
    let mut formulas = Vec::new();
    for explanation in &explanations {
        let formula = RequirementFormula::from(&explanation.required);
        if !formulas.contains(&formula) {
            formulas.push(formula);
        }
    }
    let required = formulas
        .into_iter()
        .reduce(|left, right| RequirementFormula::Join(Box::new(left), Box::new(right)));
    Ok(ProgramResidualQuery {
        goal: goal.clone(),
        required,
        explanations,
        metrics,
        failure,
    })
}

fn admissible(required: &Antecedent, allowed: &BTreeSet<Abducible>) -> bool {
    match required {
        Antecedent::Unit => true,
        Antecedent::Event(event) => {
            event.proposition.kind == PropositionKind::Seeded
                && allowed.contains(&Abducible {
                    verb: event.verb.clone(),
                    polarity: event.polarity,
                })
        }
        Antecedent::And(left, right) => admissible(left, allowed) && admissible(right, allowed),
    }
}

/// A finite certificate: each rule witness must also be a forward instance.
/// Hence its goal belongs to the least Horn closure of evidence plus hypotheses,
/// even when computing that closure would not terminate.
fn replay_proof(
    proof: &ResidualProof,
    rules: &[&Rule],
    evidence: &[Event],
    allowed: &BTreeSet<Abducible>,
) -> Option<Vec<Event>> {
    match proof {
        ResidualProof::Unit => Some(Vec::new()),
        ResidualProof::Evidence(event) => evidence.contains(event).then(|| vec![event.clone()]),
        ResidualProof::Hypothesis(event) => {
            admissible(&Antecedent::Event(event.clone()), allowed).then(|| vec![event.clone()])
        }
        ResidualProof::Tensor(left, right) => {
            let mut events = replay_proof(left, rules, evidence, allowed)?;
            events.extend(replay_proof(right, rules, evidence, allowed)?);
            Some(events)
        }
        ResidualProof::Rule {
            rule,
            goal,
            antecedent,
            ..
        } => {
            let rule = rules.iter().find(|candidate| candidate.name == rule.name)?;
            let events = replay_proof(antecedent, rules, evidence, allowed)?;
            let derived = instantiate_from_witness(rule, &events)?;
            (derived.event == *goal).then(|| vec![derived.event])
        }
    }
}

pub(super) fn validate_event(event: &Event, verbs: &BTreeMap<&str, &Verb>) -> Result<(), Error> {
    let Some(verb) = verbs.get(event.verb.as_str()) else {
        return Err(Error::new(format!(
            "unknown query/evidence verb `{}`",
            event.verb
        )));
    };
    let kind = match verb.kind {
        VerbKind::Seeded => PropositionKind::Seeded,
        VerbKind::Derived => PropositionKind::Derived,
        VerbKind::Effect => PropositionKind::Effect,
    };
    if event.args.len() != verb.args.len() || event.proposition.kind != kind {
        return Err(Error::new(format!(
            "query/evidence signature mismatch for `{}`",
            event.verb
        )));
    }
    // Entity identity types and grounding are caller preconditions. State times
    // are deliberately not validated here: they do not determine logical identity.
    Ok(())
}

struct Search<'a> {
    rules: Vec<&'a Rule>,
    evidence: &'a [Event],
    abducibles: BTreeSet<Abducible>,
    limits: SearchLimits,
    metrics: SearchMetrics,
    active: Vec<Event>,
    constraints: Option<&'a BTreeMap<String, super::query::Bindings>>,
}

impl Search<'_> {
    fn visit(&mut self, depth: usize) -> Result<(), Error> {
        engine::visit(self.limits, &mut self.metrics, depth)
    }

    fn branch(
        &mut self,
        required: Antecedent,
        proof: ResidualProof,
    ) -> Result<ResidualExplanation, Error> {
        engine::reserve_branch(self.limits, &mut self.metrics)?;
        Ok(ResidualExplanation { required, proof })
    }

    fn event(&mut self, goal: &Event, depth: usize) -> Result<Vec<ResidualExplanation>, Error> {
        self.visit(depth)?;
        match engine::classify_event(self.evidence, &self.abducibles, goal, &self.active)? {
            engine::EventDisposition::Missing => return Ok(Vec::new()),
            engine::EventDisposition::Expand => {}
            leaf => {
                engine::reserve_branch(self.limits, &mut self.metrics)?;
                return Ok(vec![engine::leaf_explanation(leaf).unwrap()]);
            }
        }
        self.active.push(goal.clone());
        let mut explanations = Vec::new();
        for index in 0..self.rules.len() {
            let rule = self.rules[index];
            let Some(expansion) =
                engine::expand_rule(rule, goal, self.constraints, &mut self.metrics, false)?
            else {
                continue;
            };
            for candidate in self.antecedent(&expansion.antecedent, depth + 1)? {
                engine::reserve_branch(self.limits, &mut self.metrics)?;
                explanations.push(engine::rule_explanation(
                    &expansion.rule,
                    &expansion.goal,
                    &expansion.substitution,
                    candidate,
                ));
            }
        }
        self.active.pop();
        Ok(explanations)
    }

    fn antecedent(
        &mut self,
        antecedent: &Antecedent,
        depth: usize,
    ) -> Result<Vec<ResidualExplanation>, Error> {
        self.visit(depth)?;
        match antecedent {
            Antecedent::Unit => Ok(vec![self.branch(Antecedent::Unit, ResidualProof::Unit)?]),
            Antecedent::Event(event) => self.event(event, depth + 1),
            Antecedent::And(left, right) => {
                let left = self.antecedent(left, depth + 1)?;
                if left.is_empty() {
                    return Ok(Vec::new());
                }
                let right = self.antecedent(right, depth + 1)?;
                let mut products = Vec::new();
                for a in &left {
                    for b in &right {
                        engine::reserve_branch(self.limits, &mut self.metrics)?;
                        products.push(engine::tensor_explanations(a, b));
                    }
                }
                Ok(products)
            }
        }
    }
}

#[cfg(test)]
mod tests;
