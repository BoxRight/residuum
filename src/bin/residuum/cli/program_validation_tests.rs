//! Source-defined finite workloads; verification and measurements only.
use residuum::programs::program_rule_origin;
use residuum::typed::*;
use residuum::{
    DerivationClock, IncrementalProgramClosure, close_program, close_program_reference,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("validation-{}", self.0),
        }
    }
}
fn load(file: &str) -> Module {
    residuum::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments")
            .join(file),
    )
    .unwrap()
}
fn experiment<'a>(module: &'a Module, name: &str) -> &'a Experiment {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}
fn observations(c: &Closure) -> (BTreeSet<String>, BTreeMap<String, usize>) {
    let known = c.known.iter().map(|e| format!("{e:?}")).collect();
    let mut proofs = BTreeMap::new();
    for d in &c.derivations {
        *proofs
            .entry(format!("{:?} {:?}", d.event, d.record))
            .or_default() += 1;
    }
    (known, proofs)
}
fn equivalent(actual: &Closure, expected: &Closure, module: &Module) {
    assert_eq!(observations(actual), observations(expected));
    for c in [actual, expected] {
        assert_eq!(c.known.len(), c.known.iter().collect::<BTreeSet<_>>().len());
        assert!(observations(c).1.values().all(|n| *n == 1));
        assert_eq!(
            c.derivations.len(),
            c.derivations
                .iter()
                .map(|d| &d.derived_at.name)
                .collect::<BTreeSet<_>>()
                .len()
        );
        for d in &c.derivations {
            assert!(d.record.antecedents.iter().all(|a| c.known.contains(a)));
            if let Some(origin) = program_rule_origin(module, &d.record.rule) {
                assert!(!origin.program.is_empty());
                assert!(!origin.defining_module.is_empty());
                assert!(origin.rule_origin.file.as_ref().unwrap().is_file());
            }
        }
    }
}
fn validate_schedules(module: &Module, seeds: &[Event], schedules: Vec<Vec<Vec<Event>>>) {
    let expected = close_program_reference(seeds.to_vec(), module, &mut Clock::default()).unwrap();
    equivalent(
        &close_program(seeds.to_vec(), module, &mut Clock::default()).unwrap(),
        &expected,
        module,
    );
    for batches in schedules {
        let mut clock = Clock::default();
        let mut state = IncrementalProgramClosure::new(Vec::new(), module, &mut clock).unwrap();
        let mut seen = Vec::new();
        for batch in batches {
            let old = state.closure().derivations.clone();
            let delta = state.update(batch.clone(), &mut clock);
            seen.extend(batch);
            let reference =
                close_program_reference(seen.clone(), module, &mut Clock::default()).unwrap();
            equivalent(state.closure(), &reference, module);
            assert_eq!(
                &state.closure().derivations[..old.len()],
                old.as_slice(),
                "existing stamps/proofs changed"
            );
            assert!(
                delta
                    .derivations
                    .iter()
                    .all(|d| state.closure().derivations.contains(d))
            );
        }
        equivalent(state.closure(), &expected, module);
        let stamped = clock.0;
        let no_work = state.update(seeds.to_vec(), &mut clock);
        assert!(
            no_work.known.is_empty()
                && no_work.derivations.is_empty()
                && no_work.activations.is_empty()
        );
        assert_eq!(no_work.stats.rules_examined, 0);
        assert_eq!(clock.0, stamped);
    }
}

