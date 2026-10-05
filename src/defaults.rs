use std::collections::{BTreeMap, BTreeSet};

use crate::derive::{match_antecedents, substitution_record};
use crate::elab::Error;
use crate::typed::{Declaration, DefaultRef, Event, Module, ProgramRef, Rule, Substitution};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicabilityMatch {
    pub antecedents: Vec<Event>,
    pub substitution: Substitution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicableDefault {
    pub default: DefaultRef,
    pub matches: Vec<ApplicabilityMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyStatus {
    NotChecked,
}

/// Provisional scope: any witness blocks the entire parametrized rule lambda.
/// This is not yet a decision about defaults evaluated per ground instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockingScope {
    RuleLambda,
    ProgramLambda,
    Lambda, // Mixed Rule/Program targets, still whole-lambda blocking.
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultBlock {
    pub default: DefaultRef,
    pub evidence: Vec<Event>,
    pub substitution: Substitution,
}

/// A provisional extension: applicability, blocking and priority have been evaluated,
/// but consistency between M1 and M2 has not been checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultExtension {
    pub applicable: Vec<ApplicableDefault>,
    pub blocked: Vec<DefaultBlock>,
    pub blocking_scope: BlockingScope,
    pub defeated: Vec<DefaultRef>,
    pub selected: Vec<DefaultRef>,
    pub consistency: ConsistencyStatus,
    rules: Vec<Rule>,
    programs: Vec<ProgramRef>,
}

impl DefaultExtension {
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
    /// Selected autonomous units, not an external harness association.
    pub fn programs(&self) -> &[ProgramRef] {
        &self.programs
    }
}

/// Context is Known, supplied by the caller (for example Closure.known).
/// A condition needs at least one shared-substitution match; no condition is top.
/// This selects complete lambdas globally, not individual ground instances.
/// Blocking uses the same supplied Known, before priority; it does not compute
/// a prospective closure or infer blockers from the absence of evidence.
pub fn evaluate_defaults(module: &Module, context: &[Event]) -> Result<DefaultExtension, Error> {
    let mut applicable = Vec::new();
    for declaration in &module.declarations {
        let Declaration::Default(default) = declaration else {
            continue;
        };
        let matches = match &default.condition {
            None => vec![ApplicabilityMatch {
                antecedents: Vec::new(),
                substitution: Substitution {
                    bindings: Vec::new(),
                },
            }],
            Some(condition) => match_antecedents(condition, context, &default.parameters)
                .into_iter()
                .map(|matched| ApplicabilityMatch {
                    antecedents: matched.antecedents,
                    substitution: substitution_record(&matched.substitutions),
                })
                .collect(),
        };
        if !matches.is_empty() {
            applicable.push(ApplicableDefault {
                default: DefaultRef {
                    name: default.name.clone(),
                },
                matches,
            });
        }
    }
    let applicable_names: BTreeSet<_> = applicable
        .iter()
        .map(|item| item.default.name.as_str())
        .collect();
    let mut blocked = Vec::new();
    for declaration in &module.declarations {
        let Declaration::Default(default) = declaration else {
            continue;
        };
        if !applicable_names.contains(default.name.as_str()) {
            continue;
        }
        if let Some(blocking) = &default.blocking {
            for matched in match_antecedents(blocking, context, &default.parameters) {
                blocked.push(DefaultBlock {
                    default: DefaultRef {
                        name: default.name.clone(),
                    },
                    evidence: matched.antecedents,
                    substitution: substitution_record(&matched.substitutions),
                });
            }
        }
    }
    let blocked_names: BTreeSet<_> = blocked
        .iter()
        .map(|item| item.default.name.as_str())
        .collect();
    let candidates: Vec<_> = applicable_names
        .difference(&blocked_names)
        .copied()
        .collect();
    let selection = select_candidates(module, &candidates, &blocked_names)?;
    Ok(DefaultExtension {
        applicable,
        blocked,
        blocking_scope: {
            let targets: Vec<_> = module
                .declarations
                .iter()
                .filter_map(|d| match d {
                    Declaration::Default(d) => Some(d.body.is_program()),
                    _ => None,
                })
                .collect();
            if targets.iter().all(|p| !p) {
                BlockingScope::RuleLambda
            } else if targets.iter().all(|p| *p) {
                BlockingScope::ProgramLambda
            } else {
                BlockingScope::Lambda
            }
        },
        defeated: selection.defeated,
        selected: selection.selected,
        rules: selection.rules,
        programs: selection.programs,
        consistency: ConsistencyStatus::NotChecked,
    })
}

struct PrioritySelection {
    defeated: Vec<DefaultRef>,
    selected: Vec<DefaultRef>,
    rules: Vec<Rule>,
    programs: Vec<ProgramRef>,
}

/// Provisional priority harness with externally chosen candidates.
/// It deliberately bypasses applicability, blocking and consistency. For context-based
/// evaluation, use evaluate_defaults(...).rules().
pub fn select_active_rules(module: &Module, activated: &[&str]) -> Result<Vec<Rule>, Error> {
    Ok(select_candidates(module, activated, &BTreeSet::new())?.rules)
}

/// An activated exception defeats its target (and that target's lower priorities),
/// without implicitly activating any of them.
fn select_candidates(
    module: &Module,
    activated: &[&str],
    blocked: &BTreeSet<&str>,
) -> Result<PrioritySelection, Error> {
    let mut defaults = BTreeMap::new();
    let mut rules = BTreeMap::new();
    let mut programs = BTreeMap::new();
    for declaration in &module.declarations {
        match declaration {
            Declaration::Default(default) => {
                if defaults.insert(default.name.as_str(), default).is_some() {
                    return Err(Error::new(format!("duplicate default `{}`", default.name)));
                }
            }
            Declaration::Rule(rule) => {
                if rules.insert(rule.name.as_str(), rule).is_some() {
                    return Err(Error::new(format!("duplicate rule `{}`", rule.name)));
                }
            }
            Declaration::Program(program) => {
                if programs
                    .insert(program.identity.as_str(), program)
                    .is_some()
                {
                    return Err(Error::new(format!(
                        "duplicate program `{}`",
                        program.identity
                    )));
                }
            }
            _ => {}
        }
    }

    let activated: BTreeSet<_> = activated.iter().copied().collect();
    let mut defeated = BTreeSet::new();
    for name in &activated {
        let mut current = *defaults
            .get(name)
            .ok_or_else(|| Error::new(format!("unknown activated default `{name}`")))?;
        let mut visited = BTreeSet::from([current.name.as_str()]);
        while let Some(to) = current.body.exception_parent() {
            // A blocked default contributes no outgoing exception edge,
            // including when reached through a chain of priorities.
            if blocked.contains(current.name.as_str()) {
                break;
            }
            if !visited.insert(to.name.as_str()) {
                return Err(Error::new(format!(
                    "cyclic default priority at `{}`",
                    to.name
                )));
            }
            defeated.insert(to.name.as_str());
            current = *defaults.get(to.name.as_str()).ok_or_else(|| {
                Error::new(format!(
                    "unknown default `{}` in exception default",
                    to.name
                ))
            })?;
        }
    }

    let surviving: BTreeSet<_> = activated.difference(&defeated).copied().collect();
    let mut selected_rules = BTreeSet::new();
    let mut selected_programs = BTreeSet::new();
    for name in &surviving {
        let body = &defaults[name].body;
        let target = body.target_name();
        if body.is_program() {
            if !programs.contains_key(target) {
                return Err(Error::new(format!(
                    "unknown program `{target}` in default `{name}`"
                )));
            }
            selected_programs.insert(target);
        } else {
            if !rules.contains_key(target) {
                return Err(Error::new(format!(
                    "unknown rule `{target}` in default `{name}`"
                )));
            }
            selected_rules.insert(target);
        }
    }

    // Resolve whole surviving units first. Only then lower their bindings.
    // Applicability substitutions never turn global selection into per-instance selection.
    let mut rules = Vec::new();
    let mut program_refs = Vec::new();
    for declaration in &module.declarations {
        match declaration {
            Declaration::Rule(rule) if selected_rules.contains(rule.name.as_str()) => {
                rules.push(rule.clone())
            }
            Declaration::Program(program)
                if selected_programs.contains(program.identity.as_str()) =>
            {
                program_refs.push(ProgramRef {
                    name: program.identity.clone(),
                });
                rules.extend(crate::programs::compile_program(program, module)?);
            }
            _ => {}
        }
    }
    let mut selected = Vec::new();
    let mut defeated_defaults = Vec::new();
    for declaration in &module.declarations {
        if let Declaration::Default(default) = declaration {
            if surviving.contains(default.name.as_str()) {
                selected.push(DefaultRef {
                    name: default.name.clone(),
                });
            } else if activated.contains(default.name.as_str())
                && defeated.contains(default.name.as_str())
            {
                defeated_defaults.push(DefaultRef {
                    name: default.name.clone(),
                });
            }
        }
    }
    Ok(PrioritySelection {
        rules,
        programs: program_refs,
        selected,
        defeated: defeated_defaults,
    })
}
