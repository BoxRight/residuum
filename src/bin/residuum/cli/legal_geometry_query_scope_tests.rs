//! AST dependency analysis only. Never enables a candidate or resolves existentials.
use super::*;
use residuum::programs::{compile_program, program_rule_origin};
use residuum::{IncrementalProgramClosure, SearchLimits, query_program_residual, query_residual};

type Key = (String, Polarity, usize);
fn key(e: &Event) -> Key {
    (e.verb.clone(), e.polarity, e.args.len())
}
fn horn(r: &Rule) -> HornClause {
    match &r.body {
        RuleBody::HornClause(c) => c.clone(),
        RuleBody::ResidualClause(c) => c.unresiduate(),
    }
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

// Conservative signature cone: ignores arguments/times, but never polarity/arity.
// All producers are retained, including producers whose parameters are unresolved.
fn cone(rs: &[Rule], goal: &Event) -> (BTreeSet<String>, BTreeSet<Key>) {
    let mut needed = BTreeSet::from([key(goal)]);
    let mut selected = BTreeSet::new();
    loop {
        let before = (needed.len(), selected.len());
        for r in rs {
            let c = horn(r);
            if needed.contains(&key(&c.consequent)) {
                selected.insert(r.name.clone());
                let mut premises = Vec::new();
                atoms(&c.antecedent, &mut premises);
                needed.extend(premises.into_iter().map(key));
            }
        }
        if before == (needed.len(), selected.len()) {
            break;
        }
    }
    (selected, needed)
}

fn term_vars(t: &Term, out: &mut BTreeSet<String>) {
    match t {
        Term::Var(n) => {
            out.insert(n.clone());
        }
        Term::Constructor { argument, .. } => term_vars(argument, out),
        Term::RecordLiteral { fields, .. } => {
            for f in fields {
                term_vars(&f.value, out);
            }
        }
        _ => (),
    }
}
fn event_vars(e: &Event) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for t in &e.args {
        term_vars(t, &mut out);
    }
    term_vars(&e.proposition.time.to_term(), &mut out);
    out
}
fn parameter_report(r: &Rule) -> (Vec<Arg>, Vec<Arg>, Vec<Arg>) {
    let c = horn(r);
    let head = event_vars(&c.consequent);
    let mut body = BTreeSet::new();
    let mut premises = Vec::new();
    atoms(&c.antecedent, &mut premises);
    for e in premises {
        body.extend(event_vars(e));
    }
    let visible = r
        .parameters
        .iter()
        .filter(|p| head.contains(&p.name))
        .cloned()
        .collect();
    let hidden = r
        .parameters
        .iter()
        .filter(|p| body.contains(&p.name) && !head.contains(&p.name))
        .cloned()
        .collect();
    let unused = r
        .parameters
        .iter()
        .filter(|p| !body.contains(&p.name) && !head.contains(&p.name))
        .cloned()
        .collect();
    (visible, hidden, unused)
}

// This adapter is safe here only for fixed ordinary Horn rules. It does not
// convert Verb.program definitions/default targets into mandatory rules.
fn ordinary_view(m: &Module, rs: &[Rule]) -> Module {
    assert!(
        !m.declarations
            .iter()
            .any(|d| matches!(d, Declaration::Default(_)))
    );
    assert!(
        !m.declarations
            .iter()
            .any(|d| matches!(d, Declaration::Verb(v) if v.program.is_some()))
    );
    let mut out = m.clone();
    out.declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    for r in rs {
        if let Some(p) = program_rule_origin(m, &r.name) {
            out.origins.insert(r.name.clone(), p.rule_origin);
        }
        out.declarations.push(Declaration::Rule(r.clone()));
    }
    out
}
fn goal_proofs(c: &Closure, goal: &Event) -> Vec<String> {
    let mut out: Vec<_> = c
        .derivations
        .iter()
        .filter(|d| d.event == *goal)
        .map(|d| format!("{:?} | {:?}", d.event, d.record))
        .collect();
    out.sort();
    out
}

