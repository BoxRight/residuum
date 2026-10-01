use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

mod incremental;
pub use incremental::{ClosureDelta, IncrementalClosure, IncrementalStats};

use crate::typed::{
    Antecedent, Arg, Closure, Declaration, DerivationRecord, DerivationTime, DerivedEvent, Event,
    Module, Polarity, PropositionType, Rule, RuleBody, StateTime, StateTransitionType,
    Substitution, SubstitutionBinding, SubstitutionValue, Term, TimeExpr, UnstampedDerivation,
    Verb, VerbKind,
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
    // Forward inference consumes the Horn view; it does not assert or query a residual.
    let clause = match &rule.body {
        RuleBody::HornClause(clause) => Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => Cow::Owned(clause.unresiduate()),
    };
    match_antecedents(&clause.antecedent, known, &rule.parameters)
        .into_iter()
        .filter_map(|matched| finish_instantiation(rule, &clause.consequent, matched))
        .collect()
}

fn finish_instantiation(
    rule: &Rule,
    consequent: &Event,
    matched: AntecedentMatch,
) -> Option<UnstampedDerivation> {
    // Check references in the template, not names inside substituted ground
    // values. A runtime identity may legitimately have a binder's spelling.
    if has_unbound_rule_parameter(consequent, &rule.parameters, &matched.substitutions) {
        return None;
    }
    let event = substitute_event(consequent, &matched.substitutions);
    Some(UnstampedDerivation {
        event,
        record: DerivationRecord {
            rule: rule.name.clone(),
            antecedents: matched.antecedents,
            substitution: substitution_record(&matched.substitutions),
        },
    })
}

/// Replay one ordered antecedent witness with the forward matcher's semantics.
/// No search, clock, or closure: each atom consumes exactly one witness event.
pub(crate) fn instantiate_from_witness(
    rule: &Rule,
    witness: &[Event],
) -> Option<UnstampedDerivation> {
    let clause = match &rule.body {
        RuleBody::HornClause(clause) => Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => Cow::Owned(clause.unresiduate()),
    };
    fn replay(
        antecedent: &Antecedent,
        events: &mut std::slice::Iter<'_, Event>,
        bindings: &mut BTreeMap<String, SubstitutionValue>,
        parameters: &[Arg],
    ) -> Option<()> {
        match antecedent {
            Antecedent::Unit => Some(()),
            Antecedent::Event(pattern) => {
                match_event(pattern, events.next()?, parameters, bindings)
            }
            Antecedent::And(left, right) => {
                replay(left, events, bindings, parameters)?;
                replay(right, events, bindings, parameters)
            }
        }
    }
    let mut events = witness.iter();
    let mut substitutions = BTreeMap::new();
    replay(
        &clause.antecedent,
        &mut events,
        &mut substitutions,
        &rule.parameters,
    )?;
    if events.next().is_some() {
        return None;
    }
    finish_instantiation(
        rule,
        &clause.consequent,
        AntecedentMatch {
            antecedents: witness.to_vec(),
            substitutions,
        },
    )
}

