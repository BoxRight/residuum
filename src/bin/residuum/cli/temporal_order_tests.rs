//! Certification of the definitional temporal-order program built with
//! `after` (docs/legal-semantic-temporal-order.md). Dates are PropositionTime
//! arguments after^n(root); the proposition index is the dossier index and is
//! never a date. Every closure is computed by `close_program`; the oracle in
//! this file is ordinary integer arithmetic on (root, n) and is used only to
//! compare.
use residuum::typed::{
    Antecedent, Closure, Constructor, Declaration, DerivationTime, Event, Module, RuleBody, Term,
    TimeExpr,
};
use residuum::{DerivationClock, close_program, load_formal_module};
use std::collections::BTreeSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

const FIXTURES: &str = "docs/legal-geometry/semantic-domain/temporal-order";
const GENERATED: &str = "target/semantic-domain-temporal-order";
const ORDER_MODULE: &str = "TemporalOrder";
const MAJORITY: i64 = 18;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn load(relative: &str) -> Module {
    load_formal_module(root().join(FIXTURES).join(relative)).unwrap()
}

struct Clock {
    count: usize,
    limit: usize,
}
impl Clock {
    fn new(limit: usize) -> Self {
        Self { count: 0, limit }
    }
}
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.count += 1;
        assert!(
            self.count <= self.limit,
            "derivation bound {} exceeded",
            self.limit
        );
        DerivationTime {
            name: format!("order-{}", self.count),
        }
    }
}

fn after_n(root: &str, n: usize) -> String {
    format!("{}{root}{}", "after(".repeat(n), ")".repeat(n))
}

/// A generated case module: persons and roots declared, one experiment.
fn case(name: &str, roots: &[&str], persons: &[&str], seeds: &[String]) -> (Module, Vec<Event>) {
    let dir = root().join(GENERATED);
    std::fs::create_dir_all(&dir).unwrap();
    let mut source = format!(
        "module Case_{name}\n\nimport \"../../{FIXTURES}/temporal_order.res\"\n\nconst t0 : PropositionTime\n"
    );
    for r in roots {
        source += &format!("const {r} : PropositionTime\n");
    }
    for p in persons {
        source += &format!("const {p} : Person\n");
    }
    source += &format!(
        "\nexperiment {name} {{\n    seeds {{\n{}\n    }}\n    query residual\n    datePoint({}) @ t0\n}}\n",
        seeds
            .iter()
            .map(|s| format!("        {s} @ t0,"))
            .collect::<Vec<_>>()
            .join("\n"),
        roots[0]
    );
    let file = dir.join(format!("{name}.res"));
    std::fs::write(&file, source).unwrap();
    let module = load_formal_module(&file).unwrap();
    let seeds = module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) => Some(e.input.clone()),
            _ => None,
        })
        .unwrap();
    (module, seeds)
}

fn close(module: &Module, seeds: &[Event], limit: usize) -> Closure {
    close_program(seeds.to_vec(), module, &mut Clock::new(limit)).unwrap()
}

/// (root, n) of a date term after^n(root).
fn date(term: &Term) -> (String, i64) {
    match term {
        Term::Const(r) => (r.clone(), 0),
        Term::Constructor {
            constructor: Constructor::After,
            argument,
        } => {
            let (r, n) = date(argument);
            (r, n + 1)
        }
        other => panic!("date term expected, found {other:?}"),
    }
}

fn constant(term: &Term) -> String {
    match term {
        Term::Const(c) => c.clone(),
        other => panic!("constant expected, found {other:?}"),
    }
}

fn pairs(closure: &Closure, verb: &str) -> BTreeSet<((String, i64), (String, i64))> {
    closure
        .known
        .iter()
        .filter(|e| e.verb == verb)
        .map(|e| (date(&e.args[0]), date(&e.args[1])))
        .collect()
}

fn conditions(closure: &Closure, verb: &str) -> BTreeSet<(String, (String, i64))> {
    closure
        .known
        .iter()
        .filter(|e| e.verb == verb)
        .map(|e| (constant(&e.args[0]), date(&e.args[1])))
        .collect()
}

fn atoms(a: &Antecedent) -> Vec<&Event> {
    match a {
        Antecedent::Unit => vec![],
        Antecedent::Event(e) => vec![e],
        Antecedent::And(l, r) => [atoms(l), atoms(r)].concat(),
    }
}

