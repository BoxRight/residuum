//! Bounded observations over unchanged source ontologies, not new engine semantics.
use residuum::derive::{DerivationClock, instantiate_from_known};
use residuum::experiments::render_event;
use residuum::typed::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};

#[path = "legal_geometry_extreme_tests.rs"]
mod extremes;

#[path = "legal_geometry_frontier_tests.rs"]
mod frontiers;

#[path = "legal_geometry_residual_tests.rs"]
mod residual_comparison;

#[path = "legal_geometry_explicit_query_tests.rs"]
mod explicit_queries;

#[path = "legal_geometry_query_scope_tests.rs"]
mod query_dependencies;

#[path = "legal_geometry_residual_activation_tests.rs"]
mod residual_activation;

#[path = "legal_geometry_active_frontier_tests.rs"]
mod active_frontiers;

#[path = "legal_geometry_partition_tests.rs"]
mod partitions;

#[path = "legal_geometry_recalcitrance_tests.rs"]
mod recalcitrance;

#[path = "legal_geometry_conversation_tests.rs"]
mod conversation;

#[path = "legal_geometry_typed_explanation_tests.rs"]
mod typed_explanations;

#[path = "legal_geometry_riggs_tests.rs"]
mod riggs;

#[path = "legal_geometry_structural_generation_tests.rs"]
mod structural_generation;

struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("geometry-{}", self.0),
        }
    }
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn module(name: &str) -> Module {
    residuum::load_formal_module(
        root().join(format!("examples/experiments/legal_geometry/{name}.res")),
    )
    .unwrap()
}
fn experiment<'a>(m: &'a Module, name: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}
fn close(m: &Module, events: Vec<Event>) -> Closure {
    residuum::close_program(events, m, &mut Clock(0)).unwrap()
}
fn events(c: &Closure) -> BTreeSet<Event> {
    c.known.iter().cloned().collect()
}
fn proofs(c: &Closure) -> Vec<String> {
    let mut result: Vec<_> = c
        .derivations
        .iter()
        .map(|d| format!("{:?}\t{:?}", d.event, d.record))
        .collect();
    result.sort();
    result
}
fn full_events(c: &Closure) -> BTreeSet<String> {
    c.known.iter().map(|e| format!("{e:?}")).collect()
}
fn universe(m: &Module) -> Vec<Event> {
    let sigma: Vec<_> = m
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) if e.input_kind == ExperimentInputKind::Seeds => {
                Some(e.input.clone())
            }
            _ => None,
        })
        .flatten()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    close(m, sigma)
        .known
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn save(name: &str, text: &str) {
    let dir = root().join("target/legal-geometry");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}
