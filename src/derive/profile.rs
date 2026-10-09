//! Test-only observations of the existing evaluator; no scheduling changes.
use crate::typed::UnstampedDerivation;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static SCAN_KNOWN: Cell<bool> = const { Cell::new(false) };
    static STRUCTURAL_REJECTS: Cell<u64> = const { Cell::new(0) };
    static PROBES: Cell<u64> = const { Cell::new(0) };
    static HEADER_REJECTS: Cell<u64> = const { Cell::new(0) };
    static DATA: RefCell<Snapshot> = RefCell::new(Snapshot::default());
}

#[derive(Clone, Debug, Default)]
pub(super) struct Snapshot {
    pub rounds: usize,
    pub evaluations: usize,
    pub associated_evaluations: usize,
    pub candidates: usize,
    pub duplicates: usize,
    pub comparisons: u64,
    pub probes: u64,
    pub header_rejects: u64,
    pub structural_rejects: u64,
    pub activated: usize,
    preparation: Duration,
    activation: Duration,
    matching: Duration,
    dedup: Duration,
    retention: Duration,
    rules: BTreeMap<String, (usize, usize, Duration)>,
}

pub(super) struct Session {
    name: &'static str,
    started: Instant,
}
impl Session {
    pub fn new(name: &'static str) -> Self {
        ENABLED.with(|on| assert!(!on.replace(true), "nested profiler"));
        DATA.with(|data| *data.borrow_mut() = Snapshot::default());
        PROBES.with(|c| c.set(0));
        HEADER_REJECTS.with(|c| c.set(0));
        STRUCTURAL_REJECTS.with(|c| c.set(0));
        println!("PROFILE BEGIN {name}");
        Self {
            name,
            started: Instant::now(),
        }
    }
    pub fn finish(self) -> Snapshot {
        let mut data = DATA.with(|data| data.borrow().clone());
        data.probes = PROBES.with(Cell::get);
        data.header_rejects = HEADER_REJECTS.with(Cell::get);
        data.structural_rejects = STRUCTURAL_REJECTS.with(Cell::get);
        let wall = self.started.elapsed();
        let measured =
            data.preparation + data.activation + data.matching + data.dedup + data.retention;
        println!(
            "PROFILE END {} wall_ms={:.3} preparation_ms={:.3} activation_ms={:.3} matching_ms={:.3} dedup_ms={:.3} retention_ms={:.3} remainder_ms={:.3} rounds={} evaluations={} associated_evaluations={} activated={} candidates={} duplicates={} comparisons={} probes={} header_rejects={} structural_rejects={}",
            self.name,
            ms(wall),
            ms(data.preparation),
            ms(data.activation),
            ms(data.matching),
            ms(data.dedup),
            ms(data.retention),
            ms(wall.saturating_sub(measured)),
            data.rounds,
            data.evaluations,
            data.associated_evaluations,
            data.activated,
            data.candidates,
            data.duplicates,
            data.comparisons,
            data.probes,
            data.header_rejects,
            data.structural_rejects
        );
        let mut rules = data.rules.iter().collect::<Vec<_>>();
        rules.sort_by_key(|(_, (_, _, elapsed))| std::cmp::Reverse(*elapsed));
        for (name, (calls, candidates, elapsed)) in rules.into_iter().take(10) {
            println!(
                "PROFILE RULE {name} calls={calls} candidates={candidates} matching_ms={:.3}",
                ms(*elapsed)
            );
        }
        data
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        ENABLED.with(|on| on.set(false));
    }
}
fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}
pub(super) fn start() -> Option<Instant> {
    ENABLED.with(|on| on.get().then(Instant::now))
}
pub(super) fn phase(name: &str, timer: Option<Instant>) {
    let Some(timer) = timer else {
        return;
    };
    let elapsed = timer.elapsed();
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        match name {
            "preparation" => data.preparation += elapsed,
            "retention" => data.retention += elapsed,
            _ => unreachable!(),
        }
    });
}
pub(super) fn activation(timer: Option<Instant>, _frontier: usize, count: usize) {
    let Some(timer) = timer else {
        return;
    };
    let elapsed = timer.elapsed();
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.activation += elapsed;
        data.activated = count;
    });
}
pub(super) fn rule(name: &str, associated: bool, timer: Option<Instant>, count: usize) {
    let Some(timer) = timer else {
        return;
    };
    let elapsed = timer.elapsed();
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.matching += elapsed;
        data.evaluations += 1;
        data.associated_evaluations += usize::from(associated);
        data.candidates += count;
        let entry = data.rules.entry(name.to_string()).or_default();
        entry.0 += 1;
        entry.1 += count;
        entry.2 += elapsed;
    });
}
pub(super) fn probe(header_reject: bool) {
    if ENABLED.with(Cell::get) {
        if header_reject {
            HEADER_REJECTS.with(|c| c.set(c.get() + 1));
        } else {
            PROBES.with(|c| c.set(c.get() + 1));
        }
    }
}
pub(super) fn contains(seen: &[UnstampedDerivation], candidate: &UnstampedDerivation) -> bool {
    let Some(timer) = start() else {
        return seen.contains(candidate);
    };
    // Same ordered equality comparisons, with the same first-match exit as contains.
    let mut comparisons = 0;
    let found = seen.iter().any(|old| {
        comparisons += 1;
        old == candidate
    });
    let elapsed = timer.elapsed();
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.dedup += elapsed;
        data.comparisons += comparisons;
        data.duplicates += usize::from(found);
    });
    found
}
pub(super) fn round(known: usize, justifications: usize, frontier: usize) {
    if !ENABLED.with(Cell::get) {
        return;
    }
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.rounds += 1;
        // Flushed by the harness's nocapture: partial statistics survive a timeout.
        println!("PROFILE ROUND {} known={known} justifications={justifications} next_frontier={frontier} activated={} evaluations={} candidates={} duplicates={} comparisons={} probes={} header_rejects={} structural_rejects={} activation_ms={:.3} matching_ms={:.3} dedup_ms={:.3} retention_ms={:.3}",
            data.rounds, data.activated, data.evaluations, data.candidates, data.duplicates, data.comparisons, PROBES.with(Cell::get), HEADER_REJECTS.with(Cell::get), STRUCTURAL_REJECTS.with(Cell::get),
            ms(data.activation), ms(data.matching), ms(data.dedup), ms(data.retention));
    });
}

// Test-only selection of the old full-scan access path. All other evaluation,
// activation, scheduling and deduplication code is shared with the indexed path.
pub(super) fn scan_known() -> bool {
    SCAN_KNOWN.with(Cell::get)
}
pub(super) struct LinearScan(bool);
pub(super) fn linear_scan() -> LinearScan {
    LinearScan(SCAN_KNOWN.with(|mode| mode.replace(true)))
}
impl Drop for LinearScan {
    fn drop(&mut self) {
        SCAN_KNOWN.with(|mode| mode.set(self.0));
    }
}
pub(super) fn structural_reject() {
    if ENABLED.with(Cell::get) {
        STRUCTURAL_REJECTS.with(|c| c.set(c.get() + 1));
    }
}
