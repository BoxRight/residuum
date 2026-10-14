//! Finite hypothetical revisions, with external labels and unchanged ontology.
use super::*;

const SYMBOLS: [&str; 6] = ["O", "R", "A", "P", "M", "D"];

#[path = "legal_geometry_revision_dependency_tests.rs"]
mod dependencies;

struct Study {
    module: Module,
    catalog: Vec<Event>,
    goal: Event,
    labels: Vec<(String, usize, bool)>,
    universe: Vec<Event>,
    clauses: Vec<(usize, usize)>,
}

fn mask_of(input: &[Event], domain: &[Event]) -> usize {
    input.iter().fold(0, |mask, e| {
        mask | (1 << domain.iter().position(|p| p == e).expect("closed universe"))
    })
}

fn label(mask: usize) -> String {
    let out: String = SYMBOLS
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, s)| *s)
        .collect();
    if out.is_empty() { "empty".into() } else { out }
}

// Independent bitset saturation: no residual routines or forward evaluator.
fn saturate(mut input: usize, clauses: &[(usize, usize)]) -> usize {
    loop {
        let before = input;
        for &(body, head) in clauses {
            if body & input == body {
                input |= head;
            }
        }
        if before == input {
            return input;
        }
    }
}

impl Study {
    fn new() -> Self {
        let module = module("region_revision");
        let catalog = experiment(&module, "catalog").input.clone();
        let goal = experiment(&module, "catalog").goal.clone();
        assert_eq!(catalog.len(), 6);
        assert_eq!(
            catalog.iter().map(|e| e.verb.as_str()).collect::<Vec<_>>(),
            [
                "obligationFiled",
                "right",
                "Act",
                "permissionFiled",
                "omissionProhibition",
                "disputedPerformance"
            ]
        );
        let text = std::fs::read_to_string(
            root().join("examples/experiments/legal_geometry/region_revision_labels.tsv"),
        )
        .unwrap();
        let labels = text
            .lines()
            .filter(|l| !l.starts_with('#'))
            .skip(1)
            .map(|line| {
                let (name, value) = line.split_once('\t').unwrap();
                let ex = experiment(&module, name);
                assert_eq!(ex.goal, goal);
                (
                    name.into(),
                    mask_of(&ex.input, &catalog),
                    match value {
                        "1" => true,
                        "0" => false,
                        _ => panic!("external Boolean label"),
                    },
                )
            })
            .collect();
        let mut input = catalog.clone();
        input.extend(experiment(&module, "theoryViolation").input.clone());
        let universe: Vec<_> = events(&close(&module, input)).into_iter().collect();
        assert!(universe.len() <= 14, "explicit exponential study bound");
        assert!(universe.contains(&goal));
        let clauses: Vec<_> = module
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Rule(r) => Some(instantiate_from_known(r, &universe)),
                _ => None,
            })
            .flatten()
            .map(|d| {
                (
                    mask_of(&d.record.antecedents, &universe),
                    mask_of(&[d.event], &universe),
                )
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        assert_eq!(clauses.len(), 7);
        Self {
            module,
            catalog,
            goal,
            labels,
            universe,
            clauses,
        }
    }

    fn projected(&self, mask: usize) -> usize {
        mask_of(&subset(&self.catalog, mask), &self.universe)
    }

    fn model_clauses(&self, package: &[usize]) -> Vec<(usize, usize)> {
        let mut clauses = self.clauses.clone();
        let head = mask_of(std::slice::from_ref(&self.goal), &self.universe);
        clauses.extend(package.iter().map(|&h| (self.projected(h), head)));
        clauses
    }

    fn has_goal(&self, closed: usize) -> bool {
        closed & mask_of(std::slice::from_ref(&self.goal), &self.universe) != 0
    }

    fn region(&self, package: &[usize]) -> BTreeSet<usize> {
        let clauses = self.model_clauses(package);
        (0..64)
            .filter(|&s| self.has_goal(saturate(self.projected(s), &clauses)))
            .collect()
    }

    fn admissible(&self) -> Vec<usize> {
        let baseline = self.region(&[]);
        (0..64)
            .filter(|&h| {
                self.labels
                    .iter()
                    .any(|(_, s, positive)| *positive && !baseline.contains(s) && h & s == h)
                    && self
                        .labels
                        .iter()
                        .all(|(_, s, positive)| *positive || h & s != h)
            })
            .collect()
    }

    fn compatible(&self, package: &[usize]) -> bool {
        let region = self.region(package);
        self.labels
            .iter()
            .all(|(_, s, positive)| region.contains(s) == *positive)
    }

    fn hypothetical(&self, package: &[usize]) -> Module {
        let mut module = self.module.clone();
        for &h in package {
            let name = format!("hypotheticalCompliance{}", label(h));
            let mut values = subset(&self.catalog, h).into_iter().map(Antecedent::Event);
            let antecedent = values
                .next()
                .map(|first| values.fold(first, |a, b| Antecedent::And(Box::new(a), Box::new(b))))
                .unwrap_or(Antecedent::Unit);
            module.declarations.push(Declaration::Rule(Rule {
                name: name.clone(),
                parameters: Vec::new(),
                body: RuleBody::HornClause(HornClause {
                    antecedent,
                    consequent: self.goal.clone(),
                }),
            }));
            module.origins.insert(
                name,
                DeclarationOrigin {
                    module: "HypotheticalComplianceRevision".into(),
                    file: None,
                },
            );
        }
        module
    }
}

