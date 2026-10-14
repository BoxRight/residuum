//! Layered residual search. Open obligations are never certified explanations.
//! Completion reuses the existing forward replay and active certification.
use super::engine;
use super::*;
use crate::derive::DerivationClock;
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
        match self {
            Self::Pending { .. } => None,
            Self::Unit => Some(ResidualExplanation {
                required: Antecedent::Unit,
                proof: ResidualProof::Unit,
            }),
            Self::Evidence(e) => {
                engine::leaf_explanation(engine::EventDisposition::Evidence(e.clone()))
            }
            Self::Hypothesis(e) => {
                engine::leaf_explanation(engine::EventDisposition::Hypothesis(e.clone()))
            }
            Self::Tensor(a, b) => {
                let a = a.explanation()?;
                let b = b.explanation()?;
                Some(engine::tensor_explanations(&a, &b))
            }
            Self::Rule {
                rule,
                goal,
                substitution,
                antecedent,
            } => {
                let child = antecedent.explanation()?;
                Some(engine::rule_explanation(rule, goal, substitution, child))
            }
        }
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
        let default = engine::default_abducibles(module);
        let allowed = allowed.unwrap_or(&default);
        let rules = engine::validate_search(module, evidence, goal, limits, allowed)?
            .into_iter()
            .cloned()
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
        engine::visit(self.limits, &mut self.metrics, depth)
    }
    fn allocation(&mut self) -> Result<(), Error> {
        engine::reserve_branch(self.limits, &mut self.metrics)
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
                match engine::classify_event(&self.evidence, &self.allowed, goal, ancestors)? {
                    engine::EventDisposition::Evidence(event) => {
                        self.allocation()?;
                        vec![ResidualSearchTree::Evidence(event)]
                    }
                    engine::EventDisposition::Hypothesis(event) => {
                        self.allocation()?;
                        vec![ResidualSearchTree::Hypothesis(event)]
                    }
                    engine::EventDisposition::Missing => Vec::new(),
                    engine::EventDisposition::Expand => {
                        let mut path = ancestors.clone();
                        path.push(goal.clone());
                        let mut alternatives = Vec::new();
                        for rule in self.rules.clone() {
                            let Some(expansion) = engine::expand_rule(
                                &rule,
                                goal,
                                self.constraints.as_ref(),
                                &mut self.metrics,
                                true,
                            )?
                            else {
                                continue;
                            };
                            let child = self.antecedent(&expansion.antecedent, depth + 1, &path)?;
                            self.allocation()?;
                            alternatives.push(ResidualSearchTree::Rule {
                                rule: expansion.rule,
                                goal: expansion.goal,
                                substitution: expansion.substitution,
                                antecedent: Box::new(child),
                            });
                        }
                        alternatives
                    }
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
