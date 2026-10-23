//! `execute`: the forward closure of `run`, then its derived effects applied to
//! the experiment's declared initial state. The closure never reads the state.
use super::*;
use residuum::experiments::ExperimentResult;
use residuum::runtime::{
    FieldName, ObjectId, OperationKind, ResolvedOperation, StateSnapshot, Store, Value,
    effect_operations,
};
use residuum::typed::{
    Effect, InitialValue, Polarity, PropositionKind, StateOperation, StateTime, Verb,
};
use std::collections::BTreeSet;

pub(super) fn render_execution(
    module: &residuum::typed::Module,
    result: &ExperimentResult,
) -> String {
    let mut output = String::new();
    let verbs: BTreeMap<&str, &Verb> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) => Some((v.name.as_str(), v)),
            _ => None,
        })
        .collect();
    let effects: Vec<&Event> = result
        .closure
        .known
        .iter()
        .filter(|e| e.proposition.kind == PropositionKind::Effect)
        .collect();
    let (positive, negative): (Vec<&Event>, Vec<&Event>) = effects
        .into_iter()
        .partition(|e| e.polarity == Polarity::Positive);
    writeln!(output, "  execution:").unwrap();
    for event in &negative {
        writeln!(
            output,
            "    not executed: {} (evidential negation)",
            render_event(event)
        )
        .unwrap();
    }
    let Some(state) = &result.experiment.state else {
        if positive.is_empty() {
            writeln!(output, "    no initial state; no effects derived").unwrap();
        } else {
            writeln!(output, "    no initial state declared; not executed:").unwrap();
            for event in &positive {
                writeln!(output, "      {}", render_event(event)).unwrap();
            }
        }
        return output;
    };
    let TimeExpr::At(base) = &state.time else {
        unreachable!("elaboration lowers the state time to a constant");
    };

    let mut store = Store::new();
    let mut fields = BTreeSet::new();
    for field in &state.fields {
        let key = (
            ObjectId(field.object.clone()),
            FieldName(field.field.clone()),
        );
        let value = match &field.value {
            // Set elements are constants, as an effect body's literal constants are.
            InitialValue::Set(values) => {
                Value::Set(values.iter().cloned().map(Value::Const).collect())
            }
            // Field paths traverse a reference only when it is an ObjectId.
            InitialValue::Reference(object) => Value::Object(ObjectId(object.clone())),
        };
        store = store.with_value(key.0.clone(), key.1.clone(), value);
        fields.insert(key);
    }
    let mut snapshot = StateSnapshot {
        time: StateTime::At(state.time.clone()),
        store,
    };
    writeln!(
        output,
        "    state @ {}: {}",
        base,
        render_store(&snapshot.store, &fields)
    )
    .unwrap();

    // Effects are grouped by input time and applied in order of `after` depth
    // over the state's base time. Events at other times are reported.
    let mut groups: BTreeMap<usize, Vec<&Event>> = BTreeMap::new();
    for event in positive {
        match event.transition.as_ref().map(|t| &t.input) {
            Some(StateTime::At(time)) if depth_over(time, base).is_some() => groups
                .entry(depth_over(time, base).unwrap())
                .or_default()
                .push(event),
            _ => writeln!(
                output,
                "    not executed: {} (input time not ordered from {base})",
                render_event(event)
            )
            .unwrap(),
        }
    }

    let mut depth = 0;
    for (input_depth, events) in groups {
        if input_depth < depth {
            writeln!(
                output,
                "    conflict @ {}: {} start before the current state @ {}",
                render_time(&after_n(base, input_depth)),
                events
                    .iter()
                    .map(|e| render_event(e))
                    .collect::<Vec<_>>()
                    .join(", "),
                render_time(&after_n(base, depth))
            )
            .unwrap();
            writeln!(output, "    execution stopped").unwrap();
            return output;
        }
        // No effect between two times leaves the store unchanged.
        snapshot.time = StateTime::At(after_n(base, input_depth));
        let mut resolved = Vec::new();
        for event in &events {
            let runtime_event = runtime_event(event, verbs[event.verb.as_str()]);
            match effect_operations(&runtime_event, verbs[event.verb.as_str()], &snapshot) {
                Ok((time, operations)) => resolved.push((*event, runtime_event, time, operations)),
                Err(error) => {
                    writeln!(output, "    error: {}: {error}", render_event(event)).unwrap();
                    writeln!(output, "    execution stopped").unwrap();
                    return output;
                }
            }
        }
        let outputs: BTreeSet<&StateTime> = resolved.iter().map(|(_, _, time, _)| time).collect();
        let conflicts = conflicts(&resolved);
        if outputs.len() > 1 || !conflicts.is_empty() {
            for line in conflicts {
                writeln!(
                    output,
                    "    conflict @ {}: {line}",
                    render_time(&after_n(base, input_depth))
                )
                .unwrap();
            }
            if outputs.len() > 1 {
                writeln!(
                    output,
                    "    conflict @ {}: effects with the same input time have different output times",
                    render_time(&after_n(base, input_depth))
                )
                .unwrap();
            }
            writeln!(output, "    execution stopped").unwrap();
            return output;
        }
        let input = snapshot.clone();
        for (event, runtime_event, _, operations) in &resolved {
            for operation in operations {
                fields.insert((operation.object.clone(), operation.field.clone()));
            }
            // Operations within a group commute (no conflict), so each effect
            // reads the group's input time and the store so far.
            let step = StateSnapshot {
                time: input.time.clone(),
                store: snapshot.store.clone(),
            };
            match residuum::execute_effect(runtime_event, verbs[event.verb.as_str()], &step) {
                Ok(next) => snapshot = next,
                Err(error) => {
                    writeln!(output, "    error: {}: {error}", render_event(event)).unwrap();
                    writeln!(output, "    execution stopped").unwrap();
                    return output;
                }
            }
            writeln!(
                output,
                "    apply {} -> {}: {}",
                render_state_time(&input.time),
                render_state_time(&snapshot.time),
                render_event(event)
            )
            .unwrap();
        }
        writeln!(
            output,
            "    state @ {}: {}",
            render_state_time(&snapshot.time),
            render_store(&snapshot.store, &fields)
        )
        .unwrap();
        depth = match &snapshot.time {
            StateTime::At(time) => depth_over(time, base).unwrap_or(input_depth),
            StateTime::Unresolved => input_depth,
        };
    }
    output
}

