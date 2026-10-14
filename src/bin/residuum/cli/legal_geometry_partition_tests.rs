//! Analysis of saved observations only: no parsing, elaboration or inference.
use super::{root, save};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

const N: usize = 9;
const H: [&str; N] = [
    "AP",
    "AM+AD",
    "AM+AD+AV",
    "APM+AD",
    "APM+AD+AV",
    "AM+APD",
    "AM+APD+APV",
    "APM+APD",
    "APM+APD+APV",
];
const ALL_PAIRS: u64 = (1 << 36) - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Partition([u8; N]);

impl Partition {
    fn of<T: Eq>(signatures: &[T]) -> Self {
        assert_eq!(signatures.len(), N);
        let mut ids = [0; N];
        let mut next = 0;
        for i in 0..N {
            if let Some(j) = (0..i).find(|&j| signatures[i] == signatures[j]) {
                ids[i] = ids[j];
            } else {
                ids[i] = next;
                next += 1;
            }
        }
        Self(ids)
    }
    fn bottom() -> Self {
        Self(std::array::from_fn(|i| i as u8))
    }
    fn top() -> Self {
        Self([0; N])
    }
    fn id(self) -> String {
        self.0.iter().map(|i| char::from(b'0' + i)).collect()
    }
    fn blocks(self) -> usize {
        usize::from(*self.0.iter().max().unwrap()) + 1
    }
    fn equal_pairs(self) -> u64 {
        let mut out = 0;
        let mut bit = 0;
        for i in 0..N {
            for j in i + 1..N {
                if self.0[i] == self.0[j] {
                    out |= 1 << bit
                }
                bit += 1;
            }
        }
        out
    }
    fn distinguished(self) -> u64 {
        ALL_PAIRS & !self.equal_pairs()
    }
    // Finer <= coarser. Distinctions grow when moving DOWN this order.
    fn le(self, other: Self) -> bool {
        self.equal_pairs() & !other.equal_pairs() == 0
    }
    fn meet(self, other: Self) -> Self {
        Self::of(&(0..N).map(|i| (self.0[i], other.0[i])).collect::<Vec<_>>())
    }
    fn join(self, other: Self) -> Self {
        let mut reachable = [[false; N]; N];
        for (i, row) in reachable.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                *value = self.0[i] == self.0[j] || other.0[i] == other.0[j];
            }
        }
        for k in 0..N {
            for i in 0..N {
                for j in 0..N {
                    reachable[i][j] |= reachable[i][k] && reachable[k][j];
                }
            }
        }
        Self::of(&reachable)
    }
}

fn all_partitions() -> Vec<Partition> {
    fn visit(i: usize, greatest: u8, ids: &mut [u8; N], out: &mut Vec<Partition>) {
        if i == N {
            out.push(Partition(*ids));
            return;
        }
        for value in 0..=greatest + 1 {
            ids[i] = value;
            visit(i + 1, greatest.max(value), ids, out);
        }
    }
    let mut out = Vec::new();
    visit(1, 0, &mut [0; N], &mut out);
    out
}

#[derive(Clone)]
struct Observation {
    name: String,
    partition: Partition,
}

fn aggregate(observations: &[Observation]) -> Partition {
    observations
        .iter()
        .fold(Partition::top(), |p, o| p.meet(o.partition))
}

fn rows(file: &str, header: &str) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(root().join("docs/legal-geometry").join(file)).unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(header));
    lines
        .map(|line| {
            let fields: Vec<_> = line.split('\t').map(String::from).collect();
            assert_eq!(fields.len(), header.split('\t').count());
            fields
        })
        .collect()
}

fn hypothesis(name: &str) -> usize {
    H.iter().position(|h| *h == name).unwrap()
}
fn boolean(value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("stored Boolean"),
    }
}

struct Data {
    theory_events: Vec<Observation>,
    seeded_events: Vec<Observation>,
    theory_proofs: Vec<Observation>,
    seeded_proofs: Vec<Observation>,
}

