//! Version spaces of monotone justificatory functions, never approved norms.
use super::*;

const N: usize = 16;
const PREFIX: &str = "justificatorySupport";

fn artifact(name: &str, text: &str) {
    save(&format!("recalcitrance-{name}"), text);
    let dir = root().join("docs/legal-geometry/recalcitrance");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

fn name(mask: usize) -> String {
    let out: String = ["O", "R", "A", "P", "N"]
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, s)| *s)
        .collect();
    if out.is_empty() { "empty".into() } else { out }
}

fn family(theta: &[usize]) -> String {
    if theta.is_empty() {
        return "false".into();
    }
    theta.iter().map(|&s| name(s)).collect::<Vec<_>>().join("|")
}

fn predict(theta: &[usize], input: usize) -> bool {
    theta.iter().any(|&b| b & input == b)
}

// Enumerate antichains directly, not the power set of possible families.
fn functions() -> Vec<Vec<usize>> {
    fn visit(next: usize, theta: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        out.push(theta.clone());
        for s in next..N {
            if theta.iter().all(|&t| s & t != s && s & t != t) {
                theta.push(s);
                visit(s + 1, theta, out);
                theta.pop();
            }
        }
    }
    let mut out = Vec::new();
    visit(0, &mut Vec::new(), &mut out);
    out.sort();
    assert_eq!(out.len(), 168);
    let regions: BTreeSet<_> = out.iter().map(|h| region(h)).collect();
    assert_eq!(regions.len(), out.len(), "no duplicate functions");
    out
}

fn region(theta: &[usize]) -> usize {
    (0..N)
        .filter(|&s| predict(theta, s))
        .fold(0, |r, s| r | (1 << s))
}

fn input_mask(events: &[Event], domain: &[Event]) -> usize {
    events.iter().fold(0, |mask, e| {
        mask | (1
            << domain
                .iter()
                .position(|p| p == e)
                .expect("explicit universe"))
    })
}

struct Observation {
    case: String,
    mask: usize,
    value: bool,
    source: String,
    rationale: String,
}

struct Study {
    module: Module,
    seeds: Vec<Event>,
    goal: Event,
    observations: Vec<Observation>,
    families: Vec<Vec<usize>>,
    universe: Vec<Event>,
    baseline: Vec<Closure>,
}

impl Study {
    fn new() -> Self {
        let module = module("recalcitrance_cases");
        let seeds = experiment(&module, "geometryCatalog").input.clone();
        assert_eq!(seeds.len(), 5);
        assert_eq!(
            seeds.iter().map(|e| e.verb.as_str()).collect::<Vec<_>>(),
            ["obligationFiled", "right", "Act", "permissionFiled", "Act"]
        );
        assert!(
            seeds
                .iter()
                .all(|e| e.proposition.kind == PropositionKind::Seeded)
        );
        assert_eq!(seeds[4].polarity, Polarity::EvidentialNot);
        assert_eq!(seeds[4].args, seeds[2].args);
        let goal = experiment(&module, "catalog").goal.clone();
        assert_eq!(experiment(&module, "catalog").input, seeds[..4]);
        assert!(
            !module
                .declarations
                .iter()
                .any(|d| matches!(d, Declaration::Program(_) | Declaration::Default(_)))
        );
        assert_eq!(
            module
                .declarations
                .iter()
                .filter(|d| matches!(d, Declaration::Rule(_)))
                .count(),
            27
        );
        let baseline: Vec<_> = (0..32).map(|s| close(&module, subset(&seeds, s))).collect();
        // Exhaustive geometry uses exactly the initial four-Seeded domain.
        // ~Act supplies a separate, wider consequence/provenance control.
        let universe: Vec<_> = events(&baseline[15]).into_iter().collect();
        assert!(universe.len() <= 13, "bound the geometric power set");
        for (s, c) in baseline.iter().enumerate() {
            audit_proofs(&module, c);
            assert!(events(c).is_subset(&events(&baseline[31])));
            assert_eq!(c.known.contains(&goal), s & 5 == 5 || s & 6 == 6);
        }
        let text = std::fs::read_to_string(
            root().join("examples/experiments/legal_geometry/recalcitrance_labels.tsv"),
        )
        .unwrap();
        let observations = text
            .lines()
            .filter(|l| !l.starts_with('#'))
            .skip(1)
            .enumerate()
            .map(|(i, line)| {
                let columns: Vec<_> = line.split('\t').collect();
                assert_eq!(columns.len(), 6);
                assert_eq!(columns[0].parse::<usize>().unwrap(), i + 1);
                assert_eq!(columns[4], "hypothetical_external_assumption");
                let ex = experiment(&module, columns[1]);
                assert_eq!(ex.input_kind, ExperimentInputKind::Seeds);
                assert_eq!(ex.goal, goal);
                Observation {
                    case: columns[1].into(),
                    mask: input_mask(&ex.input, &seeds[..4]),
                    value: match columns[2] {
                        "0" => false,
                        "1" => true,
                        _ => panic!("external Boolean valuation"),
                    },
                    source: columns[3].into(),
                    rationale: columns[5].into(),
                }
            })
            .collect();
        Self {
            module,
            seeds,
            goal,
            observations,
            families: functions(),
            universe,
            baseline,
        }
    }