fn has_unbound_rule_parameter(
    event: &Event,
    parameters: &[Arg],
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> bool {
    let parameter_names = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    event
        .args
        .iter()
        .any(|term| term_has_unbound_parameter(term, &parameter_names, substitutions))
        || time_has_unbound_parameter(&event.proposition.time, &parameter_names, substitutions)
}

fn term_has_unbound_parameter(
    term: &Term,
    parameter_names: &BTreeSet<&str>,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> bool {
    match term {
        Term::Var(name) => {
            parameter_names.contains(name.as_str())
                && !matches!(substitutions.get(name), Some(SubstitutionValue::Term(_)))
        }
        Term::Const(_) | Term::None => false,
        Term::RecordLiteral { fields, .. } => fields
            .iter()
            .any(|field| term_has_unbound_parameter(&field.value, parameter_names, substitutions)),
    }
}

fn time_has_unbound_parameter(
    time: &TimeExpr,
    parameter_names: &BTreeSet<&str>,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> bool {
    match time {
        TimeExpr::At(name) => {
            parameter_names.contains(name.as_str())
                && !matches!(substitutions.get(name), Some(SubstitutionValue::Time(_)))
        }
        TimeExpr::After(time) => time_has_unbound_parameter(time, parameter_names, substitutions),
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
    close_with_programs(seeds, rules, &[], clock)
}

/// Module-aware Horn closure. Associated rules are definitions, activated by
/// positive known instances of Seeded/Derived verbs; no state is executed.
/// As with `close`, the caller must ensure its program reaches a finite closure.
pub fn close_program<C: DerivationClock>(
    seeds: Vec<Event>,
    module: &Module,
    clock: &mut C,
) -> Result<Closure, crate::elab::Error> {
    use crate::elab::Error;
    if module
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Default(_)))
    {
        return Err(Error::new(
            "close_program requires a module without defaults",
        ));
    }
    let definitions: BTreeSet<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) => v.program.as_ref().map(|r| r.name.as_str()),
            _ => None,
        })
        .collect();
    let mut programs = Vec::new();
    for declaration in &module.declarations {
        let Declaration::Verb(verb) = declaration else {
            continue;
        };
        let Some(reference) = &verb.program else {
            continue;
        };
        let targets: Vec<_> = module
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) if r.name == reference.name => Some(r),
                _ => None,
            })
            .collect();
        let [rule] = targets.as_slice() else {
            return Err(Error::new(format!(
                "associated rule {} must resolve uniquely",
                reference.name
            )));
        };
        let expected: Vec<_> = verb
            .args
            .iter()
            .map(|a| a.ty.clone())
            .chain([crate::typed::Type::PropositionTime])
            .collect();
        if rule
            .parameters
            .iter()
            .map(|a| a.ty.clone())
            .collect::<Vec<_>>()
            != expected
        {
            return Err(Error::new(format!(
                "incompatible associated rule {}",
                rule.name
            )));
        }
        if matches!(verb.kind, VerbKind::Seeded | VerbKind::Derived) {
            programs.push((verb, *rule));
        }
    }
    let ordinary: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) if !definitions.contains(r.name.as_str()) => Some(r.clone()),
            _ => None,
        })
        .collect();
    Ok(close_with_programs(seeds, &ordinary, &programs, clock))
}

struct ActivatedProgram<'a> {
    rule: &'a Rule,
    trigger: Event,
    bindings: BTreeMap<String, SubstitutionValue>,
}

fn instantiate_program(
    program: &ActivatedProgram<'_>,
    known: &[Event],
) -> Vec<UnstampedDerivation> {
    let clause = match &program.rule.body {
        RuleBody::HornClause(clause) => Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => Cow::Owned(clause.unresiduate()),
    };
    match_antecedents_with(
        &clause.antecedent,
        known,
        &program.rule.parameters,
        program.bindings.clone(),
        Vec::new(),
    )
    .into_iter()
    .filter_map(|mut matched| {
        // The activation itself is evidence, even for a body equal to I.
        // Preserve existing body provenance when it already cites that fact.
        if !matched.antecedents.contains(&program.trigger) {
            matched.antecedents.insert(0, program.trigger.clone());
        }
        finish_instantiation(program.rule, &clause.consequent, matched)
    })
    .collect()
}

