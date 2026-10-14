use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::derive::{substitute_antecedent, substitute_event};
use crate::surface::{
    CnlDeclaration, CnlModule, FormalDeclaration, FormalModule, SurfaceAntecedent,
    SurfaceDefaultBody, SurfaceEvent, SurfaceFieldAccess, SurfaceQueryOptions, SurfaceQueryScope,
    SurfaceRuleBody, SurfaceStateOperation, SurfaceTerm, SurfaceTime, SurfaceVerbKind,
};
use crate::typed::{
    Antecedent, Arg, BehaviorKind, BehaviorRef, Constant, Declaration, Default, DefaultBody,
    DefaultRef, Effect, Entity, Event, Experiment, ExperimentInputKind, ExperimentQueryKind, Field,
    FieldAccess, HornClause, HornQueryScope, Module, Polarity, ProgramRef, PropositionKind,
    PropositionType, QueryBinding, QueryOptions, QueryParameterScope, RecordFieldValue, Residual,
    ResidualClause, Rule, RuleBody, RuleRef, StateOperation, StateTime, StateTransitionType,
    SubstitutionValue, Term, TimeExpr, Type, Verb, VerbKind,
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
    elaborate_formal_with_dependencies(module, &[])
}

/// Dependencies have already been resolved and independently elaborated.
/// This extends the symbol environment, not the source text or SurfaceAST.
pub(crate) fn elaborate_formal_with_dependencies(
    module: FormalModule,
    imported: &[Declaration],
) -> Result<Module, Error> {
    let mut declarations = imported.to_vec();
    let mut verb_sigs = BTreeMap::new();
    let mut entities = BTreeSet::new();
    let mut entity_fields = BTreeMap::new();
    let mut rule_parameters = BTreeMap::new();
    let mut programs_by_name = BTreeMap::new();
    let mut rules_by_name = BTreeMap::new();
    let mut constants = BTreeMap::new();
    let mut defaults_by_name = BTreeMap::new();
    let mut experiments = Vec::new();

    for declaration in imported {
        match declaration {
            Declaration::Entity(entity) => {
                entities.insert(entity.name.clone());
                entity_fields.insert(entity.name.clone(), entity.fields.clone());
            }
            Declaration::Const(constant) => {
                constants.insert(constant.name.clone(), constant.ty.clone());
            }
            Declaration::Verb(verb) => {
                verb_sigs.insert(
                    verb.name.clone(),
                    VerbSig {
                        kind: verb.kind.clone(),
                        arg_types: verb.args.iter().map(|a| a.ty.clone()).collect(),
                    },
                );
            }
            Declaration::Rule(rule) => {
                rule_parameters.insert(rule.name.clone(), rule.parameters.clone());
                rules_by_name.insert(rule.name.clone(), rule.clone());
            }
            Declaration::Program(program) => {
                programs_by_name.insert(program.identity.clone(), program.clone());
            }
            Declaration::Default(default) => {
                defaults_by_name.insert(default.name.clone(), default.clone());
            }
            Declaration::Experiment(_) => return Err(Error::new("experiments cannot be imported")),
        }
    }

    for declaration in module.declarations {
        match declaration {
            FormalDeclaration::Import(path) => {
                return Err(Error::new(format!(
                    "unresolved import `{path}`; use load_formal_module with a source path"
                )));
            }
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
                    .map(|effect| {
                        lower_effect(effect.operations, &args, &entity_fields, &constants)
                    })
                    .transpose()?;
                declarations.push(Declaration::Verb(Verb {
                    name: verb.name,
                    kind: lower_kind(verb.kind),
                    args,
                    effect,
                    program: verb.program.map(|name| BehaviorRef {
                        name,
                        kind: BehaviorKind::Rule,
                    }),
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
            FormalDeclaration::Program(program) => {
                if declarations.iter().any(|d| d.name() == program.name) {
                    return Err(Error::new(format!("duplicate symbol `{}`", program.name)));
                }
                let lowered = crate::programs::lower_program(
                    program,
                    &module.name,
                    &declarations,
                    &constants,
                    &entity_fields,
                )?;
                programs_by_name.insert(lowered.identity.clone(), lowered.clone());
                declarations.push(Declaration::Program(lowered));
            }
            FormalDeclaration::Default(default) => {
                let lowered = lower_default(
                    default.name,
                    default.body,
                    &rule_parameters,
                    &programs_by_name,
                    &defaults_by_name,
                )?;
                let mut lowered = lowered;
                if let Some(condition) = default.condition {
                    if matches!(
                        lowered.body,
                        DefaultBody::Supernormal { .. } | DefaultBody::SupernormalProgram { .. }
                    ) {
                        return Err(Error::new(
                            "supernormal defaults do not support a condition",
                        ));
                    }
                    check_default_condition(
                        &condition,
                        &lowered.parameters,
                        &verb_sigs,
                        &constants,
                        &declarations,
                    )?;
                    lowered.condition = Some(lower_antecedent(
                        condition,
                        &verb_sigs,
                        &constants,
                        &lowered.parameters,
                        &declarations,
                    )?);
                }
                if let Some(blocking) = default.blocking {
                    check_default_condition(
                        &blocking,
                        &lowered.parameters,
                        &verb_sigs,
                        &constants,
                        &declarations,
                    )?;
                    lowered.blocking = Some(lower_antecedent(
                        blocking,
                        &verb_sigs,
                        &constants,
                        &lowered.parameters,
                        &declarations,
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
    let program_parameters: BTreeMap<_, _> = declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Program(p) => Some((p.identity.clone(), p.parameters.clone())),
            _ => None,
        })
        .collect();
    let mut declaration_counts = BTreeMap::new();
    for declaration in &declarations {
        *declaration_counts
            .entry(declaration.name().to_string())
            .or_insert(0usize) += 1;
    }
    for declaration in &mut declarations {
        let Declaration::Verb(verb) = declaration else {
            continue;
        };
        let Some(reference) = &mut verb.program else {
            continue;
        };
        if declaration_counts
            .get(&reference.name)
            .copied()
            .unwrap_or(0)
            > 1
        {
            return Err(Error::new(format!(
                "ambiguous associated rule `{}` for verb `{}`",
                reference.name, verb.name
            )));
        }
        let parameters = if let Some(parameters) = program_parameters.get(&reference.name) {
            reference.kind = BehaviorKind::Program;
            parameters
        } else {
            &rules_by_name
                .get(&reference.name)
                .ok_or_else(|| {
                    Error::new(format!(
                        "unknown associated rule `{}` for verb `{}`",
                        reference.name, verb.name
                    ))
                })?
                .parameters
        };
        let mut expected: Vec<_> = verb.args.iter().map(|arg| arg.ty.clone()).collect();
        expected.push(Type::PropositionTime);
        let actual: Vec<_> = parameters.iter().map(|arg| arg.ty.clone()).collect();
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
        origins: BTreeMap::new(),
    };
    // Query scope validation needs original producer identities during lowering;
    // the file resolver replaces these source-only origins after linking.
    result.origins = source_origins(&result);
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
        let query_options = lower_query_options(
            experiment.query_options,
            experiment.query_kind,
            &goal,
            &result,
            &entity_fields,
            &constants,
        )?;
        result
            .declarations
            .push(Declaration::Experiment(Experiment {
                name: experiment.name,
                input_kind: experiment.input_kind,
                input,
                query_kind: experiment.query_kind,
                goal,
                query_options,
            }));
    }
    result.origins = source_origins(&result);
    Ok(result)
}

fn source_origins(result: &Module) -> BTreeMap<String, crate::typed::DeclarationOrigin> {
    result
        .declarations
        .iter()
        .map(|d| {
            (
                d.name().to_string(),
                crate::typed::DeclarationOrigin {
                    module: result.name.clone(),
                    file: None,
                },
            )
        })
        .collect()
}

fn lower_query_options(
    source: SurfaceQueryOptions,
    kind: ExperimentQueryKind,
    goal: &Event,
    module: &Module,
    fields: &BTreeMap<String, Vec<Field>>,
    constants: &BTreeMap<String, Type>,
) -> Result<QueryOptions, Error> {
    if kind != ExperimentQueryKind::Residual
        && (source.active || source.scope.is_some() || source.bindings.is_some())
    {
        return Err(Error::new(
            "fourFusion does not accept active mode, residual search scope or bindings",
        ));
    }
    if source.bindings.is_some() && source.scope.is_none() {
        return Err(Error::new(
            "explicit query bindings require an explicit search scope",
        ));
    }
    let scope = source.scope.map(|scope| match scope {
        SurfaceQueryScope::Rules(names) => {
            HornQueryScope::Rules(names.into_iter().map(|name| RuleRef { name }).collect())
        }
        SurfaceQueryScope::Program(name) => HornQueryScope::Program(ProgramRef { name }),
    });
    let bindings = source
        .bindings
        .unwrap_or_default()
        .into_iter()
        .map(|binding| {
            let scope = match binding.scope.kind {
                BehaviorKind::Rule => QueryParameterScope::Rule(RuleRef {
                    name: binding.scope.name,
                }),
                BehaviorKind::Program => QueryParameterScope::Program(ProgramRef {
                    name: binding.scope.name,
                }),
            };
            let expected =
                crate::residual::query_parameter_type(module, &scope, &binding.parameter)?;
            // No query lambda scope: names in values resolve to module constants,
            // never to homonymous Rule or Program parameter names.
            let (value, actual) = lower_term(binding.value, &[], fields, constants)?;
            let ty = if value == Term::None {
                expected
            } else {
                actual
            };
            Ok(QueryBinding {
                scope,
                parameter: binding.parameter,
                ty,
                value: SubstitutionValue::Term(value),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let mut options = QueryOptions {
        active: source.active,
        scope,
        bindings,
    };
    crate::residual::validate_query_options(module, &mut options, goal)?;
    Ok(options)
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
    let fields = declaration_fields(&module.declarations);
    for (term, expected) in event.args.iter().zip(&signature.arg_types) {
        if let SurfaceTerm::Var(name) = term {
            if !constants.contains_key(name) {
                return Err(Error::new(format!("unknown ground constant `{name}`")));
            }
        }
        let (_, actual) = lower_term(term.clone(), &[], &fields, constants)?;
        if !rule_type_assignable(&actual, expected, &module.declarations) {
            return Err(Error::new(format!(
                "ground constant `{term}` is not assignable to {expected:?}"
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
    lower_event(event, verbs, constants, &[], &module.declarations)
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
                    .map(|effect| {
                        lower_effect(effect.operations, &args, &entity_fields, &constants)
                    })
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

    let origins = declarations
        .iter()
        .map(|d| {
            (
                d.name().to_string(),
                crate::typed::DeclarationOrigin {
                    module: module.name.clone(),
                    file: None,
                },
            )
        })
        .collect();
    Ok(Module {
        name: module.name,
        declarations,
        origins,
    })
}

fn lower_default(
    name: String,
    body: SurfaceDefaultBody,
    rule_parameters: &BTreeMap<String, Vec<Arg>>,
    programs: &BTreeMap<String, crate::typed::Program>,
    defaults: &BTreeMap<String, Default>,
) -> Result<Default, Error> {
    let (rule, kind) = match &body {
        SurfaceDefaultBody::Supernormal { rule } => (rule, "supernormal"),
        SurfaceDefaultBody::Exception { rule, .. } => (rule, "exception"),
    };
    let program = programs.get(rule);
    let parameters = match program {
        Some(program) => program.parameters.clone(),
        None => rule_parameters.get(rule).cloned().ok_or_else(|| {
            Error::new(format!(
                "unknown rule or program `{rule}` in {kind} default"
            ))
        })?,
    };
    let body = match body {
        SurfaceDefaultBody::Supernormal { rule } => match program {
            Some(_) => DefaultBody::SupernormalProgram {
                program: crate::typed::ProgramRef { name: rule },
            },
            None => DefaultBody::Supernormal {
                rule: RuleRef { name: rule },
            },
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
            match program {
                Some(_) => DefaultBody::ExceptionProgram {
                    program: crate::typed::ProgramRef { name: rule },
                    to: DefaultRef { name: to },
                },
                None => DefaultBody::Exception {
                    rule: RuleRef { name: rule },
                    to: DefaultRef { name: to },
                },
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
    declarations: &[Declaration],
) -> Result<(), Error> {
    match condition {
        SurfaceAntecedent::Unit => Ok(()),
        SurfaceAntecedent::And(left, right) => {
            check_default_condition(left, parameters, verb_sigs, constants, declarations)?;
            check_default_condition(right, parameters, verb_sigs, constants, declarations)
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
            for (term, expected) in event.args.iter().zip(&signature.arg_types) {
                if let SurfaceTerm::Var(name) = term {
                    if !parameters.iter().any(|p| p.name == *name) && !constants.contains_key(name)
                    {
                        return Err(Error::new(format!(
                            "unknown default condition term `{name}`"
                        )));
                    }
                }
                let (_, actual) = lower_term(
                    term.clone(),
                    parameters,
                    &declaration_fields(declarations),
                    constants,
                )?;
                if !is_assignable(&actual, expected) {
                    return Err(Error::new(format!(
                        "default condition term `{term}` is not assignable to {:?}",
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
                for term in &conclusion.args {
                    collect_term_params(term, &mut params);
                }
                if let Some(time) = &conclusion.time {
                    params.push(time_name(time));
                }
                dedup(&mut params);
                // Declared constants are ground, not inferred lambda binders.
                params.retain(|name| !constants.contains_key(name));
            }
            let parameters = infer_rule_params(
                &params,
                &premise,
                &conclusion,
                verb_sigs,
                entities,
                declarations,
            )?;
            check_rule_antecedent(&premise, &parameters, verb_sigs, constants, declarations)?;
            check_rule_event(&conclusion, &parameters, verb_sigs, constants, declarations)?;
            let consequent =
                lower_event(conclusion, verb_sigs, constants, &parameters, declarations)?;
            if consequent.polarity == Polarity::EvidentialNot
                && consequent.proposition.kind == PropositionKind::Seeded
            {
                return Err(Error::new(format!(
                    "EvidentialNot consequent `{}` must be Derived; seeded verbs cannot derive negative evidence",
                    consequent.verb
                )));
            }
            let antecedent =
                lower_antecedent(premise, verb_sigs, constants, &parameters, declarations)?;
            Ok(Rule {
                name,
                parameters,
                body: RuleBody::HornClause(HornClause {
                    antecedent,
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
            // One typed value substitutes into both argument and index positions.
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
                let value = SubstitutionValue::Term(Term::Const(arg.clone()))
                    .canonical_for_type(&parameter.ty)
                    .ok_or_else(|| Error::new("invalid typed rule binding"))?;
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

fn declaration_fields(declarations: &[Declaration]) -> BTreeMap<String, Vec<Field>> {
    declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Entity(e) => Some((e.name.clone(), e.fields.clone())),
            _ => None,
        })
        .collect()
}

fn lower_antecedent(
    antecedent: SurfaceAntecedent,
    verb_sigs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
    parameters: &[Arg],
    declarations: &[Declaration],
) -> Result<Antecedent, Error> {
    match antecedent {
        SurfaceAntecedent::Unit => Ok(Antecedent::Unit),
        SurfaceAntecedent::Event(event) => Ok(Antecedent::Event(lower_event(
            event,
            verb_sigs,
            constants,
            parameters,
            declarations,
        )?)),
        SurfaceAntecedent::And(left, right) => Ok(Antecedent::And(
            Box::new(lower_antecedent(
                *left,
                verb_sigs,
                constants,
                parameters,
                declarations,
            )?),
            Box::new(lower_antecedent(
                *right,
                verb_sigs,
                constants,
                parameters,
                declarations,
            )?),
        )),
    }
}

fn lower_event(
    event: SurfaceEvent,
    verb_sigs: &BTreeMap<String, VerbSig>,
    constants: &BTreeMap<String, Type>,
    parameters: &[Arg],
    declarations: &[Declaration],
) -> Result<Event, Error> {
    let signature = verb_sigs
        .get(&event.verb)
        .ok_or_else(|| Error::new(format!("unknown verb `{}`", event.verb)))?;
    let time = event
        .time
        .ok_or_else(|| Error::new(format!("unresolved PropositionTime for `{}`", event.verb)))?;
    let parameter_names = parameters.iter().map(|p| p.name.clone()).collect();
    let time = proposition_time(&time, &parameter_names);
    let fields = declaration_fields(declarations);
    Ok(Event {
        polarity: event.polarity,
        verb: event.verb,
        args: event
            .args
            .into_iter()
            .map(|term| lower_term(term, parameters, &fields, constants).map(|(term, _)| term))
            .collect::<Result<_, _>>()?,
        proposition: PropositionType {
            kind: match signature.kind {
                VerbKind::Seeded => PropositionKind::Seeded,
                VerbKind::Derived => PropositionKind::Derived,
                VerbKind::Effect => PropositionKind::Effect,
            },
            time: time.clone(),
        },
        transition: effect_transition(signature.kind, &time),
    })
}

pub(crate) fn substitute_rule_body(
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
    let fields = declaration_fields(declarations);
    for (term, expected) in event.args.iter().zip(&signature.arg_types) {
        if let SurfaceTerm::Var(name) = term {
            if resolve(name).is_none() {
                return Err(Error::new(format!("unknown rule term `{name}`")));
            }
        }
        let (_, actual) = lower_term(term.clone(), parameters, &fields, constants)?;
        if !rule_type_assignable(&actual, expected, declarations) {
            return Err(Error::new(format!(
                "rule term `{term}` of type {actual:?} is not assignable to {expected:?}"
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

pub(crate) fn rule_type_assignable(
    actual: &Type,
    expected: &Type,
    declarations: &[Declaration],
) -> bool {
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

fn proposition_time(time: &SurfaceTime, parameters: &BTreeSet<String>) -> TimeExpr {
    let named = |name: &String| {
        if parameters.contains(name) {
            TimeExpr::Var(name.clone())
        } else {
            TimeExpr::At(name.clone())
        }
    };
    match time {
        SurfaceTime::At(name) => named(name),
        SurfaceTime::After(name) => TimeExpr::After(Box::new(named(name))),
    }
}

fn effect_transition(kind: VerbKind, time: &TimeExpr) -> Option<StateTransitionType> {
    if kind != VerbKind::Effect {
        return None;
    }
    Some(StateTransitionType {
        input: match time {
            TimeExpr::After(input) => StateTime::At((**input).clone()),
            TimeExpr::At(_) | TimeExpr::Var(_) => StateTime::Unresolved,
        },
        output: StateTime::At(time.clone()),
    })
}

fn lower_effect(
    operations: Vec<SurfaceStateOperation>,
    args: &[Arg],
    entity_fields: &BTreeMap<String, Vec<Field>>,
    constants: &BTreeMap<String, Type>,
) -> Result<Effect, Error> {
    let operations = operations
        .into_iter()
        .map(|operation| lower_state_operation(operation, args, entity_fields, constants))
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
    constants: &BTreeMap<String, Type>,
) -> Result<StateOperation, Error> {
    match operation {
        SurfaceStateOperation::AddToSet { target, value } => {
            let (target, value) =
                check_set_operation(target, value, args, entity_fields, constants)?;
            Ok(StateOperation::AddToSet { target, value })
        }
        SurfaceStateOperation::RemoveFromSet { target, value } => {
            let (target, value) =
                check_set_operation(target, value, args, entity_fields, constants)?;
            Ok(StateOperation::RemoveFromSet { target, value })
        }
    }
}

fn check_set_operation(
    target: SurfaceFieldAccess,
    value: SurfaceTerm,
    args: &[Arg],
    entity_fields: &BTreeMap<String, Vec<Field>>,
    constants: &BTreeMap<String, Type>,
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
    let (value, value_ty) = lower_term(value, args, entity_fields, constants)?;
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

pub(crate) fn lower_term(
    term: SurfaceTerm,
    args: &[Arg],
    entity_fields: &BTreeMap<String, Vec<Field>>,
    constants: &BTreeMap<String, Type>,
) -> Result<(Term, Type), Error> {
    match term {
        SurfaceTerm::Constructor {
            constructor,
            argument,
        } => {
            let (input, output) = constructor.signature();
            let (argument, actual) = lower_term(*argument, args, entity_fields, constants)?;
            if actual != input {
                return Err(Error::new(format!(
                    "after requires PropositionTime, got {actual:?}"
                )));
            }
            Ok((
                Term::Constructor {
                    constructor,
                    argument: Box::new(argument),
                },
                output,
            ))
        }
        SurfaceTerm::Var(name) => {
            // Effect parameters shadow module constants, as rule parameters do.
            // Keep a constant ground so runtime materialization never treats it
            // as an argument or an ObjectId.
            if let Some(arg) = args.iter().find(|arg| arg.name == name) {
                Ok((Term::Var(name), arg.ty.clone()))
            } else if let Some(ty) = constants.get(&name) {
                Ok((Term::Const(name), ty.clone()))
            } else {
                Err(Error::new(format!(
                    "unknown effect argument or constant `{name}`"
                )))
            }
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
                let (term, value_ty) = lower_term(value, args, entity_fields, constants)?;
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

pub(crate) fn is_assignable(actual: &Type, expected: &Type) -> bool {
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
        Term::Constructor { argument, .. } => format!("after({})", describe_term(argument)),
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
    declarations: &[Declaration],
) -> Result<Vec<Arg>, Error> {
    let mut inferred = BTreeMap::new();
    seed_role_types(requested, entities, &mut inferred);
    infer_antecedent_args(
        premise,
        verb_sigs,
        &mut inferred,
        &declaration_fields(declarations),
    )?;
    infer_event_args(
        conclusion,
        verb_sigs,
        &mut inferred,
        &declaration_fields(declarations),
    )?;
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
            for term in &event.args {
                collect_term_params(term, params);
            }
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
    fields: &BTreeMap<String, Vec<Field>>,
) -> Result<(), Error> {
    match antecedent {
        SurfaceAntecedent::Unit => Ok(()),
        SurfaceAntecedent::Event(event) => infer_event_args(event, verb_sigs, inferred, fields),
        SurfaceAntecedent::And(left, right) => {
            infer_antecedent_args(left, verb_sigs, inferred, fields)?;
            infer_antecedent_args(right, verb_sigs, inferred, fields)
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
    fields: &BTreeMap<String, Vec<Field>>,
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

    for (term, ty) in event.args.iter().zip(&signature.arg_types) {
        infer_term(term, ty, inferred, fields)?;
    }
    Ok(())
}

fn collect_term_params(term: &SurfaceTerm, params: &mut Vec<String>) {
    match term {
        SurfaceTerm::Var(name) => params.push(name.clone()),
        SurfaceTerm::Constructor { argument, .. } => collect_term_params(argument, params),
        SurfaceTerm::RecordLiteral { fields, .. } => {
            for field in fields {
                collect_term_params(&field.value, params);
            }
        }
        SurfaceTerm::None => {}
    }
}

fn infer_term(
    term: &SurfaceTerm,
    ty: &Type,
    inferred: &mut BTreeMap<String, Type>,
    fields: &BTreeMap<String, Vec<Field>>,
) -> Result<(), Error> {
    match term {
        SurfaceTerm::Var(name) => {
            if let Some(existing) = inferred.get(name) {
                if existing == ty || is_assignable(existing, ty) {
                    return Ok(());
                }
                if !is_assignable(ty, existing) {
                    return Err(Error::new(format!(
                        "conflicting inferred types for `{name}`"
                    )));
                }
            }
            inferred.insert(name.clone(), ty.clone());
        }
        SurfaceTerm::Constructor {
            constructor,
            argument,
        } => {
            let (input, output) = constructor.signature();
            if !is_assignable(&output, ty) {
                return Err(Error::new(format!(
                    "after result PropositionTime is not assignable to {ty:?}"
                )));
            }
            infer_term(argument, &input, inferred, fields)?;
        }
        SurfaceTerm::RecordLiteral {
            entity,
            fields: values,
        } => {
            let schema = fields
                .get(entity)
                .ok_or_else(|| Error::new(format!("unknown record entity `{entity}`")))?;
            for value in values {
                let field = schema
                    .iter()
                    .find(|f| f.name == value.field)
                    .ok_or_else(|| {
                        Error::new(format!("unknown field `{entity}.{}`", value.field))
                    })?;
                infer_term(&value.value, &field.ty, inferred, fields)?;
            }
        }
        SurfaceTerm::None => {}
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
