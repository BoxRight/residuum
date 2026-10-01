use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::typed::{
    Effect, Event, FieldAccess, Polarity, PropositionKind, StateOperation, StateTime,
    StateTransitionType, Term, Verb, VerbKind,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Object(ObjectId),
    Const(String),
    None,
    Record {
        entity: String,
        fields: BTreeMap<FieldName, Value>,
    },
    Set(BTreeSet<Value>),
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
        self.fields.insert(
            (object, field),
            Value::Set(values.into_iter().map(Value::Object).collect()),
        );
        self
    }

    pub fn with_value(mut self, object: ObjectId, field: FieldName, value: Value) -> Self {
        self.fields.insert((object, field), value);
        self
    }

    pub fn value(&self, object: &ObjectId, field: &FieldName) -> Option<&Value> {
        self.fields.get(&(object.clone(), field.clone()))
    }

    pub fn set(&self, object: &ObjectId, field: &FieldName) -> Option<&BTreeSet<Value>> {
        match self.value(object, field) {
            Some(Value::Set(values)) => Some(values),
            _ => None,
        }
    }

    fn set_mut(
        &mut self,
        object: &ObjectId,
        field: &FieldName,
    ) -> Result<&mut BTreeSet<Value>, Error> {
        match self.fields.get_mut(&(object.clone(), field.clone())) {
            Some(Value::Set(values)) => Ok(values),
            Some(_) => Err(Error::new(format!(
                "field `{}.{}` is not a set",
                object.0, field.0
            ))),
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
    if event.polarity == Polarity::EvidentialNot {
        return Err(Error::new(
            "EvidentialNot has no executable positive StateTransform",
        ));
    }
    if event.proposition.kind != PropositionKind::Effect {
        return Err(Error::new("event is not an Effect"));
    }
    if verb.name != event.verb {
        return Err(Error::new(format!(
            "event `{}` does not match verb `{}`",
            event.verb, verb.name
        )));
    }
    if verb.kind != VerbKind::Effect {
        return Err(Error::new("verb is not an effect verb"));
    }
    if event.args.len() != verb.args.len() {
        return Err(Error::new(format!(
            "verb `{}` expects {} runtime args, got {}",
            verb.name,
            verb.args.len(),
            event.args.len()
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
    if output_time != &event.proposition.time {
        return Err(Error::new(
            "effect output StateTime differs from PropositionTime",
        ));
    }

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
    if matches!(transition.input, StateTime::Unresolved)
        || matches!(input_time, StateTime::Unresolved)
    {
        return Err(Error::new("effect input StateTime is unresolved"));
    }
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
            let target = resolve_field_access(target, verb, event, store)?;
            let value = resolve_term(value, verb, event)?;
            store.set_mut(&target.0, &target.1)?.insert(value);
        }
        StateOperation::RemoveFromSet { target, value } => {
            let target = resolve_field_access(target, verb, event, store)?;
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
    store: &Store,
) -> Result<(ObjectId, FieldName), Error> {
    let Some((last, path)) = access.path.split_last() else {
        return Err(Error::new("runtime field path is empty"));
    };
    let Value::Object(mut object) = resolve_term(&access.base, verb, event)? else {
        return Err(Error::new(
            "runtime field path requires an object reference base",
        ));
    };
    for segment in path {
        let field = FieldName(segment.clone());
        match store.value(&object, &field) {
            Some(Value::Object(next)) => object = next.clone(),
            Some(_) => {
                return Err(Error::new(format!(
                    "field `{}.{}` is not an object reference",
                    object.0, field.0
                )));
            }
            None => {
                return Err(Error::new(format!(
                    "missing field `{}.{}`",
                    object.0, field.0
                )));
            }
        }
    }
    Ok((object, FieldName(last.clone())))
}

fn resolve_term(term: &Term, verb: &Verb, event: &Event) -> Result<Value, Error> {
    match term {
        Term::Var(name) => {
            let index = verb
                .args
                .iter()
                .position(|arg| arg.name == *name)
                .ok_or_else(|| Error::new(format!("unknown effect argument `{name}`")))?;
            let value = event
                .args
                .get(index)
                .ok_or_else(|| Error::new(format!("missing event argument `{name}`")))?;
            materialize_term(value, &mut |name| {
                Ok(Value::Object(ObjectId(name.to_string())))
            })
        }
        _ => materialize_term(term, &mut |name| {
            resolve_term(&Term::Var(name.to_string()), verb, event)
        }),
    }
}

fn materialize_term(
    term: &Term,
    variable: &mut impl FnMut(&str) -> Result<Value, Error>,
) -> Result<Value, Error> {
    match term {
        Term::Var(name) => variable(name),
        Term::Const(name) => Ok(Value::Const(name.clone())),
        Term::None => Ok(Value::None),
        Term::RecordLiteral { entity, fields } => {
            let mut values = BTreeMap::new();
            for field in fields {
                let value = materialize_term(&field.value, variable)?;
                if values
                    .insert(FieldName(field.field.clone()), value)
                    .is_some()
                {
                    return Err(Error::new(format!(
                        "duplicate runtime record field `{}`",
                        field.field
                    )));
                }
            }
            Ok(Value::Record {
                entity: entity.clone(),
                fields: values,
            })
        }
    }
}
