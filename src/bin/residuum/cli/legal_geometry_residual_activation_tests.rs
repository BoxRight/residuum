//! Active-query regressions and the earlier guarded-search prototype.
use super::*;
use residuum::{IncrementalProgramClosure, SearchLimits, query_program_residual};

#[test]
fn active_query_api_requires_the_matching_trigger_and_certifies_every_answer() {
    let m = residuum::load_formal_module(
        root().join("examples/experiments/associated_program_waits.res"),
    )
    .unwrap();
    let original = m.clone();
    let ex = experiment(&m, "noInstance");
    let base = close(&m, ex.input.clone());
    let before = proofs(&base);
    let ready = base
        .known
        .iter()
        .find(|e| e.verb == "ready")
        .unwrap()
        .clone();
    let signal = ex.input[0].clone();
    let trigger = experiment(&m, "later")
        .input
        .iter()
        .find(|e| e.verb == "starts")
        .unwrap()
        .clone();
    let mut wrong = trigger.clone();
    wrong.proposition.time = TimeExpr::At("sigma".into());
    let mut negative = trigger.clone();
    negative.polarity = Polarity::EvidentialNot;
    let mut report = String::from("case\trequirement\tactivations\tforward_records\n");
    for (name, input, expected) in [
        ("empty", vec![], vec![trigger.clone(), signal.clone()]),
        ("ready", vec![ready.clone()], vec![trigger.clone()]),
        ("signals", vec![signal], vec![trigger.clone()]),
        (
            "wrong-time",
            vec![wrong, ready.clone()],
            vec![trigger.clone()],
        ),
        (
            "negative",
            vec![negative.clone(), ready.clone()],
            vec![trigger.clone()],
        ),
        ("positive", vec![trigger.clone(), ready.clone()], vec![]),
        (
            "both",
            vec![negative, trigger.clone(), ready.clone()],
            vec![],
        ),
    ] {
        let evidence = input.clone();
        let query = residuum::query_active_program_residual(
            &m,
            &input,
            &ex.goal,
            SearchLimits::default(),
            &mut Clock(0),
        )
        .unwrap();
        assert!(query.failure.is_none());
        assert_eq!(query.explanations.len(), 1);
        for answer in &query.explanations {
            let mut atoms_out = Vec::new();
            atoms(&answer.candidate.required, &mut atoms_out);
            assert_eq!(
                atoms_out.into_iter().cloned().collect::<BTreeSet<_>>(),
                expected.iter().cloned().collect()
            );
            assert!(answer.hypothetical.known.contains(&ex.goal));
            let activation = answer
                .activations
                .iter()
                .find(|a| a.rule.name == "onStart")
                .unwrap();
            assert_eq!(activation.trigger, trigger);
            assert_eq!(activation.behavior.name, "onStart");
            assert_eq!(
                activation.rule_origin.as_ref().unwrap().module,
                "AssociatedProgramWaits"
            );
            assert!(!activation.substitution.bindings.is_empty());
            let record = answer
                .hypothetical
                .derivations
                .iter()
                .find(|d| d.event == ex.goal)
                .unwrap();
            assert_eq!(record.record.rule, "onStart");
            assert_eq!(
                record.record.antecedents,
                vec![trigger.clone(), ready.clone()]
            );
            writeln!(
                report,
                "{name}\t{:?}\t{:?}\t{:?}",
                answer.candidate.required,
                answer.activations,
                proofs(&answer.hypothetical)
            )
            .unwrap();
        }
        assert_eq!(input, evidence);
    }
    let forbidden = residuum::query_active_program_residual_with_abducibles(
        &m,
        &[ready],
        &ex.goal,
        &[],
        SearchLimits::default(),
        &mut Clock(0),
    )
    .unwrap();
    assert!(forbidden.explanations.is_empty());
    assert!(forbidden.required.is_none());
    assert_eq!(
        forbidden.failure,
        Some(residuum::AbductionFailure::NotAbducible)
    );
    assert_eq!(proofs(&base), before);
    assert_eq!(m, original);
    save("active-residual-api.tsv", &report);
}