#[test]
fn compliance_dependency_cone_preserves_goal_proofs_without_resolving_hidden_parameters() {
    let g = module("general");
    let p = module("patrimonial");
    let goal = &experiment(&g, "theoryCompliance").input[0];
    let expected: BTreeSet<_> = [
        "obligationCompliance",
        "rightCompliance",
        "filedObligation",
        "statutoryObligation",
        "enactLaw",
        "capacityPower",
        "grantedCapacity",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let mut report = String::from("model\trule\torigin\thead_visible\tbody_only\tunused\n");
    let mut edges = String::from(
        "model\trule\tpremise_literal\tconsequent_literal\tpremise_pattern\tconsequent_pattern\n",
    );
    let mut preservation = String::from("model\tcase\tgoal_present\tgoal_justifications\n");
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let all = rules(m);
        let (relevant, needed) = cone(&all, goal);
        assert_eq!(relevant, expected);
        assert_eq!(needed.len(), 10);
        let selected: Vec<_> = all
            .into_iter()
            .filter(|r| relevant.contains(&r.name))
            .collect();
        let scoped = ordinary_view(m, &selected);
        let mut hidden_total = 0;
        for r in &selected {
            let c = horn(r);
            let mut premises = Vec::new();
            atoms(&c.antecedent, &mut premises);
            for e in premises {
                writeln!(
                    edges,
                    "{label}\t{}\t{}\t{}\t{e:?}\t{:?}",
                    r.name,
                    literal(e),
                    literal(&c.consequent),
                    c.consequent
                )
                .unwrap();
            }
            let (visible, hidden, unused) = parameter_report(r);
            hidden_total += hidden.len();
            assert!(unused.is_empty());
            writeln!(
                report,
                "{label}\t{}\t{:?}\t{visible:?}\t{hidden:?}\t{unused:?}",
                r.name,
                m.origin(&r.name).unwrap()
            )
            .unwrap();
            if horn(r).consequent.verb == goal.verb {
                let local = query_residual(r, goal, &[]).unwrap();
                assert_eq!(local.len(), 1);
                assert_eq!(local[0].parameters, hidden);
                assert_eq!(local[0].goal_substitution.bindings.len(), visible.len());
            }
        }
        assert_eq!(hidden_total, 12);
        assert!(
            query_program_residual(&scoped, &[], goal, SearchLimits::default())
                .unwrap_err()
                .to_string()
                .contains("not determined by the goal")
        );
        let v = &experiment(m, "performance").input;
        let inputs = [v[0].clone(), v[1].clone(), v[5].clone()];
        let mut cases: Vec<_> = (0..8)
            .map(|i| (format!("performance-{i}"), subset(&inputs, i)))
            .collect();
        let sigma = m
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Experiment(e) if e.input_kind == ExperimentInputKind::Seeds => {
                    Some(e.input.clone())
                }
                _ => None,
            })
            .flatten()
            .collect();
        cases.push(("whole-synthetic-evidence".into(), sigma));
        for (name, s) in cases {
            let full = close(m, s.clone());
            let small = close(&scoped, s);
            assert_eq!(full.known.contains(goal), small.known.contains(goal));
            assert_eq!(goal_proofs(&full, goal), goal_proofs(&small, goal));
            audit_proofs(&scoped, &small);
            writeln!(
                preservation,
                "{label}\t{name}\t{}\t{:?}",
                small.known.contains(goal),
                goal_proofs(&small, goal)
            )
            .unwrap();
        }
    }
    save("query-dependency-cone.tsv", &report);
    save("query-dependency-edges.tsv", &edges);
    save("query-dependency-preservation.tsv", &preservation);
}

#[test]
fn dependency_slicing_preserves_explicit_program_bindings_but_does_not_choose_them() {
    let m = module("explicit_substitutions");
    let goal = &experiment(&m, "suppliedContext").goal;
    let mut report = String::from("program\trelevant_compiled_rules\tquery_result\n");
    for name in ["CompliancePartial", "ComplianceBound", "ComplianceGround"] {
        let p = m
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Program(p) if p.identity == name => Some(p),
                _ => None,
            })
            .unwrap();
        let compiled = compile_program(p, &m).unwrap();
        let (relevant, _) = cone(&compiled, goal);
        assert_eq!(relevant.len(), 3);
        let selected: Vec<_> = compiled
            .iter()
            .filter(|r| relevant.contains(&r.name))
            .cloned()
            .collect();
        let slice = ordinary_view(&m, &selected);
        let context = ordinary_view(&m, &compiled);
        match query_program_residual(&slice, &[], goal, SearchLimits::default()) {
            Ok(q) => {
                assert_ne!(name, "CompliancePartial");
                let previous =
                    query_program_residual(&context, &[], goal, SearchLimits::default()).unwrap();
                assert_eq!(q.explanations, previous.explanations);
                assert_eq!(q.explanations.len(), 2);
                writeln!(report, "{name}\t{relevant:?}\t{:?}", q.explanations).unwrap();
            }
            Err(e) => {
                assert_eq!(name, "CompliancePartial");
                assert!(e.to_string().contains("not determined by the goal"));
                writeln!(report, "{name}\t{relevant:?}\t{e}").unwrap();
            }
        }
    }
    // Merely declaring these Programs has not disabled the imported Rules.
    assert_eq!(rules(&m).len(), 27);
    assert_eq!(cone(&rules(&m), goal).0.len(), 7);
    save("query-dependency-explicit-context.tsv", &report);
}

