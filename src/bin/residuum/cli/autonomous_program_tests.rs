//! Proposed Program representation, confined to the finite experiment harness.
//! Rule, the elaborator, close, defaults and DerivationRecord stay unchanged.
use super::*;
use residuum::DerivationClock;
use residuum::typed::{
    Antecedent, Arg, Closure, DeclarationOrigin, DerivationTime, Module, RecordFieldValue, Rule,
    RuleBody, StateTime, Type,
};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("program-{}", self.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    rule: Rule,
    origin: DeclarationOrigin,
    // Explicit scope links: member binder -> program binder. Extra binders
    // belong to that Rule; imported environment binders are never included.
    shared: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Environment {
    symbols: Module,
    active_rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Program {
    id: String,
    parameters: Vec<Arg>,
    rules: Vec<Member>,
    environment: Environment,
}

#[derive(Debug)]
struct ProgramProof {
    program: String,
    producer: String,
    origin: DeclarationOrigin,
    imported: bool,
    proof: DerivedEvent,
}

fn load(relative: &str) -> Module {
    super::default_comparison::legacy_rule_reference(
        residuum::load_formal_module(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples/legal/modules")
                .join(relative),
        )
        .unwrap(),
    )
}

fn rule<'a>(module: &'a Module, name: &str) -> &'a Rule {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(r) if r.name == name => Some(r),
            _ => None,
        })
        .unwrap()
}

fn constant_type<'a>(module: &'a Module, name: &str) -> &'a Type {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Const(c) if c.name == name => Some(&c.ty),
            _ => None,
        })
        .unwrap()
}

// Independent finite model of the proposed scope substitution. It does not
// replace or export the compiler's beta implementation. Ground .res clauses
// provide the independent oracle, and the existing matcher checks proof replay.
fn substitute_term(term: &Term, bindings: &BTreeMap<String, String>) -> Term {
    match term {
        Term::Var(name) => bindings
            .get(name)
            .map(|c| Term::Const(c.clone()))
            .unwrap_or_else(|| term.clone()),
        Term::Constructor {
            constructor,
            argument,
        } => Term::Constructor {
            constructor: *constructor,
            argument: Box::new(substitute_term(argument, bindings)),
        },
        Term::RecordLiteral { entity, fields } => Term::RecordLiteral {
            entity: entity.clone(),
            fields: fields
                .iter()
                .map(|f| RecordFieldValue {
                    field: f.field.clone(),
                    value: substitute_term(&f.value, bindings),
                })
                .collect(),
        },
        _ => term.clone(),
    }
}

fn substitute_time(time: &TimeExpr, bindings: &BTreeMap<String, String>) -> TimeExpr {
    match time {
        TimeExpr::At(_) => time.clone(),
        TimeExpr::Var(name) => bindings
            .get(name)
            .map(|value| TimeExpr::At(value.clone()))
            .unwrap_or_else(|| time.clone()),
        TimeExpr::After(inner) => TimeExpr::After(Box::new(substitute_time(inner, bindings))),
    }
}

fn substitute_event(event: &Event, bindings: &BTreeMap<String, String>) -> Event {
    let mut result = event.clone();
    result.args = result
        .args
        .iter()
        .map(|t| substitute_term(t, bindings))
        .collect();
    result.proposition.time = substitute_time(&result.proposition.time, bindings);
    if let Some(transition) = &mut result.transition {
        for state_time in [&mut transition.input, &mut transition.output] {
            if let StateTime::At(time) = state_time {
                *time = substitute_time(time, bindings);
            }
        }
    }
    result
}

fn substitute_antecedent(a: &Antecedent, bindings: &BTreeMap<String, String>) -> Antecedent {
    match a {
        Antecedent::Unit => Antecedent::Unit,
        Antecedent::Event(e) => Antecedent::Event(substitute_event(e, bindings)),
        Antecedent::And(left, right) => Antecedent::And(
            Box::new(substitute_antecedent(left, bindings)),
            Box::new(substitute_antecedent(right, bindings)),
        ),
    }
}

fn substitute_body(body: &RuleBody, bindings: &BTreeMap<String, String>) -> RuleBody {
    match body {
        RuleBody::HornClause(c) => RuleBody::HornClause(residuum::typed::HornClause {
            antecedent: substitute_antecedent(&c.antecedent, bindings),
            consequent: substitute_event(&c.consequent, bindings),
        }),
        RuleBody::ResidualClause(c) => RuleBody::ResidualClause(residuum::typed::ResidualClause {
            antecedent: substitute_antecedent(&c.antecedent, bindings),
            consequent: residuum::typed::Residual {
                required: substitute_antecedent(&c.consequent.required, bindings),
                consequent: substitute_event(&c.consequent.consequent, bindings),
            },
        }),
    }
}

