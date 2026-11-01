//! `execute` prints the `run` closure and then applies the derived effects to
//! each experiment's declared state. The closure never reads that state.
use super::*;
use residuum::runtime::{
    FieldName, ObjectId, OperationKind, ResolvedOperation, StateSnapshot, Store, Value,
    effect_operations,
};
use residuum::typed::{Declaration, StateTime, Term, TimeExpr};

const SELLS: &str = include_str!("../../../../examples/execution/sells_execute.res");
const LITERAL: &str = include_str!("../../../../examples/execution/literal_execute.res");

fn section<'a>(output: &'a str, name: &str) -> &'a str {
    let start = output.find(&format!("experiment {name}\n")).unwrap();
    let rest = &output[start + 1..];
    &output[start..start + 1 + rest.find("\nexperiment ").map_or(rest.len(), |end| end + 1)]
}

fn execution(output: &str, name: &str) -> String {
    section(output, name)
        .split_once("  execution:\n")
        .unwrap()
        .1
        .to_string()
}

#[test]
fn execute_prints_the_run_output_and_then_each_execution() {
    let run = execute(Command::Run, SELLS).unwrap();
    let executed = execute(Command::Execute, SELLS).unwrap();
    let without_execution: String = executed
        .split_inclusive('\n')
        .scan(false, |inside, line| {
            if line == "  execution:\n" {
                *inside = true;
            } else if line.starts_with("experiment ") {
                *inside = false;
            }
            Some(if *inside { "" } else { line })
        })
        .collect();
    assert_eq!(without_execution, run);
    assert_eq!(executed.matches("  execution:\n").count(), 4);
}

#[test]
fn execute_applies_effects_in_time_order_from_the_declared_state() {
    let output = execute(Command::Execute, SELLS).unwrap();
    assert_eq!(
        execution(&output, "sale"),
        "    state @ tau: juan.assets = {car}, pedro.assets = {}\n\
         \x20   apply tau -> after(tau): give(juan, car, pedro) @ after(tau)\n\
         \x20   state @ after(tau): juan.assets = {}, pedro.assets = {car}\n"
    );
    assert_eq!(
        execution(&output, "resale"),
        "    state @ tau: charlie.assets = {}, juan.assets = {car}, pedro.assets = {}\n\
         \x20   apply tau -> after(tau): give(juan, car, pedro) @ after(tau)\n\
         \x20   state @ after(tau): charlie.assets = {}, juan.assets = {}, pedro.assets = {car}\n\
         \x20   apply after(tau) -> after(after(tau)): give(pedro, car, charlie) @ after(after(tau))\n\
         \x20   state @ after(after(tau)): charlie.assets = {car}, juan.assets = {}, pedro.assets = {}\n"
    );
    assert_eq!(
        execution(&output, "stateless"),
        "    no initial state declared; not executed:\n\
         \x20     give(juan, car, pedro) @ after(tau)\n"
    );
}

#[test]
fn execute_reports_order_dependent_effects_without_applying_them() {
    let output = execute(Command::Execute, SELLS).unwrap();
    assert_eq!(
        execution(&output, "crossed"),
        "    state @ tau: juan.assets = {car}, pedro.assets = {car}\n\
         \x20   conflict @ tau: give(juan, car, pedro) @ after(tau) and give(pedro, car, juan) @ after(tau) remove and add car on juan.assets\n\
         \x20   conflict @ tau: give(juan, car, pedro) @ after(tau) and give(pedro, car, juan) @ after(tau) add and remove car on pedro.assets\n\
         \x20   execution stopped\n"
    );
}

#[test]
fn execute_matches_literal_body_constants_with_declared_elements() {
    let output = execute(Command::Execute, LITERAL).unwrap();
    assert_eq!(
        execution(&output, "cleared"),
        "    state @ tau: bucket.markers = {Fixed}\n\
         \x20   apply tau -> after(tau): removeMarker(bucket) @ after(tau)\n\
         \x20   state @ after(tau): bucket.markers = {}\n"
    );
}

#[test]
fn execute_reports_runtime_errors_and_stops() {
    // pedro.assets is not declared, so the effect's target set is missing.
    let source = SELLS.replace(
        "state @ tau { juan.assets = {car}, pedro.assets = {}, }",
        "state @ tau { juan.assets = {car} }",
    );
    let output = execute(Command::Execute, &source).unwrap();
    assert_eq!(
        execution(&output, "sale"),
        "    state @ tau: juan.assets = {car}\n\
         \x20   error: give(juan, car, pedro) @ after(tau): missing set field `pedro.assets`\n\
         \x20   execution stopped\n"
    );
}

