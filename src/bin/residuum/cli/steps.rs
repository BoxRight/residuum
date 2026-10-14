//! Bounded presentation adapters over the stepwise evaluator APIs.
use super::*;
use residuum::experiments::{DefaultSession, FourSession};
use residuum::typed::{ExperimentInputKind, ExperimentQueryKind, Rule};
use residuum::{ForwardSession, ResidualSession};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepOptions {
    pub max_steps: usize,
    pub trace: bool,
}
impl StepOptions {
    pub fn parse(args: &[std::ffi::OsString]) -> Result<Option<Self>, Error> {
        let mut max_steps = None;
        let mut trace = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].to_str() {
                Some("--steps") if max_steps.is_none() => {
                    i += 1;
                    max_steps = Some(
                        args.get(i)
                            .and_then(|a| a.to_str())
                            .and_then(|s| s.parse::<usize>().ok())
                            .filter(|n| (1..=4096).contains(n))
                            .ok_or_else(|| Error::new("--steps requires an integer in 1..=4096"))?,
                    );
                }
                Some("--trace") if !trace => trace = true,
                _ => return Err(Error::new(format!("invalid step option\n{USAGE}"))),
            }
            i += 1;
        }
        Ok(if max_steps.is_some() || trace {
            Some(Self {
                max_steps: max_steps.unwrap_or(64),
                trace,
            })
        } else {
            None
        })
    }
}
pub fn execute_file_steps(
    command: Command,
    path: &std::path::Path,
    options: StepOptions,
) -> Result<String, Error> {
    render(command, &residuum::load_formal_module(path)?, options)
}
#[cfg(test)]
fn execute_steps(command: Command, source: &str, options: StepOptions) -> Result<String, Error> {
    render(command, &residuum::parse_formal_to_typed(source)?, options)
}
fn events(events: &[Event]) -> String {
    events
        .iter()
        .map(render_event)
        .collect::<Vec<_>>()
        .join(", ")
}
fn record(snapshots: &mut Vec<String>, snapshot: String, trace: bool) {
    if !trace {
        snapshots.clear();
    }
    snapshots.push(snapshot);
}
fn forward(step: &residuum::ForwardStep, explain: bool) -> String {
    let mut output = format!(
        "  round {}\n    T(X): {{{}}}\n    delta: {{{}}}\n    Known: {{{}}}\n",
        step.round,
        events(&step.consequences),
        events(&step.added),
        events(&step.known)
    );
    if explain {
        for proof in &step.witnesses {
            writeln!(
                output,
                "    rule {} | conclusion {} | substitution {} | antecedents {}",
                proof.record.rule,
                render_event(&proof.event),
                render_substitution(&proof.record.substitution),
                if proof.record.antecedents.is_empty() {
                    "I".into()
                } else {
                    events(&proof.record.antecedents)
                }
            )
            .unwrap();
        }
        for proof in &step.derivations {
            writeln!(
                output,
                "    new justification {} | derivedAt {}",
                proof.record.rule, proof.derived_at.name
            )
            .unwrap();
        }
    }
    output
}
fn render(
    command: Command,
    module: &residuum::typed::Module,
    options: StepOptions,
) -> Result<String, Error> {
    if !(1..=4096).contains(&options.max_steps) {
        return Err(Error::new("step budget must be in 1..=4096"));
    }
    let defaults = module
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Default(_)));
    if command == Command::Query
        && module
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Program(_)))
    {
        return Err(Error::new(
            "Program currently supports forward execution; residual/FOUR experiment integration is not implemented",
        ));
    }
    if command == Command::Query && defaults {
        return Err(Error::new(
            "experimental runner requires a monotonic module without defaults",
        ));
    }
    let rules: Vec<Rule> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let experiments: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) => Some(e),
            _ => None,
        })
        .collect();
    if experiments.is_empty() {
        return Err(Error::new("module contains no experiments"));
    }
    let mut output = String::new();
    for experiment in experiments {
        writeln!(
            output,
            "experiment {}\n  goal: {}",
            experiment.name,
            render_event(&experiment.goal)
        )
        .unwrap();
        let mut clock = Clock::default();
        let mut complete = false;
        let mut snapshots = Vec::new();
        let mut final_output = String::new();
        if command != Command::Query {
            if experiment.input_kind != ExperimentInputKind::Seeds {
                return Err(Error::new(
                    "forward experiments require seed inputs, not premises",
                ));
            }
            if defaults {
                let mut session = DefaultSession::new(module, experiment)?;
                for _ in 0..options.max_steps {
                    let step = session.step(&mut clock)?;
                    let mut snapshot = format!("  phase {:?}\n", step.phase);
                    if let Some(f) = &step.forward {
                        snapshot.push_str(&forward(f, command == Command::Explain));
                    }
                    if let Some(selection) = &step.selection {
                        writeln!(
                            snapshot,
                            "    selected: {}",
                            selection
                                .selected
                                .iter()
                                .map(|d| d.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                        .unwrap();
                    }
                    complete = step.complete;
                    record(&mut snapshots, snapshot, options.trace);
                    if complete {
                        break;
                    }
                }
                if let Some(result) = session.result() {
                    writeln!(
                        final_output,
                        "  goal supported: {}",
                        result.observation.horn_goal
                    )
                    .unwrap();
                }
            } else {
                let mut session = ForwardSession::for_module(experiment.input.clone(), module)?;
                for _ in 0..options.max_steps {
                    let step = session.step(&mut clock);
                    record(
                        &mut snapshots,
                        forward(&step, command == Command::Explain),
                        options.trace,
                    );
                    complete = step.complete;
                    if complete {
                        break;
                    }
                }
                writeln!(
                    final_output,
                    "  goal supported so far: {}",
                    session.closure().known.contains(&experiment.goal)
                )
                .unwrap();
            }
        } else {
            match experiment.query_kind {
                ExperimentQueryKind::Residual => {
                    let mut session = ResidualSession::new(
                        module,
                        &experiment.input,
                        &experiment.goal,
                        residuum::SearchLimits::default(),
                        None,
                    )?;
                    for _ in 0..options.max_steps {
                        let step = session.step()?;
                        record(
                            &mut snapshots,
                            format!(
                                "  round {}\n    search branches: {}\n    pending obligations: {{{}}}\n",
                                step.round,
                                step.branches.len(),
                                events(&step.pending)
                            ),
                            options.trace,
                        );
                        complete = step.complete;
                        if complete {
                            break;
                        }
                    }
                    if let Some(result) = session.result() {
                        writeln!(
                            final_output,
                            "  requirement: {}",
                            result
                                .required
                                .as_ref()
                                .map(residuum::experiments::render_requirement)
                                .unwrap_or_else(|| "no explanation".into())
                        )
                        .unwrap();
                        writeln!(
                            final_output,
                            "  certified branches: {}",
                            result.explanations.len()
                        )
                        .unwrap();
                    }
                }
                ExperimentQueryKind::FourFusion => {
                    let mut session = FourSession::new(experiment, &rules)?;
                    for _ in 0..options.max_steps {
                        let step = session.step();
                        record(
                            &mut snapshots,
                            format!(
                                "  round {}\n    constraint: {}\n    eliminated: {}\n    remaining interpretations: {} / {}\n",
                                step.round,
                                step.constraint.unwrap_or_else(|| "none".into()),
                                step.eliminated.len(),
                                step.surviving.len(),
                                step.interpretations
                            ),
                            options.trace,
                        );
                        complete = step.complete;
                        if complete {
                            break;
                        }
                    }
                    if let Some(result) = session.result() {
                        writeln!(
                            final_output,
                            "  FOUR fusion: {} ({} models / {} interpretations)",
                            result.entailed, result.models, result.interpretations
                        )
                        .unwrap();
                    }
                }
            }
        }
        if options.trace {
            for snapshot in snapshots {
                output.push_str(&snapshot);
            }
        } else if let Some(last) = snapshots.last() {
            output.push_str(last);
        }
        output.push_str(&final_output);
        writeln!(
            output,
            "  status: {}",
            if complete {
                "complete"
            } else {
                "step limit reached; incomplete"
            }
        )
        .unwrap();
    }
    Ok(output)
}
#[derive(Default)]
struct Clock(usize);
impl residuum::DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> residuum::typed::DerivationTime {
        self.0 += 1;
        residuum::typed::DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CHAIN: &str = "module StepCli\nconst tau : PropositionTime\nseeded verb A()\nderived verb B()\nderived verb C()\nrule ab() = A() @ tau <= B() @ tau\nrule bc() = B() @ tau <= C() @ tau\nexperiment e { seeds { A() @ tau } query residual C() @ tau }";
    #[test]
    fn options_preserve_legacy_dispatch_and_reject_invalid_budgets() {
        assert_eq!(StepOptions::parse(&[]).unwrap(), None);
        assert_eq!(
            StepOptions::parse(&["--trace".into()]).unwrap(),
            Some(StepOptions {
                max_steps: 64,
                trace: true
            })
        );
        for args in [
            vec!["--steps".into()],
            vec!["--steps".into(), "0".into()],
            vec!["--steps".into(), "4097".into()],
            vec!["--trace".into(), "--trace".into()],
        ] {
            assert!(StepOptions::parse(&args).is_err());
        }
    }
    #[test]
    fn forward_one_step_and_trace_expose_synchronous_progress_and_completion() {
        let one = execute_steps(
            Command::Run,
            CHAIN,
            StepOptions {
                max_steps: 1,
                trace: false,
            },
        )
        .unwrap();
        assert!(one.contains("T(X): {B() @ tau}"));
        assert!(one.contains("goal supported so far: false"));
        assert!(one.contains("incomplete"));
        let trace = execute_steps(
            Command::Run,
            CHAIN,
            StepOptions {
                max_steps: 10,
                trace: true,
            },
        )
        .unwrap();
        assert!(
            trace.contains("round 1")
                && trace.contains("round 3")
                && trace.contains("status: complete")
        );
        let explain = execute_steps(
            Command::Explain,
            CHAIN,
            StepOptions {
                max_steps: 3,
                trace: true,
            },
        )
        .unwrap();
        assert!(
            explain.contains("rule ab")
                && explain.contains("antecedents A()")
                && explain.contains("derivedAt")
        );
    }
    #[test]
    fn query_prefix_is_not_an_explanation_and_complete_trace_matches_queries() {
        let prefix = execute_steps(
            Command::Query,
            CHAIN,
            StepOptions {
                max_steps: 1,
                trace: true,
            },
        )
        .unwrap();
        assert!(prefix.contains("pending obligations"));
        assert!(!prefix.contains("requirement:"));
        assert!(prefix.contains("incomplete"));
        let full = execute_steps(
            Command::Query,
            CHAIN,
            StepOptions {
                max_steps: 10,
                trace: true,
            },
        )
        .unwrap();
        assert!(full.contains("requirement: I"));
        assert!(full.contains("certified branches: 1"));
        assert!(full.contains("status: complete"));
        let four = include_str!("../../../../examples/experiments/four_counterexample.res");
        let prefix = execute_steps(
            Command::Query,
            four,
            StepOptions {
                max_steps: 1,
                trace: true,
            },
        )
        .unwrap();
        assert!(!prefix.contains("FOUR fusion:"));
        let full = execute_steps(
            Command::Query,
            four,
            StepOptions {
                max_steps: 10,
                trace: true,
            },
        )
        .unwrap();
        assert!(full.contains("FOUR fusion: true (3 models / 16 interpretations)"));
    }
    #[test]
    fn default_trace_selects_once_and_existing_invocations_keep_their_output() {
        let source = include_str!("../../../../examples/legal/case_sale_defaults_formal.res");
        let output = execute_steps(
            Command::Run,
            source,
            StepOptions {
                max_steps: 64,
                trace: true,
            },
        )
        .unwrap();
        assert_eq!(output.matches("phase Selection").count(), 4);
        assert_eq!(output.matches("status: complete").count(), 4);
        assert!(
            super::super::execute(Command::Run, CHAIN)
                .unwrap()
                .contains("Known after close")
        );
        assert!(
            execute_steps(
                Command::Query,
                source,
                StepOptions {
                    max_steps: 64,
                    trace: true
                }
            )
            .is_err()
        );
    }
}