fn atoms<'a>(a: &'a Antecedent, output: &mut Vec<&'a Event>) {
    match a {
        Antecedent::Unit => (),
        Antecedent::Event(e) => output.push(e),
        Antecedent::And(a, b) => {
            atoms(a, output);
            atoms(b, output);
        }
    }
}
fn literal(e: &Event) -> String {
    format!(
        "{}{}",
        if e.polarity == Polarity::EvidentialNot {
            "~"
        } else {
            ""
        },
        e.verb
    )
}
fn type_name(t: &Type) -> String {
    match t {
        Type::Entity(n) => n.clone(),
        Type::Set(t) | Type::Optional(t) => type_name(t),
        Type::PropositionTime => "PropositionTime".into(),
        Type::DerivationTime => "DerivationTime".into(),
    }
}
fn source_graph(path: &Path, visited: &mut BTreeSet<PathBuf>, output: &mut String) {
    let path = path.canonicalize().unwrap();
    if !visited.insert(path.clone()) {
        return;
    }
    let ast = residuum::parse_formal(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for d in ast.declarations {
        if let residuum::surface::FormalDeclaration::Import(relative) = d {
            let child = path
                .parent()
                .unwrap()
                .join(relative)
                .canonicalize()
                .unwrap();
            let child_ast =
                residuum::parse_formal(&std::fs::read_to_string(&child).unwrap()).unwrap();
            writeln!(
                output,
                "{}\t{}\t{}\t{}",
                ast.name,
                child_ast.name,
                path.display(),
                child.display()
            )
            .unwrap();
            source_graph(&child, visited, output);
        }
    }
}
fn export_ast(label: &str, m: &Module) {
    let mut inventory = String::from("kind\tname\torigin_module\torigin_file\ttyped_ast\n");
    let mut structure = String::from("source\ttarget\tfield\tdeclared_type\n");
    let mut edges = String::from("source\ttarget\tkind\trule\torigin\n");
    let mut signatures = String::from("verb\tcategory\targuments\teffect\tprogram\n");
    let mut effect_paths =
        String::from("verb\toperation\ttarget\tbase_type\ttarget_type\tvalue\ttransition\n");
    for d in &m.declarations {
        if matches!(d, Declaration::Experiment(_))
            || d.name().starts_with('g') && matches!(d, Declaration::Const(_))
        {
            continue;
        }
        let origin = m.origin(d.name()).unwrap();
        let kind = match d {
            Declaration::Entity(_) => "entity",
            Declaration::Const(_) => "constant",
            Declaration::Verb(_) => "verb",
            Declaration::Rule(_) => "rule",
            _ => panic!("unexpected ontology declaration"),
        };
        writeln!(
            inventory,
            "{kind}\t{}\t{}\t{}\t{d:?}",
            d.name(),
            origin.module,
            origin.file.as_ref().unwrap().display()
        )
        .unwrap();
        match d {
            Declaration::Entity(e) => {
                for f in &e.fields {
                    writeln!(
                        structure,
                        "{}\t{}\t{}\t{:?}",
                        e.name,
                        type_name(&f.ty),
                        f.name,
                        f.ty
                    )
                    .unwrap();
                }
                for parent in &e.supertypes {
                    writeln!(structure, "{}\t{parent}\t<:\tEntity", e.name).unwrap();
                }
            }
            Declaration::Verb(v) => {
                writeln!(
                    signatures,
                    "{}\t{:?}\t{:?}\t{:?}\t{:?}",
                    v.name, v.kind, v.args, v.effect, v.program
                )
                .unwrap();
                if let Some(Effect::StateTransform {
                    transition,
                    operations,
                }) = &v.effect
                {
                    for operation in operations {
                        let (kind, target, value) = match operation {
                            StateOperation::AddToSet { target, value } => ("insert", target, value),
                            StateOperation::RemoveFromSet { target, value } => {
                                ("remove", target, value)
                            }
                        };
                        let Term::Var(base) = &target.base else {
                            panic!("unexpected effect base")
                        };
                        let base_type = &v.args.iter().find(|a| &a.name == base).unwrap().ty;
                        let mut target_type = base_type.clone();
                        for field in &target.path {
                            let Type::Entity(name) = &target_type else {
                                panic!("unexpected intermediate path type")
                            };
                            let entity = m
                                .declarations
                                .iter()
                                .find_map(|d| match d {
                                    Declaration::Entity(e) if &e.name == name => Some(e),
                                    _ => None,
                                })
                                .unwrap();
                            target_type = entity
                                .fields
                                .iter()
                                .find(|f| &f.name == field)
                                .unwrap()
                                .ty
                                .clone();
                        }
                        writeln!(effect_paths,"{}\t{kind}\t{}.{}\t{base_type:?}\t{target_type:?}\t{value:?}\t{transition:?}",v.name,base,target.path.join(".")).unwrap();
                    }
                }
            }
            Declaration::Rule(r) => {
                let RuleBody::HornClause(c) = &r.body else {
                    panic!("ontology is Horn")
                };
                let mut premises = Vec::new();
                atoms(&c.antecedent, &mut premises);
                for p in premises {
                    writeln!(
                        edges,
                        "{}\trule:{}\tpremise\t{}\t{}",
                        literal(p),
                        r.name,
                        r.name,
                        origin.module
                    )
                    .unwrap();
                }
                writeln!(
                    edges,
                    "rule:{}\t{}\tconsequence\t{}\t{}",
                    r.name,
                    literal(&c.consequent),
                    r.name,
                    origin.module
                )
                .unwrap();
            }
            _ => (),
        }
    }
    save(&format!("{label}-inventory.tsv"), &inventory);
    save(&format!("{label}-structure.tsv"), &structure);
    save(&format!("{label}-inference.tsv"), &edges);
    save(&format!("{label}-verbs.tsv"), &signatures);
    save(&format!("{label}-effects.tsv"), &effect_paths);
}
fn audit_proofs(m: &Module, c: &Closure) {
    let known = events(c);
    for d in &c.derivations {
        assert!(d.record.antecedents.iter().all(|a| known.contains(a)));
        let rule = m
            .declarations
            .iter()
            .find_map(|x| match x {
                Declaration::Rule(r) if r.name == d.record.rule => Some(r),
                _ => None,
            })
            .unwrap();
        assert!(
            instantiate_from_known(rule, &d.record.antecedents)
                .iter()
                .any(|p| format!("{:?}", p.event) == format!("{:?}", d.event)
                    && p.record == d.record)
        );
        assert!(m.origin(&d.record.rule).is_some());
    }
}
#[test]
fn correspondence_and_typed_inventory() {
    let g = module("general");
    let p = module("patrimonial");
    let r = module("reference");
    let ontology = |m: &Module| -> BTreeMap<String, Declaration> {
        m.declarations
            .iter()
            .filter(|d| {
                !matches!(d, Declaration::Experiment(_))
                    && !(d.name().starts_with('g') && matches!(d, Declaration::Const(_)))
            })
            .map(|d| (d.name().to_owned(), d.clone()))
            .collect()
    };
    assert_eq!(ontology(&p), ontology(&r));
    for (name, d) in ontology(&g) {
        assert_eq!(ontology(&p).get(&name), Some(&d));
    }
    assert_eq!(ontology(&p).len() - ontology(&g).len(), 3);
    let rule_count = |m: &Module| {
        m.declarations
            .iter()
            .filter(|d| matches!(d, Declaration::Rule(_)))
            .count()
    };
    assert_eq!(rule_count(&g), 27);
    assert_eq!(rule_count(&p), 28);
    for d in &p.declarations {
        if matches!(d, Declaration::Experiment(_))
            || d.name().starts_with('g') && matches!(d, Declaration::Const(_))
        {
            continue;
        }
        assert_eq!(
            p.origin(d.name()).unwrap().module,
            if ["deliver", "molest", "complianceAsset"].contains(&d.name()) {
                "LegalPatrimonial"
            } else {
                "LegalGeneral"
            }
        );
        assert_eq!(r.origin(d.name()).unwrap().module, "LegalMonotonicCase");
    }
    for (name, m) in [("general", &g), ("patrimonial", &p), ("reference", &r)] {
        export_ast(name, m);
        let mut graph = String::from("importer\timported\timporter_file\timported_file\n");
        source_graph(
            &root().join(format!("examples/experiments/legal_geometry/{name}.res")),
            &mut BTreeSet::new(),
            &mut graph,
        );
        save(&format!("{name}-imports.tsv"), &graph);
    }
    println!(
        "28 typed rule bodies identical; origins checked separately; AST inventories exported"
    );
}
fn subset(input: &[Event], mask: usize) -> Vec<Event> {
    input
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, e)| e.clone())
        .collect()
}
fn study_window(name: &str) {
    let g = module("general");
    let p = module("patrimonial");
    let reference = module("reference");
    let input = &experiment(&g, name).input;
    let mut states = String::from("model\tmask\tinput\tclosure\tproofs\n");
    let mut supports = String::from("model\tevent\tminimal_mask\tsupport\n");
    let mut closed = String::from("model\tmask\tclosed_events\n");
    let mut inclusions = String::from("model\tlower_mask\tupper_mask\n");
    let mut stored = Vec::new();
    for mask in 0..1 << input.len() {
        let s = subset(input, mask);
        let cg = close(&g, s.clone());
        let cp = close(&p, s.clone());
        let cr = close(&reference, s.clone());
        assert_eq!(full_events(&cp), full_events(&cr));
        assert_eq!(proofs(&cp), proofs(&cr));
        assert!(events(&cg).is_subset(&events(&cp)));
        for (model, m, c) in [("general", &g, &cg), ("patrimonial", &p, &cp)] {
            assert!(s.iter().all(|e| c.known.contains(e)));
            assert_eq!(full_events(c), full_events(&close(m, c.known.clone())));
            audit_proofs(m, c);
            writeln!(
                states,
                "{model}\t{mask}\t{:?}\t{:?}\t{}",
                s.iter().map(render_event).collect::<Vec<_>>(),
                c.known.iter().map(render_event).collect::<BTreeSet<_>>(),
                c.derivations.len()
            )
            .unwrap();
            writeln!(closed, "{model}\t{mask}\t{:?}", events(c)).unwrap();
        }
        stored.push((events(&cg), events(&cp)));
    }
    for (label, column) in [("general", 0), ("patrimonial", 1)] {
        let get = |i: usize| {
            if column == 0 {
                &stored[i].0
            } else {
                &stored[i].1
            }
        };
        let universe: BTreeSet<_> = stored
            .iter()
            .flat_map(|s| if column == 0 { s.0.iter() } else { s.1.iter() })
            .cloned()
            .collect();
        for mask in 0..stored.len() {
            // Cover comparisons suffice for all comparable input pairs.
            for bit in 0..input.len() {
                let upper = mask | (1 << bit);
                assert!(get(mask).is_subset(get(upper)));
                if upper != mask {
                    writeln!(inclusions, "{label}\t{mask}\t{upper}").unwrap();
                }
            }
            // No source predicate is a rule head: the input projection is injective.
            let projection: BTreeSet<_> = get(mask)
                .intersection(&input.iter().cloned().collect())
                .cloned()
                .collect();
            assert_eq!(projection, subset(input, mask).into_iter().collect());
        }
        for event in universe {
            for mask in 0..stored.len() {
                if get(mask).contains(&event)
                    && (0..input.len())
                        .filter(|bit| mask & (1 << bit) != 0)
                        .all(|bit| !get(mask & !(1 << bit)).contains(&event))
                {
                    writeln!(
                        supports,
                        "{label}\t{}\t{mask}\t{:?}",
                        render_event(&event),
                        subset(input, mask)
                            .iter()
                            .map(render_event)
                            .collect::<Vec<_>>()
                    )
                    .unwrap();
                }
            }
        }
    }
    save(&format!("{name}-states.tsv"), &states);
    save(&format!("{name}-frontiers.tsv"), &supports);
    save(&format!("{name}-closed.tsv"), &closed);
    save(&format!("{name}-inclusions.tsv"), &inclusions);
    let mut provenance =
        String::from("model\tevent\trule_module\trule_file\tordered_witness_and_substitution\n");
    for (label, m) in [
        ("general", &g),
        ("patrimonial", &p),
        ("reference", &reference),
    ] {
        for d in close(m, input.clone()).derivations {
            let o = m.origin(&d.record.rule).unwrap();
            writeln!(
                provenance,
                "{label}\t{:?}\t{}\t{}\t{:?}",
                d.event,
                o.module,
                o.file.as_ref().unwrap().display(),
                d.record
            )
            .unwrap();
        }
    }
    save(&format!("{name}-provenance.tsv"), &provenance);
    println!(
        "{name}: {} subsets, 2 closure operators, reference equality including proof multiplicity; extensivity/idempotence/inclusion covers passed",
        stored.len()
    );
}
#[test]
fn performance_regions_and_closures() {
    study_window("performance");
}
#[test]
fn authority_regions_and_closures() {
    study_window("authority");
}