fn load_relative(path: &str) -> Module {
    residuum::load_formal_module(root().join(path)).unwrap()
}
// Ownership is resolution metadata, not applicability or a ground body match.
fn controlled(m: &Module) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for d in &m.declarations {
        let Declaration::Default(d) = d else {
            continue;
        };
        if d.body.is_program() {
            let p = m
                .declarations
                .iter()
                .find_map(|x| match x {
                    Declaration::Program(p) if p.identity == d.body.target_name() => Some(p),
                    _ => None,
                })
                .unwrap();
            out.extend(p.rule_uses.iter().map(|u| u.rule.name.clone()));
        } else {
            out.insert(d.body.target_name().into());
        }
    }
    out
}
fn associated(m: &Module) -> BTreeMap<String, Vec<Key>> {
    let mut out: BTreeMap<String, Vec<Key>> = BTreeMap::new();
    for d in &m.declarations {
        let Declaration::Verb(v) = d else {
            continue;
        };
        let Some(reference) = &v.program else {
            continue;
        };
        let names = match reference.kind {
            BehaviorKind::Rule => vec![reference.name.clone()],
            BehaviorKind::Program => m
                .declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Program(p) if p.identity == reference.name => {
                        Some(p.rule_uses.iter().map(|u| u.rule.name.clone()).collect())
                    }
                    _ => None,
                })
                .unwrap(),
        };
        for n in names {
            out.entry(n)
                .or_default()
                .push((v.name.clone(), Polarity::Positive, v.args.len()));
        }
    }
    out
}

#[test]
fn default_dependencies_never_promote_unselected_rule_uses_to_mandatory_rules() {
    let m = load_relative("examples/legal/modules/defaults_scenario.res");
    let owned = controlled(&m);
    let guarded = associated(&m);
    assert_eq!(owned.len(), 5);
    let all = rules(&m);
    let mut report = String::from(
        "scenario\tselected_programs\tpotential_controlled\tactive_relevant_compiled\tproductive_goal_rules\tbody_only_parameters\n",
    );
    for result in residuum::experiments::run_default_experiments(&m)
        .unwrap()
        .into_iter()
        .take(4)
    {
        let goal = &result.observation.experiment.goal;
        let potential = cone(&all, goal).0;
        assert!(potential.contains("normalSale") && potential.contains("retentionSale"));
        let mut eligible: Vec<_> = all
            .iter()
            .filter(|r| !owned.contains(&r.name) && !guarded.contains_key(&r.name))
            .cloned()
            .collect();
        eligible.extend(result.extension.rules().iter().cloned());
        let active = cone(&eligible, goal).0;
        let selected = &result.extension.programs()[0].name;
        let (expected_program, expected_goal) = match result.observation.experiment.name.as_str() {
            "noRetention" => ("normalSaleProgram", true),
            "retentionUnpaid" => ("retentionSaleProgram", false),
            "retentionPaid" | "normalBlocked" => ("retentionSaleProgram", true),
            _ => panic!("unexpected scenario"),
        };
        assert_eq!(selected, expected_program);
        assert_eq!(result.observation.horn_goal, expected_goal);
        // No associated trigger is known in these four cases, even after closing.
        assert!(guarded.values().flatten().all(|guard| {
            !result
                .observation
                .closure
                .known
                .iter()
                .any(|e| key(e) == *guard)
        }));
        assert!(
            active
                .iter()
                .any(|n| n.starts_with(&format!("{selected}::")))
        );
        let other = if selected == "normalSaleProgram" {
            "retentionSaleProgram"
        } else {
            "normalSaleProgram"
        };
        assert!(active.iter().all(|n| !n.starts_with(&format!("{other}::"))));
        assert!(!active.contains("normalSale") && !active.contains("retentionSale"));
        let relevant_compiled: Vec<_> = active
            .iter()
            .filter(|n| program_rule_origin(&m, n).is_some())
            .cloned()
            .collect();
        let productive: Vec<_> = result
            .observation
            .closure
            .derivations
            .iter()
            .filter(|d| d.event == *goal)
            .map(|d| d.record.rule.clone())
            .collect();
        assert_eq!(!productive.is_empty(), result.observation.horn_goal);
        assert!(productive.iter().all(|n| active.contains(n)));
        assert!(
            result
                .context
                .known
                .iter()
                .all(|e| e.verb != "selectedReciprocity")
        );
        let parameters: Vec<_> = eligible
            .iter()
            .filter(|r| active.contains(&r.name))
            .map(|r| (r.name.clone(), parameter_report(r).1))
            .collect();
        writeln!(
            report,
            "{}\t{:?}\t{:?}\t{relevant_compiled:?}\t{productive:?}\t{parameters:?}",
            result.observation.experiment.name,
            result.extension.programs(),
            potential.intersection(&owned).collect::<Vec<_>>()
        )
        .unwrap();
    }
    save("query-dependency-default-status.tsv", &report);
}

