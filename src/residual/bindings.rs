//! Scoped, typed query constraints. Neither rules nor evidence are specialized
//! in place; search still replays the original producers for certification.
use std::collections::{BTreeMap, BTreeSet};

use super::active::{certify, guarded_view};
use super::query::{Bindings, constrained_goal, query_with_constraints, unify_term};
use super::{Abducible, ActiveProgramResidualQuery, ProgramResidualQuery, SearchLimits};
use crate::derive::{DerivationClock, substitute_term};
use crate::elab::{Error, rule_type_assignable};
use crate::programs::{compile_program, program_rule_origin, shared_parameter};
use crate::typed::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryParameterScope {
    Rule(RuleRef),
    /// A remaining shared parameter of this resolved Program identity.
    Program(ProgramRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryBinding {
    pub scope: QueryParameterScope,
    pub parameter: String,
    /// Runtime Var identities carry a caller-supplied type. Constants and
    /// structured values are also checked against their module declarations.
    pub ty: Type,
    pub value: SubstitutionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HornQueryScope {
    /// Exactly these ordinary Rule declarations; no implicit dependency slicing.
    Rules(Vec<RuleRef>),
    /// Exactly the compiled RuleUses, preserving their original producer names.
    Program(ProgramRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryWithBindings<T> {
    pub query: T,
    /// Validated supplied constraints, with canonical temporal representation.
    pub bindings: Vec<QueryBinding>,
}

/// Local reduction, including Unit branches. An empty vector means the head
/// does not match; pending branches retain every unbound typed parameter.
pub fn query_residual_with_bindings(
    module: &Module,
    rule: &Rule,
    goal: &Event,
    known: &[Event],
    bindings: &[QueryBinding],
) -> Result<QueryWithBindings<Vec<ResidualRequirement>>, Error> {
    let rules = vec![rule.clone()];
    let (constraints, bindings) = resolve(module, &rules, &BTreeMap::new(), bindings)?;
    let empty = Bindings::new();
    Ok(QueryWithBindings {
        query: query_with_constraints(
            rule,
            goal,
            known,
            constraints.get(&rule.name).unwrap_or(&empty),
            true,
        )?,
        bindings,
    })
}

/// Scoped Horn search with immutable evidence. Unresolved recursive parameters
/// are typed errors, never existential enumeration or discarded producers.
pub fn query_program_residual_with_bindings(
    module: &Module,
    scope: &HornQueryScope,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
) -> Result<QueryWithBindings<ProgramResidualQuery>, Error> {
    if module
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Default(_)))
    {
        return Err(Error::new(
            "bound residual queries require a monotonic module without defaults",
        ));
    }
    let rules = match scope {
        HornQueryScope::Rules(references) => {
            let mut selected = BTreeSet::new();
            let mut rules = Vec::new();
            for reference in references {
                if !selected.insert(&reference.name) {
                    return Err(Error::new("duplicate query scope rule"));
                }
                rules.push(find_rule(module, &reference.name)?.clone());
            }
            rules
        }
        HornQueryScope::Program(reference) => {
            compile_program(find_program(module, &reference.name)?, module)?
        }
    };
    let (constraints, bindings) = resolve(module, &rules, &BTreeMap::new(), bindings)?;
    preflight(&rules, goal, &constraints)?;
    let mut view = module.clone();
    view.declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    view.declarations
        .extend(rules.into_iter().map(Declaration::Rule));
    let defaults = default_abducibles(module);
    Ok(QueryWithBindings {
        query: super::program::query_with_constraints(
            &view,
            evidence,
            goal,
            allowed.unwrap_or(&defaults),
            limits,
            Some(&constraints),
        )?,
        bindings,
    })
}

/// Guarded search followed by hypothetical sufficiency against the unchanged
/// original close_program. Supplied bindings never become activation evidence.
pub fn query_active_program_residual_with_bindings<C: DerivationClock>(
    module: &Module,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
    clock: &mut C,
) -> Result<QueryWithBindings<ActiveProgramResidualQuery>, Error> {
    let (view, templates) = guarded_view(module)?;
    let rules: Vec<_> = view
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let (constraints, bindings) = resolve(module, &rules, &templates, bindings)?;
    preflight(&rules, goal, &constraints)?;
    let defaults = default_abducibles(module);
    let search = super::program::query_with_constraints(
        &view,
        evidence,
        goal,
        allowed.unwrap_or(&defaults),
        limits,
        Some(&constraints),
    )?;
    Ok(QueryWithBindings {
        query: certify(module, evidence, goal, search, &templates, clock)?,
        bindings,
    })
}

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

fn find_rule<'a>(module: &'a Module, name: &str) -> Result<&'a Rule, Error> {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == name => Some(r),
            _ => None,
        })
        .ok_or_else(|| Error::new(format!("unknown query rule `{name}`")))
}
fn find_program<'a>(module: &'a Module, name: &str) -> Result<&'a Program, Error> {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Program(p) if p.identity == name => Some(p),
            _ => None,
        })
        .ok_or_else(|| Error::new(format!("unknown query program `{name}`")))
}