fn bind_rule(rule: &Rule, constants: &BTreeMap<String, String>, symbols: &Module) -> Rule {
    for (parameter, constant) in constants {
        let binder = rule
            .parameters
            .iter()
            .find(|p| &p.name == parameter)
            .unwrap();
        assert_eq!(&binder.ty, constant_type(symbols, constant));
        // A PropositionTime constant may occur as a verb argument as well as
        // a proposition index. The proposed adapter replaces both positions.
    }
    Rule {
        name: rule.name.clone(),
        parameters: rule
            .parameters
            .iter()
            .filter(|p| !constants.contains_key(&p.name))
            .cloned()
            .collect(),
        body: substitute_body(&rule.body, constants),
    }
}

impl Program {
    fn partial(&self, constants: &[&str]) -> Self {
        assert!(constants.len() <= self.parameters.len());
        let applied: BTreeMap<_, _> = self
            .parameters
            .iter()
            .zip(constants)
            .map(|(p, c)| {
                assert_eq!(&p.ty, constant_type(&self.environment.symbols, c));
                (p.name.clone(), c.to_string())
            })
            .collect();
        let mut result = self.clone();
        result.parameters = self.parameters[constants.len()..].to_vec();
        for member in &mut result.rules {
            let local: BTreeMap<_, _> = member
                .shared
                .iter()
                .filter_map(|(local, parent)| {
                    applied.get(parent).map(|c| (local.clone(), c.clone()))
                })
                .collect();
            member.rule = bind_rule(&member.rule, &local, &self.environment.symbols);
            member
                .shared
                .retain(|_, parent| !applied.contains_key(parent));
        }
        // The imported Rule has its own t/subject/id binders, not this lambda's.
        assert_eq!(result.environment, self.environment);
        result
    }

    fn close(&self, input: &[Event]) -> (Closure, Vec<ProgramProof>) {
        let mut rules: Vec<_> = self.rules.iter().map(|m| m.rule.clone()).collect();
        rules.extend(self.environment.active_rules.clone());
        let closure = residuum::close(input.to_vec(), &rules, &mut Clock::default());
        let frames = closure
            .derivations
            .iter()
            .map(|p| {
                let owned = self.rules.iter().find(|m| m.rule.name == p.record.rule);
                let origin = owned
                    .map(|m| &m.origin)
                    .unwrap_or_else(|| self.environment.symbols.origin(&p.record.rule).unwrap());
                ProgramProof {
                    program: self.id.clone(),
                    producer: p.record.rule.clone(),
                    origin: origin.clone(),
                    imported: owned.is_none(),
                    proof: p.clone(),
                }
            })
            .collect();
        (closure, frames)
    }
}

fn program(module: &Module, entry: &str, dependent: &str, package: &str) -> Program {
    let parameters = rule(module, entry).parameters.clone();
    let members = [entry, "reciprocalTransferPermission", dependent]
        .into_iter()
        .map(|name| {
            let original = rule(module, name);
            let captures = if name == "reciprocalTransferPermission" {
                BTreeMap::from([("package".into(), package.into())])
            } else {
                BTreeMap::new()
            };
            let rule = bind_rule(original, &captures, module);
            // These are explicit experiment scope declarations, validated against
            // actual typed parameters. This is not universal name-based inference.
            let shared = parameters
                .iter()
                .filter_map(|parent| {
                    rule.parameters
                        .iter()
                        .find(|p| p.name == parent.name)
                        .map(|local| {
                            assert_eq!(local.ty, parent.ty);
                            (local.name.clone(), parent.name.clone())
                        })
                })
                .collect();
            Member {
                rule,
                shared,
                origin: module.origin(name).unwrap().clone(),
            }
        })
        .collect();
    Program {
        id: format!("{}:{entry}", module.origin(entry).unwrap().module),
        parameters,
        rules: members,
        environment: Environment {
            symbols: module.clone(),
            // Captured import dependencies are explicit. Holding symbols does
            // not implicitly enable every Rule present in the imported Module.
            active_rules: vec![rule(module, "obligationCreatesRelation").clone()],
        },
    }
}

fn normal(module: &Module) -> Program {
    program(module, "normalSale", "plainTransferObligation", "Plain")
}

fn retention(module: &Module) -> Program {
    program(
        module,
        "retentionSale",
        "paidRetentionObligation",
        "Retention",
    )
}