#[test]
fn legislation_transfer_and_evidential_boundaries() {
    study_window("legislation");
    study_window("transferCase");
    study_window("holding");
    let g = module("general");
    let mut result = String::new();
    for name in ["otherTime", "otherAuthority"] {
        let x = experiment(&g, name);
        let c = close(&g, x.input.clone());
        let has = |v: &str| c.known.iter().any(|e| e.verb == v);
        assert!(!has("inconsistent"));
        assert_eq!(has("violation"), name == "otherAuthority");
        writeln!(result, "{name}\t{:?}", c.known).unwrap();
    }
    let c = close(&g, experiment(&g, "transferCase").input.clone());
    let give = c.known.iter().find(|e| e.verb == "give").unwrap();
    assert_eq!(
        give.proposition.time,
        TimeExpr::After(Box::new(TimeExpr::At("gTau".into())))
    );
    assert_eq!(
        give.transition,
        Some(StateTransitionType {
            input: StateTime::At(TimeExpr::At("gTau".into())),
            output: StateTime::At(give.proposition.time.clone())
        })
    );
    assert_eq!(c.known.iter().filter(|e| e.verb == "Act").count(), 1);
    let pair = experiment(&g, "performance").input[5..].to_vec();
    let pair_closure = close(&g, pair);
    assert!(pair_closure.known.iter().any(|e| e.verb == "inconsistent"));
    assert!(
        !pair_closure
            .known
            .iter()
            .any(|e| e.verb == "violation" || e.verb == "law")
    );
    let extinction = close(&g, experiment(&g, "extinction").input.clone());
    assert!(extinction.known.iter().any(|e| e.verb == "extinguished"));
    assert!(extinction.known.iter().any(|e| e.verb == "compliance"));
    save("boundaries.tsv", &result);
    println!(
        "matched times, exact evidential opposition, terminal State transition and no explosion checked"
    );
}

