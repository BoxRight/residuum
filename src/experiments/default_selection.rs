//! A bounded, one-shot orchestration of the existing default and Horn APIs.
//! No new default semantics: mandatory closure -> evaluate -> selected closure.
use std::collections::BTreeSet;

use crate::defaults::{DefaultExtension, evaluate_defaults};
use crate::elab::Error;
use crate::typed::{Closure, Declaration, Module};

use super::{ExperimentResult, run_forward_experiments};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultExperimentResult {
    pub context: Closure,
    pub extension: DefaultExtension,
    pub observation: ExperimentResult,
}

/// Each experiment is independent. Applicability and blockers see the mandatory
/// Horn closure, never consequences of a default before it has been selected.
/// Selection is not reconsidered after the final closure (no Reiter iteration).
pub fn run_default_experiments(module: &Module) -> Result<Vec<DefaultExperimentResult>, Error> {
    let mandatory = mandatory_module(module)?;
    // Reuse the existing runner's seed-only and acyclic termination checks.
    let contexts = run_forward_experiments(&mandatory)?;
    let mut results = Vec::new();
    for context in contexts {
        let extension = evaluate_defaults(module, &context.closure.known)?;
        let mut active = mandatory.clone();
        active
            .declarations
            .retain(|d| !matches!(d, Declaration::Experiment(_)));
        // All selected member clauses and the unchanged mandatory environment
        // participate in one module-aware fixpoint, never per-member closures.
        active
            .declarations
            .extend(extension.rules().iter().cloned().map(Declaration::Rule));
        active
            .declarations
            .push(Declaration::Experiment(context.experiment));
        let observation = run_forward_experiments(&active)?.remove(0);
        results.push(DefaultExperimentResult {
            context: context.closure,
            extension,
            observation,
        });
    }
    Ok(results)
}

pub(super) fn mandatory_module(module: &Module) -> Result<Module, Error> {
    let mut controlled = BTreeSet::new();
    let mut controlled_programs = BTreeSet::new();
    for declaration in &module.declarations {
        let Declaration::Default(default) = declaration else {
            continue;
        };
        if default.body.is_program() {
            let name = default.body.target_name();
            let program = module
                .declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Program(p) if p.identity == name => Some(p),
                    _ => None,
                })
                .ok_or_else(|| Error::new(format!("unresolved default program `{name}`")))?;
            controlled_programs.insert(name);
            // Only actual members are controlled. Legal rules in the lexical
            // environment remain mandatory and are never cloned into the package.
            controlled.extend(program.rule_uses.iter().map(|u| u.rule.name.as_str()));
        } else {
            controlled.insert(default.body.target_name());
        }
    }
    if controlled.is_empty() {
        return Err(Error::new(
            "default experiments require a default declaration",
        ));
    }
    // A default-controlled lambda must not bypass selection by being activated
    // through Verb.program. That combination has no agreed semantics yet.
    for declaration in &module.declarations {
        if let Declaration::Verb(verb) = declaration {
            if verb
                .program
                .as_ref()
                .is_some_and(|r| controlled.contains(r.name.as_str()) || controlled_programs.contains(r.name.as_str())
                    || module.declarations.iter().any(|d| matches!(d, Declaration::Program(p)
                        if p.identity == r.name && p.rule_uses.iter().any(|u| controlled.contains(u.rule.name.as_str())))))
            {
                return Err(Error::new(
                    "default-controlled rules cannot also be associated verb programs in this runner",
                ));
            }
        }
    }
    let mut mandatory = module.clone();
    mandatory.declarations.retain(|d| match d {
        Declaration::Default(_) => false,
        Declaration::Rule(r) => !controlled.contains(r.name.as_str()),
        _ => true,
    });
    Ok(mandatory)
}