fn canonical(program: &Program, closure: &Closure) -> Closure {
    let derivations = closure
        .derivations
        .iter()
        .map(|p| {
            let original = rule(&program.environment.symbols, &p.record.rule);
            let candidates: Vec<_> =
                residuum::instantiate_from_known(original, &p.record.antecedents)
                    .into_iter()
                    .filter(|r| r.event == p.event && r.record.antecedents == p.record.antecedents)
                    .collect();
            assert_eq!(candidates.len(), 1);
            let replay = candidates.into_iter().next().unwrap();
            assert_eq!(replay.event, p.event);
            assert_eq!(replay.event.proposition, p.event.proposition);
            assert_eq!(replay.event.transition, p.event.transition);
            residuum::stamp(replay, p.derived_at.clone())
        })
        .collect();
    Closure {
        known: closure.known.clone(),
        derivations,
    }
}

fn same_closure(a: &Closure, b: &Closure) {
    assert_eq!(
        a.known.iter().cloned().collect::<BTreeSet<_>>(),
        b.known.iter().cloned().collect::<BTreeSet<_>>()
    );
    assert!(a.known.iter().all(|x| {
        b.known
            .iter()
            .any(|y| x == y && x.proposition == y.proposition && x.transition == y.transition)
    }));
    assert_eq!(a.derivations.len(), b.derivations.len());
    for p in &a.derivations {
        assert_eq!(
            a.derivations
                .iter()
                .filter(|q| q.event == p.event && q.record == p.record)
                .count(),
            b.derivations
                .iter()
                .filter(|q| q.event == p.event
                    && q.record == p.record
                    && q.event.proposition == p.event.proposition
                    && q.event.transition == p.event.transition)
                .count()
        );
    }
}

fn write_report(file: &str, report: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(file);
    std::fs::write(&path, report).unwrap();
    println!("{report}\nReport: {}", path.display());
}

#[test]
fn captured_program_bundles_follow_existing_default_selection_and_match_sales() {
    let module = load("defaults_scenario.res");
    let before = module.clone();
    let results = run_default_experiments(&module).unwrap();
    let normal = normal(&module);
    let retention = retention(&module);
    assert_eq!(normal.rules.len(), 3);
    let bundles: BTreeMap<_, _> = [normal, retention]
        .into_iter()
        .map(|p| (p.rules[0].rule.name.clone(), p))
        .collect();
    let mut report = String::from(
        "Historical Program prototype against the RuleRef reference; native ProgramRef integration is checked separately.\nscenario | selected whole bundle | same events/times/substitutions/provenance\n",
    );
    for result in results.iter().take(4) {
        let saved = result.clone();
        // This adapter is an explicit experiment association. No change to
        // applicability -> blocking -> priority, or fallback name heuristics.
        let selected =
            residuum::defaults::evaluate_defaults(&module, &result.context.known).unwrap();
        assert_eq!(selected, result.extension);
        let [entry] = selected.rules() else {
            panic!("one selected entry")
        };
        let bundle = &bundles[&entry.name];
        let input = &result.observation.experiment.input;
        let before_selection = residuum::close(
            input.clone(),
            &bundle.environment.active_rules,
            &mut Clock::default(),
        );
        assert_eq!(before_selection.known, *input);
        assert!(before_selection.derivations.is_empty());
        let (closure, frames) = bundle.close(input);
        same_closure(&canonical(bundle, &closure), &result.observation.closure);
        assert_eq!(frames.len(), closure.derivations.len());
        for frame in frames {
            assert_eq!(frame.program, bundle.id);
            assert_eq!(frame.producer, frame.proof.record.rule);
            assert_eq!(
                frame.origin.module,
                if frame.imported {
                    "LegalMonotonicCase"
                } else {
                    "Sales"
                }
            );
            report.push_str(&format!(
                "  {} / {} [{}] -> {} | {}\n",
                frame.program,
                frame.producer,
                frame.origin.module,
                render_event(&frame.proof.event),
                if frame.imported {
                    "captured import"
                } else {
                    "owned member"
                }
            ));
        }
        report.push_str(&format!(
            "{} | {} | equal modulo explicit package capture; derivedAt external\n",
            result.observation.experiment.name, bundle.id
        ));
        assert_eq!(&saved, result);
    }
    // Autonomy: execution receives only Program + Known, not the surrounding
    // Module's currently active rules. Imported targets were captured once.
    let mut ambient = module.clone();
    ambient
        .declarations
        .retain(|d| !matches!(d, Declaration::Rule(_)));
    let bundle = &bundles["normalSale"];
    assert!(
        bundle
            .environment
            .symbols
            .origin("obligationCreatesRelation")
            .is_some()
    );
    let input = &results[0].observation.experiment.input;
    let (closure, _) = bundle.close(input);
    same_closure(
        &canonical(bundle, &closure),
        &results[0].observation.closure,
    );
    assert_eq!(module, before);
    report.push_str("No runtime/state execution; no rules injected into the real Module. Default-to-Program association and ProgramProof frames are test-only metadata, not language features. Equality established for four source scenarios, not arbitrary contexts.\n");
    write_report("autonomous-program-selection.txt", &report);
}