/// Separation contract: in every rule of the given origin, all atoms share one
/// variable proposition index, and no date variable is used as an index.
fn separation_violations(module: &Module, origin: &str) -> Vec<String> {
    let mut violations = Vec::new();
    for d in &module.declarations {
        let Declaration::Rule(rule) = d else { continue };
        if module.origin(&rule.name).map(|o| o.module.as_str()) != Some(origin) {
            continue;
        }
        let RuleBody::HornClause(c) = &rule.body else {
            violations.push(format!("{}: not Horn", rule.name));
            continue;
        };
        let mut events = atoms(&c.antecedent);
        events.push(&c.consequent);
        let indices: BTreeSet<_> = events
            .iter()
            .map(|e| format!("{:?}", e.proposition.time))
            .collect();
        if indices.len() != 1 || !matches!(events[0].proposition.time, TimeExpr::Var(_)) {
            violations.push(format!("{}: index {:?}", rule.name, indices));
            continue;
        }
        let TimeExpr::Var(index) = &events[0].proposition.time else {
            unreachable!()
        };
        let mut argument_vars = BTreeSet::new();
        for e in &events {
            for a in &e.args {
                collect_vars(a, &mut argument_vars);
            }
        }
        if argument_vars.contains(index) {
            violations.push(format!(
                "{}: index `{index}` is also a date argument",
                rule.name
            ));
        }
    }
    violations
}

fn collect_vars(term: &Term, into: &mut BTreeSet<String>) {
    match term {
        Term::Var(v) => {
            into.insert(v.clone());
        }
        Term::Constructor { argument, .. } => collect_vars(argument, into),
        _ => {}
    }
}

#[test]
fn temporal_order_keeps_dates_as_arguments_and_one_dossier_index() {
    let order = load("temporal_order.res");
    assert_eq!(
        separation_violations(&order, ORDER_MODULE),
        Vec::<String>::new()
    );
    // No rule of the program produces a normative position.
    for d in &order.declarations {
        if let Declaration::Rule(rule) = d {
            if order.origin(&rule.name).unwrap().module == ORDER_MODULE {
                let RuleBody::HornClause(c) = &rule.body else {
                    unreachable!()
                };
                assert!(
                    ![
                        "obligation",
                        "permission",
                        "prohibition",
                        "right",
                        "omissionProhibition"
                    ]
                    .contains(&c.consequent.verb.as_str()),
                    "{}",
                    rule.name
                );
            }
        }
    }
    // Counterexample: the dossier date used as the index elaborates and is
    // rejected by the structural check.
    let bad = load("counterexamples/index_as_date.res");
    let violations = separation_violations(&bad, "TemporalOrderIndexAsDate");
    assert_eq!(violations.len(), 1, "{violations:?}");
}

#[test]
fn before_is_the_strict_successor_order_on_each_spine() {
    const N: usize = 30;
    const M: usize = 5;
    let (module, seeds) = case(
        "spines",
        &["epoch", "other"],
        &[],
        &[
            format!("datePoint({})", after_n("epoch", N)),
            format!("datePoint({})", after_n("other", M)),
        ],
    );
    let bound = 2 * (N + 1) * (N + 1) + 2 * (M + 1) * (M + 1) + 10;
    let closure = close(&module, &seeds, bound);
    let spine = |r: &str, n: usize| -> BTreeSet<((String, i64), (String, i64))> {
        let mut expected = BTreeSet::new();
        for i in 0..=n as i64 {
            for j in i + 1..=n as i64 {
                expected.insert(((r.to_string(), i), (r.to_string(), j)));
            }
        }
        expected
    };
    let mut expected = spine("epoch", N);
    expected.extend(spine("other", M));
    assert_eq!(pairs(&closure, "before"), expected);
    let mut reflexive = expected.clone();
    for (r, n) in [("epoch", N), ("other", M)] {
        for i in 0..=n as i64 {
            reflexive.insert(((r.into(), i), (r.into(), i)));
        }
    }
    assert_eq!(pairs(&closure, "notAfter"), reflexive);
    // Dates on different roots are never ordered: incomparable, not false.
    assert!(pairs(&closure, "before").iter().all(|(a, b)| a.0 == b.0));
    // Termination: every date in the closure is a subterm of a seeded date.
    let points: BTreeSet<_> = closure
        .known
        .iter()
        .filter(|e| e.verb == "datePoint")
        .map(|e| date(&e.args[0]))
        .collect();
    assert_eq!(points.len(), (N + 1) + (M + 1));
    assert!(
        points
            .iter()
            .all(|(r, n)| (r == "epoch" && *n <= N as i64) || (r == "other" && *n <= M as i64))
    );
}