#[test]
fn active_query_api_keeps_unit_guard_and_proves_derived_triggers() {
    for (file, leaf, branches) in [
        ("sells_unit_associated_program.res", "sells", 1),
        ("sells_derived_associated_program.res", "confirmedSale", 2),
    ] {
        let m =
            residuum::load_formal_module(root().join("examples/experiments").join(file)).unwrap();
        let ex = experiment(&m, "real");
        for input in [vec![], ex.input.clone()] {
            let q = residuum::query_active_program_residual(
                &m,
                &input,
                &ex.goal,
                SearchLimits::default(),
                &mut Clock(0),
            )
            .unwrap();
            assert_eq!(q.explanations.len(), branches);
            for answer in q.explanations {
                let mut h = Vec::new();
                atoms(&answer.candidate.required, &mut h);
                if input.is_empty() {
                    assert_eq!(h.len(), 1);
                    assert_eq!(h[0].verb, leaf);
                } else {
                    assert!(h.is_empty());
                }
                assert!(answer.hypothetical.known.contains(&ex.goal));
                assert_eq!(
                    answer
                        .hypothetical
                        .derivations
                        .iter()
                        .filter(|d| d.event == ex.goal)
                        .count(),
                    branches
                );
                assert!(answer.activations.iter().all(|a| a.trigger.verb == "sells"));
                if leaf == "confirmedSale" {
                    assert!(
                        answer
                            .hypothetical
                            .derivations
                            .iter()
                            .any(|d| d.event.verb == "sells" && d.record.rule == "confirm")
                    );
                }
            }
        }
    }
}

#[test]
fn active_query_api_preserves_shared_program_rule_uses_and_distinct_associations() {
    let m = residuum::load_formal_module(
        root().join("examples/experiments/associated_multi_rule_program.res"),
    )
    .unwrap();
    let goal = &experiment(&m, "absent").goal;
    let q = residuum::query_active_program_residual(
        &m,
        &[],
        goal,
        SearchLimits::default(),
        &mut Clock(0),
    )
    .unwrap();
    assert!(q.failure.is_none());
    assert_eq!(q.explanations.len(), 4);
    let mut supports = BTreeSet::new();
    let mut report = String::from("branch\trequirement\tactivations\tforward_records\n");
    for (i, answer) in q.explanations.iter().enumerate() {
        let mut h = Vec::new();
        atoms(&answer.candidate.required, &mut h);
        supports.insert(
            h.into_iter()
                .map(|e| e.verb.clone())
                .collect::<BTreeSet<_>>(),
        );
        assert!(answer.hypothetical.known.contains(goal));
        assert_eq!(answer.activations.len(), 2);
        for a in &answer.activations {
            assert_eq!(a.behavior.kind, BehaviorKind::Program);
            assert_eq!(a.behavior.name, "Chain");
            let origin = a.program_origin.as_ref().unwrap();
            assert_eq!(origin.program, "Chain");
            assert_eq!(origin.defining_module, "AssociatedMultiRuleProgram");
            assert!(a.rule.name.starts_with("Chain::"));
            assert_eq!(a.trigger.proposition.time, TimeExpr::At("tau".into()));
        }
        let original = close(
            &m,
            answer
                .hypothetical
                .known
                .iter()
                .filter(|e| e.proposition.kind == PropositionKind::Seeded)
                .cloned()
                .collect(),
        );
        assert_eq!(proofs(&answer.hypothetical), proofs(&original));
        writeln!(
            report,
            "{i}\t{:?}\t{:?}\t{:?}",
            answer.candidate.required,
            answer.activations,
            proofs(&answer.hypothetical)
        )
        .unwrap();
    }
    assert_eq!(
        supports,
        BTreeSet::from([
            BTreeSet::from(["starts".into()]),
            BTreeSet::from(["requests".into()]),
            BTreeSet::from(["starts".into(), "requests".into()])
        ])
    );
    let present = close(&m, experiment(&m, "real").input.clone());
    let evidenced = residuum::query_active_program_residual(
        &m,
        &present.known,
        goal,
        SearchLimits::default(),
        &mut Clock(0),
    )
    .unwrap();
    assert_eq!(evidenced.required, Some(residuum::RequirementFormula::Unit));
    assert!(evidenced.explanations[0].activations.is_empty());
    save("active-residual-program.tsv", &report);
}