#[test]
fn implicit_verb_program_guard_is_observable_but_is_not_a_horn_premise() {
    let m = load_relative("examples/experiments/sells_unit_associated_program.res");
    let guards = associated(&m);
    let goal = &experiment(&m, "real").goal;
    let all = rules(&m);
    let (relevant, mut dependencies) = cone(&all, goal);
    assert_eq!(relevant, BTreeSet::from(["sellsProgram".into()]));
    assert_eq!(dependencies.len(), 1); // Unit contains no explicit predecessor.
    dependencies.extend(guards["sellsProgram"].iter().cloned());
    assert!(dependencies.contains(&("sells".into(), Polarity::Positive, 3)));
    let raw = query_program_residual(&m, &[], goal, SearchLimits::default()).unwrap();
    assert!(raw.explanations.is_empty());
    assert_eq!(
        raw.failure,
        Some(residuum::AbductionFailure::NotForwardSupported)
    );
    assert_eq!(raw.metrics.head_matches, 1);
    assert_eq!(raw.metrics.rejected_not_forward_supported, 1);
    assert!(!close(&m, Vec::new()).known.contains(goal));
    let mut report =
        String::from("case\timplicit_guards\tactivations\tgoal_present\tgoal_proofs\n");
    for name in ["absent", "negativeOnly", "real"] {
        let ex = experiment(&m, name);
        let before = ex.input.clone();
        let mut clock = Clock(0);
        let mut state = IncrementalProgramClosure::new(Vec::new(), &m, &mut clock).unwrap();
        let delta = state.update(ex.input.clone(), &mut clock);
        assert_eq!(ex.input, before);
        let known = state.closure();
        assert_eq!(known.known.contains(goal), name == "real");
        assert_eq!(delta.activations.len(), usize::from(name == "real"));
        if name == "real" {
            assert_eq!(delta.activations[0].trigger, ex.input[0]);
            assert!(
                known
                    .derivations
                    .iter()
                    .filter(|d| d.event == *goal)
                    .all(|d| d.record.antecedents.contains(&ex.input[0]))
            );
        }
        writeln!(
            report,
            "{name}\t{:?}\t{:?}\t{}\t{:?}",
            guards["sellsProgram"],
            delta.activations,
            known.known.contains(goal),
            goal_proofs(known, goal)
        )
        .unwrap();
    }
    save("query-dependency-activation-guard.tsv", &report);
    // Here the ordinary Horn certificate succeeds, but lacks the implicit
    // starts guard. This contrasts two existing APIs, not a new query policy.
    let waits = load_relative("examples/experiments/associated_program_waits.res");
    let absent = experiment(&waits, "noInstance");
    let known = close(&waits, absent.input.clone());
    assert!(known.known.iter().any(|e| e.verb == "ready"));
    assert!(!known.known.iter().any(|e| e.verb == "starts"));
    assert!(!known.known.contains(&absent.goal));
    let before = proofs(&known);
    let unguarded =
        query_program_residual(&waits, &known.known, &absent.goal, SearchLimits::default())
            .unwrap();
    assert_eq!(unguarded.explanations.len(), 1);
    assert_eq!(unguarded.explanations[0].required, Antecedent::Unit);
    assert_eq!(proofs(&known), before);
    assert!(
        !close(&waits, known.known.clone())
            .known
            .contains(&absent.goal)
    );
    save(
        "query-dependency-activation-boundary.tsv",
        &format!(
            "case\tquery_result\tmodule_forward_goal\tquery_detail\nassociated-Unit-with-local-parameters\tNotForwardSupported\tfalse\t{raw:?}\nready-known-without-starts\tUnit\tfalse\t{:?}\n",
            unguarded.explanations[0].proof
        ),
    );
}