fn packages(candidates: &[usize]) -> Vec<Vec<usize>> {
    (0..1 << candidates.len())
        .map(|m| {
            candidates
                .iter()
                .enumerate()
                .filter(|(i, _)| m & (1 << i) != 0)
                .map(|(_, h)| *h)
                .collect()
        })
        .collect()
}

fn package_name(package: &[usize]) -> String {
    if package.is_empty() {
        "baseline".into()
    } else {
        package
            .iter()
            .map(|&h| label(h))
            .collect::<Vec<_>>()
            .join("+")
    }
}

fn minimal_packages(study: &Study) -> Vec<Vec<usize>> {
    packages(&study.admissible())
        .into_iter()
        .filter(|p| {
            study.compatible(p)
                && (0..p.len()).all(|i| {
                    let mut less = p.clone();
                    less.remove(i);
                    !study.compatible(&less)
                })
        })
        .collect()
}

fn frontier(region: &BTreeSet<usize>) -> Vec<usize> {
    region
        .iter()
        .copied()
        .filter(|s| !region.iter().any(|t| t != s && t & s == *t))
        .collect()
}

#[test]
fn externally_labelled_compliance_revisions_enumerate_all_minimal_extensions() {
    let study = Study::new();
    let baseline = study.region(&[]);
    assert_eq!(
        frontier(&baseline)
            .iter()
            .map(|&h| label(h))
            .collect::<Vec<_>>(),
        ["OA", "RA"]
    );
    assert_eq!(
        study
            .labels
            .iter()
            .filter(|(_, s, p)| *p && !baseline.contains(s))
            .count(),
        2
    );
    assert!(
        study
            .labels
            .iter()
            .filter(|(_, _, p)| !p)
            .all(|(_, s, _)| !baseline.contains(s))
    );
    let candidates = study.admissible();
    assert_eq!(
        candidates
            .iter()
            .map(|&h| label(h))
            .collect::<BTreeSet<_>>(),
        ["AP", "AM", "APM", "AD", "APD"]
            .into_iter()
            .map(String::from)
            .collect()
    );
    let all = packages(&candidates);
    let compatible: Vec<_> = all.iter().filter(|p| study.compatible(p)).collect();
    let minimal = minimal_packages(&study);
    assert_eq!(minimal.len(), 5);
    let fewest = compatible.iter().map(|p| p.len()).min().unwrap();
    assert_eq!(fewest, 1);
    let mut summary = String::from(
        "package\trules\tcompatible\trule_deletion_minimal\tminimum_rule_count\tcoverage_minimal\tadded_cases\tfrontier\n",
    );
    let additions: Vec<BTreeSet<usize>> = all
        .iter()
        .map(|p| study.region(p).difference(&baseline).copied().collect())
        .collect();
    let mut coverage_minimal = BTreeSet::new();
    for (i, p) in all.iter().enumerate() {
        let valid = study.compatible(p);
        let cover_min = valid
            && !all.iter().enumerate().any(|(j, q)| {
                study.compatible(q)
                    && additions[j] != additions[i]
                    && additions[j].is_subset(&additions[i])
            });
        if cover_min {
            coverage_minimal.insert(additions[i].clone());
        }
        writeln!(
            summary,
            "{}\t{}\t{valid}\t{}\t{}\t{cover_min}\t{}\t{}",
            package_name(p),
            p.len(),
            minimal.contains(p),
            valid && p.len() == fewest,
            additions[i].len(),
            frontier(&study.region(p))
                .iter()
                .map(|&h| label(h))
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    assert_eq!(coverage_minimal.len(), 1);
    let mut supports = String::from("support\tsupport_inclusion_minimal\tcone\tnew_cone\n");
    let mut rules = String::from(
        "// Hypothetical ground rules only; NOT approved normative revisions.\nmodule HypotheticalComplianceRevision\nimport \"../../examples/experiments/legal_geometry/general.res\"\n\n",
    );
    // All 64 supports are generated; these five alone contribute to positive
    // observations without capturing any negative observation.
    for &h in &candidates {
        let cone: Vec<_> = (0..64).filter(|s| h & s == h).collect();
        let extra: BTreeSet<_> = cone
            .iter()
            .copied()
            .filter(|s| !baseline.contains(s))
            .collect();
        assert_eq!(
            extra,
            study.region(&[h]).difference(&baseline).copied().collect()
        );
        let minimal_support = !candidates.iter().any(|&g| g != h && g & h == g);
        writeln!(
            supports,
            "{}\t{minimal_support}\t{}\t{}",
            label(h),
            cone.len(),
            extra.len()
        )
        .unwrap();
        writeln!(
            rules,
            "rule hypotheticalCompliance{}() =\n    {}\n    <= {}\n",
            label(h),
            subset(&study.catalog, h)
                .iter()
                .map(render_event)
                .collect::<Vec<_>>()
                .join("\n    & "),
            render_event(&study.goal)
        )
        .unwrap();
    }
    let mut distinguish = String::from("left\tright\tcase\tleft_derives\tright_derives\n");
    for (i, p) in minimal.iter().enumerate() {
        for q in &minimal[i + 1..] {
            let pr = study.region(p);
            let qr = study.region(q);
            let diff: BTreeSet<_> = pr.symmetric_difference(&qr).copied().collect();
            assert!(!diff.is_empty());
            for s in frontier(&diff) {
                assert!(!study.labels.iter().any(|(_, known, _)| *known == s));
                writeln!(
                    distinguish,
                    "{}\t{}\t{}\t{}\t{}",
                    package_name(p),
                    package_name(q),
                    label(s),
                    pr.contains(&s),
                    qr.contains(&s)
                )
                .unwrap();
            }
        }
    }
    let mut cones = String::from("case\tmask\tbaseline\tAP\tAM\tAPM\tAD\tAPD\n");
    for s in 0..64 {
        write!(cones, "{}\t{s}\t{}", label(s), baseline.contains(&s)).unwrap();
        for &h in &candidates {
            write!(cones, "\t{}", h & s == h).unwrap();
        }
        writeln!(cones).unwrap();
    }
    save("revision-supports.tsv", &supports);
    save("revision-packages.tsv", &summary);
    save("revision-distinguishing-cases.tsv", &distinguish);
    save("revision-cones.tsv", &cones);
    save("hypothetical-compliance-supports.res", &rules);
    // Round-trip the generated ground hypotheses through the real frontend;
    // the experiment does not bypass type checking to manufacture a rule.
    let parsed = residuum::load_formal_module(
        root().join("target/legal-geometry/hypothetical-compliance-supports.res"),
    )
    .unwrap();
    for &h in &candidates {
        let expected = study.hypothetical(&[h]);
        let rule_name = format!("hypotheticalCompliance{}", label(h));
        let find = |m: &Module| {
            m.declarations
                .iter()
                .find(|d| d.name() == rule_name)
                .unwrap()
                .clone()
        };
        assert_eq!(find(&expected), find(&parsed));
    }
    println!(
        "catalog=6, states=64, labels={}, candidates=5, packages=32, compatible={}, deletion-minimal=5, coverage-minimal regions=1",
        study.labels.len(),
        compatible.len()
    );
}

#[test]
fn hypothetical_compliance_revisions_preserve_independent_closures_and_provenance() {
    let study = Study::new();
    let original = study.module.clone();
    let mut models = minimal_packages(&study);
    models.insert(0, vec![]);
    let mut proof_report = String::from("package\tinput\tevent\trecord\torigin\n");
    let mut closure_report = String::from("package\tinput\tgoal\tclosed_events\tjustifications\n");
    for p in &models {
        let hypothetical = study.hypothetical(p);
        let before = hypothetical.clone();
        let clauses = study.model_clauses(p);
        for s in 0..64 {
            let input = subset(&study.catalog, s);
            let kept = input.clone();
            let reference = residuum::derive::close_program_reference(
                input.clone(),
                &hypothetical,
                &mut Clock(1000),
            )
            .unwrap();
            let actual = close(&hypothetical, input.clone());
            assert_eq!(input, kept);
            assert_eq!(full_events(&reference), full_events(&actual));
            assert_eq!(proofs(&reference), proofs(&actual));
            assert_eq!(
                mask_of(&actual.known, &study.universe),
                saturate(study.projected(s), &clauses)
            );
            audit_proofs(&hypothetical, &actual);
            for d in &actual.derivations {
                let origin = hypothetical.origin(&d.record.rule).unwrap();
                if d.record.rule.starts_with("hypotheticalCompliance") {
                    assert_eq!(origin.module, "HypotheticalComplianceRevision");
                    assert_eq!(d.event, study.goal);
                } else {
                    assert_eq!(Some(origin), original.origin(&d.record.rule));
                }
                writeln!(
                    proof_report,
                    "{}\t{}\t{:?}\t{:?}\t{:?}",
                    package_name(p),
                    label(s),
                    d.event,
                    d.record,
                    origin
                )
                .unwrap();
            }
            writeln!(
                closure_report,
                "{}\t{}\t{}\t{}\t{}",
                package_name(p),
                label(s),
                actual.known.contains(&study.goal),
                actual.known.len(),
                actual.derivations.len()
            )
            .unwrap();
        }
        assert_eq!(hypothetical, before);
    }
    assert_eq!(study.module, original);
    // Each original declaration remains identical and in its original order.
    save("revision-provenance.tsv", &proof_report);
    save("revision-engine-closures.tsv", &closure_report);
    println!(
        "384 states: independent bitset/reference/incremental closures and ordered provenance agree; original module and inputs unchanged"
    );
}

fn closure_table(clauses: &[(usize, usize)], n: usize) -> Vec<usize> {
    (0..1 << n).map(|s| saturate(s, clauses)).collect()
}

fn anti_exchange_witness(table: &[usize], n: usize) -> Option<(usize, usize, usize)> {
    let mut closed: Vec<_> = (0..table.len()).filter(|&s| table[s] == s).collect();
    closed.sort_by_key(|s| (s.count_ones(), *s));
    for f in closed {
        for x in 0..n {
            for y in x + 1..n {
                let (xb, yb) = (1 << x, 1 << y);
                if f & (xb | yb) == 0 && table[f | yb] & xb != 0 && table[f | xb] & yb != 0 {
                    return Some((f, x, y));
                }
            }
        }
    }
    None
}

#[test]
fn hypothetical_compliance_revisions_check_convex_geometry_and_counterexamples() {
    let study = Study::new();
    let candidates = study.admissible();
    let mut models = packages(&candidates); // check even redundant/nonminimal packages
    models.retain(|p| study.compatible(p));
    models.insert(0, vec![]);
    let n = study.universe.len();
    let mut report = String::from(
        "package\tuniverse\ttheory_states\tclosed_states\textensive\tmonotone\tidempotent\tempty_closed\tanti_exchange\tunique_extreme_basis\n",
    );
    let mut counterexamples = String::from(
        "package\tclosed_F\tx\ty\tclosure_F_x\tclosure_F_y\tminimal_bases_of_common_closure\n",
    );
    let mut anti_results = BTreeMap::new();
    for p in &models {
        let table = closure_table(&study.model_clauses(p), n);
        assert_eq!(table[0], 0);
        for (s, &f) in table.iter().enumerate() {
            assert_eq!(s & f, s);
            assert_eq!(table[f], f);
            for i in 0..n {
                assert_eq!(f & table[s | (1 << i)], f);
            }
        }
        let closed: Vec<_> = (0..table.len()).filter(|&s| table[s] == s).collect();
        let extreme_generates = closed.iter().all(|&f| {
            let ex = (0..n)
                .filter(|&i| f & (1 << i) != 0 && table[f & !(1 << i)] & (1 << i) == 0)
                .fold(0, |ex, i| ex | (1 << i));
            table[ex] == f
        });
        let witness = anti_exchange_witness(&table, n);
        assert_eq!(extreme_generates, witness.is_none());
        if let Some((f, x, y)) = witness {
            let hypothetical = study.hypothetical(p);
            let fx = close(&hypothetical, subset(&study.universe, f | (1 << x)));
            let fy = close(&hypothetical, subset(&study.universe, f | (1 << y)));
            assert!(fx.known.contains(&study.universe[y]));
            assert!(fy.known.contains(&study.universe[x]));
            assert_eq!(mask_of(&fx.known, &study.universe), table[f | (1 << x)]);
            assert_eq!(mask_of(&fy.known, &study.universe), table[f | (1 << y)]);
            audit_proofs(&hypothetical, &fx);
            audit_proofs(&hypothetical, &fy);
            let common = table[f | (1 << x)];
            assert_eq!(common, table[f | (1 << y)]);
            let generators: BTreeSet<_> =
                (0..table.len()).filter(|&s| table[s] == common).collect();
            let bases = frontier(&generators);
            assert!(bases.len() >= 2);
            writeln!(
                counterexamples,
                "{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
                package_name(p),
                subset(&study.universe, f),
                study.universe[x],
                study.universe[y],
                events(&fx),
                events(&fy),
                bases
                    .iter()
                    .map(|&s| subset(&study.universe, s))
                    .collect::<Vec<_>>()
            )
            .unwrap();
        }
        anti_results.insert(package_name(p), witness.is_none());
        writeln!(
            report,
            "{}\t{n}\t{}\t{}\ttrue\ttrue\ttrue\ttrue\t{}\t{extreme_generates}",
            package_name(p),
            table.len(),
            closed.len(),
            witness.is_none()
        )
        .unwrap();
    }
    assert_eq!(anti_results["baseline"], true);
    let minimal = minimal_packages(&study);
    assert_eq!(
        minimal
            .iter()
            .filter(|p| anti_results[&package_name(p)])
            .count(),
        1
    );
    let ap = 4 | 8;
    let apd = ap | 32;
    assert_eq!(
        closure_table(&study.model_clauses(&[ap]), n),
        closure_table(&study.model_clauses(&[ap, apd]), n)
    );
    assert!(
        anti_exchange_witness(&closure_table(&study.model_clauses(&[ap, apd]), n), n).is_none(),
        "a syntactic cycle alone is not a necessary failure when its rule is redundant"
    );
    let mut domain = String::from("index\tevent\n");
    for (i, e) in study.universe.iter().enumerate() {
        writeln!(domain, "{i}\t{e:?}").unwrap();
    }
    save("revision-geometric-universe.tsv", &domain);
    save("revision-geometric-properties.tsv", &report);
    save(
        "revision-anti-exchange-counterexamples.tsv",
        &counterexamples,
    );
    println!(
        "{} packages checked over all {} theory states each (E={n}); among 5 minimal packages only AP preserves anti-exchange; redundant APD does not defeat AP geometry",
        models.len(),
        1 << n
    );
}