#[test]
fn execute_rejects_steps_and_invalid_initial_states() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/execution/sells_execute.res");
    let steps = StepOptions {
        max_steps: 4,
        trace: false,
    };
    assert_eq!(
        execute_file_steps(Command::Execute, &path, steps)
            .unwrap_err()
            .to_string(),
        "execute does not support --steps or --trace"
    );
    let state = "state @ tau { juan.assets = {car}, pedro.assets = {}, }";
    for (replacement, message) in [
        (
            "state @ car { juan.assets = {car} }",
            "state time `car` must be a PropositionTime constant",
        ),
        (
            "state @ tau { tau.assets = {} }",
            "state object `tau` must be an entity constant",
        ),
        (
            "state @ tau { juan.owner = {} }",
            "unknown field `Person.owner`",
        ),
        (
            "state @ tau { juan.assets = {bike} }",
            "unknown state constant `bike`",
        ),
        (
            "state @ tau { juan.assets = {pedro} }",
            "state constant `pedro` is not assignable to Entity(\"Thing\")",
        ),
        (
            "state @ tau { juan.assets = {}, juan.assets = {car} }",
            "duplicate state field `juan.assets`",
        ),
    ] {
        let error = execute(Command::Execute, &SELLS.replace(state, replacement)).unwrap_err();
        assert_eq!(error.to_string(), message, "{replacement}");
    }
}

#[test]
fn run_and_query_ignore_the_initial_state() {
    let stateless = SELLS.replace(
        "state @ tau { juan.assets = {car}, pedro.assets = {}, }",
        "",
    );
    assert_eq!(
        execute(Command::Run, SELLS).unwrap(),
        execute(Command::Run, &stateless).unwrap()
    );
    assert_eq!(
        execute(Command::Query, SELLS).unwrap(),
        execute(Command::Query, &stateless).unwrap()
    );
}

#[test]
fn effect_operations_resolve_without_applying() {
    let module = residuum::parse_formal_to_typed(SELLS).unwrap();
    let give = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == "give" => Some(v),
            _ => None,
        })
        .unwrap();
    let result = residuum::experiments::run_forward_experiments(&module).unwrap();
    let mut event = result[0]
        .closure
        .known
        .iter()
        .find(|e| e.verb == "give")
        .unwrap()
        .clone();
    event.args[0] = Term::Var("juan".into());
    event.args[2] = Term::Var("pedro".into());
    let input = StateSnapshot {
        time: StateTime::At(TimeExpr::At("tau".into())),
        store: Store::new(),
    };
    let (time, operations) = effect_operations(&event, give, &input).unwrap();
    assert_eq!(
        time,
        StateTime::At(TimeExpr::After(Box::new(TimeExpr::At("tau".into()))))
    );
    let operation = |kind, object: &str| ResolvedOperation {
        kind,
        object: ObjectId(object.into()),
        field: FieldName("assets".into()),
        value: Value::Const("car".into()),
    };
    assert_eq!(
        operations,
        vec![
            operation(OperationKind::Remove, "juan"),
            operation(OperationKind::Add, "pedro"),
        ]
    );
    // Resolution does not need the sets; application does.
    assert_eq!(
        residuum::execute_effect(&event, give, &input)
            .unwrap_err()
            .to_string(),
        "missing set field `juan.assets`"
    );
}

fn patrimony_execution() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/execution/patrimony_execute.res");
    execute_file(Command::Execute, &path).unwrap()
}

#[test]
fn execute_follows_declared_references_to_reach_sets() {
    assert_eq!(
        execution(&patrimony_execution(), "sale"),
        "    state @ tau: juan.patrimony = juanPatrimony, juanPatrimony.assets = {car}, \
         pedro.patrimony = pedroPatrimony, pedroPatrimony.assets = {}\n\
         \x20   apply tau -> after(tau): give(juan, car, pedro) @ after(tau)\n\
         \x20   state @ after(tau): juan.patrimony = juanPatrimony, juanPatrimony.assets = {}, \
         pedro.patrimony = pedroPatrimony, pedroPatrimony.assets = {car}\n"
    );
}

/// Writes an edited copy into its own directory outside the repository, next to
/// a copy of the imported legal module, and executes it.
fn execute_edited(source: &str, name: &str) -> Result<String, Error> {
    let legal =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/legal/give_formal.res");
    let dir = std::env::temp_dir().join(format!("residuum-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(legal, dir.join("give_formal.res")).unwrap();
    let path = dir.join("case.res");
    std::fs::write(
        &path,
        source.replace("../legal/give_formal.res", "./give_formal.res"),
    )
    .unwrap();
    let result = execute_file(Command::Execute, &path);
    std::fs::remove_dir_all(&dir).unwrap();
    result
}

#[test]
fn execute_validates_reference_fields() {
    let source = include_str!("../../../../examples/execution/patrimony_execute.res");
    let declared = "juan.patrimony = juanPatrimony,";
    for (index, (replacement, message)) in [
        (
            "juan.patrimony = {juanPatrimony},",
            "state field `juan.patrimony` is not a set",
        ),
        (
            "juan.patrimony = pedro,",
            "state constant `pedro` is not assignable to Entity(\"Patrimony\")",
        ),
        (
            "juan.patrimony = nobody,",
            "unknown state constant `nobody`",
        ),
        (
            "juanPatrimony.rights = juanPatrimony,",
            "state field `juanPatrimony.rights` is not an entity reference",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let error =
            execute_edited(&source.replace(declared, replacement), &index.to_string()).unwrap_err();
        assert!(
            error.to_string().ends_with(message),
            "{replacement}: {error}"
        );
    }
    // Without the reference the runtime cannot reach the set, and says so.
    let output = execute_edited(&source.replace(declared, ""), "missing").unwrap();
    assert!(
        execution(&output, "sale")
            .contains("error: give(juan, car, pedro) @ after(tau): missing field `juan.patrimony`")
    );
}