fn observations() -> Data {
    let cases = [
        "case_0005",
        "case_0065",
        "case_0513",
        "case_2049",
        "case_2561",
    ];
    let case_labels = ["AD", "AM", "AP", "AV", "APV"];
    let old_domain: Vec<_> = rows(
        "region-revision/revision-geometric-universe.tsv",
        "index\tevent",
    )
    .into_iter()
    .enumerate()
    .map(|(i, r)| {
        assert_eq!(r[0].parse::<usize>().unwrap(), i);
        r[1].clone()
    })
    .collect();
    assert_eq!(old_domain.len(), 12);
    let mut old_events: Vec<Vec<BTreeSet<String>>> = cases
        .iter()
        .map(|case| {
            let mask = case.trim_start_matches("case_").parse::<usize>().unwrap();
            let input: BTreeSet<_> = old_domain
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, e)| e.clone())
                .collect();
            vec![input; N]
        })
        .collect();
    let mut old_proofs: Vec<Vec<Vec<String>>> = vec![vec![Vec::new(); N]; 5];
    for r in rows(
        "identifiability/identifiability-provenance.tsv",
        "hypothesis\tcase\tevent\trecord\torigin",
    ) {
        let c = cases.iter().position(|case| *case == r[1]).unwrap();
        let h = hypothesis(&r[0]);
        old_events[c][h].insert(r[2].clone());
        old_proofs[c][h].push(r[2..].join("\t")); // retain ordered witnesses, origins and multiplicity
    }
    let mut seen = BTreeSet::new();
    for r in rows(
        "identifiability/identifiability-engine-closures.tsv",
        "hypothesis\tcase\tcompliance\tknown_events\tjustifications",
    ) {
        let c = cases.iter().position(|case| *case == r[1]).unwrap();
        let h = hypothesis(&r[0]);
        assert!(seen.insert((c, h)));
        assert_eq!(old_events[c][h].len(), r[3].parse::<usize>().unwrap());
        assert_eq!(old_proofs[c][h].len(), r[4].parse::<usize>().unwrap());
        assert_eq!(old_events[c][h].contains(&old_domain[1]), boolean(&r[2]));
    }
    assert_eq!(seen.len(), 45);
    for signatures in &mut old_proofs {
        for proofs in signatures {
            proofs.sort();
        }
    }
    let theory_events = case_labels
        .iter()
        .enumerate()
        .map(|(c, label)| Observation {
            name: (*label).into(),
            partition: Partition::of(&old_events[c]),
        })
        .collect();
    let theory_proofs = case_labels
        .iter()
        .enumerate()
        .map(|(c, label)| Observation {
            name: (*label).into(),
            partition: Partition::of(&old_proofs[c]),
        })
        .collect();
    let mut saved = vec![vec![None; N]; 128];
    let mut names = vec![String::new(); 128];
    for r in rows(
        "observability/observability-engine-closures.tsv",
        "hypothesis\tseed_mask\tinput\tclosed_mask\tcompliance\tevents\tjustifications",
    ) {
        let h = hypothesis(&r[0]);
        let s = r[1].parse::<usize>().unwrap();
        let mask = r[3].parse::<usize>().unwrap();
        assert!(saved[s][h].replace(mask).is_none());
        assert_eq!(mask.count_ones() as usize, r[5].parse::<usize>().unwrap());
        assert_eq!(mask & (1 << 2) != 0, boolean(&r[4]));
        if names[s].is_empty() {
            names[s] = r[2].clone();
        } else {
            assert_eq!(names[s], r[2]);
        }
    }
    let seeded_events = saved
        .into_iter()
        .enumerate()
        .map(|(s, values)| Observation {
            name: format!("seed_{s:03}:{}", names[s]),
            partition: Partition::of(&values.into_iter().map(Option::unwrap).collect::<Vec<_>>()),
        })
        .collect();
    let mut catalog_proofs = vec![Vec::new(); N];
    for r in rows(
        "observability/observability-hypothesis-provenance.tsv",
        "context\tevent\trecord\torigin",
    ) {
        if r[0].contains('/') {
            continue;
        } // background controls are not complete for all nine hypotheses
        catalog_proofs[hypothesis(&r[0])].push(r[1..].join("\t"));
    }
    for proofs in &mut catalog_proofs {
        assert!(!proofs.is_empty());
        proofs.sort();
    }
    // All nine complete traces here are the stored closure with Seeded mask 127.
    let summary = rows(
        "observability/observability-engine-closures.tsv",
        "hypothesis\tseed_mask\tinput\tclosed_mask\tcompliance\tevents\tjustifications",
    );
    for r in summary.into_iter().filter(|r| r[1] == "127") {
        assert_eq!(
            catalog_proofs[hypothesis(&r[0])].len(),
            r[6].parse::<usize>().unwrap()
        );
    }
    let seeded_proofs = vec![Observation {
        name: "seed_127:O+R+B+M+P+A+N".into(),
        partition: Partition::of(&catalog_proofs),
    }];
    Data {
        theory_events,
        seeded_events,
        theory_proofs,
        seeded_proofs,
    }
}

