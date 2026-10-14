use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::derive::{bind_typed, substitute_antecedent, substitute_event, substitution_record};
use crate::elab::Error;
use crate::typed::{
    Antecedent, Arg, Event, ResidualRequirement, Rule, RuleBody, RuleRef, SubstitutionValue, Term,
    TimeExpr,
};

pub(super) type Bindings = BTreeMap<String, SubstitutionValue>;

/// Query one Horn/residual lambda against a ground goal and immutable evidence.
/// First constrain its head by the goal, then remove evidenced antecedent atoms.
/// Empty Known yields the complete instantiated antecedent.
/// I is the identity of a zero-factor logical product, not empty knowledge.
/// This is a local reduction, not search across rules or globally minimal abduction.
pub fn query_residual(
    rule: &Rule,
    goal: &Event,
    known: &[Event],
) -> Result<Vec<ResidualRequirement>, Error> {
    query_with_constraints(rule, goal, known, &BTreeMap::new(), false)
}

pub(super) fn query_with_constraints(
    rule: &Rule,
    goal: &Event,
    known: &[Event],
    constraints: &Bindings,
    retain_unit: bool,
) -> Result<Vec<ResidualRequirement>, Error> {
    let Some((antecedent, goal_bindings)) = constrained_goal(rule, goal, constraints)? else {
        return Ok(Vec::new());
    };
    let scope = &rule.parameters;
    let reduced = reduce(
        &antecedent,
        known,
        &scope,
        goal_bindings.clone(),
        Vec::new(),
    );
    // One complete justification suffices: this rule has no pending requirement.
    if !retain_unit
        && reduced
            .iter()
            .any(|matched| matched.required == Antecedent::Unit)
    {
        return Ok(Vec::new());
    }
    Ok(reduced
        .into_iter()
        .map(|matched| ResidualRequirement {
            rule: RuleRef {
                name: rule.name.clone(),
            },
            evidence: matched.evidence,
            goal_substitution: substitution_record(&goal_bindings),
            substitution: substitution_record(&matched.bindings),
            goal: goal.clone(),
            required: substitute_antecedent(&matched.required, &matched.bindings),
            parameters: rule
                .parameters
                .iter()
                .filter(|arg| !matched.bindings.contains_key(&arg.name))
                .cloned()
                .collect(),
        })
        .collect())
}

/// Merge constraints after matching the original head, so a conflict is an
/// error rather than a silently absent rule. Values retain their typed identity.
pub(super) fn constrained_goal(
    rule: &Rule,
    goal: &Event,
    constraints: &Bindings,
) -> Result<Option<(Antecedent, Bindings)>, Error> {
    let Some((antecedent, mut bindings)) = match_goal(rule, goal) else {
        return Ok(None);
    };
    for (name, value) in constraints {
        let parameter = rule
            .parameters
            .iter()
            .find(|p| p.name == *name)
            .ok_or_else(|| Error::new(format!("unknown query parameter `{}.{name}`", rule.name)))?;
        crate::derive::bind_typed(parameter, value.clone(), &mut bindings).ok_or_else(|| {
            Error::new(format!(
                "query binding `{}.{name}` is inconsistent with the goal",
                rule.name
            ))
        })?;
    }
    Ok(Some((antecedent, bindings)))
}

// Return the original antecedent plus goal constraints. Keeping pattern scope
// until substitution avoids capturing runtime identifiers that resemble binders.
pub(super) fn match_goal(rule: &Rule, goal: &Event) -> Option<(Antecedent, Bindings)> {
    let clause = match &rule.body {
        RuleBody::HornClause(clause) => Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => Cow::Owned(clause.unresiduate()),
    };
    let scope = &rule.parameters;
    let mut goal_bindings = Bindings::new();
    // Event equality is logical: an unresolved state interpretation must not
    // prevent this requirement from being resolved through another rule.
    if !unify_event(&clause.consequent, goal, &scope, &mut goal_bindings)
        || substitute_event(&clause.consequent, &goal_bindings) != *goal
    {
        return None;
    }
    Some((clause.antecedent.clone(), goal_bindings))
}

struct Reduction {
    required: Antecedent,
    bindings: Bindings,
    evidence: Vec<Event>,
}

