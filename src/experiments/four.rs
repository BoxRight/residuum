//! Finite reference evaluator: FOUR fusion and the unit filter {T,B}.
//! This is a query layer; it never augments the Horn closure.

use super::{Error, FourEntailment, horn};
use crate::typed::{Antecedent, Event, Experiment, Polarity, Rule, Term};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
struct Value {
    p: bool,
    n: bool,
}
impl Value {
    fn tensor(self, other: Self) -> Self {
        Self {
            p: self.p && other.p,
            n: (!other.p || self.n) && (!self.p || other.n),
        }
    }
    fn le(self, other: Self) -> bool {
        (!self.p || other.p) && (!other.n || self.n)
    }
}

fn key(event: &Event) -> Event {
    let mut key = event.clone();
    key.polarity = Polarity::Positive;
    key
}
fn collect(antecedent: &Antecedent, atoms: &mut BTreeSet<Event>) {
    match antecedent {
        Antecedent::Unit => {}
        Antecedent::Event(event) => {
            atoms.insert(key(event));
        }
        Antecedent::And(a, b) => {
            collect(a, atoms);
            collect(b, atoms);
        }
    }
}
fn value(event: &Event, world: &BTreeMap<Event, Value>) -> Value {
    let value = world[&key(event)];
    if event.polarity == Polarity::Positive {
        value
    } else {
        Value {
            p: value.n,
            n: value.p,
        }
    }
}
fn evaluate(antecedent: &Antecedent, world: &BTreeMap<Event, Value>) -> Value {
    match antecedent {
        Antecedent::Unit => Value { p: true, n: true },
        Antecedent::Event(event) => value(event, world),
        Antecedent::And(a, b) => evaluate(a, world).tensor(evaluate(b, world)),
    }
}
pub(super) fn entailment(experiment: &Experiment, rules: &[Rule]) -> Result<FourEntailment, Error> {
    let (atoms, clauses) = prepare(experiment, rules)?;
    let interpretations = 1 << (2 * atoms.len());
    let mut models = 0;
    let mut entailed = true;
    for mask in 0..interpretations {
        let world = world(&atoms, mask);
        if experiment.input.iter().all(|fact| value(fact, &world).p)
            && clauses.iter().all(|clause| {
                evaluate(&clause.antecedent, &world).le(value(&clause.consequent, &world))
            })
        {
            models += 1;
            entailed &= value(&experiment.goal, &world).p;
        }
    }
    Ok(FourEntailment {
        interpretations,
        models,
        entailed,
    })
}

fn prepare(
    experiment: &Experiment,
    rules: &[Rule],
) -> Result<(Vec<Event>, Vec<crate::typed::HornClause>), Error> {
    if rules.iter().any(|rule| !rule.parameters.is_empty()) {
        return Err(Error::new(
            "fourFusion fixture comparison requires ground rules",
        ));
    }
    let clauses: Vec<_> = rules.iter().map(horn).collect();
    let mut atoms: BTreeSet<Event> = experiment
        .input
        .iter()
        .chain([&experiment.goal])
        .map(key)
        .collect();
    for clause in &clauses {
        collect(&clause.antecedent, &mut atoms);
        atoms.insert(key(&clause.consequent));
    }
    if atoms.len() > 4 {
        return Err(Error::new(
            "fourFusion fixture comparison supports at most four ground atoms",
        ));
    }
    // This finite comparison retains its single-index experimental domain.
    // The scoped forward matcher itself also supports other literal indices.
    if atoms.iter().any(|atom| {
        atom.proposition.time != experiment.goal.proposition.time
            || atom.args.iter().any(|arg| !matches!(arg, Term::Const(_)))
    }) {
        return Err(Error::new(
            "fourFusion comparison requires constant arguments and one shared proposition time",
        ));
    }
    let atoms: Vec<_> = atoms.into_iter().collect();
    Ok((atoms, clauses))
}

fn world(atoms: &[Event], mask: usize) -> BTreeMap<Event, Value> {
    atoms
        .iter()
        .enumerate()
        .map(|(i, atom)| {
            (
                atom.clone(),
                Value {
                    p: mask & (1 << (2 * i)) != 0,
                    n: mask & (1 << (2 * i + 1)) != 0,
                },
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FourStep {
    pub round: usize,
    pub constraint: Option<String>,
    /// Interpretation IDs use two bits per atom in atoms(), in sorted order.
    pub eliminated: Vec<usize>,
    pub surviving: Vec<usize>,
    pub interpretations: usize,
    /// None while any theory constraint is still pending.
    pub entailed: Option<bool>,
    pub complete: bool,
}

#[derive(Debug, Clone)]
pub struct FourSession {
    experiment: Experiment,
    rules: Vec<Rule>,
    atoms: Vec<Event>,
    clauses: Vec<crate::typed::HornClause>,
    remaining: Vec<usize>,
    cursor: usize,
    round: usize,
    result: Option<FourEntailment>,
}
impl FourSession {
    pub fn new(experiment: &Experiment, rules: &[Rule]) -> Result<Self, Error> {
        let (atoms, clauses) = prepare(experiment, rules)?;
        let remaining = (0..1 << (2 * atoms.len())).collect();
        Ok(Self {
            experiment: experiment.clone(),
            rules: rules.to_vec(),
            atoms,
            clauses,
            remaining,
            cursor: 0,
            round: 0,
            result: None,
        })
    }
    pub fn atoms(&self) -> &[Event] {
        &self.atoms
    }
    pub fn result(&self) -> Option<&FourEntailment> {
        self.result.as_ref()
    }
    pub fn step(&mut self) -> FourStep {
        let mut eliminated = Vec::new();
        let mut constraint = None;
        let facts = self.experiment.input.len();
        if self.cursor < facts + self.clauses.len() {
            constraint = Some(if self.cursor < facts {
                format!(
                    "premise {}",
                    super::render_event(&self.experiment.input[self.cursor])
                )
            } else {
                format!("rule {}", self.rules[self.cursor - facts].name)
            });
            let mut next = Vec::new();
            for mask in &self.remaining {
                let world = world(&self.atoms, *mask);
                let satisfies = if self.cursor < facts {
                    value(&self.experiment.input[self.cursor], &world).p
                } else {
                    let clause = &self.clauses[self.cursor - facts];
                    evaluate(&clause.antecedent, &world).le(value(&clause.consequent, &world))
                };
                if satisfies {
                    next.push(*mask);
                } else {
                    eliminated.push(*mask);
                }
            }
            self.remaining = next;
            self.cursor += 1;
        }
        let complete = self.cursor == facts + self.clauses.len();
        if complete {
            let entailed = self
                .remaining
                .iter()
                .all(|mask| value(&self.experiment.goal, &world(&self.atoms, *mask)).p);
            self.result = Some(FourEntailment {
                interpretations: 1 << (2 * self.atoms.len()),
                models: self.remaining.len(),
                entailed,
            });
        }
        self.round += 1;
        FourStep {
            round: self.round,
            constraint,
            eliminated,
            surviving: self.remaining.clone(),
            interpretations: 1 << (2 * self.atoms.len()),
            entailed: self.result.as_ref().map(|r| r.entailed),
            complete,
        }
    }
    pub fn trace(&mut self, max_steps: usize) -> Vec<FourStep> {
        let mut steps = Vec::new();
        for _ in 0..max_steps {
            let step = self.step();
            let done = step.complete;
            steps.push(step);
            if done {
                break;
            }
        }
        steps
    }
}