#[test]
fn common_universe_quotients_and_lattice_counterexamples() {
    let g = module("general");
    let p = module("patrimonial");
    let sigma: Vec<Event> = g
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) if e.input_kind == ExperimentInputKind::Seeds => {
                Some(e.input.clone())
            }
            _ => None,
        })
        .flatten()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let cg = close(&g, sigma.clone());
    let cp = close(&p, sigma.clone());
    let reference = module("reference");
    let cr = close(&reference, sigma.clone());
    assert_eq!(full_events(&cp), full_events(&cr));
    assert_eq!(proofs(&cp), proofs(&cr));
    let e = events(&cp);
    assert_eq!(events(&close(&g, e.iter().cloned().collect())), e);
    assert_eq!(events(&close(&p, e.iter().cloned().collect())), e);
    let mut universe = String::from("id\tevent\ttyped_event\n");
    let mut initial = String::from("id\tevent\n");
    for (i, x) in e.iter().enumerate() {
        writeln!(universe, "E{i:03}\t{}\t{x:?}", render_event(x)).unwrap();
        if sigma.contains(x) {
            writeln!(initial, "E{i:03}\t{}", render_event(x)).unwrap();
        }
    }
    save("universe.tsv", &universe);
    save("initial-evidence.tsv", &initial);
    let mut provenance = String::from(
        "model\tevent\tderived_at\trule_module\trule_file\tordered_witness_and_substitution\n",
    );
    for (label, m, c) in [
        ("general", &g, &cg),
        ("patrimonial", &p, &cp),
        ("reference", &reference, &cr),
    ] {
        audit_proofs(m, c);
        let used: BTreeSet<_> = c
            .derivations
            .iter()
            .map(|d| d.record.rule.as_str())
            .collect();
        let declared: BTreeSet<_> = m
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) => Some(r.name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            used, declared,
            "every ontology rule has a real witness in the global synthetic state"
        );
        for d in &c.derivations {
            let origin = m.origin(&d.record.rule).unwrap();
            writeln!(
                provenance,
                "{label}\t{:?}\t{:?}\t{}\t{}\t{:?}",
                d.event,
                d.derived_at,
                origin.module,
                origin.file.as_ref().unwrap().display(),
                d.record
            )
            .unwrap();
        }
    }
    save("whole-universe-provenance.tsv", &provenance);
    let perf = &experiment(&g, "performance").input;
    let asset = experiment(&g, "theoryAsset").input[0].clone();
    let compliance = experiment(&g, "theoryCompliance").input[0].clone();
    let obligation = experiment(&g, "theoryObligation").input[0].clone();
    let s = vec![perf[0].clone(), perf[5].clone()];
    let mut t = s.clone();
    t.push(asset.clone());
    assert_ne!(events(&close(&g, s.clone())), events(&close(&g, t.clone())));
    assert_eq!(events(&close(&p, s.clone())), events(&close(&p, t.clone())));
    // Closure-equivalent but distinct states, even without the extension.
    let sg = close(&g, s.clone());
    let mut sc = s.clone();
    sc.push(compliance.clone());
    assert_eq!(events(&sg), events(&close(&g, sc)));
    let mut report = String::from("example\tmodel\tleft\tright\n");
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let x = events(&close(m, vec![compliance.clone()]));
        let y = events(&close(m, vec![perf[0].clone()]));
        let z = events(&close(m, vec![perf[5].clone()]));
        let join = |a: &BTreeSet<Event>, b: &BTreeSet<Event>| {
            events(&close(m, a.union(b).cloned().collect()))
        };
        let meet = |a: &BTreeSet<Event>, b: &BTreeSet<Event>| {
            a.intersection(b).cloned().collect::<BTreeSet<_>>()
        };
        let lhs = meet(&x, &join(&y, &z));
        let rhs = join(&meet(&x, &y), &meet(&x, &z));
        assert_ne!(lhs, rhs);
        writeln!(
            report,
            "nondistributive_closed_lattice\t{label}\t{lhs:?}\t{rhs:?}"
        )
        .unwrap();
        assert_ne!(join(&y, &z), y.union(&z).cloned().collect());
        let o = events(&close(m, vec![obligation.clone()]));
        assert!(!meet(&y, &o).is_empty());
        assert!(close(m, Vec::new()).known.is_empty());
        let ca = events(&close(m, s.clone()));
        let ra = events(&close(m, vec![perf[1].clone(), perf[5].clone()]));
        assert!(ca.contains(&compliance) && ra.contains(&compliance));
        assert!(!close(m, vec![perf[5].clone()]).known.contains(&compliance));
    }
    // Enumerate a theory-state window, not just admissible source seeds.
    let u = vec![perf[0].clone(), perf[5].clone(), compliance, asset];
    let mut gs = Vec::new();
    let mut ps = Vec::new();
    let mut states = String::from("mask\tinput\tgeneral\tpatrimonial\n");
    for mask in 0..16 {
        let s = subset(&u, mask);
        let cg = close(&g, s.clone());
        let cp = close(&p, s.clone());
        assert_eq!(events(&close(&p, cg.known.clone())), events(&cp));
        assert_eq!(events(&close(&g, cp.known.clone())), events(&cp));
        writeln!(
            states,
            "{mask}\t{:?}\t{:?}\t{:?}",
            s,
            events(&cg),
            events(&cp)
        )
        .unwrap();
        gs.push(events(&cg));
        ps.push(events(&cp));
    }
    for i in 0..16 {
        for j in 0..16 {
            if gs[i] == gs[j] {
                assert_eq!(ps[i], ps[j]);
            }
        }
    }
    let count = |sets: &Vec<BTreeSet<Event>>| sets.iter().cloned().collect::<BTreeSet<_>>().len();
    writeln!(
        report,
        "quotient_classes\tgeneral\t{}\tpatrimonial={} (16 theory states)",
        count(&gs),
        count(&ps)
    )
    .unwrap();
    writeln!(
        report,
        "universe\tcommon\t{}\tclosed in both ontologies",
        e.len()
    )
    .unwrap();
    save("counterexamples.tsv", &report);
    save("theory-states.tsv", &states);
    println!(
        "common E={}, general classes={}, patrimonial classes={}; counterexamples and absorption checked",
        e.len(),
        count(&gs),
        count(&ps)
    );
}