fn close_with_programs<C: DerivationClock>(
    seeds: Vec<Event>,
    rules: &[Rule],
    programs: &[(&Verb, &Rule)],
    clock: &mut C,
) -> Closure {
    let mut known_keys = BTreeSet::new();
    // Logical identity ignores transition metadata. Keep the first supplied
    // representation; interpreting or merging state transitions is separate.
    let mut known: Vec<Event> = seeds
        .into_iter()
        .filter(|event| known_keys.insert(event.clone()))
        .collect();
    let mut frontier = known.clone();
    let mut derivations = Vec::new();
    let mut seen_unstamped = Vec::new();
    let mut initial_round = true;
    let mut activated = Vec::new();

    // I may justify a ground consequent even when there are no input facts.
    while initial_round || !frontier.is_empty() {
        initial_round = false;
        let mut next_frontier = Vec::new();

        for trigger in &frontier {
            for (verb, rule) in programs {
                if trigger.verb != verb.name
                    || trigger.polarity != Polarity::Positive
                    || trigger.args.len() != verb.args.len()
                {
                    continue;
                }
                let mut bindings: BTreeMap<_, _> = rule
                    .parameters
                    .iter()
                    .zip(&trigger.args)
                    .map(|(parameter, value)| {
                        (
                            parameter.name.clone(),
                            SubstitutionValue::Term(value.clone()),
                        )
                    })
                    .collect();
                bindings.insert(
                    rule.parameters.last().unwrap().name.clone(),
                    SubstitutionValue::Time(trigger.proposition.time.clone()),
                );
                activated.push(ActivatedProgram {
                    rule,
                    trigger: trigger.clone(),
                    bindings,
                });
            }
        }

        let sources = rules.iter().map(|rule| (rule, None)).chain(
            activated
                .iter()
                .map(|program| (program.rule, Some(program))),
        );
        for (rule, program) in sources {
            let candidates = match program {
                Some(program) => instantiate_program(program, &known),
                None => instantiate_from_known(rule, &known),
            };
            for unstamped in candidates {
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
pub(crate) struct AntecedentMatch {
    pub(crate) antecedents: Vec<Event>,
    pub(crate) substitutions: BTreeMap<String, SubstitutionValue>,
}

pub(crate) fn match_antecedents(
    antecedent: &Antecedent,
    known: &[Event],
    parameters: &[Arg],
) -> Vec<AntecedentMatch> {
    match_antecedents_with(antecedent, known, parameters, BTreeMap::new(), Vec::new())
}

pub(crate) fn match_antecedents_with(
    antecedent: &Antecedent,
    known: &[Event],
    parameters: &[Arg],
    substitutions: BTreeMap<String, SubstitutionValue>,
    antecedents: Vec<Event>,
) -> Vec<AntecedentMatch> {
    match antecedent {
        Antecedent::Unit => vec![AntecedentMatch {
            antecedents,
            substitutions,
        }],
        Antecedent::Event(pattern) => known
            .iter()
            .filter_map(|event| {
                let mut substitutions = substitutions.clone();
                match_event(pattern, event, parameters, &mut substitutions)?;
                let mut antecedents = antecedents.clone();
                antecedents.push(event.clone());
                Some(AntecedentMatch {
                    antecedents,
                    substitutions,
                })
            })
            .collect(),
        Antecedent::And(left, right) => {
            match_antecedents_with(left, known, parameters, substitutions, antecedents)
                .into_iter()
                .flat_map(|matched| {
                    match_antecedents_with(
                        right,
                        known,
                        parameters,
                        matched.substitutions,
                        matched.antecedents,
                    )
                })
                .collect()
        }
    }
}

fn match_event(
    pattern: &Event,
    event: &Event,
    parameters: &[Arg],
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    if pattern.polarity != event.polarity
        || pattern.verb != event.verb
        || pattern.args.len() != event.args.len()
    {
        return None;
    }

    for (pattern, value) in pattern.args.iter().zip(&event.args) {
        bind_term(pattern, value, parameters, substitutions)?;
    }
    bind_time(
        &pattern.proposition.time,
        &event.proposition.time,
        parameters,
        substitutions,
    )
}

fn bind_term(
    pattern: &Term,
    value: &Term,
    parameters: &[Arg],
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    match pattern {
        Term::Var(name) if parameters.iter().any(|parameter| parameter.name == *name) => {
            bind(name, SubstitutionValue::Term(value.clone()), substitutions)
        }
        _ => (pattern == value).then_some(()),
    }
}

fn bind_time(
    pattern: &TimeExpr,
    value: &TimeExpr,
    parameters: &[Arg],
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    match (pattern, value) {
        (TimeExpr::At(name), _) if parameters.iter().any(|parameter| parameter.name == *name) => {
            bind(name, SubstitutionValue::Time(value.clone()), substitutions)
        }
        (TimeExpr::After(pattern_inner), TimeExpr::After(value_inner)) => {
            bind_time(pattern_inner, value_inner, parameters, substitutions)
        }
        _ => (pattern == value).then_some(()),
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

pub(crate) fn substitute_antecedent(
    antecedent: &Antecedent,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> Antecedent {
    match antecedent {
        Antecedent::Unit => Antecedent::Unit,
        Antecedent::Event(event) => Antecedent::Event(substitute_event(event, substitutions)),
        Antecedent::And(left, right) => Antecedent::And(
            Box::new(substitute_antecedent(left, substitutions)),
            Box::new(substitute_antecedent(right, substitutions)),
        ),
    }
}

pub(crate) fn substitute_event(
    event: &Event,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> Event {
    Event {
        polarity: event.polarity,
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

pub(crate) fn substitution_record(
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> Substitution {
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