#[test]
fn active_query_api_certifies_composed_programs_and_chained_activations() {
    let m = module("residual_activation_chain_scenario");
    let original = m.clone();
    let ex = experiment(&m, "real");
    let trigger = ex.input[0].clone();
    let real = close(&m, ex.input.clone());
    let event = |name: &str| real.known.iter().find(|e| e.verb == name).unwrap().clone();
    let handoff = event("handoff");
    let universe = vec![
        trigger.clone(),
        event("middle"),
        handoff.clone(),
        event("processed"),
    ];
    assert_eq!(real.known.len(), 5);
    assert_eq!(real.derivations.len(), 4);
    assert_eq!(handoff.proposition.time, TimeExpr::At("tau".into()));
    assert_eq!(
        ex.goal.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("tau".into())))
    );

    let mut cases: Vec<_> = (0..16)
        .map(|mask| (format!("subset-{mask}"), subset(&universe, mask)))
        .collect();
    for name in ["negative", "wrongTime", "wrongObject"] {
        cases.push((name.into(), experiment(&m, name).input.clone()));
    }
    cases.push(("goal-evidenced".into(), vec![ex.goal.clone()]));
    let mut report = String::from("case\trequirement\tactivations\tforward_records\n");
    for (name, input) in cases {
        let before = input.clone();
        let q = residuum::query_active_program_residual(
            &m,
            &input,
            &ex.goal,
            SearchLimits::default(),
            &mut Clock(0),
        )
        .unwrap();
        assert!(q.failure.is_none(), "{name}");
        assert_eq!(q.explanations.len(), 1, "{name}");
        let answer = &q.explanations[0];
        let mut pending = Vec::new();
        atoms(&answer.candidate.required, &mut pending);
        let expected =
            if input.contains(&trigger) || input.contains(&handoff) || input.contains(&ex.goal) {
                BTreeSet::new()
            } else {
                BTreeSet::from([trigger.clone()])
            };
        assert_eq!(
            pending
                .iter()
                .map(|e| (*e).clone())
                .collect::<BTreeSet<_>>(),
            expected,
            "{name}"
        );
        if expected.is_empty() {
            assert_eq!(
                q.required,
                Some(residuum::RequirementFormula::Unit),
                "{name}"
            );
        }
        let mut hypothetical_input = input.clone();
        hypothetical_input.extend(pending.into_iter().cloned());
        let oracle =
            residuum::derive::close_program_reference(hypothetical_input, &m, &mut Clock(1000))
                .unwrap();
        assert!(answer.hypothetical.known.contains(&ex.goal));
        assert_eq!(
            full_events(&answer.hypothetical),
            full_events(&oracle),
            "{name}"
        );
        // Sorted full records preserve multiplicity, ordered antecedents and
        // canonical substitutions; only independent clock stamps are excluded.
        assert_eq!(proofs(&answer.hypothetical), proofs(&oracle), "{name}");
        for a in &answer.activations {
            assert_eq!(a.behavior.kind, BehaviorKind::Program);
            let origin = a.program_origin.as_ref().unwrap();
            assert_eq!(origin.program, a.behavior.name);
            assert_eq!(origin.defining_module, "ResidualActivationChain");
            assert_eq!(origin.rule_origin.module, "ResidualActivationChain");
            assert_eq!(a.rule_origin.as_ref().unwrap(), &origin.rule_origin);
            assert_eq!(
                a.trigger,
                if a.behavior.name == "Prepare" {
                    trigger.clone()
                } else {
                    handoff.clone()
                }
            );
            assert!(
                a.substitution
                    .bindings
                    .iter()
                    .any(|b| b.value == SubstitutionValue::Time(TimeExpr::At("tau".into())))
            );
            assert!(
                a.substitution
                    .bindings
                    .iter()
                    .any(|b| b.value == SubstitutionValue::Term(Term::Const("item".into())))
            );
        }
        for d in &answer.hypothetical.derivations {
            let origin = residuum::programs::program_rule_origin(&m, &d.record.rule).unwrap();
            assert_eq!(origin.defining_module, "ResidualActivationChain");
            assert_eq!(origin.rule_origin.module, "ResidualActivationChain");
            let instance = if origin.program == "Prepare" {
                let mut e = trigger.clone();
                e.args = d.event.args.clone();
                e.proposition.time = d.event.proposition.time.clone();
                e
            } else {
                let mut e = handoff.clone();
                e.args = d.event.args.clone();
                let TimeExpr::After(t) = &d.event.proposition.time else {
                    panic!("successor expected")
                };
                e.proposition.time = *t.clone();
                e
            };
            assert_eq!(d.record.antecedents[0], instance);
            assert_eq!(
                d.record.antecedents.len(),
                if origin.rule_use == 0 { 1 } else { 2 }
            );
        }
        if name == "subset-0" {
            let producers: BTreeSet<_> = answer
                .activations
                .iter()
                .map(|a| a.rule.name.as_str())
                .collect();
            assert_eq!(
                producers,
                BTreeSet::from([
                    "Prepare::0::prepareFirst",
                    "Prepare::1::prepareSecond",
                    "Finish::0::finishFirst",
                    "Finish::1::finishSecond",
                ])
            );
            assert_eq!(full_events(&answer.hypothetical), full_events(&real));
            assert_eq!(proofs(&answer.hypothetical), proofs(&real));
        }
        if input.contains(&ex.goal) {
            assert!(answer.activations.is_empty());
        }
        writeln!(
            report,
            "{name}\t{:?}\t{:?}\t{:?}",
            q.required,
            answer.activations,
            proofs(&answer.hypothetical)
        )
        .unwrap();
        assert_eq!(input, before);
    }
    assert_eq!(m, original);
    save("active-residual-chained-programs.tsv", &report);
}

