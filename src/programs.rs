//! Program elaboration and lowering to existing single-clause Rules.
//! No default selection, state execution, or independent per-member closure.
use crate::elab::{Error, lower_term, rule_type_assignable, substitute_rule_body};
use crate::surface::{FormalProgram, SurfaceProgramBody};
use crate::typed::*;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn lower_program(
    source: FormalProgram,
    defining_module: &str,
    declarations: &[Declaration],
    constants: &BTreeMap<String, Type>,
    fields: &BTreeMap<String, Vec<Field>>,
) -> Result<Program, Error> {
    match source.body {
        SurfaceProgramBody::Application { program, args } => {
            let base = declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Program(p) if p.identity == program => Some(p),
                    _ => None,
                })
                .ok_or_else(|| Error::new(format!("unknown program `{program}`")))?;
            let mut result = specialize_program(base, &args, constants)?;
            result.identity = source.name;
            // A specialized closure retains the definition's lexical scope.
            Ok(result)
        }
        SurfaceProgramBody::Definition { parameters, uses } => {
            let parameters: Vec<_> = parameters
                .into_iter()
                .map(|a| Arg {
                    name: a.name,
                    ty: a.ty,
                })
                .collect();
            let mut names = BTreeSet::new();
            for p in &parameters {
                if !names.insert(&p.name) {
                    return Err(Error::new(format!(
                        "duplicate program parameter `{}`",
                        p.name
                    )));
                }
                validate_type(&p.ty, declarations)?;
            }
            let mut rule_uses = Vec::new();
            for member in uses {
                let rule = find_rule(declarations, &member.rule)?;
                let mut seen = BTreeSet::new();
                let mut bindings = Vec::new();
                for binding in member.bindings {
                    if !seen.insert(binding.field.clone()) {
                        return Err(Error::new(format!("duplicate binding `{}`", binding.field)));
                    }
                    let parameter = rule
                        .parameters
                        .iter()
                        .find(|p| p.name == binding.field)
                        .ok_or_else(|| {
                            Error::new(format!(
                                "unknown parameter `{}.{}`",
                                rule.name, binding.field
                            ))
                        })?;
                    let (value, actual) =
                        lower_term(binding.value, &parameters, fields, constants)?;
                    if !rule_type_assignable(&actual, &parameter.ty, declarations) {
                        return Err(Error::new(format!(
                            "binding `{}.{}` has type {actual:?}; expected {:?}",
                            rule.name, parameter.name, parameter.ty
                        )));
                    }
                    bindings.push(ProgramBinding {
                        parameter: binding.field,
                        value,
                    });
                }
                rule_uses.push(RuleUse {
                    rule: RuleRef { name: member.rule },
                    bindings,
                });
            }
            Ok(Program {
                identity: source.name,
                parameters,
                rule_uses,
                defining_module: defining_module.into(),
            })
        }
    }
}

fn validate_type(ty: &Type, declarations: &[Declaration]) -> Result<(), Error> {
    match ty {
        Type::Entity(name)
            if !declarations
                .iter()
                .any(|d| matches!(d, Declaration::Entity(e) if e.name == *name)) =>
        {
            Err(Error::new(format!(
                "unknown program parameter type `{name}`"
            )))
        }
        Type::Set(t) | Type::Optional(t) => validate_type(t, declarations),
        _ => Ok(()),
    }
}

pub fn specialize_program(
    program: &Program,
    args: &[String],
    constants: &BTreeMap<String, Type>,
) -> Result<Program, Error> {
    if args.len() > program.parameters.len() {
        return Err(Error::new("too many applied program arguments"));
    }
    let mut substitution = BTreeMap::new();
    for (arg, p) in args.iter().zip(&program.parameters) {
        let ty = constants.get(arg).ok_or_else(|| {
            Error::new(format!("unknown constant `{arg}` in program application"))
        })?;
        if ty != &p.ty {
            return Err(Error::new(format!(
                "constant `{arg}` is not assignable to program parameter `{}`",
                p.name
            )));
        }
        substitution.insert(p.name.clone(), Term::Const(arg.clone()));
    }
    let mut result = program.clone();
    result.parameters = program.parameters[args.len()..].to_vec();
    for member in &mut result.rule_uses {
        for binding in &mut member.bindings {
            binding.value = substitute_term(&binding.value, &substitution);
        }
    }
    Ok(result)
}