    fn remaining(&self, stage: usize) -> Vec<usize> {
        (0..self.families.len())
            .filter(|&h| {
                self.observations[..stage]
                    .iter()
                    .all(|o| predict(&self.families[h], o.mask) == o.value)
            })
            .collect()
    }

    fn revision(&self, templates: &Module, theta: &[usize]) -> Module {
        // Private experimental copy. No original file or declaration is edited.
        // Restrictions require replacement, not addition of positive Horn rules.
        let mut result = templates.clone();
        result.declarations.retain(|d| {
            if let Declaration::Rule(r) = d {
                if ["obligationCompliance", "rightCompliance"].contains(&r.name.as_str()) {
                    return false;
                }
                if let Some(id) = r.name.strip_prefix(PREFIX) {
                    return theta.contains(&id.parse::<usize>().unwrap());
                }
            }
            true
        });
        let names: BTreeSet<_> = result
            .declarations
            .iter()
            .map(|d| d.name().to_string())
            .collect();
        result.origins.retain(|name, _| names.contains(name));
        assert_eq!(
            result
                .declarations
                .iter()
                .filter(|d| matches!(d, Declaration::Rule(_)))
                .count(),
            25 + theta.len()
        );
        for d in &self.module.declarations {
            if matches!(d, Declaration::Experiment(_))
                || ["obligationCompliance", "rightCompliance"].contains(&d.name())
            {
                continue;
            }
            let retained = result
                .declarations
                .iter()
                .find(|r| r.name() == d.name())
                .unwrap();
            assert_eq!(format!("{d:?}"), format!("{retained:?}"));
            assert_eq!(self.module.origin(d.name()), result.origin(d.name()));
        }
        result
    }
}

fn partition(signatures: Vec<Vec<usize>>) -> Vec<usize> {
    let mut labels = BTreeMap::new();
    signatures
        .into_iter()
        .map(|s| {
            let next = labels.len();
            *labels.entry(s).or_insert(next)
        })
        .collect()
}

fn meet(a: &[usize], b: &[usize]) -> Vec<usize> {
    partition(a.iter().zip(b).map(|(&x, &y)| vec![x, y]).collect())
}

fn refines(a: &[usize], b: &[usize]) -> bool {
    (0..a.len()).all(|i| (0..a.len()).all(|j| a[i] != a[j] || b[i] == b[j]))
}

fn minimal_difference(a: &[usize], b: &[usize]) -> Vec<usize> {
    let difference: Vec<_> = (0..N).filter(|&s| predict(a, s) != predict(b, s)).collect();
    difference
        .iter()
        .copied()
        .filter(|&s| !difference.iter().any(|&t| t != s && t & s == t))
        .collect()
}