#[test]
fn active_query_api_does_not_replace_horn_unit_or_activate_unguarded_programs() {
    let m = module("residual_activation_chain_scenario");
    let original = m.clone();
    let ex = experiment(&m, "empty");
    let absent = close(&m, vec![]);
    assert!(absent.known.is_empty());
    assert!(absent.derivations.is_empty());
    // A parameterized I body supplies no bindings to the historical replay.
    // Keep that limitation separate from the ground Unit regression and guards.
    let unbound = query_program_residual(&m, &[], &ex.goal, SearchLimits::default()).unwrap();
    assert_eq!(unbound.required, None);
    assert_eq!(
        unbound.failure,
        Some(residuum::AbductionFailure::NotForwardSupported)
    );
    // Explicit processed evidence binds finishSecond under ordinary Horn. It
    // does not activate Finish without handoff under executable semantics.
    let real = close(&m, experiment(&m, "real").input.clone());
    let processed = real
        .known
        .iter()
        .find(|e| e.verb == "processed")
        .unwrap()
        .clone();
    let evidence = vec![processed];
    let before = evidence.clone();
    assert!(!close(&m, evidence.clone()).known.contains(&ex.goal));
    let horn = query_program_residual(&m, &evidence, &ex.goal, SearchLimits::default()).unwrap();
    assert_eq!(horn.required, Some(residuum::RequirementFormula::Unit));
    let active = residuum::query_active_program_residual_with_abducibles(
        &m,
        &evidence,
        &ex.goal,
        &[],
        SearchLimits::default(),
        &mut Clock(0),
    )
    .unwrap();
    assert_eq!(active.required, None);
    assert!(active.explanations.is_empty());
    assert_eq!(
        active.failure,
        Some(residuum::AbductionFailure::NotAbducible)
    );
    assert_eq!(m, original);
    assert_eq!(evidence, before);
    assert_eq!(full_events(&close(&m, vec![])), full_events(&absent));
}