#[test]
fn majority_conditions_match_year_arithmetic_at_the_boundaries() {
    // Birth and dossier offsets on one spine, including d < b, d = b,
    // d = b + 17, d = b + 18 and d > b + 18.
    let births = [0usize, 3, 10];
    let offsets = [-2i64, -1, 0, 1, 17, 18, 19, 25];
    let mut checked = 0;
    for b in births {
        for off in offsets {
            let d = b as i64 + off;
            if d < 0 {
                continue;
            }
            let name = format!("b{b}_d{d}");
            let (module, seeds) = case(
                &name,
                &["epoch"],
                &["ana"],
                &[
                    format!("born(ana, {})", after_n("epoch", b)),
                    format!("dossierDate({})", after_n("epoch", d as usize)),
                ],
            );
            let closure = close(&module, &seeds, 20_000);
            let at = |verb| conditions(&closure, verb);
            let here: BTreeSet<_> = [("ana".to_string(), ("epoch".to_string(), d))].into();
            let (b, d) = (b as i64, d);
            let minor = b <= d && d < b + MAJORITY;
            let adult = d >= b + MAJORITY;
            let unborn = d < b;
            assert_eq!(at("minorAt") == here, minor, "{name} minor");
            assert_eq!(at("minorAt").is_empty(), !minor, "{name} minor");
            assert_eq!(at("adultAt") == here, adult, "{name} adult");
            assert_eq!(at("adultAt").is_empty(), !adult, "{name} adult");
            assert_eq!(at("bornAfterDossier") == here, unborn, "{name} unborn");
            assert_eq!(at("bornAfterDossier").is_empty(), !unborn, "{name} unborn");
            // Exactly one condition holds on a single spine.
            assert_eq!([minor, adult, unborn].iter().filter(|x| **x).count(), 1);
            checked += 1;
        }
    }
    assert_eq!(checked, 22);
}

#[test]
fn counterexamples_mark_the_limits_of_the_construction() {
    // 1. Different roots: no order, hence no condition. Indeterminate, not
    //    a negative answer.
    let (module, seeds) = case(
        "cross_root",
        &["epoch", "other"],
        &["ana"],
        &[
            format!("born(ana, {})", after_n("epoch", 2)),
            format!("dossierDate({})", after_n("other", 40)),
        ],
    );
    let closure = close(&module, &seeds, 20_000);
    for verb in ["minorAt", "adultAt", "bornAfterDossier"] {
        assert!(conditions(&closure, verb).is_empty(), "{verb}");
    }

    // 2. Contradictory births: monotone closure derives both conditions for
    //    the same person and date; the data are not repaired.
    let (module, seeds) = case(
        "two_births",
        &["epoch"],
        &["ana"],
        &[
            format!("born(ana, {})", after_n("epoch", 1)),
            format!("born(ana, {})", after_n("epoch", 20)),
            format!("dossierDate({})", after_n("epoch", 25)),
        ],
    );
    let closure = close(&module, &seeds, 20_000);
    let both: BTreeSet<_> = [("ana".to_string(), ("epoch".to_string(), 25))].into();
    assert_eq!(conditions(&closure, "minorAt"), both);
    assert_eq!(conditions(&closure, "adultAt"), both);

    // 3. Year granularity: two births in the same year are the same date term;
    //    the program cannot distinguish months. Expressed by identity of terms.
    assert_eq!(after_n("epoch", 7), after_n("epoch", 7));

    // 4. Ascending rule: the set of dates is unbounded and the closure exceeds
    //    any fixed bound (here 5,000 derivations).
    let ascending = load("counterexamples/ascending.res");
    let (_, seeds) = case(
        "ascending_seed",
        &["epoch"],
        &[],
        &[format!("datePoint({})", after_n("epoch", 1))],
    );
    let result = catch_unwind(AssertUnwindSafe(|| {
        close_program(seeds.clone(), &ascending, &mut Clock::new(5_000))
    }));
    assert!(
        result.is_err(),
        "the ascending closure stayed within the bound"
    );
}
