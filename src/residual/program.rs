//! Enumerate ground residual proof branches over a typed monotonic program.
//! Hypotheses, proof witnesses, and the logical join stay outside Known.

use std::collections::{BTreeMap, BTreeSet};

use super::query::match_goal;
use crate::derive::{instantiate_from_witness, substitute_antecedent, substitution_record};
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
    pub required: Option<RequirementFormula>,
    /// Separate proof branches are retained even when their requirements agree.
    pub explanations: Vec<ResidualExplanation>,
    pub metrics: SearchMetrics,
    /// A failed query proposes no hypotheses and is distinct from Unit.
    pub failure: Option<AbductionFailure>,
}

/// Search proof branches against immutable evidence, with Seeded abducible by
/// default in both explicit polarities. Returned branches are forward-certified.
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
    let abducibles: Vec<_> = program
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Verb(verb) if verb.is_abducible() => Some(verb),
            _ => None,
        })
        .flat_map(|verb| {
            [Polarity::Positive, Polarity::EvidentialNot].map(|polarity| Abducible {
                verb: verb.name.clone(),
                polarity,
            })
        })
        .collect();
    query_program_residual_with_abducibles(program, evidence, goal, &abducibles, limits)
}

/// Optional explicit restriction of the default seeded hypotheses. This is an
/// admissibility policy, never a preference or a second mathematical order.
pub fn query_program_residual_with_abducibles(
    program: &Module,
    evidence: &[Event],
    goal: &Event,
    abducibles: &[Abducible],
    limits: SearchLimits,
) -> Result<ProgramResidualQuery, Error> {
    if limits.max_depth == 0
        || limits.max_depth > 64
        || limits.max_nodes == 0
        || limits.max_branches == 0
    {
        return Err(Error::new(
            "invalid residual search limits (depth must be 1..=64)",
        ));
    }
    let mut rules = Vec::new();
    let mut verbs = BTreeMap::new();
    for declaration in &program.declarations {
        match declaration {
            Declaration::Verb(verb) => {
                verbs.insert(verb.name.as_str(), verb);
            }
            Declaration::Rule(rule) => rules.push(rule),
            Declaration::Default(_) => {
                return Err(Error::new(
                    "program residual search requires a monotonic program without defaults",
                ));
            }
            _ => {}
        }
    }
    validate_event(goal, &verbs)?;
    for event in evidence {
        validate_event(event, &verbs)?;
    }
    for abducible in abducibles {
        if !verbs
            .get(abducible.verb.as_str())
            .is_some_and(|verb| verb.kind == VerbKind::Seeded)
        {
            return Err(Error::new(format!(
                "abducible `{}` must be a declared seeded verb",
                abducible.verb
            )));
        }
    }
    let mut search = Search {
        rules,
        evidence,
        abducibles: abducibles.iter().cloned().collect(),
        limits,
        metrics: SearchMetrics::default(),
        active: Vec::new(),
    };
    let candidates = search.event(goal, 0)?;
    let mut explanations = Vec::new();
    let mut rejected = 0;
    for candidate in candidates {
        // The final boundary checks admissibility and replays the actual program
        // rules using forward semantics. No hypothetical facts enter Known.
        if !admissible(&candidate.required, &search.abducibles) {
            continue;
        }
        if let Some(events) = replay_proof(
            &candidate.proof,
            &search.rules,
            evidence,
            &search.abducibles,
        ) {
            if events.as_slice() == std::slice::from_ref(goal) {
                explanations.push(candidate);
                continue;
            }
        }
        rejected += 1;
    }
    search.metrics.rejected_not_forward_supported = rejected;
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
        metrics: search.metrics,
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

fn validate_event(event: &Event, verbs: &BTreeMap<&str, &Verb>) -> Result<(), Error> {
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
}

impl Search<'_> {
    fn visit(&mut self, depth: usize) -> Result<(), Error> {
        if depth > self.limits.max_depth || self.metrics.nodes >= self.limits.max_nodes {
            return Err(Error::new(
                "residual search depth/node limit exceeded; no complete result",
            ));
        }
        self.metrics.nodes += 1;
        Ok(())
    }

    fn branch(
        &mut self,
        required: Antecedent,
        proof: ResidualProof,
    ) -> Result<ResidualExplanation, Error> {
        if self.metrics.branches_generated >= self.limits.max_branches {
            return Err(Error::new(
                "residual search branch limit exceeded; no complete result",
            ));
        }
        self.metrics.branches_generated += 1;
        Ok(ResidualExplanation { required, proof })
    }

    fn event(&mut self, goal: &Event, depth: usize) -> Result<Vec<ResidualExplanation>, Error> {
        self.visit(depth)?;
        if let Some(event) = self.evidence.iter().find(|event| *event == goal) {
            return Ok(vec![self.branch(
                Antecedent::Unit,
                ResidualProof::Evidence(event.clone()),
            )?]);
        }
        if goal.proposition.kind == PropositionKind::Seeded {
            if self.abducibles.contains(&Abducible {
                verb: goal.verb.clone(),
                polarity: goal.polarity,
            }) {
                return Ok(vec![self.branch(
                    Antecedent::Event(goal.clone()),
                    ResidualProof::Hypothesis(goal.clone()),
                )?]);
            }
            return Ok(Vec::new());
        }
        if self.active.contains(goal) {
            return Err(Error::new(format!(
                "cyclic residual search at `{}`; no complete result",
                goal.verb
            )));
        }
        self.active.push(goal.clone());
        let mut explanations = Vec::new();
        for index in 0..self.rules.len() {
            let rule = self.rules[index];
            self.metrics.rules_examined += 1;
            let Some((pattern, bindings)) = match_goal(rule, goal) else {
                continue;
            };
            self.metrics.head_matches += 1;
            if let Some(parameter) = rule
                .parameters
                .iter()
                .find(|arg| !bindings.contains_key(&arg.name))
            {
                return Err(Error::new(format!(
                    "rule `{}` has parameter `{}` not determined by the goal",
                    rule.name, parameter.name
                )));
            }
            let antecedent = substitute_antecedent(&pattern, &bindings);
            for candidate in self.antecedent(&antecedent, depth + 1)? {
                let proof = ResidualProof::Rule {
                    rule: RuleRef {
                        name: rule.name.clone(),
                    },
                    goal: goal.clone(),
                    substitution: substitution_record(&bindings),
                    antecedent: Box::new(candidate.proof),
                };
                explanations.push(self.branch(candidate.required, proof)?);
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
                        // Check budget before allocating/cloning a product proof.
                        if self.metrics.branches_generated >= self.limits.max_branches {
                            return Err(Error::new(
                                "residual search branch limit exceeded; no complete result",
                            ));
                        }
                        let required = match (&a.required, &b.required) {
                            (Antecedent::Unit, other) | (other, Antecedent::Unit) => other.clone(),
                            (a, b) => Antecedent::And(Box::new(a.clone()), Box::new(b.clone())),
                        };
                        products.push(self.branch(
                            required,
                            ResidualProof::Tensor(
                                Box::new(a.proof.clone()),
                                Box::new(b.proof.clone()),
                            ),
                        )?);
                    }
                }
                Ok(products)
            }
        }
    }
}

#[cfg(test)]
mod tests;
