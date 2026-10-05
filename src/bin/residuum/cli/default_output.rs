//! Presentation of selection records returned by the existing default calculus.
use std::fmt::Write;

use residuum::experiments::{DefaultExperimentResult, render_event, render_results};
use residuum::typed::{Declaration, Module};

use super::{Command, render_explanations, render_substitution};

pub(super) fn render(
    module: &Module,
    results: &[DefaultExperimentResult],
    command: Command,
) -> String {
    let mut output = String::new();
    for result in results {
        let extension = &result.extension;
        writeln!(output, "experiment {}", result.observation.experiment.name).unwrap();
        writeln!(
            output,
            "  default context: mandatory Horn closure ({} events)",
            result.context.known.len()
        )
        .unwrap();
        writeln!(output, "  blocking scope: {:?}", extension.blocking_scope).unwrap();
        writeln!(output, "  consistency: {:?}", extension.consistency).unwrap();
        for declaration in &module.declarations {
            let Declaration::Default(default) = declaration else {
                continue;
            };
            let target = default.body.target_name();
            let applicable = extension
                .applicable
                .iter()
                .find(|d| d.default.name == default.name);
            let blocks: Vec<_> = extension
                .blocked
                .iter()
                .filter(|d| d.default.name == default.name)
                .collect();
            let status = if !blocks.is_empty() {
                "blocked"
            } else if extension.defeated.iter().any(|d| d.name == default.name) {
                "defeated"
            } else if extension.selected.iter().any(|d| d.name == default.name) {
                "selected"
            } else {
                "not applicable"
            };
            writeln!(output, "  default {} -> {}: {status}", default.name, target).unwrap();
            if let Some(origin) = module.origin(&default.name).filter(|o| o.file.is_some()) {
                writeln!(
                    output,
                    "    default origin: {} ({})",
                    origin.module,
                    origin.file.as_ref().unwrap().display()
                )
                .unwrap();
                if let Some(origin) = module.origin(target) {
                    writeln!(output, "    lambda origin: {}", origin.module).unwrap();
                }
            }
            match applicable {
                None => {
                    writeln!(output, "    applicability: no matching evidence").unwrap();
                }
                Some(matches) => {
                    for matched in &matches.matches {
                        if default.condition.is_none() {
                            writeln!(output, "    applicability: unconditional").unwrap();
                        } else {
                            writeln!(
                                output,
                                "    applicability evidence: {}",
                                matched
                                    .antecedents
                                    .iter()
                                    .map(render_event)
                                    .collect::<Vec<_>>()
                                    .join(" & ")
                            )
                            .unwrap();
                            writeln!(
                                output,
                                "      substitution: {}",
                                render_substitution(&matched.substitution)
                            )
                            .unwrap();
                        }
                    }
                }
            }
            for block in &blocks {
                writeln!(
                    output,
                    "    blocking evidence: {}",
                    block
                        .evidence
                        .iter()
                        .map(render_event)
                        .collect::<Vec<_>>()
                        .join(" & ")
                )
                .unwrap();
                writeln!(
                    output,
                    "      substitution: {}",
                    render_substitution(&block.substitution)
                )
                .unwrap();
            }
            if applicable.is_some() && blocks.is_empty() {
                writeln!(output, "    blocking: no matching blocker").unwrap();
            }
            if let Some(to) = default.body.exception_parent() {
                writeln!(output, "    declared priority: exception to {}", to.name).unwrap();
            }
            if status == "defeated" {
                writeln!(
                    output,
                    "    priority: displaced by an applicable, unblocked exception"
                )
                .unwrap();
            } else if status == "selected" {
                writeln!(output, "    selection: applicable, unblocked, not defeated").unwrap();
            }
        }
        writeln!(
            output,
            "  {}: {}",
            if extension.programs().is_empty() {
                "selected default lambdas"
            } else {
                "compiled rules from selected defaults"
            },
            extension
                .rules()
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        if !extension.programs().is_empty() {
            writeln!(
                output,
                "  selected programs: {}",
                extension
                    .programs()
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .unwrap();
        }
        let observation = std::slice::from_ref(&result.observation);
        let text = match command {
            Command::Run => render_results(observation),
            Command::Explain => render_explanations(module, observation),
            Command::Query => unreachable!("default queries remain outside this adapter"),
        };
        // Reuse the existing observation renderer without repeating its header.
        output.push_str(text.split_once('\n').unwrap().1);
    }
    output
}
