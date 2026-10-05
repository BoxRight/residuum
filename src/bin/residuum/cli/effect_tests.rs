//! Source fixtures declare the effects; these tests only inspect and execute them.
use residuum::runtime::{FieldName, ObjectId, StateSnapshot, Store, Value};
use residuum::typed::{
    Declaration, DerivationTime, Effect, Event, Module, Polarity, PropositionKind, PropositionType,
    RuleBody, StateOperation, StateTime, Term, TimeExpr, Verb, VerbKind,
};
use residuum::{DerivationClock, close, execute_effect};

const CONSTANTS: &str = include_str!("../../../../examples/experiments/effect_body_constants.res");
const MODES: &str = include_str!("../../../../examples/legal/do_notdo_formal.res");
const MODE_INPUTS: &str = include_str!("../../../../examples/experiments/effect_modes_inputs.res");

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("test-{}", self.0),
        }
    }
}

fn after(time: &TimeExpr) -> TimeExpr {
    TimeExpr::After(Box::new(time.clone()))
}

// These are external ObjectId bindings, following the existing runtime API;
// module constants such as deliver and the two modes remain Term::Const.
fn request(name: &str, time: TimeExpr) -> Event {
    Event {
        verb: name.into(),
        args: vec![
            Term::Var("juan".into()),
            Term::Const("deliver".into()),
            Term::Var("car".into()),
            Term::Var("pedro".into()),
        ],
        polarity: Polarity::Positive,
        proposition: PropositionType {
            kind: PropositionKind::Seeded,
            time,
        },
        transition: None,
    }
}

fn snapshot() -> StateSnapshot {
    let mut store = Store::new();
    for person in ["juan", "pedro"] {
        let patrimony = ObjectId(format!("{person}-patrimony"));
        store = store.with_value(
            ObjectId(person.into()),
            FieldName("patrimony".into()),
            Value::Object(patrimony.clone()),
        );
        for field in ["assets", "rights", "obligations"] {
            let values = if person == "juan" && field == "assets" {
                vec![ObjectId("car".into())]
            } else {
                vec![]
            };
            store = store.with_set(patrimony.clone(), FieldName(field.into()), values);
        }
    }
    StateSnapshot {
        time: StateTime::At(TimeExpr::At("tau".into())),
        store,
    }
}

fn expected_relation(mode: &str) -> Value {
    relation_for("deliver", mode)
}

fn relation_for(conduct: &str, mode: &str) -> Value {
    Value::Record {
        entity: "Relation".into(),
        fields: [
            (
                FieldName("debtor".into()),
                Value::Object(ObjectId("juan".into())),
            ),
            (
                FieldName("thing".into()),
                Value::Object(ObjectId("car".into())),
            ),
            (FieldName("conduct".into()), Value::Const(conduct.into())),
            (
                FieldName("creditor".into()),
                Value::Object(ObjectId("pedro".into())),
            ),
            (FieldName("mode".into()), Value::Const(mode.into())),
        ]
        .into_iter()
        .collect(),
    }
}

fn verb<'a>(module: &'a Module, name: &str) -> &'a Verb {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == name => Some(v),
            _ => None,
        })
        .unwrap()
}

fn operation_value(verb: &Verb) -> &Term {
    let Some(Effect::StateTransform { operations, .. }) = &verb.effect else {
        panic!("fixture verb has an effect body");
    };
    match &operations[0] {
        StateOperation::AddToSet { value, .. } | StateOperation::RemoveFromSet { value, .. } => {
            value
        }
    }
}

#[test]
fn effect_constants_resolve_ground_and_parameters_keep_precedence() {
    let module = residuum::parse_formal_to_typed(CONSTANTS).unwrap();
    let Term::RecordLiteral { fields, .. } = operation_value(verb(&module, "capture")) else {
        panic!("record literal");
    };
    assert_eq!(fields[0].value, Term::Const("Fixed".into()));
    let Term::RecordLiteral { fields, .. } = operation_value(verb(&module, "captureArgument"))
    else {
        panic!("record literal");
    };
    assert_eq!(fields[0].value, Term::Var("Fixed".into()));
    for name in ["addMarker", "removeMarker"] {
        assert_eq!(
            operation_value(verb(&module, name)),
            &Term::Const("Fixed".into())
        );
    }
}

#[test]
fn effect_constants_reject_unknown_names_and_wrong_field_types() {
    let unknown = CONSTANTS.replace("marker = Fixed", "marker = Unknown");
    assert!(
        residuum::parse_formal_to_typed(&unknown)
            .unwrap_err()
            .to_string()
            .contains("unknown effect argument or constant `Unknown`")
    );
    let wrong_type = CONSTANTS.replace("const Fixed : Marker", "const Fixed : Bucket");
    assert!(
        residuum::parse_formal_to_typed(&wrong_type)
            .unwrap_err()
            .to_string()
            .contains("cannot assign `Fixed` to field `Entry.marker`")
    );
}

