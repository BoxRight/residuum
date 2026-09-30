use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::surface::{
    CnlDeclaration, CnlModule, FormalDeclaration, FormalModule, SurfaceAntecedent,
    SurfaceDefaultBody, SurfaceEvent, SurfaceFieldAccess, SurfaceRuleBody, SurfaceStateOperation,
    SurfaceTerm, SurfaceTime, SurfaceVerbKind,
};
use crate::typed::{
    Antecedent, Arg, Constant, Declaration, Default, DefaultBody, Effect, Entity, Event, Field,
    FieldAccess, HornClause, Module, PropositionKind, PropositionType, RecordFieldValue, Rule,
    RuleBody, RuleRef, StateOperation, StateTime, StateTransitionType, Term, TimeExpr, Type, Verb,
    VerbKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerbSig {
    kind: VerbKind,
    arg_types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

pub fn elaborate_formal(module: FormalModule) -> Result<Module, Error> {
    let mut declarations = Vec::new();
    let mut verb_sigs = BTreeMap::new();
    let mut entities = BTreeSet::new();
    let mut entity_fields = BTreeMap::new();
    let mut rule_parameters = BTreeMap::new();
    let mut rules_by_name = BTreeMap::new();
    let mut constants = BTreeMap::new();

    for declaration in module.declarations {
        match declaration {
            FormalDeclaration::Entity(entity) => {
                entities.insert(entity.name.clone());
                let fields: Vec<_> = entity
                    .fields
                    .into_iter()
                    .map(|field| Field {
                        name: field.name,
                        ty: field.ty,
                    })
                    .collect();
                entity_fields.insert(entity.name.clone(), fields.clone());
                declarations.push(Declaration::Entity(Entity {
                    name: entity.name,
                    supertypes: entity.supertypes,
                    fields,
                }));
            }
            FormalDeclaration::Const(constant) => {
                collect_type_entities(&constant.ty, &mut entities);
                constants.insert(constant.name.clone(), constant.ty.clone());
                declarations.push(Declaration::Const(Constant {
                    name: constant.name,
                    ty: constant.ty,
                }));
            }
            FormalDeclaration::Verb(verb) => {
                let args: Vec<_> = verb
                    .args
                    .into_iter()
                    .map(|arg| Arg {
                        name: arg.name,
                        ty: arg.ty,
                    })
                    .collect();
                verb_sigs.insert(
                    verb.name.clone(),
                    VerbSig {
                        kind: lower_kind(verb.kind.clone()),
                        arg_types: args.iter().map(|arg| arg.ty.clone()).collect(),
                    },
                );
                let effect = verb
                    .effect
                    .map(|effect| lower_effect(effect.operations, &args, &entity_fields))
                    .transpose()?;
                declarations.push(Declaration::Verb(Verb {
                    name: verb.name,
                    kind: lower_kind(verb.kind),
                    args,
                    effect,
                }));
            }
            FormalDeclaration::Rule(rule) => {
                let lowered = lower_rule(
                    rule.name,
                    rule.body,
                    &verb_sigs,
                    &entities,
                    &constants,
                    &rules_by_name,
                )?;
                rule_parameters.insert(lowered.name.clone(), lowered.parameters.clone());
                rules_by_name.insert(lowered.name.clone(), lowered.clone());
                declarations.push(Declaration::Rule(lowered));
            }
            FormalDeclaration::Default(default) => {
                let SurfaceDefaultBody::Supernormal { rule } = default.body;
                let parameters = rule_parameters.get(&rule).cloned().ok_or_else(|| {
                    Error::new(format!("unknown rule `{rule}` in supernormal default"))
                })?;
                declarations.push(Declaration::Default(Default {
                    name: default.name,
                    parameters,
                    body: DefaultBody::Supernormal {
                        rule: RuleRef { name: rule },
                    },
                }));
            }
        }
    }

    Ok(Module {
        name: module.name,
        declarations,
    })
}

pub fn elaborate_cnl(module: CnlModule) -> Result<Module, Error> {
    let mut entities: BTreeMap<String, (Vec<String>, Vec<Field>)> = BTreeMap::new();
    let mut entity_mentions = BTreeSet::new();
    let mut verbs = Vec::new();
    let mut rules = Vec::new();
    let mut defaults = Vec::new();
    let mut verb_sigs = BTreeMap::new();
    let mut rule_parameters = BTreeMap::new();
    let mut rules_by_name = BTreeMap::new();
    let mut constants = BTreeMap::new();

    for declaration in module.declarations {
        match declaration {
            CnlDeclaration::Entity(entity) => {
                entity_mentions.insert(entity.name.clone());
                for supertype in &entity.supertypes {
                    entity_mentions.insert(supertype.clone());
                }
                entities
                    .entry(entity.name)
                    .or_default()
                    .0
                    .extend(entity.supertypes);
            }
            CnlDeclaration::Const(constant) => {
                collect_type_entities(&constant.ty, &mut entity_mentions);
                constants.insert(constant.name.clone(), constant.ty.clone());
                if let Type::Entity(name) = &constant.ty {
                    entities.entry(name.clone()).or_default();
                }
                verbs.push(Declaration::Const(Constant {
                    name: constant.name,
                    ty: constant.ty,
                }));
            }
            CnlDeclaration::Field(field) => {
                entity_mentions.insert(field.owner.clone());
                collect_type_entities(&field.ty, &mut entity_mentions);
                entities.entry(field.owner).or_default().1.push(Field {
                    name: field.name,
                    ty: field.ty,
                });
            }
            CnlDeclaration::Verb(verb) => {
                let args: Vec<_> = verb
                    .args
                    .into_iter()
                    .map(|arg| {
                        collect_type_entities(&arg.ty, &mut entity_mentions);
                        Arg {
                            name: arg.name,
                            ty: arg.ty,
                        }
                    })
                    .collect();
                verb_sigs.insert(
                    verb.name.clone(),
                    VerbSig {
                        kind: lower_kind(verb.kind.clone()),
                        arg_types: args.iter().map(|arg| arg.ty.clone()).collect(),
                    },
                );
                let entity_fields = cnl_entity_fields(&entities);
                let effect = verb
                    .effect
                    .map(|effect| lower_effect(effect.operations, &args, &entity_fields))
                    .transpose()?;
                verbs.push(Declaration::Verb(Verb {
                    name: verb.name,
                    kind: lower_kind(verb.kind),
                    args,
                    effect,
                }));
            }
            CnlDeclaration::Rule(rule) => {
                let lowered = lower_rule(
                    rule.name,
                    rule.body,
                    &verb_sigs,
                    &entity_mentions,
                    &constants,
                    &rules_by_name,
                )?;
                rule_parameters.insert(lowered.name.clone(), lowered.parameters.clone());
                rules_by_name.insert(lowered.name.clone(), lowered.clone());
                rules.push(Declaration::Rule(lowered));
            }
            CnlDeclaration::Default(default) => {
                let SurfaceDefaultBody::Supernormal { rule } = default.body;
                let parameters = rule_parameters.get(&rule).cloned().ok_or_else(|| {
                    Error::new(format!("unknown rule `{rule}` in supernormal default"))
                })?;
                defaults.push(Declaration::Default(Default {
                    name: default.name,
                    parameters,
                    body: DefaultBody::Supernormal {
                        rule: RuleRef { name: rule },
                    },
                }));
            }
        }
    }

    for name in entity_mentions {
        entities.entry(name).or_default();
    }

    let mut declarations = Vec::new();
    let mut entity_names: Vec<_> = entities.keys().cloned().collect();
    entity_names.sort_by_key(|name| {
        let has_fields = entities
            .get(name)
            .is_some_and(|(_, fields)| !fields.is_empty());
        (has_fields, name.clone())
    });
    for name in entity_names {
        let (supertypes, fields) = entities.remove(&name).unwrap_or_default();
        declarations.push(Declaration::Entity(Entity {
            name,
            supertypes,
            fields,
        }));
    }
    declarations.extend(verbs);
    declarations.extend(rules);
    declarations.extend(defaults);

    Ok(Module {
        name: module.name,
        declarations,
    })
}

fn lower_kind(kind: SurfaceVerbKind) -> VerbKind {
    match kind {
        SurfaceVerbKind::Seeded => VerbKind::Seeded,
        SurfaceVerbKind::Effect => VerbKind::Effect,
    }
}

fn lower_rule(
    name: String,
    body: SurfaceRuleBody,
    verb_sigs: &BTreeMap<String, VerbSig>,
    entities: &BTreeSet<String>,
    constants: &BTreeMap<String, Type>,
    rules_by_name: &BTreeMap<String, Rule>,
) -> Result<Rule, Error> {
    match body {
        SurfaceRuleBody::Horn {
            mut params,
            premise,
            conclusion,
        } => {
            if params.is_empty() {
                collect_antecedent_params(&premise, &mut params);
                params.extend(conclusion.args.iter().cloned());
                if let Some(time) = &conclusion.time {
                    params.push(time_name(time));
                }
                dedup(&mut params);
            }
            let parameters =
                infer_rule_params(&params, &premise, &conclusion, verb_sigs, entities)?;
            let parameter_names = parameters
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            Ok(Rule {
                name,
                parameters,
                body: RuleBody::HornClause(HornClause {
                    antecedent: lower_antecedent(premise, verb_sigs, constants, &parameter_names)?,
                    consequent: lower_event(conclusion, verb_sigs, constants, &parameter_names)?,
                }),
            })
        }
        SurfaceRuleBody::Application { rule, args } => {
            let base = rules_by_name
                .get(&rule)
                .ok_or_else(|| Error::new(format!("unknown rule `{rule}` in rule application")))?;
            if args.len() > base.parameters.len() {
                return Err(Error::new(format!(
                    "rule `{rule}` expects at most {} applied args, got {}",
                    base.parameters.len(),
                    args.len()
                )));
            }
            let mut substitution = BTreeMap::new();
            for (arg, parameter) in args.iter().zip(&base.parameters) {
                let actual_ty = constants.get(arg).ok_or_else(|| {
                    Error::new(format!("unknown constant `{arg}` in rule application"))
                })?;
                if actual_ty != &parameter.ty {
                    return Err(Error::new(format!(
                        "constant `{arg}` is not assignable to rule parameter `{}`",
                        parameter.name
                    )));
                }
                substitution.insert(parameter.name.clone(), Term::Const(arg.clone()));
            }
            Ok(Rule {
                name,
                parameters: base.parameters[args.len()..].to_vec(),
                body: substitute_rule_body(&base.body, &substitution),
            })
        }
    }
}

fn lower_antecedent(
    antecedent: SurfaceAntecedent,
    verb_sigs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
    parameters: &BTreeSet<String>,
) -> Result<Antecedent, Error> {
    match antecedent {
        SurfaceAntecedent::Event(event) => Ok(Antecedent::Event(lower_event(
            event, verb_sigs, constants, parameters,
        )?)),
        SurfaceAntecedent::And(left, right) => Ok(Antecedent::And(
            Box::new(lower_antecedent(*left, verb_sigs, constants, parameters)?),
            Box::new(lower_antecedent(*right, verb_sigs, constants, parameters)?),
        )),
    }
}

fn lower_event(
    event: SurfaceEvent,
    verb_sigs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
    parameters: &BTreeSet<String>,
) -> Result<Event, Error> {
    let signature = verb_sigs
        .get(&event.verb)
        .ok_or_else(|| Error::new(format!("unknown verb `{}`", event.verb)))?;
    let time = event
        .time
        .ok_or_else(|| Error::new(format!("unresolved PropositionTime for `{}`", event.verb)))?;

    Ok(Event {
        verb: event.verb,
        args: event
            .args
            .into_iter()
            .map(|name| {
                if constants.contains_key(&name) && !parameters.contains(&name) {
                    Term::Const(name)
                } else {
                    Term::Var(name)
                }
            })
            .collect(),
        proposition: PropositionType {
            kind: match signature.kind {
                VerbKind::Seeded => PropositionKind::Seeded,
                VerbKind::Effect => PropositionKind::Effect,
            },
            time: proposition_time(&time),
        },
        transition: effect_transition(signature.kind, &time),
    })
}

fn substitute_rule_body(body: &RuleBody, substitution: &BTreeMap<String, Term>) -> RuleBody {
    match body {
        RuleBody::HornClause(clause) => RuleBody::HornClause(HornClause {
            antecedent: substitute_antecedent(&clause.antecedent, substitution),
            consequent: substitute_event_terms(&clause.consequent, substitution),
        }),
    }
}

fn substitute_antecedent(
    antecedent: &Antecedent,
    substitution: &BTreeMap<String, Term>,
) -> Antecedent {
    match antecedent {
        Antecedent::Event(event) => Antecedent::Event(substitute_event_terms(event, substitution)),
        Antecedent::And(left, right) => Antecedent::And(
            Box::new(substitute_antecedent(left, substitution)),
            Box::new(substitute_antecedent(right, substitution)),
        ),
    }
}

fn substitute_event_terms(event: &Event, substitution: &BTreeMap<String, Term>) -> Event {
    Event {
        verb: event.verb.clone(),
        args: event
            .args
            .iter()
            .map(|term| substitute_term(term, substitution))
            .collect(),
        proposition: event.proposition.clone(),
        transition: event.transition.clone(),
    }
}

fn substitute_term(term: &Term, substitution: &BTreeMap<String, Term>) -> Term {
    match term {
        Term::Var(name) => substitution
            .get(name)
            .cloned()
            .unwrap_or_else(|| Term::Var(name.clone())),
        Term::Const(name) => Term::Const(name.clone()),
        Term::None => Term::None,
        Term::RecordLiteral { entity, fields } => Term::RecordLiteral {
            entity: entity.clone(),
            fields: fields
                .iter()
                .map(|field| RecordFieldValue {
                    field: field.field.clone(),
                    value: substitute_term(&field.value, substitution),
                })
                .collect(),
        },
    }
}

fn proposition_time(time: &SurfaceTime) -> TimeExpr {
    match time {
        SurfaceTime::At(time) => TimeExpr::At(time.clone()),
        SurfaceTime::After(time) => TimeExpr::After(Box::new(TimeExpr::At(time.clone()))),
    }
}

fn effect_transition(kind: VerbKind, time: &SurfaceTime) -> Option<StateTransitionType> {
    if kind != VerbKind::Effect {
        return None;
    }

    Some(match time {
        SurfaceTime::At(time) => StateTransitionType {
            input: StateTime::Unresolved,
            output: StateTime::At(TimeExpr::At(time.clone())),
        },
        SurfaceTime::After(time) => StateTransitionType {
            input: StateTime::At(TimeExpr::At(time.clone())),
            output: StateTime::At(TimeExpr::After(Box::new(TimeExpr::At(time.clone())))),
        },
    })
}

fn lower_effect(
    operations: Vec<SurfaceStateOperation>,
    args: &[Arg],
    entity_fields: &BTreeMap<String, Vec<Field>>,
) -> Result<Effect, Error> {
    let operations = operations
        .into_iter()
        .map(|operation| lower_state_operation(operation, args, entity_fields))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Effect::StateTransform {
        transition: StateTransitionType {
            input: StateTime::Unresolved,
            output: StateTime::Unresolved,
        },
        operations,
    })
}

fn lower_state_operation(
    operation: SurfaceStateOperation,
    args: &[Arg],
    entity_fields: &BTreeMap<String, Vec<Field>>,
) -> Result<StateOperation, Error> {
    match operation {
        SurfaceStateOperation::AddToSet { target, value } => {
            let (target, value) = check_set_operation(target, value, args, entity_fields)?;
            Ok(StateOperation::AddToSet { target, value })
        }
        SurfaceStateOperation::RemoveFromSet { target, value } => {
            let (target, value) = check_set_operation(target, value, args, entity_fields)?;
            Ok(StateOperation::RemoveFromSet { target, value })
        }
    }
}

fn check_set_operation(
    target: SurfaceFieldAccess,
    value: SurfaceTerm,
    args: &[Arg],
    entity_fields: &BTreeMap<String, Vec<Field>>,
) -> Result<(FieldAccess, Term), Error> {
    let base_ty = lookup_arg_type(&target.base, args)?;
    let Type::Entity(entity_name) = base_ty else {
        return Err(Error::new(format!(
            "`{}.{}` requires entity base, got {:?}",
            target.base,
            target.path.join("."),
            base_ty
        )));
    };
    let field_ty = resolve_field_path(entity_name, &target.path, entity_fields)?;
    let Type::Set(member_ty) = field_ty else {
        return Err(Error::new(format!(
            "`{}.{}` is not a set field",
            target.base,
            target.path.join(".")
        )));
    };
    let (value, value_ty) = lower_term(value, args, entity_fields)?;
    if !is_assignable(&value_ty, member_ty.as_ref()) {
        return Err(Error::new(format!(
            "cannot update `{}.{}` with `{}`",
            target.base,
            target.path.join("."),
            describe_term(&value)
        )));
    }

    Ok((
        FieldAccess {
            base: Term::Var(target.base),
            path: target.path,
        },
        value,
    ))
}

fn lower_term(
    term: SurfaceTerm,
    args: &[Arg],
    entity_fields: &BTreeMap<String, Vec<Field>>,
) -> Result<(Term, Type), Error> {
    match term {
        SurfaceTerm::Var(name) => {
            let ty = lookup_arg_type(&name, args)?.clone();
            Ok((Term::Var(name), ty))
        }
        SurfaceTerm::None => Ok((
            Term::None,
            Type::Optional(Box::new(Type::Entity("Never".to_string()))),
        )),
        SurfaceTerm::RecordLiteral { entity, fields } => {
            let declared_fields = entity_fields
                .get(&entity)
                .ok_or_else(|| Error::new(format!("unknown record entity `{entity}`")))?;
            let mut provided = BTreeMap::new();
            for field in fields {
                if provided.insert(field.field.clone(), field.value).is_some() {
                    return Err(Error::new(format!("duplicate field `{}`", field.field)));
                }
            }

            let mut lowered = Vec::new();
            for declared in declared_fields {
                let value = provided.remove(&declared.name).ok_or_else(|| {
                    Error::new(format!(
                        "missing field `{}` in `{entity}` literal",
                        declared.name
                    ))
                })?;
                let (term, value_ty) = lower_term(value, args, entity_fields)?;
                if !is_assignable(&value_ty, &declared.ty) {
                    return Err(Error::new(format!(
                        "cannot assign `{}` to field `{}.{}`",
                        describe_term(&term),
                        entity,
                        declared.name
                    )));
                }
                lowered.push(RecordFieldValue {
                    field: declared.name.clone(),
                    value: term,
                });
            }

            if let Some((field, _)) = provided.into_iter().next() {
                return Err(Error::new(format!("unknown field `{entity}.{field}`")));
            }

            Ok((
                Term::RecordLiteral {
                    entity: entity.clone(),
                    fields: lowered,
                },
                Type::Entity(entity),
            ))
        }
    }
}

fn is_assignable(actual: &Type, expected: &Type) -> bool {
    if actual == expected {
        return true;
    }

    match (actual, expected) {
        (Type::Optional(_), Type::Optional(_)) if matches_none_type(actual) => true,
        (_, Type::Optional(inner)) => is_assignable(actual, inner),
        _ => false,
    }
}

fn matches_none_type(ty: &Type) -> bool {
    matches!(ty, Type::Optional(inner) if matches!(inner.as_ref(), Type::Entity(name) if name == "Never"))
}

fn describe_term(term: &Term) -> String {
    match term {
        Term::Var(name) => name.clone(),
        Term::Const(name) => name.clone(),
        Term::None => "none".to_string(),
        Term::RecordLiteral { entity, .. } => format!("{entity} {{ ... }}"),
    }
}

fn resolve_field_path<'a>(
    entity_name: &str,
    path: &[String],
    entity_fields: &'a BTreeMap<String, Vec<Field>>,
) -> Result<&'a Type, Error> {
    let mut current_entity = entity_name;
    let mut current_ty = None;

    for (index, segment) in path.iter().enumerate() {
        let fields = entity_fields
            .get(current_entity)
            .ok_or_else(|| Error::new(format!("unknown entity `{current_entity}`")))?;
        let field = fields
            .iter()
            .find(|field| field.name == *segment)
            .ok_or_else(|| Error::new(format!("unknown field `{}.{}`", current_entity, segment)))?;
        current_ty = Some(&field.ty);

        if index + 1 < path.len() {
            let Type::Entity(next_entity) = &field.ty else {
                return Err(Error::new(format!(
                    "`{}` in `{}` does not resolve to an entity",
                    segment,
                    path.join(".")
                )));
            };
            current_entity = next_entity;
        }
    }

    current_ty.ok_or_else(|| Error::new("empty field path"))
}