#[test]
fn finite_partition_lattice_has_exact_meet_join_and_non_distributivity() {
    let lattice = all_partitions();
    assert_eq!(lattice.len(), 21147);
    assert_eq!(
        lattice.iter().copied().collect::<BTreeSet<_>>().len(),
        lattice.len()
    );
    let mut levels = [0; N + 1];
    let mut catalog = String::from("partition\tblocks\tequal_pairs_mask\n");
    for &p in &lattice {
        assert_eq!(Partition::of(&p.0), p);
        assert!(Partition::bottom().le(p) && p.le(Partition::top()));
        assert_eq!(p.meet(p), p);
        assert_eq!(p.join(p), p);
        levels[p.blocks()] += 1;
        writeln!(catalog, "{}\t{}\t{}", p.id(), p.blocks(), p.equal_pairs()).unwrap();
    }
    assert_eq!(levels[1..], [1, 255, 3025, 7770, 6951, 2646, 462, 36, 1]);
    let a = Partition([0, 0, 1, 2, 3, 4, 5, 6, 7]);
    let b = Partition([0, 1, 0, 2, 3, 4, 5, 6, 7]);
    let c = Partition([0, 1, 1, 2, 3, 4, 5, 6, 7]);
    for &x in &[a, b, c, Partition::top(), Partition::bottom()] {
        for &y in &[a, b, c, Partition::top(), Partition::bottom()] {
            let meet = x.meet(y);
            let join = x.join(y);
            assert_eq!(meet, y.meet(x));
            assert_eq!(join, y.join(x));
            assert_eq!(meet.equal_pairs(), x.equal_pairs() & y.equal_pairs());
            assert!(meet.le(x) && meet.le(y) && x.le(join) && y.le(join));
            for &z in &lattice {
                if z.le(x) && z.le(y) {
                    assert!(z.le(meet));
                }
                if x.le(z) && y.le(z) {
                    assert!(join.le(z));
                }
            }
            for &z in &[a, b, c, Partition::top(), Partition::bottom()] {
                assert_eq!(x.meet(y).meet(z), x.meet(y.meet(z)));
                assert_eq!(x.join(y).join(z), x.join(y.join(z)));
                assert_eq!(x.meet(x.join(y)), x);
                assert_eq!(x.join(x.meet(y)), x);
            }
        }
    }
    assert_ne!(a.meet(b.join(c)), a.meet(b).join(a.meet(c)));
    save("partitions-universe.tsv", &catalog);
    save(
        "partitions-non-distributivity.tsv",
        &format!(
            "a\tb\tc\ta_meet_b_join_c\ta_meet_b_join_a_meet_c\n{}\t{}\t{}\t{}\t{}\n",
            a.id(),
            b.id(),
            c.id(),
            a.meet(b.join(c)).id(),
            a.meet(b).join(a.meet(c)).id()
        ),
    );
    println!(
        "21147 canonical partitions; exact bounds checked against full universe; non-distributivity witnessed"
    );
}

