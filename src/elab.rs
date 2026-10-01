use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::derive::{substitute_antecedent, substitute_event};
use crate::surface::{
    CnlDeclaration, CnlModule, FormalDeclaration, FormalModule, SurfaceAntecedent,
    SurfaceDefaultBody, SurfaceEvent, SurfaceFieldAccess, SurfaceRuleBody, SurfaceStateOperation,
    SurfaceTerm, SurfaceTime, SurfaceVerbKind,
};
use crate::typed::{
    Antecedent, Arg, Constant, Declaration, Default, DefaultBody, DefaultRef, Effect, Entity,
    Event, Experiment, ExperimentInputKind, ExperimentQueryKind, Field, FieldAccess, HornClause,
    Module, Polarity, PropositionKind, PropositionType, RecordFieldValue, Residual, ResidualClause,
    Rule, RuleBody, RuleRef, StateOperation, StateTime, StateTransitionType, SubstitutionValue,
    Term, TimeExpr, Type, Verb, VerbKind,
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
    let mut defaults_by_name = BTreeMap::new();
    let mut experiments = Vec::new();

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
                if matches!(verb.kind, SurfaceVerbKind::Derived) && verb.effect.is_some() {
                    return Err(Error::new(format!(
                        "derived verb `{}` cannot have a StateTransform",
                        verb.name
                    )));
                }
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
                    program: verb.program.map(|name| RuleRef { name }),
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
                    &declarations,
                )?;
                rule_parameters.insert(lowered.name.clone(), lowered.parameters.clone());
                rules_by_name.insert(lowered.name.clone(), lowered.clone());
                declarations.push(Declaration::Rule(lowered));
            }
            FormalDeclaration::Default(default) => {
                let lowered = lower_default(
                    default.name,
                    default.body,
                    &rule_parameters,
                    &defaults_by_name,
                )?;
                let mut lowered = lowered;
                if let Some(condition) = default.condition {
                    if matches!(lowered.body, DefaultBody::Supernormal { .. }) {
                        return Err(Error::new(
                            "supernormal defaults do not support a condition",
                        ));
                    }
                    check_default_condition(
                        &condition,
                        &lowered.parameters,
                        &verb_sigs,
                        &constants,
                    )?;
                    let parameter_names = lowered
                        .parameters
                        .iter()
                        .map(|arg| arg.name.clone())
                        .collect();
                    lowered.condition = Some(lower_antecedent(
                        condition,
                        &verb_sigs,
                        &constants,
                        &parameter_names,
                    )?);
                }
                if let Some(blocking) = default.blocking {
                    check_default_condition(
                        &blocking,
                        &lowered.parameters,
                        &verb_sigs,
                        &constants,
                    )?;
                    let parameter_names = lowered
                        .parameters
                        .iter()
                        .map(|arg| arg.name.clone())
                        .collect();
                    lowered.blocking = Some(lower_antecedent(
                        blocking,
                        &verb_sigs,
                        &constants,
                        &parameter_names,
                    )?);
                }
                if defaults_by_name.contains_key(&lowered.name) {
                    return Err(Error::new(format!("duplicate default `{}`", lowered.name)));
                }
                defaults_by_name.insert(lowered.name.clone(), lowered.clone());
                declarations.push(Declaration::Default(lowered));
            }
            FormalDeclaration::Experiment(experiment) => experiments.push(experiment),
        }
    }

    // Verb declarations precede their rules because rules use verb signatures.
    // Resolve only this new link after elaboration; other name/order policies
    // stay unchanged. A reference retains the whole parametrized lambda.
    for declaration in &declarations {
        let Declaration::Verb(verb) = declaration else {
            continue;
        };
        let Some(reference) = &verb.program else {
            continue;
        };
        if declarations
            .iter()
            .filter(|d| {
                matches!(d,
            Declaration::Rule(rule) if rule.name == reference.name)
            })
            .count()
            > 1
        {
            return Err(Error::new(format!(
                "ambiguous associated rule `{}` for verb `{}`",
                reference.name, verb.name
            )));
        }
        let program = rules_by_name.get(&reference.name).ok_or_else(|| {
            Error::new(format!(
                "unknown associated rule `{}` for verb `{}`",
                reference.name, verb.name
            ))
        })?;
        let mut expected: Vec<_> = verb.args.iter().map(|arg| arg.ty.clone()).collect();
        expected.push(Type::PropositionTime);
        let actual: Vec<_> = program
            .parameters
            .iter()
            .map(|arg| arg.ty.clone())
            .collect();
        if actual != expected {
            return Err(Error::new(format!(
                "associated rule `{}` for verb `{}` has parameter types {:?}; expected {:?}",
                reference.name, verb.name, actual, expected
            )));
        }
    }

    let mut result = Module {
        name: module.name,
        declarations,
    };
    let mut names = BTreeSet::new();
    for experiment in experiments {
        if !names.insert(experiment.name.clone()) {
            return Err(Error::new(format!(
                "duplicate experiment `{}`",
                experiment.name
            )));
        }
        let compatible = matches!(
            (experiment.input_kind, experiment.query_kind),
            (ExperimentInputKind::Seeds, ExperimentQueryKind::Residual)
                | (
                    ExperimentInputKind::Premises,
                    ExperimentQueryKind::FourFusion
                )
        );
        if !compatible {
            return Err(Error::new(
                "residual experiments require seeds; fourFusion comparisons require logical premises",
            ));
        }
        let input = experiment
            .input
            .into_iter()
            .map(|event| {
                let lowered = lower_ground_event(event, &verb_sigs, &constants, &result)?;
                if experiment.input_kind == ExperimentInputKind::Seeds
                    && lowered.proposition.kind != PropositionKind::Seeded
                {
                    return Err(Error::new("experiment seeds must use seeded verbs"));
                }
                Ok(lowered)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let goal = lower_ground_event(experiment.goal, &verb_sigs, &constants, &result)?;
        result
            .declarations
            .push(Declaration::Experiment(Experiment {
                name: experiment.name,
                input_kind: experiment.input_kind,
                input,
                query_kind: experiment.query_kind,
                goal,
            }));
    }
    Ok(result)
}

fn lower_ground_event(
    event: SurfaceEvent,
    verbs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
    module: &Module,
) -> Result<Event, Error> {
    let signature = verbs
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
    for (name, expected) in event.args.iter().zip(&signature.arg_types) {
        let actual = constants
            .get(name)
            .ok_or_else(|| Error::new(format!("unknown ground constant `{name}`")))?;
        let subtype = match (actual, expected) {
            (Type::Entity(a), Type::Entity(b)) => module.is_subtype(a, b),
            _ => false,
        };
        if !is_assignable(actual, expected) && !subtype {
            return Err(Error::new(format!(
                "ground constant `{name}` is not assignable to {expected:?}"
            )));
        }
    }
    let time = event
        .time
        .as_ref()
        .ok_or_else(|| Error::new(format!("unresolved PropositionTime for `{}`", event.verb)))?;
    let name = time_name(time);
    if constants.get(&name) != Some(&Type::PropositionTime) {
        return Err(Error::new(format!(
            "ground time `{name}` must be a PropositionTime constant"
        )));
    }
    lower_event(event, verbs, constants, &BTreeSet::new())
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
                    program: None,
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
                    &[],
                )?;
                rule_parameters.insert(lowered.name.clone(), lowered.parameters.clone());
                rules_by_name.insert(lowered.name.clone(), lowered.clone());
                rules.push(Declaration::Rule(lowered));
            }
            CnlDeclaration::Default(default) => {
                let SurfaceDefaultBody::Supernormal { rule } = default.body else {
                    return Err(Error::new(
                        "exception defaults are supported only by the Formal frontend",
                    ));
                };
                let parameters = rule_parameters.get(&rule).cloned().ok_or_else(|| {
                    Error::new(format!("unknown rule `{rule}` in supernormal default"))
                })?;
                defaults.push(Declaration::Default(Default {
                    name: default.name,
                    parameters,
                    condition: None,
                    blocking: None,
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

fn lower_default(
    name: String,
    body: SurfaceDefaultBody,
    rule_parameters: &BTreeMap<String, Vec<Arg>>,
    defaults: &BTreeMap<String, Default>,
) -> Result<Default, Error> {
    let (rule, kind) = match &body {
        SurfaceDefaultBody::Supernormal { rule } => (rule, "supernormal"),
        SurfaceDefaultBody::Exception { rule, .. } => (rule, "exception"),
    };
    let parameters = rule_parameters
        .get(rule)
        .cloned()
        .ok_or_else(|| Error::new(format!("unknown rule `{rule}` in {kind} default")))?;
    let body = match body {
        SurfaceDefaultBody::Supernormal { rule } => DefaultBody::Supernormal {
            rule: RuleRef { name: rule },
        },
        SurfaceDefaultBody::Exception { rule, to } => {
            let target = defaults.get(&to).ok_or_else(|| {
                Error::new(format!("unknown default `{to}` in exception default"))
            })?;
            if !parameters
                .iter()
                .map(|arg| &arg.ty)
                .eq(target.parameters.iter().map(|arg| &arg.ty))
            {
                return Err(Error::new(format!(
                    "incompatible parameter types for exception default `{name}` and target `{to}`"
                )));
            }
            DefaultBody::Exception {
                rule: RuleRef { name: rule },
                to: DefaultRef { name: to },
            }
        }
    };
    Ok(Default {
        name,
        parameters,
        body,
        condition: None,
        blocking: None,
    })
}

fn check_default_condition(
    condition: &SurfaceAntecedent,
    parameters: &[Arg],
    verb_sigs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
) -> Result<(), Error> {
    match condition {
        SurfaceAntecedent::Unit => Ok(()),
        SurfaceAntecedent::And(left, right) => {
            check_default_condition(left, parameters, verb_sigs, constants)?;
            check_default_condition(right, parameters, verb_sigs, constants)
        }
        SurfaceAntecedent::Event(event) => {
            let signature = verb_sigs
                .get(&event.verb)
                .ok_or_else(|| Error::new(format!("unknown verb `{}`", event.verb)))?;
            if event.args.len() != signature.arg_types.len() {
                return Err(Error::new(format!(
                    "verb `{}` expects {} args, got {}",
                    event.verb,
                    signature.arg_types.len(),
                    event.args.len()
                )));
            }
            for (name, expected) in event.args.iter().zip(&signature.arg_types) {
                let actual = parameters
                    .iter()
                    .find(|arg| arg.name == *name)
                    .map(|arg| &arg.ty)
                    .or_else(|| constants.get(name))
                    .ok_or_else(|| {
                        Error::new(format!("unknown default condition term `{name}`"))
                    })?;
                if !is_assignable(actual, expected) {
                    return Err(Error::new(format!(
                        "default condition term `{name}` is not assignable to {:?}",
                        expected
                    )));
                }
            }
            let time = event.time.as_ref().ok_or_else(|| {
                Error::new(format!("unresolved PropositionTime for `{}`", event.verb))
            })?;
            let name = time_name(time);
            if !parameters
                .iter()
                .any(|arg| arg.name == name && arg.ty == Type::PropositionTime)
            {
                return Err(Error::new(format!(
                    "default condition time `{name}` is not a PropositionTime parameter"
                )));
            }
            Ok(())
        }
    }
}

fn lower_kind(kind: SurfaceVerbKind) -> VerbKind {
    match kind {
        SurfaceVerbKind::Seeded => VerbKind::Seeded,
        SurfaceVerbKind::Derived => VerbKind::Derived,
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
    declarations: &[Declaration],
) -> Result<Rule, Error> {
    match body {
        SurfaceRuleBody::Residual {
            params,
            premise,
            required,
            conclusion,
        } => {
            // The same lambda scope and M2 checks apply to both sides of the adjunction.
            let horn = lower_rule(
                name,
                SurfaceRuleBody::Horn {
                    params,
                    premise: SurfaceAntecedent::And(Box::new(premise), Box::new(required)),
                    conclusion,
                },
                verb_sigs,
                entities,
                constants,
                rules_by_name,
                declarations,
            )?;
            crate::residual::residuate(&horn)
        }
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
                // Declared constants are ground, not inferred lambda binders.
                params.retain(|name| !constants.contains_key(name));
            }
            let parameters =
                infer_rule_params(&params, &premise, &conclusion, verb_sigs, entities)?;
            check_rule_antecedent(&premise, &parameters, verb_sigs, constants, declarations)?;
            check_rule_event(&conclusion, &parameters, verb_sigs, constants, declarations)?;
            let parameter_names = parameters
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            let consequent = lower_event(conclusion, verb_sigs, constants, &parameter_names)?;
            if consequent.polarity == Polarity::EvidentialNot
                && consequent.proposition.kind == PropositionKind::Seeded
            {
                return Err(Error::new(format!(
                    "EvidentialNot consequent `{}` must be Derived; seeded verbs cannot derive negative evidence",
                    consequent.verb
                )));
            }
            Ok(Rule {
                name,
                parameters,
                body: RuleBody::HornClause(HornClause {
                    antecedent: lower_antecedent(premise, verb_sigs, constants, &parameter_names)?,
                    consequent,
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
                let value = if parameter.ty == Type::PropositionTime {
                    SubstitutionValue::Time(TimeExpr::At(arg.clone()))
                } else {
                    SubstitutionValue::Term(Term::Const(arg.clone()))
                };
                substitution.insert(parameter.name.clone(), value);
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
        SurfaceAntecedent::Unit => Ok(Antecedent::Unit),
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
        polarity: event.polarity,
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
                VerbKind::Derived => PropositionKind::Derived,
                VerbKind::Effect => PropositionKind::Effect,
            },
            time: proposition_time(&time),
        },
        transition: effect_transition(signature.kind, &time),
    })
}

fn substitute_rule_body(
    body: &RuleBody,
    substitution: &BTreeMap<String, SubstitutionValue>,
) -> RuleBody {
    match body {
        RuleBody::HornClause(clause) => RuleBody::HornClause(HornClause {
            antecedent: substitute_antecedent(&clause.antecedent, substitution),
            consequent: substitute_event(&clause.consequent, substitution),
        }),
        RuleBody::ResidualClause(clause) => RuleBody::ResidualClause(ResidualClause {
            antecedent: substitute_antecedent(&clause.antecedent, substitution),
            consequent: Residual {
                required: substitute_antecedent(&clause.consequent.required, substitution),
                consequent: substitute_event(&clause.consequent.consequent, substitution),
            },
        }),
    }
}

fn check_rule_antecedent(
    antecedent: &SurfaceAntecedent,
    parameters: &[Arg],
    verbs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
    declarations: &[Declaration],
) -> Result<(), Error> {
    match antecedent {
        SurfaceAntecedent::Unit => Ok(()),
        SurfaceAntecedent::Event(event) => {
            check_rule_event(event, parameters, verbs, constants, declarations)
        }
        SurfaceAntecedent::And(left, right) => {
            check_rule_antecedent(left, parameters, verbs, constants, declarations)?;
            check_rule_antecedent(right, parameters, verbs, constants, declarations)
        }
    }
}

fn check_rule_event(
    event: &SurfaceEvent,
    parameters: &[Arg],
    verbs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
    declarations: &[Declaration],
) -> Result<(), Error> {
    let signature = verbs
        .get(&event.verb)
        .ok_or_else(|| Error::new(format!("unknown verb `{}`", event.verb)))?;
    let resolve = |name: &str| {
        parameters
            .iter()
            .find(|arg| arg.name == name)
            .map(|arg| &arg.ty)
            .or_else(|| constants.get(name))
    };
    for (name, expected) in event.args.iter().zip(&signature.arg_types) {
        let actual =
            resolve(name).ok_or_else(|| Error::new(format!("unknown rule term `{name}`")))?;
        if !rule_type_assignable(actual, expected, declarations) {
            return Err(Error::new(format!(
                "rule term `{name}` of type {actual:?} is not assignable to {expected:?}"
            )));
        }
    }
    let time = event
        .time
        .as_ref()
        .ok_or_else(|| Error::new(format!("unresolved PropositionTime for `{}`", event.verb)))?;
    let name = time_name(time);
    if resolve(&name) != Some(&Type::PropositionTime) {
        return Err(Error::new(format!(
            "rule time `{name}` must have type PropositionTime"
        )));
    }
    Ok(())
}

fn rule_type_assignable(actual: &Type, expected: &Type, declarations: &[Declaration]) -> bool {
    if is_assignable(actual, expected) {
        return true;
    }
    if let Type::Optional(inner) = expected {
        return rule_type_assignable(actual, inner, declarations);
    }
    let (Type::Entity(subtype), Type::Entity(supertype)) = (actual, expected) else {
        return false;
    };
    let mut pending = vec![subtype.as_str()];
    let mut visited = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if name == supertype {
            return true;
        }
        if !visited.insert(name) {
            continue;
        }
        for declaration in declarations {
            if let Declaration::Entity(entity) = declaration {
                if entity.name == name {
                    pending.extend(entity.supertypes.iter().map(String::as_str));
                }
            }
        }
    }
    false
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
        SurfaceAntecedent::Unit => {}
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
        SurfaceAntecedent::Unit => Ok(()),
        SurfaceAntecedent::Event(event) => infer_event_args(event, verb_sigs, inferred),
        SurfaceAntecedent::And(left, right) => {
            infer_antecedent_args(left, verb_sigs, inferred)?;
            infer_antecedent_args(right, verb_sigs, inferred)
        }
    }
}

fn infer_antecedent_times(antecedent: &SurfaceAntecedent, inferred: &mut BTreeMap<String, Type>) {
    match antecedent {
        SurfaceAntecedent::Unit => {}
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