/// The adapter of the runtime tests, limited to field-access bases: a constant
/// passed to a parameter that an operation dereferences (`subject.assets`)
/// names the ObjectId of the same name. Every other argument stays a constant,
/// so a value added through an argument equals the same constant written
/// literally in an effect body or in the initial state.
fn runtime_event(event: &Event, verb: &Verb) -> Event {
    let bases: BTreeSet<&str> = match &verb.effect {
        Some(Effect::StateTransform { operations, .. }) => operations
            .iter()
            .filter_map(|operation| match operation {
                StateOperation::AddToSet { target, .. }
                | StateOperation::RemoveFromSet { target, .. } => match &target.base {
                    Term::Var(name) => Some(name.as_str()),
                    _ => None,
                },
            })
            .collect(),
        None => BTreeSet::new(),
    };
    let mut runtime = event.clone();
    for (arg, declared) in runtime.args.iter_mut().zip(&verb.args) {
        if let Term::Const(name) = &*arg {
            if bases.contains(declared.name.as_str()) {
                *arg = Term::Var(name.clone());
            }
        }
    }
    runtime
}

/// Pairs of effects in one group that add and remove the same value of the same
/// field: their result depends on the order, so they are reported, not applied.
fn conflicts(resolved: &[(&Event, Event, StateTime, Vec<ResolvedOperation>)]) -> Vec<String> {
    let mut lines = Vec::new();
    for (i, (a, _, _, ops_a)) in resolved.iter().enumerate() {
        for (b, _, _, ops_b) in &resolved[i + 1..] {
            for x in ops_a {
                for y in ops_b {
                    if x.kind != y.kind
                        && x.object == y.object
                        && x.field == y.field
                        && x.value == y.value
                    {
                        lines.push(format!(
                            "{} and {} {} {} on {}.{}",
                            render_event(a),
                            render_event(b),
                            if x.kind == OperationKind::Add {
                                "add and remove"
                            } else {
                                "remove and add"
                            },
                            render_value(&x.value),
                            x.object.0,
                            x.field.0
                        ));
                    }
                }
            }
        }
    }
    lines
}

fn depth_over(time: &TimeExpr, base: &str) -> Option<usize> {
    match time {
        TimeExpr::At(name) if name == base => Some(0),
        TimeExpr::After(inner) => depth_over(inner, base).map(|d| d + 1),
        _ => None,
    }
}

fn after_n(base: &str, n: usize) -> TimeExpr {
    (0..n).fold(TimeExpr::At(base.to_string()), |t, _| {
        TimeExpr::After(Box::new(t))
    })
}

fn render_state_time(time: &StateTime) -> String {
    match time {
        StateTime::At(time) => render_time(time),
        StateTime::Unresolved => "?".into(),
    }
}

fn render_store(store: &Store, fields: &BTreeSet<(ObjectId, FieldName)>) -> String {
    fields
        .iter()
        .map(|(object, field)| {
            let value = store
                .value(object, field)
                .map(render_value)
                .unwrap_or_else(|| "missing".into());
            format!("{}.{} = {value}", object.0, field.0)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_value(value: &Value) -> String {
    match value {
        Value::Object(ObjectId(name)) | Value::Const(name) => name.clone(),
        Value::None => "none".into(),
        Value::Set(values) => format!(
            "{{{}}}",
            values
                .iter()
                .map(render_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => format!("{other:?}"),
    }
}
