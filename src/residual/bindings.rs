//! Scoped, typed query constraints. Neither rules nor evidence are specialized
//! in place; search still replays the original producers for certification.
use std::collections::{BTreeMap, BTreeSet};

use super::active::{certify, guarded_view};
use super::query::{Bindings, constrained_goal, query_with_constraints, unify_term};
use super::{Abducible, ActiveProgramResidualQuery, ProgramResidualQuery, SearchLimits};
use crate::derive::DerivationClock;
use crate::elab::{Error, is_assignable, rule_type_assignable};
use crate::programs::{
    compile_program, program_rule_origin, rule_use_substitution, shared_parameter,
};
use crate::typed::*;

// Preserve the existing public paths while sharing the interface data with TypedAST.
pub use crate::typed::{HornQueryScope, QueryBinding, QueryParameterScope};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryWithBindings<T> {
    pub query: T,
    /// Validated supplied constraints, with canonical temporal representation.
    pub bindings: Vec<QueryBinding>,
}

/// Empty bindings delegate to the legacy local contract. Nonempty bindings
/// retain Unit branches and every unbound typed parameter; see the documented
/// local-query Unit contract in docs/residual-explicit-bindings-api.md.
pub fn query_residual_with_bindings(
    module: &Module,
    rule: &Rule,
    goal: &Event,
    known: &[Event],
    bindings: &[QueryBinding],
) -> Result<QueryWithBindings<Vec<ResidualRequirement>>, Error> {
    if bindings.is_empty() {
        return Ok(QueryWithBindings {
            query: super::query::query_residual(rule, goal, known)?,
            bindings: Vec::new(),
        });
    }
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
    if !bindings.is_empty()
        && module
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Default(_)))
    {
        return Err(Error::new(
            "bound residual queries require a monotonic module without defaults",
        ));
    }
    let rules = scoped_rules(module, scope)?;
    let view = horn_view(module, &rules);
    if bindings.is_empty() {
        let query = match allowed {
            None => super::query_program_residual(&view, evidence, goal, limits),
            Some(allowed) => super::query_program_residual_with_abducibles(
                &view, evidence, goal, allowed, limits,
            ),
        }?;
        return Ok(QueryWithBindings {
            query,
            bindings: Vec::new(),
        });
    }
    let (constraints, bindings) = resolve(module, &rules, &BTreeMap::new(), bindings)?;
    preflight(&rules, goal, &constraints)?;
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

/// The exact existing Horn scope selection, shared with source orchestration.
pub(crate) fn scoped_rules(module: &Module, scope: &HornQueryScope) -> Result<Vec<Rule>, Error> {
    match scope {
        HornQueryScope::Rules(references) => {
            let mut selected = BTreeSet::new();
            let mut rules = Vec::new();
            for reference in references {
                if !selected.insert(&reference.name) {
                    return Err(Error::new("duplicate query scope rule"));
                }
                rules.push(find_rule(module, &reference.name)?.clone());
            }
            Ok(rules)
        }
        HornQueryScope::Program(reference) => {
            compile_program(find_program(module, &reference.name)?, module)
        }
    }
}

fn horn_view(module: &Module, rules: &[Rule]) -> Module {
    let mut view = module.clone();
    view.declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    view.declarations
        .extend(rules.iter().cloned().map(Declaration::Rule));
    view
}

/// Begin layered search using the same scope selection and constraint resolver
/// as the full binding interface; retain legacy validation for empty bindings.
pub fn begin_residual_query_with_bindings(
    module: &Module,
    scope: &HornQueryScope,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
) -> Result<QueryWithBindings<super::ResidualSession>, Error> {
    let rules = scoped_rules(module, scope)?;
    let view = horn_view(module, &rules);
    if bindings.is_empty() {
        return Ok(QueryWithBindings {
            query: super::ResidualSession::new(&view, evidence, goal, limits, allowed)?,
            bindings: Vec::new(),
        });
    }
    let (constraints, bindings) = resolve(module, &rules, &BTreeMap::new(), bindings)?;
    preflight(&rules, goal, &constraints)?;
    Ok(QueryWithBindings {
        query: super::ResidualSession::new(&view, evidence, goal, limits, allowed)?
            .constrained(constraints),
        bindings,
    })
}

