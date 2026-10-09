use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

mod incremental;
mod known_index;
use known_index::{KnownIndex, KnownView};
#[cfg(test)]
mod profile;
#[cfg(test)]
mod profile_tests;
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
    instantiate_from_view(rule, KnownView::linear(known))
}

fn instantiate_from_view(rule: &Rule, known: KnownView<'_>) -> Vec<UnstampedDerivation> {
    // Forward inference consumes the Horn view; it does not assert or query a residual.
    let clause = match &rule.body {
        RuleBody::HornClause(clause) => Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => Cow::Owned(clause.unresiduate()),
    };
    match_antecedents_in(
        &clause.antecedent,
        known,
        &rule.parameters,
        BTreeMap::new(),
        Vec::new(),
    )
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
            parameter_names.contains(name.as_str()) && !substitutions.contains_key(name)
        }
        Term::Constructor { argument, .. } => {
            term_has_unbound_parameter(argument, parameter_names, substitutions)
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
        TimeExpr::At(_) => false,
        TimeExpr::Var(name) => substitutions
            .get(name)
            .and_then(SubstitutionValue::as_time)
            .is_none(),
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
    #[cfg(test)]
    let preparation = profile::start();
    if module
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Default(_)))
    {
        return Err(Error::new(
            "close_program requires a module without defaults",
        ));
    }
    let mut definitions = BTreeSet::new();
    let mut programs = Vec::new();
    for declaration in &module.declarations {
        let Declaration::Verb(verb) = declaration else {
            continue;
        };
        let Some(reference) = &verb.program else {
            continue;
        };
        let (parameters, rules) = match reference.kind {
            crate::typed::BehaviorKind::Rule => {
                let rule = module
                    .declarations
                    .iter()
                    .find_map(|d| match d {
                        Declaration::Rule(r) if r.name == reference.name => Some(r),
                        _ => None,
                    })
                    .ok_or_else(|| Error::new("unresolved associated rule"))?;
                definitions.insert(rule.name.clone());
                (rule.parameters.clone(), vec![rule.clone()])
            }
            crate::typed::BehaviorKind::Program => {
                let program = module
                    .declarations
                    .iter()
                    .find_map(|d| match d {
                        Declaration::Program(p) if p.identity == reference.name => Some(p),
                        _ => None,
                    })
                    .ok_or_else(|| Error::new("unresolved associated program"))?;
                definitions.extend(program.rule_uses.iter().map(|u| u.rule.name.clone()));
                let parameters = program
                    .parameters
                    .iter()
                    .map(|p| Arg {
                        name: crate::programs::shared_parameter(program, &p.name),
                        ty: p.ty.clone(),
                    })
                    .collect();
                (
                    parameters,
                    crate::programs::compile_program(program, module)?,
                )
            }
        };
        let expected: Vec<_> = verb
            .args
            .iter()
            .map(|a| a.ty.clone())
            .chain([crate::typed::Type::PropositionTime])
            .collect();
        if parameters.iter().map(|a| a.ty.clone()).collect::<Vec<_>>() != expected {
            return Err(Error::new(format!(
                "incompatible associated behavior {}",
                reference.name
            )));
        }
        if matches!(verb.kind, VerbKind::Seeded | VerbKind::Derived) {
            programs.push(ProgramSource {
                verb,
                parameters,
                rules,
            });
        }
    }
    let ordinary: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) if !definitions.contains(&r.name) => Some(r.clone()),
            _ => None,
        })
        .collect();
    #[cfg(test)]
    profile::phase("preparation", preparation);
    Ok(close_with_programs(seeds, &ordinary, &programs, clock))
}

struct ProgramSource<'a> {
    verb: &'a Verb,
    parameters: Vec<Arg>,
    rules: Vec<Rule>,
}

struct ActivatedProgram<'a> {
    rule: &'a Rule,
    trigger: Event,
    bindings: BTreeMap<String, SubstitutionValue>,
}