fn lookup_arg_type<'a>(name: &str, args: &'a [Arg]) -> Result<&'a Type, Error> {
    args.iter()
        .find(|arg| arg.name == name)
        .map(|arg| &arg.ty)
        .ok_or_else(|| Error::new(format!("unknown effect argument `{name}`")))
}

fn infer_rule_params(
    requested: &[String],
    premise: &SurfaceAntecedent,
    conclusion: &SurfaceEvent,
    verb_sigs: &BTreeMap<String, VerbSig>,
    entities: &BTreeSet<String>,
) -> Result<Vec<Arg>, Error> {
    let mut inferred = BTreeMap::new();
    seed_role_types(requested, entities, &mut inferred);
    infer_antecedent_args(premise, verb_sigs, &mut inferred)?;
    infer_event_args(conclusion, verb_sigs, &mut inferred)?;
    infer_antecedent_times(premise, &mut inferred);
    if let Some(time) = &conclusion.time {
        inferred.insert(time_name(time), Type::PropositionTime);
    }

    requested
        .iter()
        .map(|name| {
            let ty = inferred.get(name).cloned().ok_or_else(|| {
                Error::new(format!("could not infer type for rule parameter `{name}`"))
            })?;
            Ok(Arg {
                name: name.clone(),
                ty,
            })
        })
        .collect()
}