type ConstraintMap = BTreeMap<String, Bindings>;

fn resolve(
    module: &Module,
    rules: &[Rule],
    templates: &BTreeMap<String, super::ResidualActivation>,
    supplied: &[QueryBinding],
) -> Result<(ConstraintMap, Vec<QueryBinding>), Error> {
    let mut constraints = ConstraintMap::new();
    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();
    for binding in supplied {
        let (is_program, identity) = match &binding.scope {
            QueryParameterScope::Rule(r) => (false, &r.name),
            QueryParameterScope::Program(p) => (true, &p.name),
        };
        if !seen.insert((is_program, identity.clone(), binding.parameter.clone())) {
            return Err(Error::new(format!(
                "duplicate query binding `{identity}.{}`",
                binding.parameter
            )));
        }
        let declared = if is_program {
            &find_program(module, identity)?.parameters
        } else {
            // A standalone Rule need not be a member of the supplied environment.
            &rules
                .iter()
                .find(|r| r.name == *identity)
                .or_else(|| find_rule(module, identity).ok())
                .ok_or_else(|| Error::new(format!("unknown query rule `{identity}`")))?
                .parameters
        };
        let parameter = declared
            .iter()
            .find(|p| p.name == binding.parameter)
            .ok_or_else(|| {
                Error::new(format!(
                    "unknown query parameter `{identity}.{}`",
                    binding.parameter
                ))
            })?;
        check_type(&binding.ty, module)?;
        check_value(&binding.value, &binding.ty, module)?;
        if !rule_type_assignable(&binding.ty, &parameter.ty, &module.declarations) {
            return Err(Error::new(format!(
                "query binding `{identity}.{}` has type {:?}; expected {:?}",
                binding.parameter, binding.ty, parameter.ty
            )));
        }
        let value = binding
            .value
            .canonical_for_type(&parameter.ty)
            .ok_or_else(|| Error::new("incompatible temporal query binding"))?;
        let mut matched_scope = false;
        for rule in rules {
            let producer = templates
                .get(&rule.name)
                .map(|t| t.rule.name.as_str())
                .unwrap_or(&rule.name);
            let origin = program_rule_origin(module, producer);
            let pattern = if let Some(origin) = origin {
                let program = find_program(module, &origin.program)?;
                if is_program && program.identity == *identity {
                    Some(Term::Var(shared_parameter(program, &binding.parameter)))
                } else if !is_program && origin.rule == *identity {
                    let member = &program.rule_uses[origin.rule_use];
                    let raw = member
                        .bindings
                        .iter()
                        .find(|b| b.parameter == binding.parameter)
                        .map(|b| b.value.clone())
                        .unwrap_or_else(|| {
                            Term::Var(format!(
                                "#{}:use{}:{}",
                                program.identity, origin.rule_use, binding.parameter
                            ))
                        });
                    let shared: BTreeMap<_, _> = program
                        .parameters
                        .iter()
                        .map(|p| {
                            (
                                p.name.clone(),
                                SubstitutionValue::Term(Term::Var(shared_parameter(
                                    program, &p.name,
                                ))),
                            )
                        })
                        .collect();
                    Some(substitute_term(&raw, &shared))
                } else {
                    None
                }
            } else if !is_program && producer == identity {
                Some(Term::Var(binding.parameter.clone()))
            } else {
                None
            };
            if let Some(pattern) = pattern {
                matched_scope = true;
                let map = constraints.entry(rule.name.clone()).or_default();
                if !unify_term(&pattern, &value.as_term(), &rule.parameters, map) {
                    return Err(Error::new(format!(
                        "query binding `{identity}.{}` conflicts with another binding or a specialized RuleUse",
                        binding.parameter
                    )));
                }
            }
        }
        if !matched_scope {
            return Err(Error::new(format!(
                "query binding scope `{identity}` is outside the selected query scope"
            )));
        }
        normalized.push(QueryBinding {
            value,
            ..binding.clone()
        });
    }
    Ok((constraints, normalized))
}

