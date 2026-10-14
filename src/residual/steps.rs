//! Layered residual search. Open obligations are never certified explanations.
//! Completion reuses the existing forward replay and active certification.
use super::*;
use crate::derive::{DerivationClock, substitute_antecedent, substitution_record};
use crate::typed::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResidualSearchTree {
    Pending {
        goal: Event,
        depth: usize,
        ancestors: Vec<Event>,
    },
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
impl ResidualSearchTree {
    fn pending(&self, output: &mut Vec<Event>) {
        match self {
            Self::Pending { goal, .. } => output.push(goal.clone()),
            Self::Tensor(a, b) => {
                a.pending(output);
                b.pending(output);
            }
            Self::Rule { antecedent, .. } => antecedent.pending(output),
            _ => {}
        }
    }
    fn explanation(&self) -> Option<ResidualExplanation> {
        let (required, proof) = match self {
            Self::Pending { .. } => return None,
            Self::Unit => (Antecedent::Unit, ResidualProof::Unit),
            Self::Evidence(e) => (Antecedent::Unit, ResidualProof::Evidence(e.clone())),
            Self::Hypothesis(e) => (
                Antecedent::Event(e.clone()),
                ResidualProof::Hypothesis(e.clone()),
            ),
            Self::Tensor(a, b) => {
                let a = a.explanation()?;
                let b = b.explanation()?;
                let required = match (a.required, b.required) {
                    (Antecedent::Unit, other) | (other, Antecedent::Unit) => other,
                    (a, b) => Antecedent::And(Box::new(a), Box::new(b)),
                };
                (
                    required,
                    ResidualProof::Tensor(Box::new(a.proof), Box::new(b.proof)),
                )
            }
            Self::Rule {
                rule,
                goal,
                substitution,
                antecedent,
            } => {
                let child = antecedent.explanation()?;
                (
                    child.required,
                    ResidualProof::Rule {
                        rule: rule.clone(),
                        goal: goal.clone(),
                        substitution: substitution.clone(),
                        antecedent: Box::new(child.proof),
                    },
                )
            }
        };
        Some(ResidualExplanation { required, proof })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualStep {
    pub round: usize,
    pub branches: Vec<ResidualSearchTree>,
    pub pending: Vec<Event>,
    pub metrics: SearchMetrics,
    pub complete: bool,
}

#[derive(Debug, Clone)]
pub struct ResidualSession {
    rules: Vec<Rule>,
    evidence: Vec<Event>,
    goal: Event,
    allowed: BTreeSet<Abducible>,
    limits: SearchLimits,
    metrics: SearchMetrics,
    branches: Vec<ResidualSearchTree>,
    round: usize,
    result: Option<ProgramResidualQuery>,
    failed: Option<String>,
    constraints: Option<BTreeMap<String, super::query::Bindings>>,
}
impl ResidualSession {
    pub fn new(
        module: &Module,
        evidence: &[Event],
        goal: &Event,
        limits: SearchLimits,
        allowed: Option<&[Abducible]>,
    ) -> Result<Self, Error> {
        if limits.max_depth == 0
            || limits.max_depth > 64
            || limits.max_nodes == 0
            || limits.max_branches == 0
        {
            return Err(Error::new(
                "invalid residual search limits (depth must be 1..=64)",
            ));
        }
        if module
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Default(_)))
        {
            return Err(Error::new(
                "program residual search requires a monotonic program without defaults",
            ));
        }
        let verbs: BTreeMap<_, _> = module
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Verb(v) => Some((v.name.as_str(), v)),
                _ => None,
            })
            .collect();
        super::program::validate_event(goal, &verbs)?;
        for event in evidence {
            super::program::validate_event(event, &verbs)?;
        }
        let default: Vec<_> = verbs
            .values()
            .filter(|v| v.is_abducible())
            .flat_map(|v| {
                [Polarity::Positive, Polarity::EvidentialNot].map(|polarity| Abducible {
                    verb: v.name.clone(),
                    polarity,
                })
            })
            .collect();
        let allowed = allowed.unwrap_or(&default);
        for a in allowed {
            if !verbs
                .get(a.verb.as_str())
                .is_some_and(|v| v.kind == VerbKind::Seeded)
            {
                return Err(Error::new(format!(
                    "abducible `{}` must be a declared seeded verb",
                    a.verb
                )));
            }
        }
        let rules = module
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) => Some(r.clone()),
                _ => None,
            })
            .collect();
        Ok(Self {
            rules,
            evidence: evidence.to_vec(),
            goal: goal.clone(),
            allowed: allowed.iter().cloned().collect(),
            limits,
            metrics: SearchMetrics::default(),
            branches: vec![ResidualSearchTree::Pending {
                goal: goal.clone(),
                depth: 0,
                ancestors: Vec::new(),
            }],
            round: 0,
            result: None,
            failed: None,
            constraints: None,
        })
    }
    pub(super) fn constrained(
        mut self,
        constraints: BTreeMap<String, super::query::Bindings>,
    ) -> Self {
        self.constraints = Some(constraints);
        self
    }
    pub fn result(&self) -> Option<&ProgramResidualQuery> {
        self.result.as_ref()
    }
    pub fn evidence(&self) -> &[Event] {
        &self.evidence
    }
    pub fn step(&mut self) -> Result<ResidualStep, Error> {
        if let Some(error) = &self.failed {
            return Err(Error::new(error.clone()));
        }
        if self.result.is_none() {
            let mut next = Vec::new();
            for branch in self.branches.clone() {
                match self.advance(&branch) {
                    Ok(branches) => next.extend(branches),
                    Err(error) => {
                        self.failed = Some(error.to_string());
                        return Err(error);
                    }
                }
            }
            self.branches = next;
        }
        self.round += 1;
        let mut pending = Vec::new();
        for branch in &self.branches {
            branch.pending(&mut pending);
        }
        let complete = pending.is_empty();
        if complete && self.result.is_none() {
            let candidates = self
                .branches
                .iter()
                .map(|b| b.explanation().expect("no pending obligations"))
                .collect();
            let rules: Vec<_> = self.rules.iter().collect();
            let result = super::program::certify_candidates(
                &rules,
                &self.evidence,
                &self.goal,
                &self.allowed,
                self.metrics.clone(),
                candidates,
            )?;
            self.metrics = result.metrics.clone();
            self.result = Some(result);
        }
        Ok(ResidualStep {
            round: self.round,
            branches: self.branches.clone(),
            pending,
            metrics: self.metrics.clone(),
            complete,
        })
    }
    pub fn trace(&mut self, max_steps: usize) -> Result<Vec<ResidualStep>, Error> {
        let mut steps = Vec::new();
        for _ in 0..max_steps {
            let step = self.step()?;
            let done = step.complete;
            steps.push(step);
            if done {
                break;
            }
        }
        Ok(steps)
    }
    fn visit(&mut self, depth: usize) -> Result<(), Error> {
        if depth > self.limits.max_depth || self.metrics.nodes >= self.limits.max_nodes {
            return Err(Error::new(
                "residual search depth/node limit exceeded; no complete result",
            ));
        }
        self.metrics.nodes += 1;
        Ok(())
    }
    fn allocation(&mut self) -> Result<(), Error> {
        if self.metrics.branches_generated >= self.limits.max_branches {
            return Err(Error::new(
                "residual search branch limit exceeded; no complete result",
            ));
        }
        self.metrics.branches_generated += 1;
        Ok(())
    }
    fn antecedent(
        &mut self,
        a: &Antecedent,
        depth: usize,
        ancestors: &[Event],
    ) -> Result<ResidualSearchTree, Error> {
        self.visit(depth)?;
        Ok(match a {
            Antecedent::Unit => ResidualSearchTree::Unit,
            Antecedent::Event(goal) => ResidualSearchTree::Pending {
                goal: goal.clone(),
                depth: depth + 1,
                ancestors: ancestors.to_vec(),
            },
            Antecedent::And(a, b) => ResidualSearchTree::Tensor(
                Box::new(self.antecedent(a, depth + 1, ancestors)?),
                Box::new(self.antecedent(b, depth + 1, ancestors)?),
            ),
        })
    }
    fn advance(&mut self, tree: &ResidualSearchTree) -> Result<Vec<ResidualSearchTree>, Error> {
        Ok(match tree {
            ResidualSearchTree::Pending {
                goal,
                depth,
                ancestors,
            } => {
                self.visit(*depth)?;
                if self.evidence.contains(goal) {
                    self.allocation()?;
                    vec![ResidualSearchTree::Evidence(goal.clone())]
                } else if goal.proposition.kind == PropositionKind::Seeded {
                    if self.allowed.contains(&Abducible {
                        verb: goal.verb.clone(),
                        polarity: goal.polarity,
                    }) {
                        self.allocation()?;
                        vec![ResidualSearchTree::Hypothesis(goal.clone())]
                    } else {
                        Vec::new()
                    }
                } else {
                    if ancestors.contains(goal) {
                        return Err(Error::new(format!(
                            "cyclic residual search at `{}`; no complete result",
                            goal.verb
                        )));
                    }
                    let mut path = ancestors.clone();
                    path.push(goal.clone());
                    let mut alternatives = Vec::new();
                    for rule in self.rules.clone() {
                        self.metrics.rules_examined += 1;
                        let matched = if let Some(constraints) = &self.constraints {
                            super::query::constrained_goal(
                                &rule,
                                goal,
                                constraints.get(&rule.name).unwrap_or(&BTreeMap::new()),
                            )?
                        } else {
                            super::query::match_goal(&rule, goal)
                        };
                        let Some((pattern, bindings)) = matched else {
                            continue;
                        };
                        self.metrics.head_matches += 1;
                        let pending: Vec<_> = rule
                            .parameters
                            .iter()
                            .filter(|p| !bindings.contains_key(&p.name))
                            .collect();
                        if !pending.is_empty() {
                            return Err(Error::new(format!(
                                "rule `{}` has parameters {pending:?} not determined by the goal; no complete result",
                                rule.name
                            )));
                        }
                        let instantiated = substitute_antecedent(&pattern, &bindings);
                        let child = self.antecedent(&instantiated, depth + 1, &path)?;
                        self.allocation()?;
                        alternatives.push(ResidualSearchTree::Rule {
                            rule: RuleRef { name: rule.name },
                            goal: goal.clone(),
                            substitution: substitution_record(&bindings),
                            antecedent: Box::new(child),
                        });
                    }
                    alternatives
                }
            }
            ResidualSearchTree::Tensor(a, b) => {
                let left = self.advance(a)?;
                if left.is_empty() {
                    return Ok(Vec::new());
                }
                let right = self.advance(b)?;
                let mut products = Vec::new();
                for a in left {
                    for b in &right {
                        self.allocation()?;
                        products.push(ResidualSearchTree::Tensor(
                            Box::new(a.clone()),
                            Box::new(b.clone()),
                        ));
                    }
                }
                products
            }
            ResidualSearchTree::Rule {
                rule,
                goal,
                substitution,
                antecedent,
            } => self
                .advance(antecedent)?
                .into_iter()
                .map(|a| ResidualSearchTree::Rule {
                    rule: rule.clone(),
                    goal: goal.clone(),
                    substitution: substitution.clone(),
                    antecedent: Box::new(a),
                })
                .collect(),
            other => vec![other.clone()],
        })
    }
}

