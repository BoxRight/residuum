//! Small source-defined experiments, not a general CLI or a new Horn calculus.

use std::collections::{BTreeMap, BTreeSet};

use crate::elab::Error;
use crate::residual::{ProgramResidualQuery, RequirementFormula, SearchLimits};
use crate::typed::{
    Antecedent, Closure, Declaration, DerivationTime, Event, Experiment, ExperimentQueryKind,
    Module, Polarity, Rule, RuleBody, Term, TimeExpr,
};
use crate::{DerivationClock, close, query_program_residual};

mod four;
mod incremental;

/// Fixture composition only: flat shared declarations, not a module/import
/// system. Both sources are parsed independently before ordinary elaboration.
pub fn elaborate_composed_fixture(
    library: crate::surface::FormalModule,
    consumer: crate::surface::FormalModule,
) -> Result<Module, Error> {
    if library
        .declarations
        .iter()
        .any(|d| matches!(d, crate::surface::FormalDeclaration::Experiment(_)))
    {
        return Err(Error::new(
            "fixture library must not contain experiment inputs",
        ));
    }
    let mut declarations = library.declarations;
    declarations.extend(consumer.declarations);
    crate::elaborate_formal(crate::surface::FormalModule {
        name: consumer.name,
        declarations,
    })
}

pub use incremental::{
    IncrementalExperimentResult, render_incremental, run_incremental_experiment,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FourEntailment {
    pub interpretations: usize,
    pub models: usize,
    pub entailed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentResult {
    pub experiment: Experiment,
    pub closure: Closure,
    pub horn_goal: bool,
    pub residual: Option<ProgramResidualQuery>,
    pub four: Option<FourEntailment>,
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}

pub fn run_experiments(module: &Module) -> Result<Vec<ExperimentResult>, Error> {
    if module
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Default(_)))
    {
        return Err(Error::new(
            "experimental runner requires a monotonic module without defaults",
        ));
    }
    let rules: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(rule) => Some(rule.clone()),
            _ => None,
        })
        .collect();
    let mut results = Vec::new();
    for declaration in &module.declarations {
        let Declaration::Experiment(experiment) = declaration else {
            continue;
        };
        let (residual, four) = match experiment.query_kind {
            ExperimentQueryKind::Residual => {
                require_acyclic(&rules)?;
                (
                    Some(query_program_residual(
                        module,
                        &experiment.input,
                        &experiment.goal,
                        SearchLimits::default(),
                    )?),
                    None,
                )
            }
            ExperimentQueryKind::FourFusion => {
                // Check the finite ground universe before calling close.
                (None, Some(four::entailment(experiment, &rules)?))
            }
        };
        let closure = close(experiment.input.clone(), &rules, &mut Clock::default());
        results.push(ExperimentResult {
            experiment: experiment.clone(),
            horn_goal: closure.known.contains(&experiment.goal),
            closure,
            residual,
            four,
        });
    }
    if results.is_empty() {
        return Err(Error::new("module contains no experiments"));
    }
    Ok(results)
}

/// Forward observations only. Query goals label observations; this mode does
/// not invoke residual search, FOUR entailment, or state execution.
pub fn run_forward_experiments(module: &Module) -> Result<Vec<ExperimentResult>, Error> {
    let dependencies: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let mut activation_edges = Vec::new();
    // An associated body may be I: its implicit activation guard must also
    // participate in the runner's conservative termination check.
    for declaration in &module.declarations {
        let Declaration::Verb(verb) = declaration else {
            continue;
        };
        if !matches!(
            verb.kind,
            crate::typed::VerbKind::Seeded | crate::typed::VerbKind::Derived
        ) {
            continue;
        }
        let Some(reference) = &verb.program else {
            continue;
        };
        let rule = dependencies
            .iter()
            .find(|r| r.name == reference.name)
            .ok_or_else(|| Error::new("unresolved associated program"))?;
        activation_edges.push((verb.name.clone(), horn(rule).consequent.verb));
    }
    require_acyclic_with_edges(&dependencies, activation_edges)?;
    let mut results = Vec::new();
    for declaration in &module.declarations {
        let Declaration::Experiment(experiment) = declaration else {
            continue;
        };
        if experiment.input_kind != crate::typed::ExperimentInputKind::Seeds {
            return Err(Error::new(
                "forward experiments require seed inputs, not premises",
            ));
        }
        let closure =
            crate::close_program(experiment.input.clone(), module, &mut Clock::default())?;
        results.push(ExperimentResult {
            experiment: experiment.clone(),
            horn_goal: closure.known.contains(&experiment.goal),
            closure,
            residual: None,
            four: None,
        });
    }
    if results.is_empty() {
        return Err(Error::new("module contains no experiments"));
    }
    Ok(results)
}