fn collect_antecedent_params(antecedent: &SurfaceAntecedent, params: &mut Vec<String>) {
    match antecedent {
        SurfaceAntecedent::Event(event) => {
            params.extend(event.args.iter().cloned());
            if let Some(time) = &event.time {
                params.push(time_name(time));
            }
        }
        SurfaceAntecedent::And(left, right) => {
            collect_antecedent_params(left, params);
            collect_antecedent_params(right, params);
        }
    }
}

fn infer_antecedent_args(
    antecedent: &SurfaceAntecedent,
    verb_sigs: &BTreeMap<String, VerbSig>,
    inferred: &mut BTreeMap<String, Type>,
) -> Result<(), Error> {
    match antecedent {
        SurfaceAntecedent::Event(event) => infer_event_args(event, verb_sigs, inferred),
        SurfaceAntecedent::And(left, right) => {
            infer_antecedent_args(left, verb_sigs, inferred)?;
            infer_antecedent_args(right, verb_sigs, inferred)
        }
    }
}

fn infer_antecedent_times(antecedent: &SurfaceAntecedent, inferred: &mut BTreeMap<String, Type>) {
    match antecedent {
        SurfaceAntecedent::Event(event) => {
            if let Some(time) = &event.time {
                inferred.insert(time_name(time), Type::PropositionTime);
            }
        }
        SurfaceAntecedent::And(left, right) => {
            infer_antecedent_times(left, inferred);
            infer_antecedent_times(right, inferred);
        }
    }
}

