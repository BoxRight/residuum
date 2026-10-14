use super::*;
use std::path::Path;
#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("step-{}", self.0),
        }
    }
}
fn module(source: &str) -> Module {
    crate::parse_formal_to_typed(source).unwrap()
}
fn rules(m: &Module) -> Vec<Rule> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect()
}
fn experiment(m: &Module) -> &crate::typed::Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) => Some(e),
            _ => None,
        })
        .unwrap()
}
fn proofs(c: &Closure) -> Vec<String> {
    let mut p: Vec<_> = c
        .derivations
        .iter()
        .map(|d| format!("{:?} {:?}", d.event, d.record))
        .collect();
    p.sort();
    p
}
fn same(a: &Closure, b: &Closure) {
    assert_eq!(
        a.known.iter().cloned().collect::<BTreeSet<_>>(),
        b.known.iter().cloned().collect()
    );
    assert_eq!(proofs(a), proofs(b));
}
const CHAIN: &str = "module Steps\nconst tau : PropositionTime\nseeded verb A()\nderived verb B()\nderived verb C()\nrule ab() = A() @ tau <= B() @ tau\nrule bc() = B() @ tau <= C() @ tau\nexperiment e { seeds { A() @ tau } query residual C() @ tau }";

#[test]
fn synchronous_steps_are_rule_order_independent_and_expose_t_not_only_delta() {
    let m = module(CHAIN);
    let e = experiment(&m);
    let r = rules(&m);
    let mut a = ForwardSession::new(e.input.clone(), &r);
    let mut reversed = r.clone();
    reversed.reverse();
    let mut b = ForwardSession::new(e.input.clone(), &reversed);
    let first = a.step(&mut Clock::default());
    let other = b.step(&mut Clock::default());
    assert_eq!(first.added, other.added);
    assert_eq!(first.added[0].verb, "B");
    assert!(!first.known.contains(&e.goal));
    assert!(!first.complete);
    let second = a.step(&mut Clock::default());
    assert_eq!(second.added[0].verb, "C");
    assert_eq!(
        second
            .consequences
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["B", "C"])
    );
    assert!(a.step(&mut Clock::default()).complete);
    assert_eq!(a.closure().derivations.len(), 2);
    same(
        a.closure(),
        &close(e.input.clone(), &r, &mut Clock::default()),
    );
}

#[test]
fn unit_negative_conjunction_and_new_proofs_for_old_events_are_retained() {
    let m = module(
        "module StepProofs\nconst tau : PropositionTime\nderived verb A()\nderived verb B()\nrule axiom() = I <= A() @ tau\nrule neg() = I <= ~A() @ tau\nrule join() = A() @ tau & ~A() @ tau <= B() @ tau\nrule again() = B() @ tau <= A() @ tau\nexperiment e { seeds {} query residual B() @ tau }",
    );
    let mut s = ForwardSession::new(Vec::new(), &rules(&m));
    let mut clock = Clock::default();
    let first = s.step(&mut clock);
    assert_eq!(first.added.len(), 2);
    assert!(
        first
            .witnesses
            .iter()
            .all(|w| w.record.antecedents.is_empty())
    );
    assert_eq!(s.step(&mut clock).added.len(), 1);
    let stable = s.step(&mut clock);
    assert!(stable.complete);
    assert!(stable.added.is_empty());
    assert_eq!(stable.derivations.len(), 1);
    assert_eq!(stable.derivations[0].record.rule, "again");
    let stamps = clock.0;
    assert!(s.step(&mut clock).derivations.is_empty());
    assert_eq!(clock.0, stamps);
    same(
        s.closure(),
        &close(Vec::new(), &rules(&m), &mut Clock::default()),
    );
}

#[test]
fn program_steps_preserve_activation_guards_unit_bodies_and_original_producers() {
    let m = crate::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/associated_multi_rule_program.res"),
    )
    .unwrap();
    let e = experiment(&m);
    let original = m.clone();
    let mut s = ForwardSession::for_module(e.input.clone(), &m).unwrap();
    let first = s.step(&mut Clock::default());
    assert!(first.added.iter().any(|e| e.verb == "middle"));
    assert!(!first.known.contains(&e.goal));
    for witness in &first.witnesses {
        assert!(witness.record.antecedents.contains(&e.input[0]));
    }
    s.trace(10, &mut Clock::default());
    assert!(s.is_complete());
    same(
        s.closure(),
        &close_program(e.input.clone(), &m, &mut Clock::default()).unwrap(),
    );
    let mut negative = e.input[0].clone();
    negative.polarity = Polarity::EvidentialNot;
    let mut absent = ForwardSession::for_module(vec![negative], &m).unwrap();
    assert!(absent.step(&mut Clock::default()).added.is_empty());
    assert_eq!(m, original);
}