pub(crate) fn horn(rule: &Rule) -> crate::typed::HornClause {
    match &rule.body {
        RuleBody::HornClause(clause) => clause.clone(),
        RuleBody::ResidualClause(clause) => clause.unresiduate(),
    }
}

fn antecedent_verbs(antecedent: &Antecedent, output: &mut Vec<String>) {
    match antecedent {
        Antecedent::Unit => {}
        Antecedent::Event(event) => output.push(event.verb.clone()),
        Antecedent::And(a, b) => {
            antecedent_verbs(a, output);
            antecedent_verbs(b, output);
        }
    }
}

// Sufficient finite termination condition for this runner. close is unchanged.
fn require_acyclic(rules: &[Rule]) -> Result<(), Error> {
    require_acyclic_with_edges(rules, Vec::new())
}

fn require_acyclic_with_edges(
    rules: &[Rule],
    activation_edges: Vec<(String, String)>,
) -> Result<(), Error> {
    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (source, target) in activation_edges {
        edges.entry(source).or_default().push(target);
    }
    for rule in rules {
        let clause = horn(rule);
        let mut sources = Vec::new();
        antecedent_verbs(&clause.antecedent, &mut sources);
        for source in sources {
            edges
                .entry(source)
                .or_default()
                .push(clause.consequent.verb.clone());
        }
    }
    fn visit(
        name: &str,
        edges: &BTreeMap<String, Vec<String>>,
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> bool {
        if done.contains(name) {
            return true;
        }
        if !active.insert(name.into()) {
            return false;
        }
        for next in edges.get(name).into_iter().flatten() {
            if !visit(next, edges, active, done) {
                return false;
            }
        }
        active.remove(name);
        done.insert(name.into());
        true
    }
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    for name in edges.keys() {
        if !visit(name, &edges, &mut active, &mut done) {
            return Err(Error::new(
                "residual fixture runner supports acyclic rule dependencies only",
            ));
        }
    }
    Ok(())
}

pub fn render_results(results: &[ExperimentResult]) -> String {
    let mut output = String::new();
    for result in results {
        output.push_str(&format!("experiment {}\n", result.experiment.name));
        output.push_str(&format!(
            "  {:?}: {{{}}}\n",
            result.experiment.input_kind,
            result
                .experiment
                .input
                .iter()
                .map(render_event)
                .collect::<Vec<_>>()
                .join(", ")
        ));
        output.push_str(&format!(
            "  goal: {}\n  Horn: {}\n",
            render_event(&result.experiment.goal),
            result.horn_goal
        ));
        output.push_str(&format!(
            "  Known after close: {{{}}}\n",
            result
                .closure
                .known
                .iter()
                .map(render_event)
                .collect::<Vec<_>>()
                .join(", ")
        ));
        if let Some(residual) = &result.residual {
            output.push_str(&format!(
                "  requirement: {}\n",
                residual
                    .required
                    .as_ref()
                    .map(render_requirement)
                    .unwrap_or_else(|| "no explanation".into())
            ));
        }
        if let Some(four) = &result.four {
            output.push_str(&format!(
                "  FOUR fusion: {} ({} models / {} interpretations)\n",
                four.entailed, four.models, four.interpretations
            ));
        }
    }
    output
}

pub fn render_event(event: &Event) -> String {
    format!(
        "{}{}({}) @ {}",
        if event.polarity == Polarity::EvidentialNot {
            "~"
        } else {
            ""
        },
        event.verb,
        event
            .args
            .iter()
            .map(|term| match term {
                Term::Var(name) | Term::Const(name) => name.clone(),
                Term::None => "none".into(),
                Term::RecordLiteral { entity, .. } => format!("{entity} {{...}}"),
            })
            .collect::<Vec<_>>()
            .join(", "),
        render_time(&event.proposition.time)
    )
}
fn render_time(time: &TimeExpr) -> String {
    match time {
        TimeExpr::At(name) => name.clone(),
        TimeExpr::After(time) => format!("after({})", render_time(time)),
    }
}
pub fn render_requirement(requirement: &RequirementFormula) -> String {
    match requirement {
        RequirementFormula::Unit => "I".into(),
        RequirementFormula::Event(event) => render_event(event),
        RequirementFormula::Tensor(a, b) => {
            format!("({} ⊗ {})", render_requirement(a), render_requirement(b))
        }
        RequirementFormula::Join(a, b) => {
            format!("({} ∨ {})", render_requirement(a), render_requirement(b))
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod owns_tests;

#[cfg(test)]
mod reuse_tests;

#[cfg(test)]
mod interface_tests;

#[cfg(test)]
mod refinement_tests;

#[cfg(test)]
mod sells_tests;

#[cfg(test)]
mod associated_program_tests;

#[cfg(test)]
mod active_program_tests;

#[cfg(test)]
mod composition_structure_tests;

#[cfg(test)]
mod discrete_derivative_tests;

#[cfg(test)]
mod legal_case_tests;