fn substitute_term(term: &Term, substitution: &BTreeMap<String, Term>) -> Term {
    match term {
        Term::Var(name) => substitution
            .get(name)
            .cloned()
            .unwrap_or_else(|| term.clone()),
        Term::Constructor {
            constructor,
            argument,
        } => Term::Constructor {
            constructor: *constructor,
            argument: Box::new(substitute_term(argument, substitution)),
        },
        Term::RecordLiteral { entity, fields } => Term::RecordLiteral {
            entity: entity.clone(),
            fields: fields
                .iter()
                .map(|f| RecordFieldValue {
                    field: f.field.clone(),
                    value: substitute_term(&f.value, substitution),
                })
                .collect(),
        },
        _ => term.clone(),
    }
}

fn find_rule<'a>(declarations: &'a [Declaration], name: &str) -> Result<&'a Rule, Error> {
    declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == name => Some(r),
            _ => None,
        })
        .ok_or_else(|| Error::new(format!("unknown program member rule `{name}`")))
}

/// Hygienic binder identity. These names cannot be written as source identifiers.
pub(crate) fn shared_parameter(program: &Program, name: &str) -> String {
    format!("#{}:shared:{name}", program.identity)
}

/// Lower all uses together. The caller supplies this whole vector to one close.
/// Unbound Program parameters remain binders; unmapped Rule binders stay local.
pub fn compile_program(program: &Program, module: &Module) -> Result<Vec<Rule>, Error> {
    let mut output = Vec::new();
    for (index, member) in program.rule_uses.iter().enumerate() {
        let original = find_rule(&module.declarations, &member.rule.name)?;
        let (parameters, substitution) = rule_use_substitution(program, index, original)?;
        output.push(Rule {
            name: format!("{}::{index}::{}", program.identity, original.name),
            parameters,
            body: substitute_rule_body(&original.body, &substitution),
        });
    }
    Ok(output)
}

/// The compiler's original RuleUse binder mapping, shared with query scoping.
/// Returns typed beta values and all hygienic shared/local parameters, without
/// reducing a body or implementing a second specialization algorithm.
pub(crate) fn rule_use_substitution(
    program: &Program,
    index: usize,
    original: &Rule,
) -> Result<(Vec<Arg>, BTreeMap<String, SubstitutionValue>), Error> {
    let member = program
        .rule_uses
        .get(index)
        .filter(|member| member.rule.name == original.name)
        .ok_or_else(|| Error::new("invalid query RuleUse identity"))?;
    let shared: BTreeMap<_, _> = program
        .parameters
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                Term::Var(shared_parameter(program, &p.name)),
            )
        })
        .collect();
    let values: BTreeMap<_, _> = member
        .bindings
        .iter()
        .map(|b| (b.parameter.clone(), substitute_term(&b.value, &shared)))
        .collect();
    let mut substitution = BTreeMap::new();
    let mut parameters: Vec<_> = program
        .parameters
        .iter()
        .map(|p| Arg {
            name: shared_parameter(program, &p.name),
            ty: p.ty.clone(),
        })
        .collect();
    for p in &original.parameters {
        let value = values.get(&p.name).cloned().unwrap_or_else(|| {
            let name = format!("#{}:use{index}:{}", program.identity, p.name);
            parameters.push(Arg {
                name: name.clone(),
                ty: p.ty.clone(),
            });
            Term::Var(name)
        });
        let value = SubstitutionValue::Term(value)
            .canonical_for_type(&p.ty)
            .ok_or_else(|| Error::new("temporal binding must be a PropositionTime term"))?;
        substitution.insert(p.name.clone(), value);
    }
    Ok((parameters, substitution))
}

/// Decode a compiled producer without modifying DerivationRecord or logical Event identity.
pub fn program_rule_origin(module: &Module, producer: &str) -> Option<ProgramRuleOrigin> {
    let mut parts = producer.split("::");
    let identity = parts.next()?;
    let index: usize = parts.next()?.parse().ok()?;
    let rule = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let program = module.declarations.iter().find_map(|d| match d {
        Declaration::Program(p) if p.identity == identity => Some(p),
        _ => None,
    })?;
    let member = program.rule_uses.get(index)?;
    if member.rule.name != rule {
        return None;
    }
    Some(ProgramRuleOrigin {
        program: identity.into(),
        defining_module: program.defining_module.clone(),
        rule: rule.into(),
        rule_origin: module.origin(rule)?.clone(),
        rule_use: index,
    })
}
