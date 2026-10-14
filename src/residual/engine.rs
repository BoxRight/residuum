//! Operations extracted from the original residual search. Both traversal
//! schedules call these operations; only continuation management differs.
use super::*;
use crate::derive::{substitute_antecedent, substitution_record};
use crate::typed::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Constraints = BTreeMap<String, super::query::Bindings>;

pub(super) fn default_abducibles(module: &Module) -> Vec<Abducible> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) if v.is_abducible() => Some(v),
            _ => None,
        })
        .flat_map(|v| {
            [Polarity::Positive, Polarity::EvidentialNot].map(|polarity| Abducible {
                verb: v.name.clone(),
                polarity,
            })
        })
        .collect()
}

pub(super) fn validate_search<'a>(
    module: &'a Module,
    evidence: &[Event],
    goal: &Event,
    limits: SearchLimits,
    allowed: &[Abducible],
) -> Result<Vec<&'a Rule>, Error> {
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
    for declaration in &module.declarations {
        match declaration {
            Declaration::Verb(v) => {
                verbs.insert(v.name.as_str(), v);
            }
            Declaration::Rule(r) => rules.push(r),
            Declaration::Default(_) => {
                return Err(Error::new(
                    "program residual search requires a monotonic program without defaults",
                ));
            }
            _ => {}
        }
    }
    super::program::validate_event(goal, &verbs)?;
    for event in evidence {
        super::program::validate_event(event, &verbs)?;
    }
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
    Ok(rules)
}

pub(super) fn visit(
    limits: SearchLimits,
    metrics: &mut SearchMetrics,
    depth: usize,
) -> Result<(), Error> {
    if depth > limits.max_depth || metrics.nodes >= limits.max_nodes {
        return Err(Error::new(
            "residual search depth/node limit exceeded; no complete result",
        ));
    }
    metrics.nodes += 1;
    Ok(())
}
pub(super) fn reserve_branch(
    limits: SearchLimits,
    metrics: &mut SearchMetrics,
) -> Result<(), Error> {
    if metrics.branches_generated >= limits.max_branches {
        return Err(Error::new(
            "residual search branch limit exceeded; no complete result",
        ));
    }
    metrics.branches_generated += 1;
    Ok(())
}

pub(super) enum EventDisposition {
    Evidence(Event),
    Hypothesis(Event),
    Missing,
    Expand,
}
pub(super) fn classify_event(
    evidence: &[Event],
    allowed: &BTreeSet<Abducible>,
    goal: &Event,
    ancestors: &[Event],
) -> Result<EventDisposition, Error> {
    if let Some(event) = evidence.iter().find(|event| *event == goal) {
        return Ok(EventDisposition::Evidence(event.clone()));
    }
    if goal.proposition.kind == PropositionKind::Seeded {
        return Ok(
            if allowed.contains(&Abducible {
                verb: goal.verb.clone(),
                polarity: goal.polarity,
            }) {
                EventDisposition::Hypothesis(goal.clone())
            } else {
                EventDisposition::Missing
            },
        );
    }
    if ancestors.contains(goal) {
        return Err(Error::new(format!(
            "cyclic residual search at `{}`; no complete result",
            goal.verb
        )));
    }
    Ok(EventDisposition::Expand)
}
pub(super) fn leaf_explanation(disposition: EventDisposition) -> Option<ResidualExplanation> {
    match disposition {
        EventDisposition::Evidence(e) => Some(ResidualExplanation {
            required: Antecedent::Unit,
            proof: ResidualProof::Evidence(e),
        }),
        EventDisposition::Hypothesis(e) => Some(ResidualExplanation {
            required: Antecedent::Event(e.clone()),
            proof: ResidualProof::Hypothesis(e),
        }),
        _ => None,
    }
}

pub(super) struct RuleExpansion {
    pub rule: RuleRef,
    pub goal: Event,
    pub substitution: Substitution,
    pub antecedent: Antecedent,
}
pub(super) fn expand_rule(
    rule: &Rule,
    goal: &Event,
    constraints: Option<&Constraints>,
    metrics: &mut SearchMetrics,
    typed_pending: bool,
) -> Result<Option<RuleExpansion>, Error> {
    metrics.rules_examined += 1;
    let matched = if let Some(constraints) = constraints {
        super::query::constrained_goal(
            rule,
            goal,
            constraints.get(&rule.name).unwrap_or(&BTreeMap::new()),
        )?
    } else {
        super::query::match_goal(rule, goal)
    };
    let Some((pattern, bindings)) = matched else {
        return Ok(None);
    };
    metrics.head_matches += 1;
    let pending: Vec<_> = rule
        .parameters
        .iter()
        .filter(|p| !bindings.contains_key(&p.name))
        .collect();
    if let Some(parameter) = pending.first() {
        return Err(if typed_pending || constraints.is_some() {
            Error::new(format!(
                "rule `{}` has unresolved query parameters {pending:?}; not determined by the goal or supplied bindings; no complete result",
                rule.name
            ))
        } else {
            Error::new(format!(
                "rule `{}` has parameter `{}` not determined by the goal",
                rule.name, parameter.name
            ))
        });
    }
    Ok(Some(RuleExpansion {
        rule: RuleRef {
            name: rule.name.clone(),
        },
        goal: goal.clone(),
        substitution: substitution_record(&bindings),
        antecedent: substitute_antecedent(&pattern, &bindings),
    }))
}
pub(super) fn rule_explanation(
    rule: &RuleRef,
    goal: &Event,
    substitution: &Substitution,
    child: ResidualExplanation,
) -> ResidualExplanation {
    ResidualExplanation {
        required: child.required,
        proof: ResidualProof::Rule {
            rule: rule.clone(),
            goal: goal.clone(),
            substitution: substitution.clone(),
            antecedent: Box::new(child.proof),
        },
    }
}
pub(super) fn tensor_explanations(
    a: &ResidualExplanation,
    b: &ResidualExplanation,
) -> ResidualExplanation {
    let required = match (&a.required, &b.required) {
        (Antecedent::Unit, other) | (other, Antecedent::Unit) => other.clone(),
        (a, b) => Antecedent::And(Box::new(a.clone()), Box::new(b.clone())),
    };
    ResidualExplanation {
        required,
        proof: ResidualProof::Tensor(Box::new(a.proof.clone()), Box::new(b.proof.clone())),
    }
}
