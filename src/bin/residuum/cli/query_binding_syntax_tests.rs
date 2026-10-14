use super::*;
use std::path::Path;

#[test]
fn query_command_loads_scoped_compliance_bindings_from_source() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/experiments/legal_geometry/explicit_query_bindings.res");
    let output = execute_file(Command::Query, &path).unwrap();
    assert_eq!(output.matches("experiment ").count(), 2);
    assert!(
        output.contains(
            "search scope: rules {obligationCompliance, rightCompliance, filedObligation}"
        )
    );
    assert!(output.contains("search scope: program ComplianceContext"));
    assert!(output.contains("rule obligationCompliance.conduct = gConduct"));
    assert!(output.contains("program ComplianceContext.actEffective = gDate"));
    assert_eq!(output.matches("Known after close: {}").count(), 2);
    assert_eq!(output.matches("  requirement: ").count(), 2);
    assert!(
        output.contains(
            "Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau"
        )
    );
    assert!(!output.contains("active requirement:"));
}

#[test]
fn active_query_command_reports_guards_unit_and_original_producers() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/experiments/active_query_bindings.res");
    let output = execute_file(Command::Query, &path).unwrap();
    assert_eq!(output.matches("experiment ").count(), 3);
    assert_eq!(output.matches("  Active closure: ").count(), 3);
    assert_eq!(output.matches("  certified explanations: 2").count(), 3);
    assert!(output.contains("active requirement: starts(item, private) @ tau"));
    assert!(output.contains("active requirement: I"));
    assert!(output.contains("guard starts(item, private) @ tau | producer P::0::finish"));
    assert!(output.contains("guard starts(item, private) @ tau | producer P::1::finish"));
    let other = output.split("experiment otherActivation\n").nth(1).unwrap();
    assert!(
        other.contains("Known after close: {starts(item, public) @ tau, done(item) @ after(tau)}")
    );
    assert!(!other.contains("Known after close: {starts(item, private)"));
}

#[test]
fn forward_commands_ignore_query_constraints_and_legacy_query_errors_stay_explicit() {
    let source = include_str!("../../../../examples/experiments/active_query_bindings.res");
    let m = residuum::parse_formal_to_typed(source).unwrap();
    let mut legacy = m.clone();
    for d in &mut legacy.declarations {
        if let Declaration::Experiment(e) = d {
            e.query_options = Default::default();
        }
    }
    assert_eq!(
        execute_module(Command::Run, &m).unwrap(),
        execute_module(Command::Run, &legacy).unwrap()
    );
    assert_eq!(
        execute_module(Command::Explain, &m).unwrap(),
        execute_module(Command::Explain, &legacy).unwrap()
    );
    assert!(
        execute_module(Command::Query, &legacy)
            .unwrap_err()
            .to_string()
            .contains("Program currently supports forward execution")
    );
    let malformed = source.replace("using program P", "using program Missing");
    assert!(
        execute(Command::Query, &malformed)
            .unwrap_err()
            .to_string()
            .contains("unknown query program")
    );
}