#[test]
fn effect_modes_are_normal_record_fields_and_coexist_idempotently() {
    let module = residuum::experiments::elaborate_composed_fixture(
        residuum::parse_formal(MODES).unwrap(),
        residuum::parse_formal(MODE_INPUTS).unwrap(),
    )
    .unwrap();
    for (name, mode) in [("do", "Doing"), ("notDo", "Abstaining")] {
        let Some(Effect::StateTransform { operations, .. }) = &verb(&module, name).effect else {
            panic!("fixture effect");
        };
        assert_eq!(operations.len(), 2);
        for op in operations {
            let StateOperation::AddToSet {
                value: Term::RecordLiteral { fields, .. },
                ..
            } = op
            else {
                panic!("ordinary record insertion");
            };
            assert_eq!(
                fields.iter().find(|f| f.field == "mode").unwrap().value,
                Term::Const(mode.into())
            );
        }
    }
    let tau = TimeExpr::At("tau".into());
    let times = [
        tau.clone(),
        after(&tau),
        after(&after(&tau)),
        after(&after(&after(&tau))),
    ];
    let rules: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r.clone()),
            _ => None,
        })
        .collect();
    let seeds = times
        .iter()
        .zip([
            "doingRequired",
            "abstentionRequired",
            "doingRequired",
            "abstentionRequired",
        ])
        .map(|(time, name)| request(name, time.clone()))
        .collect();
    let input = snapshot();
    let original = input.clone();
    let closure = close(seeds, &rules, &mut Clock::default());
    assert_eq!(input, original, "derivation never executes effects");
    assert_eq!(closure.known.len(), 8);
    assert_eq!(closure.derivations.len(), 4);
    assert!(
        !closure
            .known
            .iter()
            .any(|e| e.verb == "Act" || e.polarity == Polarity::EvidentialNot)
    );
    let saved_closure = closure.clone();
    let mut state = input.clone();
    let mut stores: Vec<Store> = Vec::new();
    for (i, name) in ["do", "notDo", "do", "notDo"].into_iter().enumerate() {
        let effect = closure
            .derivations
            .iter()
            .find(|d| d.event.verb == name && d.event.proposition.time == after(&times[i]))
            .unwrap();
        let before = state.clone();
        state = execute_effect(&effect.event, verb(&module, name), &before).unwrap();
        assert_eq!(state.time, StateTime::At(after(&times[i])));
        assert_eq!(
            before.store,
            if i == 0 {
                original.store.clone()
            } else {
                stores[i - 1].clone()
            }
        );
        let relations = if i == 0 {
            vec![expected_relation("Doing")]
        } else {
            vec![expected_relation("Doing"), expected_relation("Abstaining")]
        };
        let expected: std::collections::BTreeSet<_> = relations.into_iter().collect();
        for (person, field) in [("juan", "obligations"), ("pedro", "rights")] {
            assert_eq!(
                state.store.set(
                    &ObjectId(format!("{person}-patrimony")),
                    &FieldName(field.into())
                ),
                Some(&expected)
            );
        }
        stores.push(state.store.clone());
    }
    assert_ne!(expected_relation("Doing"), expected_relation("Abstaining"));
    assert_eq!(stores[1], stores[2], "reinserting Doing is idempotent");
    assert_eq!(stores[2], stores[3], "reinserting Abstaining is idempotent");
    assert_eq!(
        closure, saved_closure,
        "execution cannot add Act or ~Act to Known"
    );
    assert_eq!(input, original);
}