#[test]
fn active_query_result_contract_preserves_unit_pending_absence_and_mixed_branches() {
    let m = residuum::load_formal_module(
        root().join("examples/experiments/associated_multi_rule_program.res"),
    )
    .unwrap();
    let original = m.clone();
    let ex = experiment(&m, "real");
    let known = close(&m, ex.input.clone());
    let before = proofs(&known);
    let mut report = String::from("case\trequired\tfailure\tbranch_requirements\n");
    for (name, evidence, allowed, expected) in [
        ("pending", vec![], None, (false, true)),
        ("mixed", ex.input.clone(), None, (true, true)),
        ("unit", known.known.clone(), None, (true, false)),
        ("no-explanation", vec![], Some(vec![]), (false, false)),
    ] {
        let evidence_before = evidence.clone();
        let q: residuum::ActiveProgramResidualQuery = if let Some(allowed) = allowed {
            residuum::query_active_program_residual_with_abducibles(
                &m,
                &evidence,
                &ex.goal,
                &allowed,
                SearchLimits::default(),
                &mut Clock(0),
            )
            .unwrap()
        } else {
            residuum::query_active_program_residual(
                &m,
                &evidence,
                &ex.goal,
                SearchLimits::default(),
                &mut Clock(0),
            )
            .unwrap()
        };
        assert_eq!(q.required.is_none(), q.explanations.is_empty(), "{name}");
        assert_eq!(q.failure.is_some(), q.explanations.is_empty(), "{name}");
        let unit = q
            .explanations
            .iter()
            .any(|a| a.candidate.required == Antecedent::Unit);
        let pending = q
            .explanations
            .iter()
            .any(|a| a.candidate.required != Antecedent::Unit);
        assert_eq!((unit, pending), expected, "{name}");
        if name == "unit" {
            assert_eq!(q.required, Some(residuum::RequirementFormula::Unit));
        }
        if name == "mixed" {
            // A successful Unit branch need not be the whole summary. Retain
            // the alternative requests hypothesis and its separate proofs.
            assert!(matches!(
                q.required,
                Some(residuum::RequirementFormula::Join(_, _))
            ));
            assert_eq!(q.explanations.len(), 4);
            for a in q
                .explanations
                .iter()
                .filter(|a| a.candidate.required != Antecedent::Unit)
            {
                let mut leaves = Vec::new();
                atoms(&a.candidate.required, &mut leaves);
                assert!(leaves.iter().all(|e| e.verb == "requests"));
            }
        }
        if name == "no-explanation" {
            assert_eq!(q.failure, Some(residuum::AbductionFailure::NotAbducible));
        }
        for a in &q.explanations {
            assert!(a.hypothetical.known.contains(&ex.goal));
        }
        writeln!(
            report,
            "{name}\t{:?}\t{:?}\t{:?}",
            q.required,
            q.failure,
            q.explanations
                .iter()
                .map(|a| &a.candidate.required)
                .collect::<Vec<_>>()
        )
        .unwrap();
        assert_eq!(evidence, evidence_before);
    }
    let error = residuum::query_active_program_residual(
        &m,
        &[],
        &ex.goal,
        SearchLimits {
            max_nodes: 1,
            ..SearchLimits::default()
        },
        &mut Clock(0),
    )
    .unwrap_err();
    assert!(error.to_string().contains("limit exceeded"));
    writeln!(report, "limit-error\tErr\t{error}\tno partial result").unwrap();
    assert_eq!(m, original);
    assert_eq!(proofs(&known), before);
    save("active-residual-result-contract.tsv", &report);
}

