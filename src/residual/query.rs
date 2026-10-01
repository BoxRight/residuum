use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::derive::{substitute_antecedent, substitute_event, substitution_record};
use crate::elab::Error;
use crate::typed::{
    Antecedent, Event, ResidualRequirement, Rule, RuleBody, RuleRef, SubstitutionValue, Term,
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
    let Some((antecedent, goal_bindings)) = match_goal(rule, goal) else {
        return Ok(Vec::new());
    };
    let scope: BTreeSet<_> = rule
        .parameters
        .iter()
        .map(|arg| arg.name.as_str())
        .collect();
    let reduced = reduce(
        &antecedent,
        known,
        &scope,
        goal_bindings.clone(),
        Vec::new(),
    );
    // One complete justification suffices: this rule has no pending requirement.
    if reduced
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

// Return the original antecedent plus goal constraints. Keeping pattern scope
// until substitution avoids capturing runtime identifiers that resemble binders.
pub(super) fn match_goal(rule: &Rule, goal: &Event) -> Option<(Antecedent, Bindings)> {
    let clause = match &rule.body {
        RuleBody::HornClause(clause) => Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => Cow::Owned(clause.unresiduate()),
    };
    let scope: BTreeSet<_> = rule
        .parameters
        .iter()
        .map(|arg| arg.name.as_str())
        .collect();
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
    scope: &BTreeSet<&str>,
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
fn unify_event(
    pattern: &Event,
    event: &Event,
    scope: &BTreeSet<&str>,
    bindings: &mut Bindings,
) -> bool {
    pattern.verb == event.verb
        && pattern.polarity == event.polarity
        && pattern.proposition.kind == event.proposition.kind
        && pattern.args.len() == event.args.len()
        && pattern
            .args
            .iter()
            .zip(&event.args)
            .all(|(pattern, value)| match pattern {
                Term::Var(name) if scope.contains(name.as_str()) => {
                    bind(name, SubstitutionValue::Term(value.clone()), bindings)
                }
                _ => pattern == value,
            })
        && unify_time(
            &pattern.proposition.time,
            &event.proposition.time,
            scope,
            bindings,
        )
}

fn unify_time(
    pattern: &TimeExpr,
    value: &TimeExpr,
    scope: &BTreeSet<&str>,
    bindings: &mut Bindings,
) -> bool {
    match (pattern, value) {
        (TimeExpr::At(name), _) if scope.contains(name.as_str()) => {
            bind(name, SubstitutionValue::Time(value.clone()), bindings)
        }
        (TimeExpr::After(pattern), TimeExpr::After(value)) => {
            unify_time(pattern, value, scope, bindings)
        }
        _ => pattern == value,
    }
}

fn bind(name: &str, value: SubstitutionValue, bindings: &mut Bindings) -> bool {
    match bindings.get(name) {
        Some(existing) => *existing == value,
        None => {
            bindings.insert(name.into(), value);
            true
        }
    }
}