#[test]
fn recalcitrant_observations_filter_antichains_and_refine_hypothesis_partitions() {
    let s = Study::new();
    let original = format!("{:?}", s.module);
    let mut families = String::from("hypothesis\tantichain\tpositive_cases\tfirst_rejected_step\n");
    for (h, theta) in s.families.iter().enumerate() {
        let rejected = s
            .observations
            .iter()
            .position(|o| predict(theta, o.mask) != o.value)
            .map(|i| (i + 1).to_string())
            .unwrap_or_else(|| "survives".into());
        writeln!(
            families,
            "H{h:03}\t{}\t{}\t{rejected}",
            family(theta),
            (0..N)
                .filter(|&x| predict(theta, x))
                .map(name)
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    let mut stages = String::from(
        "stage\tcase\tinput\tnorm\texternal_value\trecalcitrance\tbefore\tafter\thypothesis_partition_blocks\tcase_partition_blocks\tsource\trationale\n",
    );
    let mut statuses = String::from("stage\thypothesis\tstatus\tantichain\n");
    let mut shared =
        String::from("stage\tshared_minimal_supports\tunanimous_positive\tunanimous_negative\n");
    let mut kernels = String::from("stage\thypothesis\tclass\n");
    let mut cases = String::from("stage\tcase\tclass\tnorm\n");
    let mut discriminants = String::from(
        "stage\tcase\tminimal_for_pairs\twitness_h1\twitness_h2\tvalue_h1\tvalue_h2\texternal_answer\n",
    );
    let mut pairs = String::from("h1\th2\tminimal_discriminant\tvalue_h1\tvalue_h2\n");
    let mut hp = vec![0; s.families.len()];
    let mut cp = partition(
        (0..N)
            .map(|x| vec![usize::from(s.baseline[x].known.contains(&s.goal))])
            .collect(),
    );
    for stage in 0..=s.observations.len() {
        let active = s.remaining(stage);
        assert!(
            !active.is_empty(),
            "do not invent a function for an empty version space"
        );
        let p = partition(
            s.families
                .iter()
                .map(|h| {
                    s.observations[..stage]
                        .iter()
                        .map(|o| usize::from(predict(h, o.mask)))
                        .collect()
                })
                .collect(),
        );
        if stage > 0 {
            let o = &s.observations[stage - 1];
            let one = partition(
                s.families
                    .iter()
                    .map(|h| vec![usize::from(predict(h, o.mask))])
                    .collect(),
            );
            assert_eq!(p, meet(&hp, &one));
            assert!(refines(&p, &hp));
            let norm = s.baseline[o.mask].known.contains(&s.goal);
            let discrepancy = match (norm, o.value) {
                (true, false) => "overinclusion",
                (false, true) => "underinclusion",
                _ => "agreement",
            };
            writeln!(
                stages,
                "{stage}\t{}\t{}\t{norm}\t{}\t{discrepancy}\t{}\t{}\t{}\t{}\t{}\t{}",
                o.case,
                name(o.mask),
                o.value,
                s.remaining(stage - 1).len(),
                active.len(),
                p.iter().collect::<BTreeSet<_>>().len(),
                0,
                o.source,
                o.rationale
            )
            .unwrap();
        }
        hp = p;
        let next_cp = partition(
            (0..N)
                .map(|x| {
                    let mut signature = vec![usize::from(s.baseline[x].known.contains(&s.goal))];
                    signature.extend(
                        active
                            .iter()
                            .map(|&h| usize::from(predict(&s.families[h], x))),
                    );
                    signature
                })
                .collect(),
        );
        if stage > 0 {
            assert!(
                refines(&cp, &next_cp),
                "fewer candidate functions coarsen the case kernel"
            );
        }
        cp = next_cp;
        for (h, class) in hp.iter().enumerate() {
            writeln!(kernels, "{stage}\tH{h:03}\t{class}").unwrap();
            writeln!(
                statuses,
                "{stage}\tH{h:03}\t{}\t{}",
                if active.contains(&h) {
                    "survives"
                } else {
                    "excluded"
                },
                family(&s.families[h])
            )
            .unwrap();
        }
        for (x, class) in cp.iter().enumerate() {
            writeln!(
                cases,
                "{stage}\t{}\t{class}\t{}",
                name(x),
                s.baseline[x].known.contains(&s.goal)
            )
            .unwrap();
        }
        let common: BTreeSet<_> = s.families[active[0]]
            .iter()
            .copied()
            .filter(|b| active.iter().all(|&h| s.families[h].contains(b)))
            .collect();
        writeln!(
            shared,
            "{stage}\t{}\t{}\t{}",
            common.into_iter().map(name).collect::<Vec<_>>().join(","),
            (0..N)
                .filter(|&x| active.iter().all(|&h| predict(&s.families[h], x)))
                .map(name)
                .collect::<Vec<_>>()
                .join(","),
            (0..N)
                .filter(|&x| active.iter().all(|&h| !predict(&s.families[h], x)))
                .map(name)
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
        let mut pool: BTreeMap<usize, (usize, usize, usize)> = BTreeMap::new();
        for (i, &a) in active.iter().enumerate() {
            for &b in &active[i + 1..] {
                for x in minimal_difference(&s.families[a], &s.families[b]) {
                    let row = pool.entry(x).or_insert((0, a, b));
                    row.0 += 1;
                    if stage == s.observations.len() {
                        writeln!(
                            pairs,
                            "H{a:03}\tH{b:03}\t{}\t{}\t{}",
                            name(x),
                            predict(&s.families[a], x),
                            predict(&s.families[b], x)
                        )
                        .unwrap();
                    }
                }
            }
        }
        for (x, (count, a, b)) in pool {
            writeln!(
                discriminants,
                "{stage}\t{}\t{count}\tH{a:03}\tH{b:03}\t{}\t{}\tpending",
                name(x),
                predict(&s.families[a], x),
                predict(&s.families[b], x)
            )
            .unwrap();
        }
    }
    // No function is designated as true; final distinct functions remain.
    let final_ids = s.remaining(s.observations.len());
    assert_eq!(final_ids.len(), 11);
    let distinguishing_cases = [2, 3, 9, 10, 11]; // R, OR, OP, RP, ORP
    let signatures: BTreeSet<_> = final_ids
        .iter()
        .map(|&h| {
            distinguishing_cases
                .iter()
                .map(|&x| predict(&s.families[h], x))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(signatures.len(), final_ids.len());
    let mut identification =
        String::from("case\twitness_h1\twitness_h2\tother_cases_agree\texternal_answer\n");
    for &x in &distinguishing_cases {
        let witness = final_ids
            .iter()
            .enumerate()
            .find_map(|(i, &a)| {
                final_ids[i + 1..]
                    .iter()
                    .copied()
                    .find(|&b| {
                        predict(&s.families[a], x) != predict(&s.families[b], x)
                            && distinguishing_cases
                                .iter()
                                .filter(|&&t| t != x)
                                .all(|&t| predict(&s.families[a], t) == predict(&s.families[b], t))
                    })
                    .map(|b| (a, b))
            })
            .expect("each of these five observations is indispensable in this catalog");
        writeln!(
            identification,
            "{}\tH{:03}\tH{:03}\ttrue\tpending",
            name(x),
            witness.0,
            witness.1
        )
        .unwrap();
    }
    artifact("identification-cases.tsv", &identification);
    let norm_id = s.families.iter().position(|h| h == &[5, 6]).unwrap();
    assert!(!s.remaining(3).contains(&norm_id));
    assert!(
        s.families
            .iter()
            .all(|h| !(predict(h, 4) && !predict(h, 12))),
        "positive A and negative AP are impossible for any monotone hypothesis"
    );
    artifact(
        "incompatible-control.tsv",
        "positive_case\tnegative_case\tcompatible_functions\treason\nA\tAP\t0\tA subset AP; monotone language cannot fit these stipulated valuations\n",
    );
    // Case partition sizes are taken from the independently constructed kernel.
    let sizes: BTreeMap<usize, usize> =
        cases
            .lines()
            .skip(1)
            .fold(BTreeMap::new(), |mut acc, line| {
                let row: Vec<_> = line.split('\t').collect();
                let stage = row[0].parse().unwrap();
                let class = row[2].parse::<usize>().unwrap();
                acc.entry(stage)
                    .and_modify(|v| *v = (*v).max(class + 1))
                    .or_insert(class + 1);
                acc
            });
    let mut corrected = String::new();
    for (i, line) in stages.lines().enumerate() {
        if i == 0 {
            writeln!(corrected, "{line}").unwrap();
        } else {
            let mut row: Vec<_> = line.split('\t').map(String::from).collect();
            row[9] = sizes[&row[0].parse::<usize>().unwrap()].to_string();
            writeln!(corrected, "{}", row.join("\t")).unwrap();
        }
    }
    for (file, text) in [
        ("functions.tsv", families),
        ("stages.tsv", corrected),
        ("version-spaces.tsv", statuses),
        ("shared-conditions.tsv", shared),
        ("hypothesis-partitions.tsv", kernels),
        ("case-partitions.tsv", cases),
        ("discriminants.tsv", discriminants),
        ("remaining-pair-minima.tsv", pairs),
    ] {
        artifact(file, &text);
    }
    assert_eq!(original, format!("{:?}", s.module));
}

struct ProofArchive {
    ids: BTreeMap<String, usize>,
    records: String,
    occurrences: String,
}

impl ProofArchive {
    fn new() -> Self {
        Self {
            ids: BTreeMap::new(),
            records: "payload_id\tkind\tevent\trecord\torigin\n".into(),
            occurrences: "context\tpayload_id\n".into(),
        }
    }

    fn occurrence(&mut self, context: &str, payload: String) -> usize {
        let id = if let Some(&id) = self.ids.get(&payload) {
            id
        } else {
            let id = self.ids.len();
            writeln!(self.records, "{id}\t{payload}").unwrap();
            self.ids.insert(payload, id);
            id
        };
        // Every occurrence is retained: this storage encoding does not collapse
        // justifications or modify runtime provenance/deduplication.
        writeln!(self.occurrences, "{context}\t{id}").unwrap();
        id
    }

    fn export(&mut self, context: &str, m: &Module, c: &Closure) {
        audit_proofs(m, c);
        for e in &c.known {
            let payload = format!("event\t{e:?}\t-\t-");
            let id = self.occurrence(context, payload.clone());
            assert_eq!(self.ids[&payload], id);
        }
        for d in &c.derivations {
            let payload = format!(
                "derivation\t{:?}\t{:?}\t{:?}",
                d.event,
                d.record,
                m.origin(&d.record.rule).unwrap()
            );
            let id = self.occurrence(context, payload.clone());
            assert_eq!(self.ids[&payload], id);
        }
    }
}

#[test]
fn justificatory_revisions_are_hypothetical_and_certified_by_real_closures() {
    let s = Study::new();
    let templates = module("recalcitrance_revisions");
    let before = format!("{:?}", templates);
    let active = s.remaining(s.observations.len());
    let norm_region = region(&[5, 6]);
    let mut revisions = String::from(
        "hypothesis\tantichain\tadded_cases\tremoved_cases\tcompatible_final\tnew_producers\tremoved_producers\n",
    );
    let mut output =
        String::from("hypothesis\tseed_case\tnorm\tjustification\tnew_events\tlost_events\n");
    let mut proof_report = ProofArchive::new();
    for (x, c) in s.baseline.iter().enumerate() {
        proof_report.export(&format!("norm:{}", name(x)), &s.module, c);
    }
    for (h, theta) in s.families.iter().enumerate() {
        let revised = s.revision(&templates, theta);
        let r = region(theta);
        writeln!(
            revisions,
            "H{h:03}\t{}\t{}\t{}\t{}\t{}\tobligationCompliance,rightCompliance",
            family(theta),
            (0..N)
                .filter(|x| (r & !norm_region) & (1 << x) != 0)
                .map(name)
                .collect::<Vec<_>>()
                .join(","),
            (0..N)
                .filter(|x| (norm_region & !r) & (1 << x) != 0)
                .map(name)
                .collect::<Vec<_>>()
                .join(","),
            active.contains(&h),
            theta
                .iter()
                .map(|x| format!("{PREFIX}{x}"))
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
        let limit = if active.contains(&h) { 32 } else { N };
        for x in 0..limit {
            let input = subset(&s.seeds, x);
            let input_before = format!("{input:?}");
            let c = close(&revised, input.clone());
            assert_eq!(input_before, format!("{input:?}"));
            assert_eq!(c.known.contains(&s.goal), predict(theta, x & 15));
            assert!(events(&c).is_subset(&events(&s.baseline[31])));
            let actual = full_events(&c);
            let original = full_events(&s.baseline[x]);
            let added: Vec<_> = actual.difference(&original).collect();
            let removed: Vec<_> = original.difference(&actual).collect();
            writeln!(
                output,
                "H{h:03}\t{}\t{}\t{}\t{added:?}\t{removed:?}",
                name(x),
                s.baseline[x].known.contains(&s.goal),
                predict(theta, x & 15)
            )
            .unwrap();
            proof_report.export(&format!("H{h:03}:{}", name(x)), &revised, &c);
            for d in &c.derivations {
                if d.record.rule.starts_with(PREFIX) {
                    assert_eq!(
                        revised.origin(&d.record.rule).unwrap().module,
                        "HypotheticalJustificatoryRevisions"
                    );
                    assert_eq!(d.event.proposition.time, s.goal.proposition.time);
                } else {
                    assert_eq!(
                        revised.origin(&d.record.rule),
                        s.module.origin(&d.record.rule)
                    );
                }
            }
        }
    }
    assert_eq!(before, format!("{templates:?}"));
    artifact("revisions.tsv", &revisions);
    artifact("revision-consequences.tsv", &output);
    artifact("closure-provenance.tsv", &proof_report.occurrences);
    artifact("provenance-records.tsv", &proof_report.records);
}

fn ground(m: &Module, universe: &[Event]) -> Vec<(usize, usize)> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(instantiate_from_known(r, universe)),
            _ => None,
        })
        .flatten()
        .map(|d| {
            (
                input_mask(&d.record.antecedents, universe),
                input_mask(&[d.event], universe),
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn saturation(mut s: usize, rules: &[(usize, usize)]) -> usize {
    loop {
        let before = s;
        for &(a, c) in rules {
            if s & a == a {
                s |= c;
            }
        }
        if before == s {
            return s;
        }
    }
}

fn anti_exchange(table: &[usize], n: usize) -> Option<(usize, usize, usize)> {
    for (f, &closed) in table.iter().enumerate() {
        if f != closed {
            continue;
        }
        for x in 0..n {
            for y in x + 1..n {
                if f & ((1 << x) | (1 << y)) == 0
                    && table[f | (1 << x)] & (1 << y) != 0
                    && table[f | (1 << y)] & (1 << x) != 0
                {
                    return Some((f, x, y));
                }
            }
        }
    }
    None
}

#[test]
fn hypothetical_revision_geometry_checks_anti_exchange_without_using_it_as_a_filter() {
    let s = Study::new();
    let templates = module("recalcitrance_revisions");
    let n = s.universe.len();
    let bound = 1 << n;
    let mut report = String::from(
        "hypothesis\tuniverse_size\ttheories_checked\tclosed_states\textensive\tmonotone\tidempotent\tanti_exchange\tempty_closed\tcompatible_final\twider_universe_size\twider_dag\n",
    );
    let active = s.remaining(s.observations.len());
    let mut revisions = vec![("norm".to_string(), s.module.clone())];
    revisions.extend(
        s.families
            .iter()
            .enumerate()
            .map(|(h, t)| (format!("H{h:03}"), s.revision(&templates, t))),
    );
    for (id, m) in revisions {
        let clauses = ground(&m, &s.universe);
        let table: Vec<_> = (0..bound).map(|x| saturation(x, &clauses)).collect();
        let extensive = table.iter().enumerate().all(|(x, &f)| x & f == x);
        let idempotent = table.iter().all(|&f| table[f] == f);
        let monotone = table
            .iter()
            .enumerate()
            .all(|(x, &f)| (0..n).all(|i| f & table[x | (1 << i)] == f));
        assert!(extensive && monotone && idempotent);
        let witness = anti_exchange(&table, n);
        // This conclusion is checked for all candidates, not a validity policy.
        assert_eq!(witness, None);
        let wider: Vec<_> = events(&s.baseline[31]).into_iter().collect();
        let wider_n = wider.len();
        let mut reach = vec![vec![false; wider_n]; wider_n];
        for (body, head) in ground(&m, &wider) {
            for (i, row) in reach.iter_mut().enumerate() {
                for (j, r) in row.iter_mut().enumerate() {
                    if body & (1 << i) != 0 && head & (1 << j) != 0 {
                        *r = true;
                    }
                }
            }
        }
        for k in 0..wider_n {
            for i in 0..wider_n {
                for j in 0..wider_n {
                    reach[i][j] |= reach[i][k] && reach[k][j];
                }
            }
        }
        assert!(
            (0..wider_n).all(|i| !reach[i][i]),
            "the supplied controls are acyclic even with ~Act"
        );
        // Independent saturation must match the real evaluator on every Seeded
        // input in the initial four-fact catalog for this candidate.
        for x in 0..N {
            let input = subset(&s.seeds, x);
            let actual = close(&m, input.clone());
            assert_eq!(
                table[input_mask(&input, &s.universe)],
                input_mask(&actual.known, &s.universe)
            );
        }
        let compatible = id
            .strip_prefix('H')
            .is_some_and(|h| active.contains(&h.parse::<usize>().unwrap()));
        writeln!(
            report,
            "{id}\t{n}\t{bound}\t{}\t{extensive}\t{monotone}\t{idempotent}\t{}\t{}\t{compatible}\t{wider_n}\ttrue",
            table.iter().enumerate().filter(|(x, f)| *x == **f).count(),
            witness.is_none(),
            table[0] == 0
        )
        .unwrap();
    }
    artifact("geometry.tsv", &report);
}