// Experimental search view only. The original module remains the forward oracle.
// This deliberately supports unique, single-Rule associations, not Program RuleUses.
fn guarded_search_view(m: &Module) -> (Module, BTreeMap<String, Event>) {
    assert!(
        !m.declarations
            .iter()
            .any(|d| matches!(d, Declaration::Default(_) | Declaration::Program(_)))
    );
    let mut guards = BTreeMap::new();
    for d in &m.declarations {
        let Declaration::Verb(v) = d else { continue };
        let Some(reference) = &v.program else {
            continue;
        };
        assert_eq!(
            reference.kind,
            BehaviorKind::Rule,
            "unsupported association in this prototype"
        );
        assert!(matches!(v.kind, VerbKind::Seeded | VerbKind::Derived));
        let r = m
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Rule(r) if r.name == reference.name => Some(r),
                _ => None,
            })
            .unwrap();
        assert_eq!(r.parameters.len(), v.args.len() + 1);
        assert_eq!(
            r.parameters.iter().map(|p| &p.ty).collect::<Vec<_>>(),
            v.args
                .iter()
                .map(|p| &p.ty)
                .chain([&Type::PropositionTime])
                .collect::<Vec<_>>()
        );
        let guard = Event {
            verb: v.name.clone(),
            args: r.parameters[..v.args.len()]
                .iter()
                .map(|p| Term::Var(p.name.clone()))
                .collect(),
            polarity: Polarity::Positive,
            proposition: PropositionType {
                kind: match v.kind {
                    VerbKind::Seeded => PropositionKind::Seeded,
                    VerbKind::Derived => PropositionKind::Derived,
                    VerbKind::Effect => unreachable!(),
                },
                time: TimeExpr::Var(r.parameters.last().unwrap().name.clone()),
            },
            transition: None,
        };
        assert!(
            guards.insert(r.name.clone(), guard).is_none(),
            "multiple associations need separate scoped proof identities"
        );
    }
    let mut view = m.clone();
    for d in &mut view.declarations {
        let Declaration::Rule(r) = d else { continue };
        let Some(guard) = guards.get(&r.name) else {
            continue;
        };
        let RuleBody::HornClause(c) = &mut r.body else {
            panic!("prototype requires a Horn body")
        };
        let mut body = Vec::new();
        atoms(&c.antecedent, &mut body);
        if !body.contains(&guard) {
            c.antecedent = match &c.antecedent {
                Antecedent::Unit => Antecedent::Event(guard.clone()),
                other => Antecedent::And(
                    Box::new(Antecedent::Event(guard.clone())),
                    Box::new(other.clone()),
                ),
            };
        }
    }
    (view, guards)
}

#[test]
fn guarded_residual_candidates_are_certified_by_the_unchanged_active_closure() {
    let m = residuum::load_formal_module(
        root().join("examples/experiments/associated_program_waits.res"),
    )
    .unwrap();
    let (view, guards) = guarded_search_view(&m);
    let goal = experiment(&m, "noInstance").goal.clone();
    let signal = experiment(&m, "noInstance").input[0].clone();
    let base = close(&m, vec![signal.clone()]);
    let ready = base
        .known
        .iter()
        .find(|e| e.verb == "ready")
        .unwrap()
        .clone();
    let trigger = experiment(&m, "later")
        .input
        .iter()
        .find(|e| e.verb == "starts")
        .unwrap()
        .clone();
    let mut wrong = trigger.clone();
    wrong.proposition.time = TimeExpr::At("sigma".into());
    let mut negative = trigger.clone();
    negative.polarity = Polarity::EvidentialNot;
    // Validation alone rejects the old Unit candidate; it does not invent starts.
    let old = query_program_residual(&m, &base.known, &goal, SearchLimits::default()).unwrap();
    assert_eq!(old.explanations[0].required, Antecedent::Unit);
    assert!(!close(&m, base.known.clone()).known.contains(&goal));
    let before_module = m.clone();
    let mut report = String::from(
        "case\tevidence\trequirement\tguard_patterns\tquery_proof\tforward_goal_records\n",
    );
    for (name, evidence, expected) in [
        ("empty", vec![], vec![trigger.clone(), signal.clone()]),
        ("ready-only", vec![ready.clone()], vec![trigger.clone()]),
        ("signal-only", vec![signal.clone()], vec![trigger.clone()]),
        (
            "positive-trigger-and-ready",
            vec![trigger.clone(), ready.clone()],
            vec![],
        ),
        (
            "wrong-time-and-ready",
            vec![wrong, ready.clone()],
            vec![trigger.clone()],
        ),
        (
            "negative-trigger-and-ready",
            vec![negative.clone(), ready.clone()],
            vec![trigger.clone()],
        ),
        (
            "both-polarities-and-ready",
            vec![negative, trigger.clone(), ready],
            vec![],
        ),
    ] {
        let before = evidence.clone();
        let answer =
            query_program_residual(&view, &evidence, &goal, SearchLimits::default()).unwrap();
        assert!(answer.failure.is_none());
        assert_eq!(answer.explanations.len(), 1);
        let explanation = &answer.explanations[0];
        let mut leaves = Vec::new();
        atoms(&explanation.required, &mut leaves);
        let hypotheses: BTreeSet<_> = leaves.into_iter().cloned().collect();
        assert_eq!(hypotheses, expected.into_iter().collect());
        assert!(
            hypotheses
                .iter()
                .all(|e| e.proposition.kind == PropositionKind::Seeded)
        );
        let mut hypothetical = evidence.clone();
        hypothetical.extend(hypotheses);
        let c = close(&m, hypothetical);
        assert!(
            c.known.contains(&goal),
            "candidate must be sufficient under actual activation"
        );
        let records: Vec<_> = c
            .derivations
            .iter()
            .filter(|d| d.event == goal)
            .map(|d| &d.record)
            .collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].rule, "onStart");
        assert_eq!(records[0].antecedents[0], trigger);
        assert_eq!(records[0].antecedents[1].verb, "ready");
        assert_eq!(
            m.origin(&records[0].rule).unwrap(),
            view.origin(&records[0].rule).unwrap()
        );
        assert_eq!(evidence, before);
        assert_eq!(m, before_module);
        writeln!(
            report,
            "{name}\t{evidence:?}\t{:?}\t{guards:?}\t{:?}\t{records:?}",
            explanation.required, explanation.proof
        )
        .unwrap();
    }
    save("guarded-residual-candidates.tsv", &report);
    println!(
        "Seven ground queries with explicit guard projections; every hypothesis certified by unchanged close_program, no existential extension"
    );
}