#[test]
fn legal_effects_derive_then_execute_without_creating_historical_evidence() {
    let module = residuum::parse_formal_to_typed(super::LEGAL).unwrap();
    let input = snapshot();
    let original = input.clone();
    let cases = residuum::experiments::run_forward_experiments(&module).unwrap();
    assert_eq!(input, original, "the forward runner does not touch State");

    // Smoke-test the old logical observations against the same source without
    // the three terminal effect bridges. Extra effects must not change facts
    // or the non-effect justification records; clock stamps may differ.
    let mut baseline = module.clone();
    baseline.declarations.retain(|d| match d {
        Declaration::Verb(v) => v.kind != VerbKind::Effect,
        Declaration::Rule(r) => match &r.body {
            RuleBody::HornClause(c) => c.consequent.proposition.kind != PropositionKind::Effect,
            RuleBody::ResidualClause(_) => true,
        },
        _ => true,
    });
    let base_cases = residuum::experiments::run_forward_experiments(&baseline).unwrap();
    for (new, old) in cases.iter().zip(&base_cases) {
        assert_eq!(new.experiment.name, old.experiment.name);
        assert_eq!(new.horn_goal, old.horn_goal);
        let non_effects: std::collections::BTreeSet<_> = new
            .closure
            .known
            .iter()
            .filter(|e| e.proposition.kind != PropositionKind::Effect)
            .collect();
        assert_eq!(non_effects, old.closure.known.iter().collect());
        let proofs: Vec<_> = new
            .closure
            .derivations
            .iter()
            .filter(|d| d.event.proposition.kind != PropositionKind::Effect)
            .collect();
        assert_eq!(proofs.len(), old.closure.derivations.len());
        for proof in proofs {
            assert!(
                old.closure
                    .derivations
                    .iter()
                    .any(|p| p.event == proof.event && p.record == proof.record)
            );
        }
        for event in new.closure.known.iter().filter(|e| e.verb == "Act") {
            assert!(new.experiment.input.contains(event));
        }
    }

    // Explicit caller bindings to the initialized Store. This is test-side
    // binding, not a new runtime resolver: source events in Known stay Const.
    let bindings = ["juan", "pedro", "car"]
        .into_iter()
        .map(|name| (name, ObjectId(name.into())))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (scenario, effect_name, bridge, conduct, mode) in [
        (
            "fulfilledTransfer",
            "give",
            "historicalTransferEffect",
            "",
            "",
        ),
        (
            "retentionPaid",
            "do",
            "obligationCreatesRelation",
            "transfer",
            "Doing",
        ),
        (
            "prohibitedOccurrence",
            "notDo",
            "prohibitionCreatesAbstention",
            "molest",
            "Abstaining",
        ),
    ] {
        let case = cases
            .iter()
            .find(|c| c.experiment.name == scenario)
            .unwrap();
        let saved_closure = case.closure.clone();
        let derived = case
            .closure
            .derivations
            .iter()
            .find(|d| d.event.verb == effect_name && d.record.rule == bridge)
            .unwrap();
        assert_eq!(derived.record.antecedents.len(), 1);
        let substitutions = super::super::render_substitution(&derived.record.substitution);
        let (authority, effective) = match effect_name {
            "give" => ("private", "newDate"),
            "do" => ("private", "oldDate"),
            "notDo" => ("administrative", "oldDate"),
            _ => unreachable!(),
        };
        assert!(
            substitutions.contains(&format!("authority={authority}")),
            "{substitutions}"
        );
        assert!(
            substitutions.contains(&format!("effective={effective}")),
            "{substitutions}"
        );
        assert_eq!(
            derived.event.proposition.time,
            after(&TimeExpr::At("tau".into()))
        );
        assert_eq!(derived.event.polarity, Polarity::Positive);

        // Symbolic constants alone are not object references in this runtime.
        // Keep that boundary visible rather than silently changing execute_effect.
        assert!(
            execute_effect(&derived.event, verb(&module, effect_name), &input)
                .unwrap_err()
                .to_string()
                .contains("object reference base")
        );
        let mut bound = derived.event.clone();
        for arg in &mut bound.args {
            if let Term::Const(name) = arg {
                if let Some(object) = bindings.get(name.as_str()) {
                    *arg = Term::Var(object.0.clone());
                }
            }
        }
        let output = execute_effect(&bound, verb(&module, effect_name), &input).unwrap();
        let expected_store = if effect_name == "give" {
            input
                .store
                .clone()
                .with_set(
                    ObjectId("juan-patrimony".into()),
                    FieldName("assets".into()),
                    vec![],
                )
                .with_set(
                    ObjectId("pedro-patrimony".into()),
                    FieldName("assets".into()),
                    vec![ObjectId("car".into())],
                )
        } else {
            let relation = relation_for(conduct, mode);
            input
                .store
                .clone()
                .with_value(
                    ObjectId("juan-patrimony".into()),
                    FieldName("obligations".into()),
                    Value::Set([relation.clone()].into_iter().collect()),
                )
                .with_value(
                    ObjectId("pedro-patrimony".into()),
                    FieldName("rights".into()),
                    Value::Set([relation].into_iter().collect()),
                )
        };
        assert_eq!(output.store, expected_store);
        assert_eq!(output.time, StateTime::At(bound.proposition.time));
        assert_eq!(
            case.closure, saved_closure,
            "execution never adds Act or ~Act"
        );
        assert_eq!(input, original, "execution preserves its input snapshot");
    }
}