#[test]
fn derived_trigger_cannot_activate_a_program_in_the_same_round() {
    let m = crate::load_formal_module(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/experiments/associated_multi_rule_program.res"),
    )
    .unwrap();
    let e = m
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == "derived" => Some(e),
            _ => None,
        })
        .unwrap();
    let mut s = ForwardSession::for_module(e.input.clone(), &m).unwrap();
    assert_eq!(
        s.step(&mut Clock::default())
            .added
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        vec!["activated"]
    );
    assert_eq!(
        s.step(&mut Clock::default())
            .added
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        vec!["middle"]
    );
    assert_eq!(
        s.step(&mut Clock::default())
            .added
            .iter()
            .map(|e| e.verb.as_str())
            .collect::<Vec<_>>(),
        vec!["completed"]
    );
}

#[test]
fn finite_cycles_stabilize_and_infinite_temporal_prefixes_remain_incomplete() {
    let m = module(
        "module StepCycle\nconst tau : PropositionTime\nseeded verb A()\nderived verb B()\nrule ab(t) = A() @ t <= B() @ t\nrule bb(t) = B() @ t <= B() @ t\nexperiment e { seeds { A() @ tau } query residual B() @ tau }",
    );
    let mut s = ForwardSession::new(experiment(&m).input.clone(), &rules(&m));
    assert!(s.trace(5, &mut Clock::default()).last().unwrap().complete);
    let m = module(
        "module InfiniteSteps\nconst tau : PropositionTime\nderived verb B()\nrule next(t) = B() @ t <= B() @ after t\nexperiment e { premises { B() @ tau } query fourFusion B() @ after tau }",
    );
    let mut s = ForwardSession::new(experiment(&m).input.clone(), &rules(&m));
    assert!(
        s.trace(4, &mut Clock::default())
            .iter()
            .all(|r| !r.complete)
    );
    assert_eq!(s.closure().known.len(), 5);
}

#[test]
fn derivative_steps_are_phi_approximations_and_agree_with_incremental_update() {
    let m = module(CHAIN);
    let r = rules(&m);
    let e = experiment(&m);
    let base = IncrementalClosure::new(Vec::new(), &r, &mut Clock::default());
    let unchanged = base.closure().clone();
    let mut stepped = base.begin_stepwise_update(e.input.clone());
    let first = stepped.step(&mut Clock::default());
    assert_eq!(first.delta, e.input);
    assert!(first.derivations.is_empty());
    let second = stepped.step(&mut Clock::default());
    assert_eq!(second.added[0].verb, "B");
    assert!(!second.delta.contains(&e.goal));
    assert!(
        stepped
            .trace(5, &mut Clock::default())
            .last()
            .unwrap()
            .complete
    );
    let mut oracle = base.clone();
    let delta = oracle.update(e.input.clone(), &mut Clock::default());
    assert_eq!(
        stepped
            .closure()
            .known
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        delta.known.iter().cloned().collect()
    );
    same(stepped.closure(), oracle.closure());
    assert_eq!(base.closure(), &unchanged);
    let mut duplicate = oracle.begin_stepwise_update(e.input.clone());
    assert!(duplicate.step(&mut Clock::default()).complete);
    assert!(duplicate.step(&mut Clock::default()).delta.is_empty());
}

#[test]
fn derivative_conjunctive_frontier_uses_closed_base_and_retains_old_stamps() {
    let m = module(
        "module BaseDerivative\nconst tau : PropositionTime\nseeded verb P()\nseeded verb Q()\nderived verb A()\nderived verb C()\nderived verb D()\nrule axiom() = I <= A() @ tau\nrule join() = P() @ tau & Q() @ tau <= C() @ tau\nrule alternative() = P() @ tau & Q() @ tau <= C() @ tau\nrule next() = C() @ tau <= D() @ tau\nexperiment e { seeds { P() @ tau, Q() @ tau } query residual D() @ tau }",
    );
    let e = experiment(&m);
    let r = rules(&m);
    let base = IncrementalClosure::new(vec![e.input[0].clone()], &r, &mut Clock::default());
    let mut step = base.begin_stepwise_update(vec![e.input[1].clone()]);
    assert_eq!(
        step.step(&mut Clock::default()).delta,
        vec![e.input[1].clone()]
    );
    let join = step.step(&mut Clock::default());
    assert_eq!(join.added.len(), 1);
    assert_eq!(join.derivations.len(), 2);
    assert!(join.delta.iter().all(|e| !base.closure().known.contains(e)));
    assert!(
        step.trace(10, &mut Clock::default())
            .last()
            .unwrap()
            .complete
    );
    let mut oracle = base.clone();
    oracle.update(vec![e.input[1].clone()], &mut Clock::default());
    same(step.closure(), oracle.closure());
    assert_eq!(step.closure().derivations[0], base.closure().derivations[0]);
}
