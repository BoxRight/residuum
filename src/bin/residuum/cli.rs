//! Presentation adapters over the existing experimental runners.
use std::collections::BTreeMap;
use std::fmt::Write;

use residuum::elab::Error;
use residuum::experiments::{
    ExperimentResult, render_event, render_results, run_default_experiments, run_experiments,
    run_forward_experiments,
};
use residuum::typed::{
    Declaration, DerivedEvent, Event, Substitution, SubstitutionValue, Term, TimeExpr,
};

#[path = "cli/default_output.rs"]
mod default_output;
#[path = "cli/steps.rs"]
mod steps;
pub use steps::{StepOptions, execute_file_steps};

pub const USAGE: &str = "Usage: residuum <run|query|explain> <case.res> [--steps N] [--trace]\n\
    run      Forward closure and goals; no state execution\n\
    query    Declared residual or FOUR queries\n\
    explain  Forward justifications and default selection\n\
    --steps N  At most N synchronous transitions (1..=4096)\n\
    --trace    Show every transition; default budget 64\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Run,
    Query,
    Explain,
}

impl Command {
    pub fn parse(name: &str) -> Result<Self, Error> {
        match name {
            "run" => Ok(Self::Run),
            "query" => Ok(Self::Query),
            "explain" => Ok(Self::Explain),
            _ => Err(Error::new(format!("unknown command `{name}`\n{USAGE}"))),
        }
    }
}

/// No inference is implemented here: each command delegates to an existing API.
pub fn execute(command: Command, source: &str) -> Result<String, Error> {
    let module = residuum::parse_formal_to_typed(source)?;
    execute_module(command, &module)
}

pub fn execute_file(command: Command, path: &std::path::Path) -> Result<String, Error> {
    execute_module(command, &residuum::load_formal_module(path)?)
}

fn execute_module(command: Command, module: &residuum::typed::Module) -> Result<String, Error> {
    if command != Command::Query
        && module
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Default(_)))
    {
        let results = run_default_experiments(module)?;
        return Ok(default_output::render(module, &results, command));
    }
    match command {
        Command::Run => Ok(render_results(&run_forward_experiments(module)?)),
        Command::Query => Ok(render_results(&run_experiments(module)?)),
        Command::Explain => Ok(render_explanations(
            module,
            &run_forward_experiments(module)?,
        )),
    }
}

fn render_explanations(module: &residuum::typed::Module, results: &[ExperimentResult]) -> String {
    let mut output = String::new();
    for result in results {
        writeln!(output, "experiment {}", result.experiment.name).unwrap();
        writeln!(output, "  goal: {}", render_event(&result.experiment.goal)).unwrap();
        if !result.horn_goal {
            writeln!(output, "  not derived from these seeds").unwrap();
            continue;
        }

        let mut proofs: BTreeMap<&Event, Vec<&DerivedEvent>> = BTreeMap::new();
        for derivation in &result.closure.derivations {
            proofs
                .entry(&derivation.event)
                .or_default()
                .push(derivation);
        }
        // References rather than recursive expansion preserve every justification
        // without duplicating shared branches or looping through a proof cycle.
        let mut ids = BTreeMap::from([(result.experiment.goal.clone(), 1)]);
        let mut events = vec![result.experiment.goal.clone()];
        let mut index = 0;
        while index < events.len() {
            let event = events[index].clone();
            writeln!(output, "  #{} {}", index + 1, render_event(&event)).unwrap();
            if let Some(origin) = module.origin(&event.verb).filter(|o| o.file.is_some()) {
                writeln!(
                    output,
                    "    verb origin: {} ({})",
                    origin.module,
                    origin.file.as_ref().unwrap().display()
                )
                .unwrap();
                if let Some(reference) = module.declarations.iter().find_map(|d| match d {
                    Declaration::Verb(v) if v.name == event.verb => v.program.as_ref(),
                    _ => None,
                }) {
                    let target = module.origin(&reference.name).unwrap();
                    writeln!(
                        output,
                        "    associated program: {} | origin: {}",
                        reference.name, target.module
                    )
                    .unwrap();
                }
            }
            if result.experiment.input.contains(&event) {
                writeln!(output, "    seed").unwrap();
            }
            for derivation in proofs.get(&event).into_iter().flatten() {
                writeln!(
                    output,
                    "    rule {} | derivedAt {}",
                    derivation.record.rule, derivation.derived_at.name
                )
                .unwrap();
                if let Some(origin) =
                    residuum::programs::program_rule_origin(module, &derivation.record.rule)
                {
                    writeln!(
                        output,
                        "      program {} | module {} | member {} | rule {} | rule module {}",
                        origin.program,
                        origin.defining_module,
                        origin.rule_use,
                        origin.rule,
                        origin.rule_origin.module
                    )
                    .unwrap();
                }
                if let Some(origin) = module
                    .origin(&derivation.record.rule)
                    .filter(|o| o.file.is_some())
                {
                    writeln!(
                        output,
                        "      rule origin: {} ({})",
                        origin.module,
                        origin.file.as_ref().unwrap().display()
                    )
                    .unwrap();
                }
                writeln!(
                    output,
                    "      substitution: {}",
                    render_substitution(&derivation.record.substitution)
                )
                .unwrap();
                let mut references = Vec::new();
                for antecedent in &derivation.record.antecedents {
                    let id = *ids.entry(antecedent.clone()).or_insert_with(|| {
                        events.push(antecedent.clone());
                        events.len()
                    });
                    references.push(format!("#{id}"));
                }
                writeln!(
                    output,
                    "      antecedents: {}",
                    if references.is_empty() {
                        "I".into()
                    } else {
                        references.join(", ")
                    }
                )
                .unwrap();
            }
            index += 1;
        }
    }
    output
}

fn render_substitution(substitution: &Substitution) -> String {
    let bindings: Vec<_> = substitution
        .bindings
        .iter()
        .map(|binding| {
            let value = match &binding.value {
                SubstitutionValue::Term(Term::Var(name) | Term::Const(name)) => name.clone(),
                SubstitutionValue::Term(Term::None) => "none".into(),
                SubstitutionValue::Term(value) => format!("{value:?}"),
                SubstitutionValue::Time(time) => render_time(time),
            };
            format!("{}={value}", binding.parameter)
        })
        .collect();
    format!("{{{}}}", bindings.join(", "))
}

fn render_time(time: &TimeExpr) -> String {
    match time {
        TimeExpr::At(name) => name.clone(),
        TimeExpr::Var(name) => format!("?{name}"),
        TimeExpr::After(time) => format!("after({})", render_time(time)),
    }
}

#[cfg(test)]
#[path = "cli/tests.rs"]
mod tests;