#[derive(Debug, Clone)]
pub struct ActiveResidualSession {
    original: Module,
    templates: BTreeMap<String, ResidualActivation>,
    search: ResidualSession,
    result: Option<ActiveProgramResidualQuery>,
}
impl ActiveResidualSession {
    pub(super) fn constrained(
        mut self,
        constraints: BTreeMap<String, super::query::Bindings>,
    ) -> Self {
        self.search = self.search.constrained(constraints);
        self
    }
    pub fn new(
        module: &Module,
        evidence: &[Event],
        goal: &Event,
        limits: SearchLimits,
        allowed: Option<&[Abducible]>,
    ) -> Result<Self, Error> {
        let (view, templates) = super::active::guarded_view(module)?;
        Self::from_view(module, &view, templates, evidence, goal, limits, allowed)
    }
    pub(super) fn from_view(
        module: &Module,
        view: &Module,
        templates: BTreeMap<String, ResidualActivation>,
        evidence: &[Event],
        goal: &Event,
        limits: SearchLimits,
        allowed: Option<&[Abducible]>,
    ) -> Result<Self, Error> {
        Ok(Self {
            original: module.clone(),
            templates,
            search: ResidualSession::new(view, evidence, goal, limits, allowed)?,
            result: None,
        })
    }
    pub fn result(&self) -> Option<&ActiveProgramResidualQuery> {
        self.result.as_ref()
    }
    pub fn step<C: DerivationClock>(&mut self, clock: &mut C) -> Result<ResidualStep, Error> {
        let step = self.search.step()?;
        if step.complete && self.result.is_none() {
            self.result = Some(super::active::certify(
                &self.original,
                &self.search.evidence,
                &self.search.goal,
                self.search.result.as_ref().unwrap().clone(),
                &self.templates,
                clock,
            )?);
        }
        Ok(step)
    }
    pub fn trace<C: DerivationClock>(
        &mut self,
        max_steps: usize,
        clock: &mut C,
    ) -> Result<Vec<ResidualStep>, Error> {
        let mut steps = Vec::new();
        for _ in 0..max_steps {
            let step = self.step(clock)?;
            let done = step.complete;
            steps.push(step);
            if done {
                break;
            }
        }
        Ok(steps)
    }
}

#[cfg(test)]
mod tests;