// Minimal-support analysis of the finite, grounded hypergraph. Identity supports
// allow any Event already present in K, including Derived theory snapshots.
// This is an analysis algorithm; it neither adds rules nor changes close.
type Supports = BTreeSet<BTreeSet<usize>>;
fn insert_minimal(into: &mut Supports, support: BTreeSet<usize>) {
    if into.iter().any(|s| s.is_subset(&support)) {
        return;
    }
    into.retain(|s| !support.is_subset(s));
    into.insert(support);
}

fn finite_frontiers(m: &Module, e: &[Event], allowed: &BTreeSet<usize>) -> (usize, Vec<Supports>) {
    assert_eq!(e.len(), 73, "refuse enlargement of the bounded study");
    let indices: BTreeMap<_, _> = e.iter().cloned().enumerate().map(|(i, e)| (e, i)).collect();
    let ground: Vec<(usize, Vec<usize>)> = m
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(instantiate_from_known(r, e)),
            _ => None,
        })
        .flatten()
        .map(|d| {
            (
                indices[&d.event],
                d.record.antecedents.iter().map(|a| indices[a]).collect(),
            )
        })
        .collect();
    let mut supports: Vec<Supports> = (0..e.len())
        .map(|i| {
            if allowed.contains(&i) {
                Supports::from([BTreeSet::from([i])])
            } else {
                Supports::new()
            }
        })
        .collect();
    loop {
        let before = supports.clone();
        for (head, premises) in &ground {
            let mut candidates = Supports::from([BTreeSet::new()]);
            for premise in premises {
                let mut combined = Supports::new();
                for a in &candidates {
                    for b in &supports[*premise] {
                        insert_minimal(&mut combined, a.union(b).copied().collect());
                    }
                }
                candidates = combined;
            }
            for candidate in candidates {
                insert_minimal(&mut supports[*head], candidate);
            }
        }
        if supports == before {
            break;
        }
    }
    (ground.len(), supports)
}
#[test]
fn full_event_frontiers_without_powerset_enumeration() {
    let g = module("general");
    let p = module("patrimonial");
    let e = universe(&p);
    let mut output = String::from("model\tconclusion\tminimal_support_ids\tsupport\n");
    let mut totals = String::from("model\tground_clauses\tfrontier_supports\tverified_supports\n");
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        let (ground_count, supports) = finite_frontiers(m, &e, &(0..e.len()).collect());
        let mut checked = 0;
        for (i, frontier) in supports.iter().enumerate() {
            for s in frontier {
                let input: Vec<_> = s.iter().map(|i| e[*i].clone()).collect();
                assert!(close(m, input.clone()).known.contains(&e[i]));
                for bit in s {
                    let reduced: Vec<_> = s
                        .iter()
                        .filter(|i| *i != bit)
                        .map(|i| e[*i].clone())
                        .collect();
                    assert!(!close(m, reduced).known.contains(&e[i]));
                }
                let ids: Vec<_> = s.iter().map(|i| format!("E{i:03}")).collect();
                writeln!(
                    output,
                    "{label}\t{}\t{ids:?}\t{:?}",
                    render_event(&e[i]),
                    input.iter().map(render_event).collect::<Vec<_>>()
                )
                .unwrap();
                checked += 1;
            }
        }
        writeln!(
            totals,
            "{label}\t{}\t{}\t{checked}",
            ground_count,
            supports.iter().map(BTreeSet::len).sum::<usize>()
        )
        .unwrap();
        println!(
            "{label}: {} ground clauses, {checked} full-E minimal supports verified by real closure",
            ground_count
        );
    }
    save("full-event-frontiers.tsv", &output);
    save("full-event-frontier-totals.tsv", &totals);
}