#[test]
fn guarded_search_preserves_unit_guards_and_derives_instead_of_abducing_derived_triggers() {
    let mut report = String::from("module\tcase\trequirement\tproof\tforward_records\n");
    for (path, seed_verb, count) in [
        ("sells_unit_associated_program.res", "sells", 1),
        ("sells_derived_associated_program.res", "confirmedSale", 2),
    ] {
        let m =
            residuum::load_formal_module(root().join("examples/experiments").join(path)).unwrap();
        let (view, _) = guarded_search_view(&m);
        let ex = experiment(&m, "real");
        for (name, evidence) in [("absent", vec![]), ("present", ex.input.clone())] {
            let before = evidence.clone();
            let query = query_program_residual(&view, &evidence, &ex.goal, SearchLimits::default())
                .unwrap();
            assert!(query.failure.is_none());
            assert_eq!(query.explanations.len(), count);
            for explanation in query.explanations {
                let mut leaves = Vec::new();
                atoms(&explanation.required, &mut leaves);
                let h: BTreeSet<_> = leaves.into_iter().cloned().collect();
                if name == "absent" {
                    assert_eq!(h.len(), 1);
                    assert_eq!(h.iter().next().unwrap().verb, seed_verb);
                    assert_eq!(
                        h.iter().next().unwrap().proposition.kind,
                        PropositionKind::Seeded
                    );
                } else {
                    assert!(h.is_empty());
                }
                let mut hypothetical = evidence.clone();
                hypothetical.extend(h);
                let c = close(&m, hypothetical);
                assert!(c.known.contains(&ex.goal));
                let records: Vec<_> = c
                    .derivations
                    .iter()
                    .filter(|d| d.event == ex.goal)
                    .map(|d| &d.record)
                    .collect();
                assert_eq!(records.len(), count);
                if seed_verb == "confirmedSale" {
                    assert!(
                        c.derivations
                            .iter()
                            .any(|d| d.event.verb == "sells" && d.record.rule == "confirm")
                    );
                }
                assert!(records.iter().all(|r| {
                    r.antecedents
                        .iter()
                        .any(|a| a.verb == "sells" && a.polarity == Polarity::Positive)
                }));
                for record in &records {
                    assert!(m.origin(&record.rule).is_some());
                }
                writeln!(
                    report,
                    "{}\t{name}\t{:?}\t{:?}\t{records:?}",
                    m.name, explanation.required, explanation.proof
                )
                .unwrap();
            }
            assert_eq!(evidence, before);
        }
    }
    save("guarded-residual-trigger-kinds.tsv", &report);
}

