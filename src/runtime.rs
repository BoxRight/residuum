use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::typed::{
    Effect, Event, FieldAccess, PropositionKind, StateOperation, StateTime, StateTransitionType,
    Term, Verb,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldName(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Set(BTreeSet<ObjectId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    fields: BTreeMap<(ObjectId, FieldName), Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshot {
    pub time: StateTime,
    pub store: Store,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl Store {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    pub fn with_set(mut self, object: ObjectId, field: FieldName, values: Vec<ObjectId>) -> Self {
        self.fields
            .insert((object, field), Value::Set(values.into_iter().collect()));
        self
    }

    pub fn set(&self, object: &ObjectId, field: &FieldName) -> Option<&BTreeSet<ObjectId>> {
        match self.fields.get(&(object.clone(), field.clone())) {
            Some(Value::Set(values)) => Some(values),
            None => None,
        }
    }

    fn set_mut(
        &mut self,
        object: &ObjectId,
        field: &FieldName,
    ) -> Result<&mut BTreeSet<ObjectId>, Error> {
        match self.fields.get_mut(&(object.clone(), field.clone())) {
            Some(Value::Set(values)) => Ok(values),
            None => Err(Error::new(format!(
                "missing set field `{}.{}`",
                object.0, field.0
            ))),
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

pub fn execute_effect(
    event: &Event,
    verb: &Verb,
    input: &StateSnapshot,
) -> Result<StateSnapshot, Error> {
    if event.proposition.kind != PropositionKind::Effect {
        return Err(Error::new("event is not an Effect"));
    }
    if verb.name != event.verb {
        return Err(Error::new(format!(
            "event `{}` does not match verb `{}`",
            event.verb, verb.name
        )));
    }

    let transition = event
        .transition
        .as_ref()
        .ok_or_else(|| Error::new("effect event has no state transition"))?;
    require_input_time(transition, &input.time)?;
    let StateTime::At(output_time) = &transition.output else {
        return Err(Error::new("effect output StateTime is unresolved"));
    };

    let Effect::StateTransform { operations, .. } = verb
        .effect
        .as_ref()
        .ok_or_else(|| Error::new("effect verb has no StateTransform"))?;

    let mut output = StateSnapshot {
        time: StateTime::At(output_time.clone()),
        store: input.store.clone(),
    };

    for operation in operations {
        execute_operation(operation, verb, event, &mut output.store)?;
    }

    Ok(output)
}

fn require_input_time(
    transition: &StateTransitionType,
    input_time: &StateTime,
) -> Result<(), Error> {
    if &transition.input == input_time {
        Ok(())
    } else {
        Err(Error::new(
            "input StateTime does not match effect transition",
        ))
    }
}

fn execute_operation(
    operation: &StateOperation,
    verb: &Verb,
    event: &Event,
    store: &mut Store,
) -> Result<(), Error> {
    match operation {
        StateOperation::AddToSet { target, value } => {
            let target = resolve_field_access(target, verb, event)?;
            let value = resolve_term(value, verb, event)?;
            store.set_mut(&target.0, &target.1)?.insert(value);
        }
        StateOperation::RemoveFromSet { target, value } => {
            let target = resolve_field_access(target, verb, event)?;
            let value = resolve_term(value, verb, event)?;
            store.set_mut(&target.0, &target.1)?.remove(&value);
        }
    }

    Ok(())
}

fn resolve_field_access(
    access: &FieldAccess,
    verb: &Verb,
    event: &Event,
) -> Result<(ObjectId, FieldName), Error> {
    if access.path.len() != 1 {
        return Err(Error::new("runtime nested field paths are not implemented"));
    }

    Ok((
        resolve_term(&access.base, verb, event)?,
        FieldName(access.path[0].clone()),
    ))
}

fn resolve_term(term: &Term, verb: &Verb, event: &Event) -> Result<ObjectId, Error> {
    match term {
        Term::Var(name) => {
            let index = verb
                .args
                .iter()
                .position(|arg| arg.name == *name)
                .ok_or_else(|| Error::new(format!("unknown effect argument `{name}`")))?;
            let Some(Term::Var(value)) = event.args.get(index) else {
                return Err(Error::new(format!("missing event argument `{name}`")));
            };

            Ok(ObjectId(value.clone()))
        }
        Term::Const(_) | Term::None | Term::RecordLiteral { .. } => {
            Err(Error::new("runtime only resolves object identifiers"))
        }
    }
}