#[test]
fn recorded_observations_define_event_and_provenance_partitions() {
    let data = observations();
    let theory = aggregate(&data.theory_events);
    let seed = aggregate(&data.seeded_events);
    let trace_theory = aggregate(&data.theory_proofs);
    let trace_seed = aggregate(&data.seeded_proofs);
    assert_eq!(theory, Partition::bottom());
    assert_eq!(seed.blocks(), 7);
    assert_eq!(trace_theory, Partition::bottom());
    assert_eq!(trace_seed, Partition::bottom());
    let restricted: Vec<_> = data
        .seeded_events
        .iter()
        .enumerate()
        .filter(|(s, _)| s & (4 | 64) == 0)
        .map(|(_, o)| o.clone())
        .collect();
    assert_eq!(restricted.len(), 32);
    assert_eq!(aggregate(&restricted).blocks(), 3);
    assert!(seed.le(aggregate(&restricted)));
    // D is available as a THEORY premise, not as Seeded input: these are not
    // interchangeable observation protocols even though both have partitions.
    let ad = data
        .theory_events
        .iter()
        .find(|o| o.name == "AD")
        .unwrap()
        .partition;
    assert!(!ad.le(seed) && !seed.le(ad));
    assert_eq!(ad.meet(seed), Partition::bottom());
    assert_eq!(ad.join(seed), Partition::top());
    let mut inventory = String::from("index\thypothesis\n");
    for (i, h) in H.iter().enumerate() {
        writeln!(inventory, "{i}\t{h}").unwrap();
    }
    let mut report =
        String::from("protocol\tobservation\tpartition\tblocks\tdistinguished_pairs\n");
    let protocols = [
        ("theory_events", &data.theory_events),
        ("seeded_events", &data.seeded_events),
        ("theory_provenance", &data.theory_proofs),
        ("seeded_provenance", &data.seeded_proofs),
    ];
    for (name, observations) in protocols {
        for o in observations {
            writeln!(
                report,
                "{name}\t{}\t{}\t{}\t{}",
                o.name,
                o.partition.id(),
                o.partition.blocks(),
                o.partition.distinguished().count_ones()
            )
            .unwrap();
        }
        let p = aggregate(observations);
        writeln!(
            report,
            "{name}\tALL\t{}\t{}\t{}",
            p.id(),
            p.blocks(),
            p.distinguished().count_ones()
        )
        .unwrap();
    }
    let mut selected = vec![
        ("theory_events", theory),
        ("seeded_events", seed),
        ("theory_provenance", trace_theory),
        ("seeded_provenance", trace_seed),
        ("original_seeded_protocol", aggregate(&restricted)),
    ];
    let mut individual = Vec::new();
    for (protocol, observations) in &protocols {
        for o in *observations {
            if *protocol != "seeded_events" || o.partition != Partition::top() {
                individual.push((format!("{protocol}/{}", o.name), o.partition));
            }
        }
    }
    selected.extend(individual.iter().map(|(name, p)| (name.as_str(), *p)));
    let mut operations = String::from(
        "left\tright\tleft_le_right\tmeet\tmeet_blocks\tjoin\tjoin_blocks\tnew_pairs_added_to_left\n",
    );
    for &(a, p) in &selected {
        for &(b, q) in &selected {
            writeln!(
                operations,
                "{a}\t{b}\t{}\t{}\t{}\t{}\t{}\t{}",
                p.le(q),
                p.meet(q).id(),
                p.meet(q).blocks(),
                p.join(q).id(),
                p.join(q).blocks(),
                (p.meet(q).distinguished() & !p.distinguished()).count_ones()
            )
            .unwrap();
        }
    }
    save("partitions-hypotheses.tsv", &inventory);
    save("partitions-observations.tsv", &report);
    save("partitions-operations.tsv", &operations);
    println!(
        "Saved data only: five theory cases ->9; old seeds ->3; extended seeds ->7; complete traces ->9; no closures evaluated"
    );
}

struct Profile {
    partition: Partition,
    aliases: Vec<String>,
}

fn profiles(observations: &[Observation]) -> Vec<Profile> {
    let mut grouped: BTreeMap<Partition, Vec<String>> = BTreeMap::new();
    for o in observations {
        grouped.entry(o.partition).or_default().push(o.name.clone());
    }
    grouped
        .into_iter()
        .filter(|(p, _)| *p != Partition::top())
        .map(|(p, aliases)| Profile {
            partition: p,
            aliases,
        })
        .collect()
}