#[test]
fn hypothetical_trigger_obligations_preserve_dynamic_activation_and_evidence() {
    let m = residuum::load_formal_module(
        root().join("examples/experiments/associated_program_waits.res"),
    )
    .unwrap();
    let ex = experiment(&m, "noInstance");
    let known = close(&m, ex.input.clone());
    let before_events = full_events(&known);
    let before_records = proofs(&known);
    let ready = known.known.iter().find(|e| e.verb == "ready").unwrap();
    assert!(!known.known.contains(&ex.goal));
    let trigger = experiment(&m, "later")
        .input
        .iter()
        .find(|e| e.verb == "starts")
        .unwrap()
        .clone();
    assert_eq!(trigger.proposition.time, ready.proposition.time);
    let mut wrong = trigger.clone();
    wrong.proposition.time = TimeExpr::At("sigma".into());
    let mut negative = trigger.clone();
    negative.polarity = Polarity::EvidentialNot;

    // Current residual semantics is unchanged: its ordinary-clause proof lacks starts.
    let ordinary =
        query_program_residual(&m, &known.known, &ex.goal, SearchLimits::default()).unwrap();
    assert_eq!(ordinary.explanations.len(), 1);
    assert_eq!(ordinary.explanations[0].required, Antecedent::Unit);

    let mut report =
        String::from("case\thypotheses\tdynamic_goal\tnew_activations\tgoal_records\n");
    for (name, h, succeeds, activation_count) in [
        ("no-hypothesis", vec![], false, 0),
        ("wrong-time-trigger", vec![wrong], false, 1),
        ("negative-trigger", vec![negative.clone()], false, 0),
        ("matching-positive-trigger", vec![trigger.clone()], true, 1),
        (
            "both-polarities",
            vec![negative.clone(), trigger.clone()],
            true,
            1,
        ),
    ] {
        let mut hypothetical_input = known.known.clone();
        hypothetical_input.extend(h.clone());
        let recomputed = close(&m, hypothetical_input);
        assert_eq!(recomputed.known.contains(&ex.goal), succeeds);

        let mut clock = Clock(0);
        let mut incremental =
            IncrementalProgramClosure::new(known.known.clone(), &m, &mut clock).unwrap();
        let delta = incremental.update(h.clone(), &mut clock);
        assert_eq!(delta.activations.len(), activation_count);
        assert_eq!(full_events(incremental.closure()), full_events(&recomputed));
        assert_eq!(proofs(incremental.closure()), proofs(&recomputed));
        assert_eq!(full_events(&known), before_events);
        assert_eq!(proofs(&known), before_records);
        let records: Vec<_> = recomputed
            .derivations
            .iter()
            .filter(|d| d.event == ex.goal)
            .map(|d| &d.record)
            .collect();
        assert_eq!(records.len(), usize::from(succeeds));
        if succeeds {
            assert_eq!(records[0].rule, "onStart");
            assert_eq!(records[0].antecedents, vec![trigger.clone(), ready.clone()]);
            assert_eq!(
                m.origin(&records[0].rule).unwrap().module,
                "AssociatedProgramWaits"
            );
            assert_eq!(delta.activations[0].trigger, trigger);
            assert_eq!(delta.activations[0].behavior.name, "onStart");
            assert!(delta.known.iter().any(|e| e == &ex.goal));
        }
        if name == "both-polarities" {
            assert!(recomputed.known.contains(&trigger));
            assert!(recomputed.known.contains(&negative));
        }
        writeln!(
            report,
            "{name}\t{:?}\t{succeeds}\t{:?}\t{records:?}",
            h.iter().map(render_event).collect::<Vec<_>>(),
            delta.activations
        )
        .unwrap();
    }
    // Exhaustive only within this singleton declared ground hypothesis domain.
    for mask in 0..2 {
        let h = subset(std::slice::from_ref(&trigger), mask);
        let mut input = known.known.clone();
        input.extend(h);
        assert_eq!(close(&m, input).known.contains(&ex.goal), mask == 1);
    }
    save("residual-activation-contract.tsv", &report);
    println!(
        "Five hypothetical contexts checked; matching starts is the unique sufficient support in the singleton ground domain, not returned by a new solver"
    );
}