fn preflight(rules: &[Rule], goal: &Event, constraints: &ConstraintMap) -> Result<(), Error> {
    // Also validate constraints when evidence already contains the query goal.
    for rule in rules {
        if let Some(bindings) = constraints.get(&rule.name) {
            constrained_goal(rule, goal, bindings)?;
        }
    }
    Ok(())
}

fn check_type(ty: &Type, module: &Module) -> Result<(), Error> {
    match ty {
        Type::Entity(name)
            if !module
                .declarations
                .iter()
                .any(|d| matches!(d, Declaration::Entity(e) if e.name == *name)) =>
        {
            Err(Error::new(format!("unknown query binding type `{name}`")))
        }
        Type::Optional(inner) | Type::Set(inner) => check_type(inner, module),
        _ => Ok(()),
    }
}
fn check_value(value: &SubstitutionValue, ty: &Type, module: &Module) -> Result<(), Error> {
    if matches!(value, SubstitutionValue::Time(_)) && *ty != Type::PropositionTime {
        return Err(Error::new("temporal query value requires PropositionTime"));
    }
    check_term(&value.as_term(), ty, module, 0)
}
fn check_term(term: &Term, expected: &Type, module: &Module, depth: usize) -> Result<(), Error> {
    if depth > 64 {
        return Err(Error::new(
            "query binding value exceeds structural depth limit",
        ));
    }
    let mismatch = || {
        Error::new(format!(
            "query binding value {term:?} is incompatible with {expected:?}"
        ))
    };
    match term {
        Term::Const(name) => {
            let actual = module
                .declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Const(c) if c.name == *name => Some(&c.ty),
                    _ => None,
                })
                .ok_or_else(|| Error::new(format!("unknown query binding constant `{name}`")))?;
            if !rule_type_assignable(actual, expected, &module.declarations) {
                return Err(mismatch());
            }
        }
        Term::Var(_) if *expected == Type::PropositionTime => {
            return Err(Error::new(
                "query temporal binding must be resolved; omit the binding to retain a pending parameter",
            ));
        }
        Term::Var(_) => {} // Typed opaque runtime identity, never reclassified as Const.
        Term::None if matches!(expected, Type::Optional(_)) => {}
        Term::None => return Err(mismatch()),
        Term::Constructor {
            constructor,
            argument,
        } => {
            let (input, output) = constructor.signature();
            if !rule_type_assignable(&output, expected, &module.declarations) {
                return Err(mismatch());
            }
            check_term(argument, &input, module, depth + 1)?;
        }
        Term::RecordLiteral { entity, fields } => {
            if !rule_type_assignable(
                &Type::Entity(entity.clone()),
                expected,
                &module.declarations,
            ) {
                return Err(mismatch());
            }
            let declaration = module
                .declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Entity(e) if e.name == *entity => Some(e),
                    _ => None,
                })
                .ok_or_else(|| Error::new(format!("unknown query record type `{entity}`")))?;
            let mut names = BTreeSet::new();
            for field in fields {
                if !names.insert(&field.field) {
                    return Err(Error::new("duplicate query record field"));
                }
                let declared = declaration
                    .fields
                    .iter()
                    .find(|f| f.name == field.field)
                    .ok_or_else(|| {
                        Error::new(format!(
                            "unknown query record field `{}.{}`",
                            entity, field.field
                        ))
                    })?;
                check_term(&field.value, &declared.ty, module, depth + 1)?;
            }
            if fields.len() != declaration.fields.len() {
                return Err(Error::new("missing query record fields"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