#[test]
fn finite_cycles_concurrent_activations_and_late_proofs_preserve_equivalence() {
    let module = load("program_incremental_cycles.res");
    let e = experiment(&module, "concurrent");
    let seeds = &e.input;
    let partition = |requests_first: bool| {
        let (requests, other): (Vec<_>, Vec<_>) =
            seeds.iter().cloned().partition(|e| e.verb == "request");
        if requests_first {
            vec![requests, other]
        } else {
            vec![other, requests]
        }
    };
    validate_schedules(
        &module,
        seeds,
        vec![
            vec![seeds.clone()],
            seeds.iter().cloned().map(|e| vec![e]).collect(),
            seeds.iter().rev().cloned().map(|e| vec![e]).collect(),
            partition(true),
            partition(false),
        ],
    );
    let c = close_program(seeds.clone(), &module, &mut Clock::default()).unwrap();
    assert!(c.known.contains(&e.goal));
    for person in ["alice", "bob"] {
        let echo = c
            .known
            .iter()
            .find(|e| e.verb == "echo" && e.args == [Term::Const(person.into())])
            .unwrap();
        assert_eq!(
            c.derivations.iter().filter(|d| &d.event == echo).count(),
            2,
            "retain cycle and program justifications"
        );
    }
    println!(
        "VALIDATED cycles schedules=5 events={} proofs={}",
        c.known.len(),
        c.derivations.len()
    );
}

fn recursive_schedule_check(name: &str, source: &str) {
    let module = load("program_incremental_scaling.res");
    let e = experiment(&module, name);
    let (requests, evidence): (Vec<_>, Vec<_>) =
        e.input.iter().cloned().partition(|e| e.verb == source);
    validate_schedules(
        &module,
        &e.input,
        vec![
            vec![requests.clone(), evidence.clone()],
            vec![evidence.clone(), requests.clone()],
            vec![
                evidence,
                requests[..requests.len() / 2].to_vec(),
                requests[requests.len() / 2..].to_vec(),
            ],
        ],
    );
    let c = close_program(e.input.clone(), &module, &mut Clock::default()).unwrap();
    assert!(c.known.contains(&e.goal));
    for request in requests {
        let id = request.args.last().unwrap();
        let answer = c
            .known
            .iter()
            .filter(|e| e.verb == if source == "add" { "sum" } else { "product" })
            .find(|e| e.args == [id.clone()])
            .unwrap();
        let operand = |term: &Term| match term {
            Term::Const(name) => ["nZero", "nOne", "nTwo", "nThree", "nFour"]
                .iter()
                .position(|n| n == name)
                .unwrap(),
            _ => panic!("unexpected fixture operand"),
        };
        let a = operand(&request.args[0]);
        let b = operand(&request.args[1]);
        let expected = if source == "add" { a + b } else { a * b };
        fn depth(t: &TimeExpr) -> usize {
            match t {
                TimeExpr::At(n) if n == "zeroTime" => 0,
                TimeExpr::After(t) => 1 + depth(t),
                _ => panic!("non-ground result"),
            }
        }
        assert_eq!(depth(&answer.proposition.time), expected);
    }
    println!(
        "VALIDATED {name} schedules=3 events={} proofs={}",
        c.known.len(),
        c.derivations.len()
    );
}
#[test]
fn recursive_addition_concurrent_requests_preserve_all_twenty_five_answers_and_proofs() {
    recursive_schedule_check("jobs25", "add");
}
#[test]
fn recursive_multiplication_and_addition_continuations_preserve_independent_results() {
    recursive_schedule_check("multiplications", "multiply");
}

#[test]
fn distinct_rule_uses_preserve_multiple_proofs_without_extra_events() {
    let module = load("program_proof_scaling.res");
    for n in [1, 4, 16] {
        let e = experiment(&module, &format!("proofs{n}"));
        validate_schedules(&module, &e.input, vec![vec![e.input.clone()]]);
        let c = close_program(e.input.clone(), &module, &mut Clock::default()).unwrap();
        assert_eq!(c.known.len(), 2);
        assert_eq!(c.derivations.len(), n);
        assert!(c.derivations.iter().all(|d| d.event == e.goal));
        assert_eq!(
            c.derivations
                .iter()
                .map(|d| &d.record.rule)
                .collect::<BTreeSet<_>>()
                .len(),
            n
        );
    }
}