fn minimal_covers(profiles: &[Profile], target: Partition) -> Vec<usize> {
    assert!(
        profiles.len() <= 16,
        "bounded exact search over distinct partitions, not 2^128 cases"
    );
    let required = target.distinguished();
    let mut coverage = vec![0; 1 << profiles.len()];
    for mask in 1..coverage.len() {
        let bit = mask.trailing_zeros() as usize;
        coverage[mask] = coverage[mask & !(1 << bit)] | profiles[bit].partition.distinguished();
    }
    (0..coverage.len())
        .filter(|&s| {
            coverage[s] == required
                && (0..profiles.len())
                    .all(|i| s & (1 << i) == 0 || coverage[s & !(1 << i)] != required)
        })
        .collect()
}

fn expand_aliases(profiles: &[Profile], mask: usize) -> Vec<Vec<String>> {
    let mut sets = vec![Vec::new()];
    for (_, p) in profiles
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
    {
        let mut next = Vec::new();
        for prefix in &sets {
            for name in &p.aliases {
                let mut set = prefix.clone();
                set.push(name.clone());
                next.push(set);
            }
        }
        assert!(
            next.len() <= 20000,
            "bounded exact expansion of equivalent cases"
        );
        sets = next;
    }
    sets
}

fn witness_pair(mask: u64) -> (usize, usize) {
    let mut bit = 0;
    for i in 0..N {
        for j in i + 1..N {
            if mask & (1 << bit) != 0 {
                return (i, j);
            }
            bit += 1;
        }
    }
    panic!("nonempty distinction witness")
}

