use std::collections::{BTreeMap, BTreeSet};

use crate::typed::{
    Antecedent, Closure, DerivationRecord, DerivationTime, DerivedEvent, Event, PropositionType,
    Rule, RuleBody, StateTime, StateTransitionType, Substitution, SubstitutionBinding,
    SubstitutionValue, Term, TimeExpr, UnstampedDerivation,
};

pub trait DerivationClock {
    fn next_derivation_time(&mut self) -> DerivationTime;
}

pub fn instantiate(rule: &Rule, antecedent: &Event) -> Option<UnstampedDerivation> {
    instantiate_from_known(rule, std::slice::from_ref(antecedent))
        .into_iter()
        .next()
}

pub fn instantiate_from_known(rule: &Rule, known: &[Event]) -> Vec<UnstampedDerivation> {
    let RuleBody::HornClause(clause) = &rule.body;
    match_antecedents(&clause.antecedent, known)
        .into_iter()
        .filter_map(|matched| {
            let event = substitute_event(&clause.consequent, &matched.substitutions);
            if has_unresolved_rule_parameter(&event, &rule.parameters) {
                return None;
            }
            Some(UnstampedDerivation {
                event,
                record: DerivationRecord {
                    rule: rule.name.clone(),
                    antecedents: matched.antecedents,
                    substitution: substitution_record(&matched.substitutions),
                },
            })
        })
        .collect()
}

fn has_unresolved_rule_parameter(event: &Event, parameters: &[crate::typed::Arg]) -> bool {
    let parameter_names = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    event
        .args
        .iter()
        .any(|term| term_has_parameter(term, &parameter_names))
        || time_has_parameter(&event.proposition.time, &parameter_names)
}

fn term_has_parameter(term: &Term, parameter_names: &BTreeSet<&str>) -> bool {
    match term {
        Term::Var(name) => parameter_names.contains(name.as_str()),
        Term::Const(_) | Term::None => false,
        Term::RecordLiteral { fields, .. } => fields
            .iter()
            .any(|field| term_has_parameter(&field.value, parameter_names)),
    }
}

fn time_has_parameter(time: &TimeExpr, parameter_names: &BTreeSet<&str>) -> bool {
    match time {
        TimeExpr::At(name) => parameter_names.contains(name.as_str()),
        TimeExpr::After(time) => time_has_parameter(time, parameter_names),
    }
}

pub fn stamp(derivation: UnstampedDerivation, derived_at: DerivationTime) -> DerivedEvent {
    DerivedEvent {
        event: derivation.event,
        derived_at,
        record: derivation.record,
    }
}

pub fn derive_once(rule: &Rule, seed: &Event, derived_at: DerivationTime) -> Option<DerivedEvent> {
    instantiate(rule, seed).map(|derivation| stamp(derivation, derived_at))
}

pub fn close<C: DerivationClock>(seeds: Vec<Event>, rules: &[Rule], clock: &mut C) -> Closure {
    let mut known = seeds;
    let mut known_keys: BTreeSet<Event> = known.iter().cloned().collect();
    let mut frontier = known.clone();
    let mut derivations = Vec::new();
    let mut seen_unstamped = Vec::new();

    while !frontier.is_empty() {
        let mut next_frontier = Vec::new();

        for rule in rules {
            for unstamped in instantiate_from_known(rule, &known) {
                if seen_unstamped.contains(&unstamped) {
                    continue;
                }
                seen_unstamped.push(unstamped.clone());
                let is_new = known_keys.insert(unstamped.event.clone());
                let derived = stamp(unstamped, clock.next_derivation_time());
                if is_new {
                    known.push(derived.event.clone());
                    next_frontier.push(derived.event.clone());
                }
                derivations.push(derived);
            }
        }

        frontier = next_frontier;
    }

    Closure { known, derivations }
}

#[derive(Debug, Clone)]
struct AntecedentMatch {
    antecedents: Vec<Event>,
    substitutions: BTreeMap<String, SubstitutionValue>,
}

fn match_antecedents(antecedent: &Antecedent, known: &[Event]) -> Vec<AntecedentMatch> {
    match_antecedents_with(antecedent, known, BTreeMap::new(), Vec::new())
}