#[test]
fn anti_exchange_on_controlled_theory_states() {
    let g = module("general");
    let p = module("patrimonial");
    let e = universe(&p);
    let performance = &experiment(&g, "performance").input;
    let u = vec![
        performance[0].clone(),
        performance[5].clone(),
        experiment(&g, "theoryCompliance").input[0].clone(),
        experiment(&g, "theoryAsset").input[0].clone(),
    ];
    let mut report =
        String::from("model\tbase_states\tordered_pairs_checked\tnew_derivability_implications\n");
    let mut order_report = String::from("model\trank\tliteral\n");
    for (label, m) in [("general", &g), ("patrimonial", &p)] {
        assert!(close(m, Vec::new()).known.is_empty());
        let mut nodes = BTreeSet::new();
        let mut edges = BTreeSet::new();
        for d in &m.declarations {
            match d {
                Declaration::Verb(v) => {
                    assert!(v.program.is_none());
                    nodes.insert(v.name.clone());
                }
                Declaration::Rule(r) => {
                    let RuleBody::HornClause(c) = &r.body else {
                        panic!("not ordinary Horn")
                    };
                    let mut premises = Vec::new();
                    atoms(&c.antecedent, &mut premises);
                    assert!(!premises.is_empty());
                    for premise in premises {
                        let a = literal(premise);
                        let b = literal(&c.consequent);
                        nodes.insert(a.clone());
                        nodes.insert(b.clone());
                        edges.insert((a, b));
                    }
                }
                Declaration::Program(_) | Declaration::Default(_) => {
                    panic!("unexpected dynamic rule source")
                }
                _ => (),
            }
        }
        let mut rank = 0;
        while !nodes.is_empty() {
            let ready: Vec<_> = nodes
                .iter()
                .filter(|n| !edges.iter().any(|(a, b)| b == *n && nodes.contains(a)))
                .cloned()
                .collect();
            assert!(!ready.is_empty(), "cyclic inferential dependencies");
            for node in ready {
                writeln!(order_report, "{label}\t{rank}\t{node}").unwrap();
                nodes.remove(&node);
                rank += 1;
            }
        }
        let mut pairs = 0;
        let mut implications = 0;
        for mask in 0..16 {
            let s = subset(&u, mask);
            let base = events(&close(m, s.clone()));
            let extensions: BTreeMap<Event, BTreeSet<Event>> = e
                .iter()
                .filter(|x| !base.contains(*x))
                .map(|x| {
                    let mut input = s.clone();
                    input.push(x.clone());
                    (x.clone(), events(&close(m, input)))
                })
                .collect();
            for (y, cy) in &extensions {
                for (x, cx) in &extensions {
                    if x == y {
                        continue;
                    }
                    pairs += 1;
                    if cy.contains(x) {
                        implications += 1;
                        assert!(
                            !cx.contains(y),
                            "anti-exchange counterexample: {s:?}, x={x:?}, y={y:?}"
                        );
                    }
                }
            }
        }
        writeln!(report, "{label}\t16\t{pairs}\t{implications}").unwrap();
        println!("{label}: anti-exchange checked on 16 theory bases and {pairs} ordered pairs");
    }
    save("anti-exchange.tsv", &report);
    save("predicate-order.tsv", &order_report);
}

#[path = "legal_geometry_region_revision_tests.rs"]
mod region_revision;