#[test]
fn exact_minimum_case_selection_separates_results_from_justifications() {
    let data = observations();
    let protocols = [
        ("theory_events", &data.theory_events, 4),
        ("seeded_events", &data.seeded_events, 4),
        ("theory_provenance", &data.theory_proofs, 3),
        ("seeded_provenance", &data.seeded_proofs, 1),
    ];
    let mut sets =
        String::from("protocol\ttarget_partition\tsize\tcardinality_minimum\tobservations\n");
    let mut certificates = String::from(
        "protocol\tselected_observations\tremoved_observation\tindistinguishable_left\tindistinguishable_right\n",
    );
    let mut profiles_report =
        String::from("protocol\tprofile\tpartition\taliases\tnew_pairs_after_all_other_profiles\n");
    let mut summary = String::from(
        "protocol\tavailable_cases\tnontrivial_profiles\tclasses\tminimum_cardinality\tinclusion_minimal_sets\tcardinality_minimal_sets\n",
    );
    let mut combinations =
        String::from("protocol\tprofile_subset\trepresentative_observations\tpartition\tblocks\n");
    for (name, observations, expected_minimum) in protocols {
        let target = aggregate(observations);
        let profiles = profiles(observations);
        let covers = minimal_covers(&profiles, target);
        let minimum = covers.iter().map(|c| c.count_ones()).min().unwrap();
        assert_eq!(minimum, expected_minimum);
        let mut inclusion_count = 0;
        let mut minimum_count = 0;
        let mut computed_sets = BTreeSet::new();
        for mask in 0..1 << profiles.len() {
            let representatives: Vec<_> = profiles
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, p)| Observation {
                    name: p.aliases[0].clone(),
                    partition: p.partition,
                })
                .collect();
            let p = aggregate(&representatives);
            writeln!(
                combinations,
                "{name}\t{mask}\t{}\t{}\t{}",
                representatives
                    .iter()
                    .map(|o| o.name.as_str())
                    .collect::<Vec<_>>()
                    .join(";"),
                p.id(),
                p.blocks()
            )
            .unwrap();
        }
        for (i, p) in profiles.iter().enumerate() {
            let others = profiles
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .fold(Partition::top(), |q, (_, other)| q.meet(other.partition));
            writeln!(
                profiles_report,
                "{name}\t{i}\t{}\t{}\t{}",
                p.partition.id(),
                p.aliases.join(";"),
                (target.distinguished() & !others.distinguished()).count_ones()
            )
            .unwrap();
        }
        for cover in covers {
            for names in expand_aliases(&profiles, cover) {
                let mut canonical = names.clone();
                canonical.sort();
                assert!(computed_sets.insert(canonical));
                let selected: Vec<_> = names
                    .iter()
                    .map(|name| {
                        observations
                            .iter()
                            .find(|o| &o.name == name)
                            .unwrap()
                            .clone()
                    })
                    .collect();
                assert_eq!(aggregate(&selected), target);
                inclusion_count += 1;
                if names.len() == minimum as usize {
                    minimum_count += 1;
                }
                writeln!(
                    sets,
                    "{name}\t{}\t{}\t{}\t{}",
                    target.id(),
                    names.len(),
                    names.len() == minimum as usize,
                    names.join(";")
                )
                .unwrap();
                for (removed, o) in selected.iter().enumerate() {
                    let rest: Vec<_> = selected
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != removed)
                        .map(|(_, o)| o.clone())
                        .collect();
                    let weaker = aggregate(&rest);
                    assert!(target.le(weaker) && target != weaker);
                    let (a, b) = witness_pair(target.distinguished() & !weaker.distinguished());
                    assert_ne!(target.0[a], target.0[b]);
                    assert_eq!(weaker.0[a], weaker.0[b]);
                    writeln!(
                        certificates,
                        "{name}\t{}\t{}\t{}\t{}",
                        names.join(";"),
                        o.name,
                        H[a],
                        H[b]
                    )
                    .unwrap();
                }
            }
        }
        // Independent exhaustive oracle over the actual informative cases,
        // rather than profiles or pair-mask coverage. This also checks alias
        // expansion preserves ALL inclusion-minimal sets.
        let informative: Vec<_> = observations
            .iter()
            .filter(|o| o.partition != Partition::top())
            .cloned()
            .collect();
        assert!(informative.len() <= 16);
        let by_mask: Vec<_> = (0..1 << informative.len())
            .map(|mask| {
                let subset: Vec<_> = informative
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, o)| o.clone())
                    .collect();
                aggregate(&subset)
            })
            .collect();
        let oracle: BTreeSet<_> = by_mask
            .iter()
            .enumerate()
            .filter(|(mask, p)| {
                **p == target
                    && (0..informative.len())
                        .all(|i| mask & (1 << i) == 0 || by_mask[mask & !(1 << i)] != target)
            })
            .map(|(mask, _)| {
                let mut names: Vec<_> = informative
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, o)| o.name.clone())
                    .collect();
                names.sort();
                names
            })
            .collect();
        assert_eq!(computed_sets, oracle);
        writeln!(
            summary,
            "{name}\t{}\t{}\t{}\t{minimum}\t{inclusion_count}\t{minimum_count}",
            observations.len(),
            profiles.len(),
            target.blocks()
        )
        .unwrap();
    }
    // A mathematical control on the same nine points: inclusion-minimal need
    // not mean minimum cardinality. No new legal hypothesis or observation.
    let control = vec![
        Observation {
            name: "complete".into(),
            partition: Partition::bottom(),
        },
        Observation {
            name: "row".into(),
            partition: Partition::of(&(0..N).map(|i| i / 3).collect::<Vec<_>>()),
        },
        Observation {
            name: "column".into(),
            partition: Partition::of(&(0..N).map(|i| i % 3).collect::<Vec<_>>()),
        },
    ];
    let controls = profiles(&control);
    let covers = minimal_covers(&controls, Partition::bottom());
    assert_eq!(covers.len(), 2);
    assert_eq!(
        covers
            .iter()
            .map(|c| c.count_ones())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([1, 2])
    );
    save("partitions-minimal-observation-sets.tsv", &sets);
    save("partitions-minimality-certificates.tsv", &certificates);
    save("partitions-observation-profiles.tsv", &profiles_report);
    save("partitions-selection-summary.tsv", &summary);
    save("partitions-observation-combinations.tsv", &combinations);
    println!(
        "Exact nonadaptive selection from saved data: theory events 4, Seeded events 4, theory traces 3, Seeded full trace 1; no external labels answered"
    );
}