fn reduce(
    antecedent: &Antecedent,
    known: &[Event],
    scope: &[Arg],
    bindings: Bindings,
    evidence: Vec<Event>,
) -> Vec<Reduction> {
    match antecedent {
        Antecedent::Unit => vec![Reduction {
            required: Antecedent::Unit,
            bindings,
            evidence,
        }],
        Antecedent::Event(pattern) => {
            let matches: Vec<_> = known
                .iter()
                .filter_map(|event| {
                    let mut bindings = bindings.clone();
                    if !unify_event(pattern, event, scope, &mut bindings) {
                        return None;
                    }
                    let mut evidence = evidence.clone();
                    evidence.push(event.clone());
                    Some(Reduction {
                        required: Antecedent::Unit,
                        bindings,
                        evidence,
                    })
                })
                .collect();
            if matches.is_empty() {
                vec![Reduction {
                    required: antecedent.clone(),
                    bindings,
                    evidence,
                }]
            } else {
                matches
            }
        }
        Antecedent::And(left, right) => reduce(left, known, scope, bindings, evidence)
            .into_iter()
            .flat_map(|left| {
                reduce(right, known, scope, left.bindings, left.evidence)
                    .into_iter()
                    .map(move |right| Reduction {
                        required: tensor(left.required.clone(), right.required),
                        bindings: right.bindings,
                        evidence: right.evidence,
                    })
            })
            .collect(),
    }
}

fn tensor(left: Antecedent, right: Antecedent) -> Antecedent {
    match (left, right) {
        (Antecedent::Unit, right) => right,
        (left, Antecedent::Unit) => left,
        (left, right) => Antecedent::And(Box::new(left), Box::new(right)),
    }
}

// Only lambda parameters are variables. Ground Term::Var identities in the goal
// and evidence are opaque values; substituting then matching would rebind them.
fn unify_event(pattern: &Event, event: &Event, scope: &[Arg], bindings: &mut Bindings) -> bool {
    pattern.verb == event.verb
        && pattern.polarity == event.polarity
        && pattern.proposition.kind == event.proposition.kind
        && pattern.args.len() == event.args.len()
        && pattern
            .args
            .iter()
            .zip(&event.args)
            .all(|(pattern, value)| unify_term(pattern, value, scope, bindings))
        && unify_time(
            &pattern.proposition.time,
            &event.proposition.time,
            scope,
            bindings,
        )
}

pub(super) fn unify_term(
    pattern: &Term,
    value: &Term,
    scope: &[Arg],
    bindings: &mut Bindings,
) -> bool {
    match (pattern, value) {
        (Term::Var(name), _) if scope.iter().any(|p| p.name == *name) => {
            (!matches!(value, Term::Constructor { .. }) || value.is_ground())
                && bind_typed(
                    scope.iter().find(|p| p.name == *name).unwrap(),
                    SubstitutionValue::Term(value.clone()),
                    bindings,
                )
                .is_some()
        }
        (
            Term::Constructor {
                constructor: a,
                argument: p,
            },
            Term::Constructor {
                constructor: b,
                argument: v,
            },
        ) => a == b && value.is_ground() && unify_term(p, v, scope, bindings),
        (
            Term::RecordLiteral {
                entity: a,
                fields: p,
            },
            Term::RecordLiteral {
                entity: b,
                fields: v,
            },
        ) => {
            a == b
                && p.len() == v.len()
                && p.iter().zip(v).all(|(p, v)| {
                    p.field == v.field && unify_term(&p.value, &v.value, scope, bindings)
                })
        }
        _ => pattern == value,
    }
}

fn unify_time(
    pattern: &TimeExpr,
    value: &TimeExpr,
    scope: &[Arg],
    bindings: &mut Bindings,
) -> bool {
    match (pattern, value) {
        (TimeExpr::Var(name), _) if scope.iter().any(|p| p.name == *name) => bind_typed(
            scope.iter().find(|p| p.name == *name).unwrap(),
            SubstitutionValue::Time(value.clone()),
            bindings,
        )
        .is_some(),
        (TimeExpr::After(pattern), TimeExpr::After(value)) => {
            unify_time(pattern, value, scope, bindings)
        }
        _ => pattern == value,
    }
}
