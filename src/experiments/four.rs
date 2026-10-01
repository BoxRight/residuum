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
    let interpretations = 1 << (2 * atoms.len());
    let mut models = 0;
    let mut entailed = true;
    for mask in 0..interpretations {
        let world: BTreeMap<_, _> = atoms
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
            .collect();
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