#[test]
fn shared_program_beta_specializes_every_member_and_preserves_import_scopes() {
    let module = load("defaults_scenario.res");
    let original = normal(&module);
    let partial = original.partial(&["saleCase", "juan"]);
    assert_eq!(partial.parameters, original.parameters[2..]);
    let existing = load("default_comparison/program_specialization.res");
    for (member, alias) in
        partial
            .rules
            .iter()
            .zip(["normalForJuan", "permissionForJuan", "obligationForJuan"])
    {
        let mut expected = rule(&existing, alias).clone();
        expected.name = member.rule.name.clone();
        if alias == "permissionForJuan" {
            expected = bind_rule(
                &expected,
                &BTreeMap::from([("package".into(), "Plain".into())]),
                &module,
            );
        }
        assert_eq!(member.rule, expected);
    }
    let arguments = [
        "saleCase",
        "juan",
        "car",
        "pedro",
        "price",
        "saleTransferDuty",
        "saleTransferPermission",
        "private",
        "oldDate",
        "tau",
    ];
    let whole = original.partial(&arguments);
    assert_eq!(partial.partial(&arguments[2..]), whole);
    // Program-binder identity is carried by the scope links, not by a global
    // replacement of equally spelled binders in the surrounding module.
    let mut renamed_parent = original.clone();
    renamed_parent.parameters[1].name = "programSeller".into();
    for member in &mut renamed_parent.rules {
        for parent in member.shared.values_mut() {
            if parent == "seller" {
                *parent = "programSeller".into();
            }
        }
    }
    assert_eq!(renamed_parent.partial(&arguments).rules, whole.rules);
    assert!(whole.parameters.is_empty());
    let ground = load("default_comparison/autonomous_program_beta_reference.res");
    let mut report = String::from(
        "Program beta prototype: shared scope projected to every member; imported Rule scopes unchanged.\nPartial prefix: saleId=saleCase, seller=juan; eight residual program binders.\nStaged full beta equals direct full beta.\n",
    );
    for (member, oracle, existing_beta) in whole
        .rules
        .iter()
        .zip([
            ("groundEntry", "entryViaExistingBeta"),
            ("groundPermission", "permissionViaExistingBeta"),
            ("groundObligation", "obligationViaExistingBeta"),
        ])
        .map(|(m, (o, b))| (m, o, b))
    {
        assert!(member.rule.parameters.is_empty());
        assert_eq!(member.rule.body, rule(&ground, oracle).body);
        // The actual elaborator must now match the independent ground oracle;
        // Program is still a harness model, not a compiler construction.
        let existing_application = rule(&ground, existing_beta);
        let matches_ground = existing_application.body == rule(&ground, oracle).body;
        report.push_str(&format!("{}: proposed beta matches ground source oracle; existing Rule beta matches oracle={matches_ground}\n",member.rule.name));
        assert!(existing_application.parameters.is_empty());
        assert!(
            matches_ground,
            "real beta must substitute PropositionTime constants in argument positions too"
        );
    }
    let results = run_default_experiments(&module).unwrap();
    let (closed, frames) = whole.close(&results[0].observation.experiment.input);
    same_closure(&canonical(&whole, &closed), &results[0].observation.closure);
    assert!(
        frames
            .iter()
            .any(|p| p.imported && p.producer == "obligationCreatesRelation")
    );
    assert!(closed.known.iter().any(|e| e.verb == "do"
        && e.proposition.time == TimeExpr::After(Box::new(TimeExpr::At("tau".into())))));
    let scope = existing
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == "specializationScope" => Some(e),
            _ => None,
        })
        .unwrap();
    let (restricted, _) = partial.close(&scope.input);
    assert!(!restricted.known.contains(&scope.goal));
    assert!(restricted.known.iter().any(|e| e.verb == "obligation"));
    assert_eq!(original.parameters.len(), 10);
    assert_eq!(partial.environment, original.environment);
    report.push_str("Same-id/different-seller counterexample: program-wide partial beta excludes the second seller's obligation. The imported effect rule retains its own binders and produces do @ after(tau).\n");
    write_report("autonomous-program-beta.txt", &report);
}
