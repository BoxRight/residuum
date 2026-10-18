//! The existing one-shot default workflow, with explicit phase boundaries.
use super::*;
use crate::defaults::{DefaultExtension, evaluate_defaults};
use crate::derive::{ForwardSession, ForwardStep};
use crate::typed::ExperimentInputKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultPhase {
    Mandatory,
    Selection,
    Selected,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultStep {
    pub phase: DefaultPhase,
    pub forward: Option<ForwardStep>,
    pub selection: Option<DefaultExtension>,
    pub complete: bool,
}

#[derive(Debug, Clone)]
pub struct DefaultSession {
    original: Module,
    mandatory: Module,
    experiment: Experiment,
    forward: ForwardSession,
    phase: DefaultPhase,
    context: Option<Closure>,
    extension: Option<DefaultExtension>,
    result: Option<DefaultExperimentResult>,
}
impl DefaultSession {
    pub fn new(module: &Module, experiment: &Experiment) -> Result<Self, Error> {
        if experiment.input_kind != ExperimentInputKind::Seeds {
            return Err(Error::new(
                "forward experiments require seed inputs, not premises",
            ));
        }
        let mandatory = super::default_selection::mandatory_module(module)?;
        require_program_acyclic(&mandatory)?;
        let forward = ForwardSession::for_module(experiment.input.clone(), &mandatory)?;
        Ok(Self {
            original: module.clone(),
            mandatory,
            experiment: experiment.clone(),
            forward,
            phase: DefaultPhase::Mandatory,
            context: None,
            extension: None,
            result: None,
        })
    }
    pub fn result(&self) -> Option<&DefaultExperimentResult> {
        self.result.as_ref()
    }
    pub fn step(&mut self, clock: &mut dyn DerivationClock) -> Result<DefaultStep, Error> {
        let phase = self.phase;
        let mut forward = None;
        let mut selection = None;
        match phase {
            DefaultPhase::Mandatory => {
                let step = self.forward.step(clock);
                if step.complete {
                    self.context = Some(self.forward.closure().clone());
                    self.phase = DefaultPhase::Selection;
                }
                forward = Some(step);
            }
            DefaultPhase::Selection => {
                let extension =
                    evaluate_defaults(&self.original, &self.context.as_ref().unwrap().known)?;
                let mut active = self.mandatory.clone();
                active
                    .declarations
                    .extend(extension.rules().iter().cloned().map(Declaration::Rule));
                require_program_acyclic(&active)?;
                self.forward = ForwardSession::for_module(self.experiment.input.clone(), &active)?;
                self.phase = DefaultPhase::Selected;
                self.extension = Some(extension.clone());
                selection = Some(extension);
            }
            DefaultPhase::Selected => {
                let step = self.forward.step(clock);
                if step.complete {
                    let closure = self.forward.closure().clone();
                    self.result = Some(DefaultExperimentResult {
                        context: self.context.as_ref().unwrap().clone(),
                        extension: self.extension.as_ref().unwrap().clone(),
                        observation: ExperimentResult {
                            experiment: self.experiment.clone(),
                            horn_goal: closure.known.contains(&self.experiment.goal),
                            closure,
                            residual: None,
                            active_residual: None,
                            four: None,
                        },
                    });
                    self.phase = DefaultPhase::Complete;
                }
                forward = Some(step);
            }
            DefaultPhase::Complete => {}
        }
        Ok(DefaultStep {
            phase,
            forward,
            selection,
            complete: self.phase == DefaultPhase::Complete,
        })
    }
    pub fn trace(
        &mut self,
        max_steps: usize,
        clock: &mut dyn DerivationClock,
    ) -> Result<Vec<DefaultStep>, Error> {
        let mut steps = Vec::new();
        for _ in 0..max_steps {
            let step = self.step(clock)?;
            let done = step.complete;
            steps.push(step);
            if done {
                break;
            }
        }
        Ok(steps)
    }
}