pub fn begin_active_residual_query_with_bindings(
    module: &Module,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
) -> Result<QueryWithBindings<super::ActiveResidualSession>, Error> {
    begin_active_query(module, None, evidence, goal, bindings, limits, allowed)
}

pub fn begin_active_residual_query_in_scope_with_bindings(
    module: &Module,
    scope: &HornQueryScope,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
) -> Result<QueryWithBindings<super::ActiveResidualSession>, Error> {
    begin_active_query(
        module,
        Some(scope),
        evidence,
        goal,
        bindings,
        limits,
        allowed,
    )
}

fn begin_active_query(
    module: &Module,
    scope: Option<&HornQueryScope>,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
) -> Result<QueryWithBindings<super::ActiveResidualSession>, Error> {
    if scope.is_none() && bindings.is_empty() {
        return Ok(QueryWithBindings {
            query: super::ActiveResidualSession::new(module, evidence, goal, limits, allowed)?,
            bindings: Vec::new(),
        });
    }
    let (view, templates) = active_view(module, scope)?;
    let rules = view
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let (constraints, normalized) = resolve(module, &rules, &templates, bindings)?;
    preflight(&rules, goal, &constraints)?;
    let mut query = super::ActiveResidualSession::from_view(
        module, &view, templates, evidence, goal, limits, allowed,
    )?;
    if !bindings.is_empty() {
        query = query.constrained(constraints);
    }
    Ok(QueryWithBindings {
        query,
        bindings: normalized,
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
    active_query(
        module, None, evidence, goal, bindings, limits, allowed, clock,
    )
}

/// Restrict the original guarded search producers, retaining their guards.
/// Certification still runs close_program over the unchanged original module.
pub fn query_active_program_residual_in_scope_with_bindings<C: DerivationClock>(
    module: &Module,
    scope: &HornQueryScope,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
    clock: &mut C,
) -> Result<QueryWithBindings<ActiveProgramResidualQuery>, Error> {
    active_query(
        module,
        Some(scope),
        evidence,
        goal,
        bindings,
        limits,
        allowed,
        clock,
    )
}

fn active_query<C: DerivationClock>(
    module: &Module,
    scope: Option<&HornQueryScope>,
    evidence: &[Event],
    goal: &Event,
    bindings: &[QueryBinding],
    limits: SearchLimits,
    allowed: Option<&[Abducible]>,
    clock: &mut C,
) -> Result<QueryWithBindings<ActiveProgramResidualQuery>, Error> {
    if scope.is_none() && bindings.is_empty() {
        let query = match allowed {
            None => super::query_active_program_residual(module, evidence, goal, limits, clock),
            Some(allowed) => super::query_active_program_residual_with_abducibles(
                module, evidence, goal, allowed, limits, clock,
            ),
        }?;
        return Ok(QueryWithBindings {
            query,
            bindings: Vec::new(),
        });
    }
    let (view, templates) = active_view(module, scope)?;
    let rules: Vec<_> = view
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let constrained = !bindings.is_empty();
    let (constraints, bindings) = resolve(module, &rules, &templates, bindings)?;
    preflight(&rules, goal, &constraints)?;
    let defaults = default_abducibles(module);
    let search = super::program::query_with_constraints(
        &view,
        evidence,
        goal,
        allowed.unwrap_or(&defaults),
        limits,
        constrained.then_some(&constraints),
    )?;
    Ok(QueryWithBindings {
        query: certify(module, evidence, goal, search, &templates, clock)?,
        bindings,
    })
}

fn active_view(
    module: &Module,
    scope: Option<&HornQueryScope>,
) -> Result<(Module, BTreeMap<String, super::ResidualActivation>), Error> {
    let (mut view, mut templates) = guarded_view(module)?;
    if let Some(scope) = scope {
        let selected: BTreeSet<_> = scoped_rules(module, scope)?
            .into_iter()
            .map(|r| r.name)
            .collect();
        view.declarations.retain(|d| match d {
            Declaration::Rule(r) => {
                let producer = templates.get(&r.name);
                let source = producer
                    .and_then(|t| t.program_origin.as_ref())
                    .map(|o| o.rule.as_str());
                selected.contains(producer.map(|t| t.rule.name.as_str()).unwrap_or(&r.name))
                    || (matches!(scope, HornQueryScope::Rules(_))
                        && source.is_some_and(|s| selected.contains(s)))
            }
            _ => true,
        });
        let names: BTreeSet<_> = view
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) => Some(r.name.as_str()),
                _ => None,
            })
            .collect();
        templates.retain(|name, _| names.contains(name.as_str()));
    }
    Ok((view, templates))
}