fn instantiate_program(
    program: &ActivatedProgram<'_>,
    known: KnownView<'_>,
) -> Vec<UnstampedDerivation> {
    let clause = match &program.rule.body {
        RuleBody::HornClause(clause) => Cow::Borrowed(clause),
        RuleBody::ResidualClause(clause) => Cow::Owned(clause.unresiduate()),
    };
    match_antecedents_in(
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
    programs: &[ProgramSource<'_>],
    clock: &mut C,
) -> Closure {
    let mut known_keys = BTreeSet::new();
    // Logical identity ignores transition metadata. Keep the first supplied
    // representation; interpreting or merging state transitions is separate.
    let mut known: Vec<Event> = seeds
        .into_iter()
        .filter(|event| known_keys.insert(event.clone()))
        .collect();
    let mut known_index = KnownIndex::new(&known);
    let mut frontier = known.clone();
    let mut derivations = Vec::new();
    let mut seen_unstamped = Vec::new();
    let mut initial_round = true;
    let mut activated = Vec::new();

    // I may justify a ground consequent even when there are no input facts.
    while initial_round || !frontier.is_empty() {
        initial_round = false;
        let mut next_frontier = Vec::new();

        #[cfg(test)]
        let activation = profile::start();
        for trigger in &frontier {
            for source in programs {
                let verb = source.verb;
                if trigger.verb != verb.name
                    || trigger.polarity != Polarity::Positive
                    || trigger.args.len() != verb.args.len()
                {
                    continue;
                }
                let mut bindings = BTreeMap::new();
                let valid =
                    source
                        .parameters
                        .iter()
                        .zip(&trigger.args)
                        .all(|(parameter, value)| {
                            bind_typed(
                                parameter,
                                SubstitutionValue::Term(value.clone()),
                                &mut bindings,
                            )
                            .is_some()
                        })
                        && bind_typed(
                            source.parameters.last().unwrap(),
                            SubstitutionValue::Time(trigger.proposition.time.clone()),
                            &mut bindings,
                        )
                        .is_some();
                if !valid {
                    continue;
                }
                for rule in &source.rules {
                    activated.push(ActivatedProgram {
                        rule,
                        trigger: trigger.clone(),
                        bindings: bindings.clone(),
                    });
                }
            }
        }

        #[cfg(test)]
        profile::activation(activation, frontier.len(), activated.len());
        let sources = rules.iter().map(|rule| (rule, None)).chain(
            activated
                .iter()
                .map(|program| (program.rule, Some(program))),
        );
        for (rule, program) in sources {
            #[cfg(test)]
            let matching = profile::start();
            let evidence = KnownView {
                events: &known,
                index: Some(&known_index),
            };
            #[cfg(test)]
            let evidence = if profile::scan_known() {
                KnownView::linear(&known)
            } else {
                evidence
            };
            let candidates = match program {
                Some(program) => instantiate_program(program, evidence),
                None => instantiate_from_view(rule, evidence),
            };
            #[cfg(test)]
            profile::rule(&rule.name, program.is_some(), matching, candidates.len());
            for unstamped in candidates {
                #[cfg(test)]
                let duplicate = profile::contains(&seen_unstamped, &unstamped);
                #[cfg(not(test))]
                let duplicate = seen_unstamped.contains(&unstamped);
                if duplicate {
                    continue;
                }
                #[cfg(test)]
                let retention = profile::start();
                seen_unstamped.push(unstamped.clone());
                let is_new = known_keys.insert(unstamped.event.clone());
                let derived = stamp(unstamped, clock.next_derivation_time());
                if is_new {
                    known_index.insert(known.len(), &derived.event);
                    known.push(derived.event.clone());
                    next_frontier.push(derived.event.clone());
                }
                derivations.push(derived);
                #[cfg(test)]
                profile::phase("retention", retention);
            }
        }

        #[cfg(test)]
        profile::round(known.len(), derivations.len(), next_frontier.len());
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
    // Canonicalize externally supplied constraints before even a Unit match.
    let mut canonical = BTreeMap::new();
    for (name, value) in substitutions {
        let Some(parameter) = parameters.iter().find(|p| p.name == name) else {
            return Vec::new();
        };
        if bind_typed(parameter, value, &mut canonical).is_none() {
            return Vec::new();
        }
    }
    match_antecedents_in(
        antecedent,
        KnownView::linear(known),
        parameters,
        canonical,
        antecedents,
    )
}

fn match_antecedents_in(
    antecedent: &Antecedent,
    known: KnownView<'_>,
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
            .candidates(pattern)
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
            match_antecedents_in(left, known, parameters, substitutions, antecedents)
                .into_iter()
                .flat_map(|matched| {
                    match_antecedents_in(
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
    #[cfg(test)]
    profile::probe(false);
    if pattern.polarity != event.polarity
        || pattern.verb != event.verb
        || pattern.args.len() != event.args.len()
    {
        #[cfg(test)]
        profile::probe(true);
        return None;
    }

    for (pattern, value) in pattern.args.iter().zip(&event.args) {
        if bind_term(pattern, value, parameters, substitutions).is_none() {
            #[cfg(test)]
            profile::structural_reject();
            return None;
        }
    }
    let result = bind_time(
        &pattern.proposition.time,
        &event.proposition.time,
        parameters,
        substitutions,
    );
    #[cfg(test)]
    if result.is_none() {
        profile::structural_reject();
    }
    result
}

fn bind_term(
    pattern: &Term,
    value: &Term,
    parameters: &[Arg],
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    match pattern {
        Term::Var(name) if parameters.iter().any(|parameter| parameter.name == *name) => {
            if matches!(value, Term::Constructor { .. }) && !value.is_ground() {
                return None;
            }
            bind_typed(
                parameters.iter().find(|p| p.name == *name)?,
                SubstitutionValue::Term(value.clone()),
                substitutions,
            )
        }
        Term::Constructor {
            constructor,
            argument,
        } => match value {
            Term::Constructor {
                constructor: other,
                argument: actual,
            } if constructor == other && value.is_ground() => {
                bind_term(argument, actual, parameters, substitutions)
            }
            _ => None,
        },
        Term::RecordLiteral { entity, fields } => match value {
            Term::RecordLiteral {
                entity: other,
                fields: actual,
            } if entity == other && fields.len() == actual.len() => {
                for (p, v) in fields.iter().zip(actual) {
                    if p.field != v.field {
                        return None;
                    }
                    bind_term(&p.value, &v.value, parameters, substitutions)?;
                }
                Some(())
            }
            _ => None,
        },
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
        (TimeExpr::Var(name), _) if parameters.iter().any(|parameter| parameter.name == *name) => {
            bind_typed(
                parameters.iter().find(|p| p.name == *name)?,
                SubstitutionValue::Time(value.clone()),
                substitutions,
            )
        }
        (TimeExpr::After(pattern_inner), TimeExpr::After(value_inner)) => {
            bind_time(pattern_inner, value_inner, parameters, substitutions)
        }
        _ => (pattern == value).then_some(()),
    }
}

/// Shared insertion boundary for ordinary, pivoted and residual matching.
/// Stored values are canonical before substitution, provenance and deduplication.
pub(crate) fn bind_typed(
    parameter: &Arg,
    value: SubstitutionValue,
    substitutions: &mut BTreeMap<String, SubstitutionValue>,
) -> Option<()> {
    let value = value.canonical_for_type(&parameter.ty)?;
    if let Some(existing) = substitutions.get(&parameter.name) {
        if existing.canonical_for_type(&parameter.ty)? != value {
            return None;
        }
    }
    substitutions.insert(parameter.name.clone(), value);
    Some(())
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

pub(crate) fn substitute_term(
    term: &Term,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> Term {
    match term {
        Term::Var(name) => match substitutions.get(name) {
            Some(value) => value.as_term(),
            _ => Term::Var(name.clone()),
        },
        Term::Constructor {
            constructor,
            argument,
        } => Term::Constructor {
            constructor: *constructor,
            argument: Box::new(substitute_term(argument, substitutions)),
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

pub(crate) fn substitute_time(
    time: &TimeExpr,
    substitutions: &BTreeMap<String, SubstitutionValue>,
) -> TimeExpr {
    match time {
        TimeExpr::At(_) => time.clone(),
        TimeExpr::Var(name) => substitutions
            .get(name)
            .and_then(SubstitutionValue::as_time)
            .unwrap_or_else(|| time.clone()),
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