fn infer_event_args(
    event: &SurfaceEvent,
    verb_sigs: &BTreeMap<String, VerbSig>,
    inferred: &mut BTreeMap<String, Type>,
) -> Result<(), Error> {
    let signature = verb_sigs
        .get(&event.verb)
        .ok_or_else(|| Error::new(format!("unknown verb `{}`", event.verb)))?;
    if signature.arg_types.len() != event.args.len() {
        return Err(Error::new(format!(
            "verb `{}` expects {} args, got {}",
            event.verb,
            signature.arg_types.len(),
            event.args.len()
        )));
    }

    for (name, ty) in event.args.iter().zip(&signature.arg_types) {
        if let Some(existing) = inferred.get(name) {
            if existing == ty || is_assignable(existing, ty) {
                continue;
            }
            if is_assignable(ty, existing) {
                inferred.insert(name.clone(), ty.clone());
                continue;
            }
            return Err(Error::new(format!(
                "conflicting inferred types for `{name}`"
            )));
        } else {
            inferred.insert(name.clone(), ty.clone());
        }
    }

    Ok(())
}

fn collect_type_entities(ty: &Type, entities: &mut BTreeSet<String>) {
    match ty {
        Type::Entity(name) => {
            entities.insert(name.clone());
        }
        Type::Set(inner) => collect_type_entities(inner, entities),
        Type::Optional(inner) => collect_type_entities(inner, entities),
        Type::PropositionTime | Type::DerivationTime => {}
    }
}

fn cnl_entity_fields(
    entities: &BTreeMap<String, (Vec<String>, Vec<Field>)>,
) -> BTreeMap<String, Vec<Field>> {
    entities
        .iter()
        .map(|(name, (_, fields))| (name.clone(), fields.clone()))
        .collect()
}

fn seed_role_types(
    requested: &[String],
    entities: &BTreeSet<String>,
    inferred: &mut BTreeMap<String, Type>,
) {
    for name in requested {
        if let Some(entity) = entities
            .iter()
            .find(|entity| name == &entity.to_lowercase())
        {
            inferred.insert(name.clone(), Type::Entity(entity.clone()));
        }
    }
}

fn time_name(time: &SurfaceTime) -> String {
    match time {
        SurfaceTime::At(time) | SurfaceTime::After(time) => time.clone(),
    }
}

fn dedup(items: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    items.retain(|item| seen.insert(item.clone()));
}