/// Source adapter validation only: no search, hypotheses or closure at elaboration.
pub(crate) fn validate_query_options(
    module: &Module,
    options: &mut QueryOptions,
    goal: &Event,
) -> Result<(), Error> {
    if options.is_legacy() {
        return Ok(());
    }
    if options.scope.is_none() && !options.bindings.is_empty() {
        return Err(Error::new(
            "explicit query bindings require an explicit search scope",
        ));
    }
    let (rules, templates) = if options.active {
        let (view, templates) = active_view(module, options.scope.as_ref())?;
        (
            view.declarations
                .into_iter()
                .filter_map(|d| match d {
                    Declaration::Rule(r) => Some(r),
                    _ => None,
                })
                .collect(),
            templates,
        )
    } else {
        (
            scoped_rules(
                module,
                options
                    .scope
                    .as_ref()
                    .ok_or_else(|| Error::new("scoped residual query requires a search scope"))?,
            )?,
            BTreeMap::new(),
        )
    };
    let (constraints, bindings) = resolve(module, &rules, &templates, &options.bindings)?;
    preflight(&rules, goal, &constraints)?;
    options.bindings = bindings;
    Ok(())
}

pub(crate) fn query_parameter_type(
    module: &Module,
    scope: &QueryParameterScope,
    name: &str,
) -> Result<Type, Error> {
    Ok(binding_parameter(module, &[], scope, name)?.ty.clone())
}

fn binding_parameter<'a>(
    module: &'a Module,
    rules: &'a [Rule],
    scope: &QueryParameterScope,
    name: &str,
) -> Result<&'a Arg, Error> {
    let (identity, parameters) = match scope {
        QueryParameterScope::Program(p) => (&p.name, &find_program(module, &p.name)?.parameters),
        QueryParameterScope::Rule(r) => (
            &r.name,
            &rules
                .iter()
                .find(|candidate| candidate.name == r.name)
                .or_else(|| find_rule(module, &r.name).ok())
                .ok_or_else(|| Error::new(format!("unknown query rule `{}`", r.name)))?
                .parameters,
        ),
    };
    parameters
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| Error::new(format!("unknown query parameter `{identity}.{name}`")))
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
        let parameter = binding_parameter(module, rules, &binding.scope, &binding.parameter)?;
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
                    let (_, mapping) = rule_use_substitution(
                        program,
                        origin.rule_use,
                        find_rule(module, &origin.rule)?,
                    )?;
                    Some(mapping[&binding.parameter].as_term())
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
    check_term(&value.as_term(), ty, module, 0, false)
}
fn check_term(
    term: &Term,
    expected: &Type,
    module: &Module,
    depth: usize,
    record_field: bool,
) -> Result<(), Error> {
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
    let assignable = |actual: &Type| {
        if record_field {
            is_assignable(actual, expected)
        } else {
            rule_type_assignable(actual, expected, &module.declarations)
        }
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
            if !assignable(actual) {
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
            if !assignable(&output) {
                return Err(mismatch());
            }
            check_term(argument, &input, module, depth + 1, false)?;
        }
        Term::RecordLiteral { entity, fields } => {
            if !assignable(&Type::Entity(entity.clone())) {
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
                check_term(&field.value, &declared.ty, module, depth + 1, true)?;
            }
            if fields.len() != declaration.fields.len() {
                return Err(Error::new("missing query record fields"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod compatibility_tests;
#[cfg(test)]
mod session_tests;
#[cfg(test)]
mod tests;