fn match_antecedents_with(
    antecedent: &Antecedent,
    known: &[Event],
    substitutions: BTreeMap<String, SubstitutionValue>,
    antecedents: Vec<Event>,
) -> Vec<AntecedentMatch> {
    match antecedent {
        Antecedent::Event(pattern) => known
            .iter()
            .filter_map(|event| {
                let mut substitutions = substitutions.clone();
                match_event(pattern, event, &mut substitutions)?;
                let mut antecedents = antecedents.clone();
                antecedents.push(event.clone());
                Some(AntecedentMatch {
                    antecedents,
                    substitutions,
                })
            })
            .collect(),
        Antecedent::And(left, right) => {
            match_antecedents_with(left, known, substitutions, antecedents)
                .into_iter()
                .flat_map(|matched| {
                    match_antecedents_with(right, known, matched.substitutions, matched.antecedents)
                })
                .collect()
        }
    }
}

fn match_event(
    pattern: &Event,
    event: &Event,
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    if pattern.verb != event.verb || pattern.args.len() != event.args.len() {
        return None;
    }

    for (pattern, value) in pattern.args.iter().zip(&event.args) {
        bind_term(pattern, value, substitutions)?;
    }
    bind_time(
        &pattern.proposition.time,
        &event.proposition.time,
        substitutions,
    )
}

fn bind_term(
    pattern: &Term,
    value: &Term,
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    match pattern {
        Term::Var(name) => bind(name, SubstitutionValue::Term(value.clone()), substitutions),
        Term::Const(_) | Term::None | Term::RecordLiteral { .. } => {
            (pattern == value).then_some(())
        }
    }
}

fn bind_time(
    pattern: &TimeExpr,
    value: &TimeExpr,
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    match (pattern, value) {
        (TimeExpr::At(name), _) => {
            bind(name, SubstitutionValue::Time(value.clone()), substitutions)
        }
        (TimeExpr::After(pattern_inner), TimeExpr::After(value_inner)) => {
            bind_time(pattern_inner, value_inner, substitutions)
        }
        _ => None,
    }
}

fn bind(
    name: &str,
    value: SubstitutionValue,
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    if let Some(existing) = substitutions.get(name) {
        (existing == &value).then_some(())
    } else {
        substitutions.insert(name.to_string(), value);
        Some(())
    }
}

fn substitute_event(event: &Event, substitutions: &BTreeMap<String, SubstitutionValue>) -> Event {
    Event {
        verb: event.verb.clone(),
        args: event
            .args
            .iter()
            .map(|term| substitute_term(term, substitutions))
            .collect(),
        proposition: PropositionType {
            kind: event.proposition.kind.clone(),
            time: substitute_time(&event.proposition.time, substitutions),
        },
        transition: event
            .transition
            .as_ref()
            .map(|transition| substitute_transition(transition, substitutions)),
    }
}

fn substitute_term(term: &Term, substitutions: &BTreeMap<String, SubstitutionValue>) -> Term {
    match term {
        Term::Var(name) => match substitutions.get(name) {
            Some(SubstitutionValue::Term(term)) => term.clone(),
            _ => Term::Var(name.clone()),
        },
        Term::Const(name) => Term::Const(name.clone()),
        Term::None => Term::None,
        Term::RecordLiteral { entity, fields } => Term::RecordLiteral {
            entity: entity.clone(),
            fields: fields
                .iter()
                .map(|field| crate::typed::RecordFieldValue {
                    field: field.field.clone(),
                    value: substitute_term(&field.value, substitutions),
                })
                .collect(),
        },
    }
}

fn substitute_time(
    time: &TimeExpr,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> TimeExpr {
    match time {
        TimeExpr::At(name) => match substitutions.get(name) {
            Some(SubstitutionValue::Time(time)) => time.clone(),
            _ => TimeExpr::At(name.clone()),
        },
        TimeExpr::After(time) => TimeExpr::After(Box::new(substitute_time(time, substitutions))),
    }
}

fn substitute_transition(
    transition: &StateTransitionType,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> StateTransitionType {
    StateTransitionType {
        input: substitute_state_time(&transition.input, substitutions),
        output: substitute_state_time(&transition.output, substitutions),
    }
}

fn substitute_state_time(
    time: &StateTime,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> StateTime {
    match time {
        StateTime::Unresolved => StateTime::Unresolved,
        StateTime::At(time) => StateTime::At(substitute_time(time, substitutions)),
    }
}

fn substitution_record(substitutions: &BTreeMap<String, SubstitutionValue>) -> Substitution {
    Substitution {
        bindings: substitutions
            .iter()
            .map(|(parameter, value)| SubstitutionBinding {
                parameter: parameter.clone(),
                value: value.clone(),
            })
            .collect(),
    }
}
