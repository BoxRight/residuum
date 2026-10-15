//! Finite analysis layer for the LLM-guided normative exploration contract
//! (`docs/llm-guided-normative-exploration-contract.md`, §6.1).
//!
//! Query and analysis only. Nothing here augments the Horn closure or changes
//! `close`, `Rule`, `Program`, `State`, defaults or the residual API. The
//! functions port the private harness helpers `finite_frontiers`,
//! `insert_minimal`, `universe`, `kernel`/`meet`/`finer` and `classify` with
//! their fixed sizes (universe 73, `N = 9`, head `compliance`, twelve body
//! literals) replaced by parameters, and add the frontier separation pool
//! that replaces power-set enumeration of states.
//!
//! Separation by frontiers (monotone closures): let `P` be the union of the
//! minimal supports of every candidate for a goal `q`. If `R_1 ⊄ R_2` there
//! is a frontier element `B` of candidate 1 outside `R_2`, since otherwise
//! `R_1 = ∪ ↑B ⊆ R_2`. That `B` lies in `P`, so two candidates have the same
//! region iff they agree on `P`.

use std::collections::{BTreeMap, BTreeSet};

use crate::derive::instantiate_from_known;
use crate::elab::Error;
use crate::typed::{
    Antecedent, Closure, Declaration, DeclarationOrigin, Event, Module, Polarity, Rule, RuleBody,
    Term, Type, Verb, VerbKind,
};

use super::Clock;

// ---------------------------------------------------------------------------
// Installation by literal addition (`P1-add`).

/// `T ⊕ r`: a private copy of `theory` with the supplied Derived verbs (when
/// absent) and rules appended. Nothing in `theory` is removed, guarded,
/// reprioritised or rewritten. Origins of the additions are recorded under
/// `origin`; the theory's origins are untouched.
pub fn install(
    theory: &Module,
    verbs: &[Verb],
    rules: &[Rule],
    origin: &str,
) -> Result<Module, Error> {
    let mut m = theory.clone();
    for v in verbs {
        match m.declarations.iter().find(|d| d.name() == v.name) {
            None => {
                if v.kind != VerbKind::Derived {
                    return Err(Error::new(format!(
                        "S_INSTALL: candidate verb `{}` must be Derived",
                        v.name
                    )));
                }
                if v.program.is_some() {
                    return Err(Error::new(format!(
                        "S_INSTALL: candidate verb `{}` must not declare a program",
                        v.name
                    )));
                }
                m.declarations.push(Declaration::Verb(v.clone()));
                m.origins.insert(
                    v.name.clone(),
                    DeclarationOrigin {
                        module: origin.into(),
                        file: None,
                    },
                );
            }
            Some(Declaration::Verb(existing)) if existing == v => {}
            Some(_) => {
                return Err(Error::new(format!(
                    "S_INSTALL: `{}` conflicts with a theory declaration",
                    v.name
                )));
            }
        }
    }
    for r in rules {
        if m.declarations.iter().any(|d| d.name() == r.name) {
            return Err(Error::new(format!(
                "S_INSTALL: `{}` is already declared",
                r.name
            )));
        }
        m.declarations.push(Declaration::Rule(r.clone()));
        m.origins.insert(
            r.name.clone(),
            DeclarationOrigin {
                module: origin.into(),
                file: None,
            },
        );
    }
    Ok(m)
}

// ---------------------------------------------------------------------------
// Decision contract `D` and evaluation of one candidate theory.

/// How a tuple of membership results compares with the external valuation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Criterion {
    /// `κ(z) = [z_i = 1]`: the named component must be derived.
    Component(usize),
    /// `κ(z) = [z = y*]` over every component.
    Exact(Vec<bool>),
}

/// `D_struct`: ground events whose membership in the closure is observed,
/// and the criterion that reads the tuple. Fixed per episode, never changed
/// by the explorer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub components: Vec<Event>,
    pub criterion: Criterion,
}

impl Decision {
    /// A misconfigured `D` is an error, never a normative verdict: a missing
    /// component or a `y*` of the wrong length must not read as `false`.
    pub fn validate(&self) -> Result<(), Error> {
        if self.components.is_empty() {
            return Err(Error::new("D: no components declared"));
        }
        match &self.criterion {
            Criterion::Component(i) if *i >= self.components.len() => Err(Error::new(format!(
                "D: component {i} does not exist; {} declared",
                self.components.len()
            ))),
            Criterion::Exact(expected) if expected.len() != self.components.len() => {
                Err(Error::new(format!(
                    "D: y* has {} entries for {} components",
                    expected.len(),
                    self.components.len()
                )))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Evaluation {
    pub components: Vec<bool>,
    pub compatible: bool,
    pub closure: Closure,
}

/// Full closure of `module` from `seeds`, read through `decision`. Rejects
/// non-terminating activation graphs with the runner's existing check.
pub fn evaluate(
    module: &Module,
    seeds: &[Event],
    decision: &Decision,
) -> Result<Evaluation, Error> {
    decision.validate()?;
    super::require_program_acyclic(module)?;
    let closure = crate::close_program(seeds.to_vec(), module, &mut Clock::default())?;
    let components: Vec<bool> = decision
        .components
        .iter()
        .map(|q| closure.known.contains(q))
        .collect();
    let compatible = match &decision.criterion {
        Criterion::Component(i) => components[*i],
        Criterion::Exact(expected) => &components == expected,
    };
    Ok(Evaluation {
        components,
        compatible,
        closure,
    })
}

/// Conservation certificate of an addition: outside the declared new heads,
/// the extended closure must have exactly the reference events and exactly
/// the reference justification multiset (rule, antecedents, substitution).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conservation {
    pub events_preserved: bool,
    pub proofs_preserved: bool,
    /// Declared new heads that actually appear.
    pub added_heads: BTreeSet<String>,
    /// New heads that were not declared as additions.
    pub foreign_heads: BTreeSet<String>,
}

impl Conservation {
    pub fn holds(&self) -> bool {
        self.events_preserved && self.proofs_preserved && self.foreign_heads.is_empty()
    }
}

/// Every declaration and origin of `theory` is present and identical in
/// `installed`, and every addition carries an origin outside the theory's
/// modules. This is the check `Study::theory` asserted, as a function.
pub fn theory_preserved(theory: &Module, installed: &Module) -> Result<(), Error> {
    for d in &theory.declarations {
        match installed.declarations.iter().find(|p| p.name() == d.name()) {
            Some(p) if p == d => {}
            Some(_) => {
                return Err(Error::new(format!(
                    "S_ORIGIN: declaration `{}` of the theory was altered",
                    d.name()
                )));
            }
            None => {
                return Err(Error::new(format!(
                    "S_ORIGIN: declaration `{}` of the theory is missing",
                    d.name()
                )));
            }
        }
        if installed.origin(d.name()) != theory.origin(d.name()) {
            return Err(Error::new(format!(
                "S_ORIGIN: origin of `{}` changed",
                d.name()
            )));
        }
    }
    let theory_modules: BTreeSet<&str> =
        theory.origins.values().map(|o| o.module.as_str()).collect();
    for d in &installed.declarations {
        if theory.declarations.iter().any(|p| p.name() == d.name()) {
            continue;
        }
        match installed.origin(d.name()) {
            Some(o) if !theory_modules.contains(o.module.as_str()) => {}
            Some(o) => {
                return Err(Error::new(format!(
                    "S_ORIGIN: addition `{}` claims theory module `{}`",
                    d.name(),
                    o.module
                )));
            }
            None => {
                return Err(Error::new(format!(
                    "S_ORIGIN: addition `{}` has no origin",
                    d.name()
                )));
            }
        }
    }
    Ok(())
}

/// Certified scope: `declared_new_heads` must be absent from the reference
/// closure. Stripping a head the reference already produces would hide the
/// very conclusions the certificate has to examine, so that case is an error
/// rather than a weaker certificate.
pub fn certify_addition(
    reference: &Closure,
    extended: &Closure,
    declared_new_heads: &BTreeSet<String>,
) -> Result<Conservation, Error> {
    fn strip(c: &Closure, heads: &BTreeSet<String>) -> (BTreeSet<String>, BTreeMap<String, usize>) {
        let events = c
            .known
            .iter()
            .filter(|e| !heads.contains(&e.verb))
            .map(|e| format!("{e:?}"))
            .collect();
        let mut proofs = BTreeMap::new();
        for d in c
            .derivations
            .iter()
            .filter(|d| !heads.contains(&d.event.verb))
        {
            *proofs
                .entry(format!("{:?}\t{:?}", d.event, d.record))
                .or_insert(0) += 1;
        }
        (events, proofs)
    }
    let reference_verbs: BTreeSet<&str> = reference.known.iter().map(|e| e.verb.as_str()).collect();
    if let Some(h) = declared_new_heads
        .iter()
        .find(|h| reference_verbs.contains(h.as_str()))
    {
        return Err(Error::new(format!(
            "certificate scope: `{h}` is already produced by the reference; \
             conservation under an existing head needs its own certificate"
        )));
    }
    let mut added_heads = BTreeSet::new();
    let mut foreign_heads = BTreeSet::new();
    for e in &extended.known {
        if !reference_verbs.contains(e.verb.as_str()) {
            if declared_new_heads.contains(&e.verb) {
                added_heads.insert(e.verb.clone());
            } else {
                foreign_heads.insert(e.verb.clone());
            }
        }
    }
    let (ref_events, ref_proofs) = strip(reference, declared_new_heads);
    let (ext_events, ext_proofs) = strip(extended, declared_new_heads);
    Ok(Conservation {
        events_preserved: ref_events == ext_events,
        proofs_preserved: ref_proofs == ext_proofs,
        added_heads,
        foreign_heads,
    })
}

// ---------------------------------------------------------------------------
// Finite universe and minimal supports.

pub type Support = BTreeSet<usize>;
pub type Supports = BTreeSet<Support>;

/// Keep `into` an antichain of minimal supports.
pub fn insert_minimal(into: &mut Supports, support: Support) {
    if into.iter().any(|s| s.is_subset(&support)) {
        return;
    }
    into.retain(|s| !support.is_subset(s));
    into.insert(support);
}

/// `E = C_module(Σ)`, deduplicated and sorted. Replaces the harness helper
/// that read `Σ` from the module's own experiments.
pub fn universe(module: &Module, sigma: &[Event]) -> Result<Vec<Event>, Error> {
    super::require_program_acyclic(module)?;
    let closure = crate::close_program(sigma.to_vec(), module, &mut Clock::default())?;
    Ok(closure
        .known
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

/// Joint universe `E = C_{T ⊕ all candidates}(Σ)`: closed under every
/// candidate separately, so per-candidate frontiers can share indices.
/// The joint module is an analysis artefact, never a theory.
pub fn joint_universe(
    theory: &Module,
    verbs: &[Verb],
    candidates: &[Rule],
    sigma: &[Event],
) -> Result<(Module, Vec<Event>), Error> {
    let joint = install(theory, verbs, candidates, "JointAnalysis")?;
    let e = universe(&joint, sigma)?;
    Ok((joint, e))
}

/// Indices of `e` whose verb is Seeded in `module`: the admissible evidence
/// leaves of a minimal-support analysis.
pub fn seeded_indices(module: &Module, e: &[Event]) -> BTreeSet<usize> {
    let seeded: BTreeSet<&str> = module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Verb(v) if v.kind == VerbKind::Seeded => Some(v.name.as_str()),
            _ => None,
        })
        .collect();
    e.iter()
        .enumerate()
        .filter(|(_, x)| seeded.contains(x.verb.as_str()))
        .map(|(i, _)| i)
        .collect()
}

pub fn goal_index(e: &[Event], goal: &Event) -> Option<usize> {
    e.iter().position(|x| x == goal)
}

/// A case's events as a state over `e`; `None` if some event lies outside.
pub fn state_index(e: &[Event], state: &[Event]) -> Option<Support> {
    state.iter().map(|x| goal_index(e, x)).collect()
}

pub fn state_events(e: &[Event], support: &Support) -> Vec<Event> {
    support.iter().map(|i| e[*i].clone()).collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontiers {
    pub ground_clauses: usize,
    /// For each index of the universe, its antichain of minimal supports over
    /// the allowed leaves. Empty when unreachable from allowed leaves.
    pub supports: Vec<Supports>,
}

impl Frontiers {
    pub fn reachable(&self) -> usize {
        self.supports.iter().filter(|s| !s.is_empty()).count()
    }
    pub fn total_supports(&self) -> usize {
        self.supports.iter().map(BTreeSet::len).sum()
    }
}

fn rules_of(module: &Module) -> Vec<&Rule> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some(r),
            _ => None,
        })
        .collect()
}

/// Minimal supports over the finite grounded hypergraph of `module` on `e`,
/// with `allowed` as identity leaves. This is the harness algorithm without
/// its fixed universe size. It neither adds rules nor changes `close`.
pub fn frontiers(
    module: &Module,
    e: &[Event],
    allowed: &BTreeSet<usize>,
) -> Result<Frontiers, Error> {
    if module.declarations.iter().any(|d| match d {
        Declaration::Program(_) | Declaration::Default(_) => true,
        Declaration::Verb(v) => v.program.is_some(),
        _ => false,
    }) {
        return Err(Error::new(
            "frontier analysis supports ordinary rules only; programs and defaults are out of scope",
        ));
    }
    let indices: BTreeMap<Event, usize> =
        e.iter().cloned().enumerate().map(|(i, x)| (x, i)).collect();
    if indices.len() != e.len() {
        return Err(Error::new("universe contains duplicate events"));
    }
    if let Some(i) = allowed.iter().find(|i| **i >= e.len()) {
        return Err(Error::new(format!(
            "allowed leaf {i} is outside the universe"
        )));
    }
    let mut ground: Vec<(usize, Vec<usize>)> = Vec::new();
    for rule in rules_of(module) {
        for d in instantiate_from_known(rule, e) {
            let head = *indices.get(&d.event).ok_or_else(|| {
                Error::new(format!("universe is not closed under rule `{}`", rule.name))
            })?;
            let mut premises = Vec::with_capacity(d.record.antecedents.len());
            for a in &d.record.antecedents {
                premises.push(*indices.get(a).ok_or_else(|| {
                    Error::new(format!(
                        "antecedent of rule `{}` lies outside the universe",
                        rule.name
                    ))
                })?);
            }
            ground.push((head, premises));
        }
    }
    let mut supports: Vec<Supports> = (0..e.len())
        .map(|i| {
            if allowed.contains(&i) {
                Supports::from([Support::from([i])])
            } else {
                Supports::new()
            }
        })
        .collect();
    loop {
        let before = supports.clone();
        for (head, premises) in &ground {
            let mut candidates = Supports::from([Support::new()]);
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
    Ok(Frontiers {
        ground_clauses: ground.len(),
        supports,
    })
}

// ---------------------------------------------------------------------------
// Regions, separation pool, partitions and discriminants.

/// `P`: every minimal support of every candidate, sorted. Agreement on `P`
/// is agreement of regions (module documentation).
pub fn region_pool<'a>(frontiers: impl IntoIterator<Item = &'a Supports>) -> Vec<Support> {
    frontiers
        .into_iter()
        .flat_map(|f| f.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// `S ∈ R` iff some minimal support is contained in `S`.
pub fn in_region(frontier: &Supports, state: &Support) -> bool {
    frontier.iter().any(|b| b.is_subset(state))
}

pub fn signature(frontier: &Supports, pool: &[Support]) -> Vec<bool> {
    pool.iter().map(|s| in_region(frontier, s)).collect()
}

/// `R_specific ⊆ R_general`: every minimal support of the specific candidate
/// contains one of the general candidate.
pub fn covers(general: &Supports, specific: &Supports) -> bool {
    specific
        .iter()
        .all(|b| general.iter().any(|a| a.is_subset(b)))
}

/// Partition kernel: class ids in order of first appearance.
pub fn kernel<T: Ord>(signatures: impl IntoIterator<Item = T>) -> Vec<usize> {
    let mut ids = BTreeMap::new();
    signatures
        .into_iter()
        .map(|s| {
            let n = ids.len();
            *ids.entry(s).or_insert(n)
        })
        .collect()
}

fn same_carrier(a: &[usize], b: &[usize]) -> Result<(), Error> {
    if a.len() != b.len() {
        return Err(Error::new(format!(
            "partition carriers differ: {} and {} hypotheses",
            a.len(),
            b.len()
        )));
    }
    Ok(())
}

/// Position-wise meet of two kernels over the same ordered hypotheses.
pub fn meet(a: &[usize], b: &[usize]) -> Result<Vec<usize>, Error> {
    same_carrier(a, b)?;
    let mut ids = BTreeMap::new();
    Ok(a.iter()
        .zip(b)
        .map(|(&x, &y)| {
            let n = ids.len();
            *ids.entry((x, y)).or_insert(n)
        })
        .collect())
}

/// `a` refines `b`: equal in `a` implies equal in `b`.
pub fn finer(a: &[usize], b: &[usize]) -> Result<bool, Error> {
    same_carrier(a, b)?;
    Ok((0..a.len()).all(|i| (0..a.len()).all(|j| a[i] != a[j] || b[i] == b[j])))
}

/// A partition that carries the identities of its hypotheses, so that two
/// partitions are only combined when they describe the same hypotheses in
/// the same order. This is the form the explorer's logs must use once
/// candidates are added over time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partition {
    pub hypotheses: Vec<String>,
    pub kernel: Vec<usize>,
}

impl Partition {
    pub fn new<T: Ord>(hypotheses: Vec<String>, signatures: Vec<T>) -> Result<Self, Error> {
        if hypotheses.len() != signatures.len() {
            return Err(Error::new(format!(
                "partition: {} hypotheses but {} signatures",
                hypotheses.len(),
                signatures.len()
            )));
        }
        let distinct: BTreeSet<&String> = hypotheses.iter().collect();
        if distinct.len() != hypotheses.len() {
            return Err(Error::new("partition: duplicate hypothesis identifiers"));
        }
        Ok(Self {
            kernel: kernel(signatures),
            hypotheses,
        })
    }
    fn same_hypotheses(&self, other: &Self) -> Result<(), Error> {
        if self.hypotheses != other.hypotheses {
            return Err(Error::new(
                "partition: hypotheses differ or are ordered differently",
            ));
        }
        Ok(())
    }
    pub fn meet(&self, other: &Self) -> Result<Self, Error> {
        self.same_hypotheses(other)?;
        Ok(Self {
            hypotheses: self.hypotheses.clone(),
            kernel: meet(&self.kernel, &other.kernel)?,
        })
    }
    pub fn finer(&self, other: &Self) -> Result<bool, Error> {
        self.same_hypotheses(other)?;
        finer(&self.kernel, &other.kernel)
    }
    pub fn classes(&self) -> usize {
        class_count(&self.kernel)
    }
}

pub fn class_count(kernel: &[usize]) -> usize {
    kernel.iter().collect::<BTreeSet<_>>().len()
}

/// Pool positions on which the signatures disagree: where an external
/// valuation would split the hypotheses.
pub fn discriminants(signatures: &[Vec<bool>]) -> Vec<usize> {
    let Some(first) = signatures.first() else {
        return Vec::new();
    };
    (0..first.len())
        .filter(|&j| signatures.iter().any(|s| s[j] != first[j]))
        .collect()
}

// ---------------------------------------------------------------------------
// Parametrised structural validator.

/// The finite profile a proposal must satisfy. Replaces the harness constants
/// `ALPHABET`, `MAX_ATOMS`, the `compliance` head and the `16` binder bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Verbs admitted as positive heads.
    pub heads: BTreeSet<String>,
    /// Verbs admitted in bodies.
    pub body_verbs: BTreeSet<String>,
    /// Body verbs that may appear with evidential negation.
    pub evidential_not_verbs: BTreeSet<String>,
    pub max_atoms: usize,
    pub max_binders: usize,
    pub max_after_depth: usize,
    /// Unit nodes are admitted only in an otherwise empty body.
    pub max_unit_nodes: usize,
    pub require_connected: bool,
}

impl Profile {
    /// `P1` of the exploration contract (§2): intermediate heads
    /// `obligation`, `permission`; output heads `compliance`, `violation`,
    /// `enforceability`, `give`, `do`, `notDo`; the twelve signed body
    /// literals with `~Act` as the only negative one. Connection is a search
    /// restriction the episode manifest may switch on.
    pub fn p1() -> Self {
        let heads = [
            "obligation",
            "permission",
            "compliance",
            "violation",
            "enforceability",
            "give",
            "do",
            "notDo",
        ];
        let body = [
            "give",
            "do",
            "notDo",
            "Act",
            "obligationFiled",
            "obligation",
            "right",
            "permissionFiled",
            "permission",
            "prohibition",
            "omissionProhibition",
        ];
        Self {
            heads: heads.iter().map(|s| s.to_string()).collect(),
            body_verbs: body.iter().map(|s| s.to_string()).collect(),
            evidential_not_verbs: BTreeSet::from(["Act".to_string()]),
            max_atoms: 4,
            max_binders: 16,
            max_after_depth: 2,
            max_unit_nodes: 1,
            require_connected: false,
        }
    }

    /// Add a problem manifest: its experimental outputs as heads, its body
    /// literals and the verbs it admits under evidential negation.
    pub fn with_problem<'a>(
        mut self,
        heads: impl IntoIterator<Item = &'a str>,
        body_verbs: impl IntoIterator<Item = &'a str>,
        evidential_not_verbs: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        self.heads.extend(heads.into_iter().map(str::to_string));
        self.body_verbs
            .extend(body_verbs.into_iter().map(str::to_string));
        self.evidential_not_verbs
            .extend(evidential_not_verbs.into_iter().map(str::to_string));
        self
    }

    /// The harness profile of the structural generation certificate.
    pub fn legacy_compliance() -> Self {
        let body = [
            "give",
            "do",
            "notDo",
            "Act",
            "obligationFiled",
            "obligation",
            "right",
            "permissionFiled",
            "permission",
            "prohibition",
            "omissionProhibition",
        ];
        Self {
            heads: BTreeSet::from(["compliance".to_string()]),
            body_verbs: body.iter().map(|s| s.to_string()).collect(),
            evidential_not_verbs: BTreeSet::from(["Act".to_string()]),
            max_atoms: 4,
            max_binders: 16,
            max_after_depth: 2,
            max_unit_nodes: 1,
            require_connected: true,
        }
    }
}

fn atoms<'a>(a: &'a Antecedent, output: &mut Vec<&'a Event>) {
    match a {
        Antecedent::Unit => (),
        Antecedent::Event(e) => output.push(e),
        Antecedent::And(l, r) => {
            atoms(l, output);
            atoms(r, output);
        }
    }
}

fn node_counts(a: &Antecedent) -> (usize, usize) {
    match a {
        Antecedent::Unit => (1, 0),
        Antecedent::Event(_) => (0, 0),
        Antecedent::And(l, r) => {
            let (ul, al) = node_counts(l);
            let (ur, ar) = node_counts(r);
            (ul + ur, al + ar + 1)
        }
    }
}

fn vars(t: &Term, result: &mut BTreeSet<String>) {
    match t {
        Term::Var(n) => {
            result.insert(n.clone());
        }
        Term::Constructor { argument, .. } => vars(argument, result),
        Term::RecordLiteral { fields, .. } => {
            for f in fields {
                vars(&f.value, result);
            }
        }
        _ => (),
    }
}

fn event_vars(e: &Event) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for t in &e.args {
        vars(t, &mut out);
    }
    vars(&e.proposition.time.to_term(), &mut out);
    out
}

fn depth(t: &Term) -> usize {
    match t {
        Term::Constructor { argument, .. } => 1 + depth(argument),
        _ => 0,
    }
}

fn assignable(actual: &Type, expected: &Type) -> bool {
    actual == expected
        || match expected {
            Type::Optional(inner) => assignable(actual, inner),
            _ => false,
        }
}

fn term_check(
    t: &Term,
    expected: &Type,
    scope: &BTreeMap<String, Type>,
    cs: &BTreeMap<String, Type>,
    profile: &Profile,
    reasons: &mut BTreeSet<String>,
) {
    if depth(t) > profile.max_after_depth {
        reasons.insert("B_AFTER_DEPTH".into());
    }
    match t {
        Term::None => {
            if !matches!(expected, Type::Optional(_)) {
                reasons.insert("S_TYPE".into());
            }
        }
        Term::Var(n) | Term::Const(n) => {
            let actual = if matches!(t, Term::Var(_)) {
                scope.get(n)
            } else {
                cs.get(n)
            };
            match actual {
                None => {
                    reasons.insert("S_UNKNOWN_TERM".into());
                }
                Some(ty) if !assignable(ty, expected) => {
                    reasons.insert("S_TYPE".into());
                }
                _ => (),
            }
        }
        Term::Constructor { argument, .. } => {
            if !assignable(&Type::PropositionTime, expected) {
                reasons.insert("S_TYPE".into());
            }
            term_check(
                argument,
                &Type::PropositionTime,
                scope,
                cs,
                profile,
                reasons,
            );
        }
        Term::RecordLiteral { .. } => {
            reasons.insert("B_RECORD_DEPTH".into());
        }
    }
}

fn identities(e: &Event) -> BTreeSet<String> {
    fn visit(t: &Term, out: &mut BTreeSet<String>) {
        match t {
            Term::Var(n) => {
                out.insert(format!("var:{n}"));
            }
            Term::Const(n) => {
                out.insert(format!("const:{n}"));
            }
            Term::Constructor { argument, .. } => visit(argument, out),
            Term::RecordLiteral { fields, .. } => {
                for f in fields {
                    visit(&f.value, out);
                }
            }
            Term::None => (),
        }
    }
    let mut out = BTreeSet::new();
    for t in &e.args {
        visit(t, &mut out);
    }
    visit(&e.proposition.time.to_term(), &mut out);
    out
}

fn constants(m: &Module) -> BTreeMap<String, Type> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Const(c) => Some((c.name.clone(), c.ty.clone())),
            _ => None,
        })
        .collect()
}

/// Structural admission codes of `r` against `profile` in the vocabulary of
/// `m`. Empty means admitted. This is interface validation of a record-free,
/// unsubtyped Horn profile; it neither implements theta-subsumption nor
/// assesses legal adequacy.
pub fn classify(m: &Module, r: &Rule, profile: &Profile) -> BTreeSet<String> {
    let mut reasons = BTreeSet::new();
    let RuleBody::HornClause(clause) = &r.body else {
        reasons.insert("S_SYNTAX".into());
        return reasons;
    };
    let mut input = Vec::new();
    atoms(&clause.antecedent, &mut input);
    let (unit_nodes, and_nodes) = node_counts(&clause.antecedent);
    if input.len() > profile.max_atoms {
        reasons.insert("B_ATOMS".into());
    }
    if unit_nodes > profile.max_unit_nodes || (unit_nodes > 0 && !input.is_empty()) {
        reasons.insert("B_UNIT_NODES".into());
    }
    if and_nodes != input.len().saturating_sub(1) {
        reasons.insert("B_AND_NODES".into());
    }
    if r.parameters.len() > profile.max_binders {
        reasons.insert("B_VARIABLES".into());
    }
    let scope: BTreeMap<_, _> = r
        .parameters
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    if scope.len() != r.parameters.len() {
        reasons.insert("S_DUPLICATE_BINDER".into());
    }
    let cs = constants(m);
    let mut available = BTreeSet::new();
    for e in &input {
        available.extend(event_vars(e));
    }
    if !event_vars(&clause.consequent).is_subset(&available) {
        reasons.insert("S_UNBOUND_HEAD".into());
    }
    if !profile.heads.contains(&clause.consequent.verb)
        || clause.consequent.polarity != Polarity::Positive
    {
        reasons.insert("P_HEAD_INTERFACE".into());
    }
    for (head, e) in input
        .iter()
        .map(|e| (false, *e))
        .chain(std::iter::once((true, &clause.consequent)))
    {
        let Some(v) = m.declarations.iter().find_map(|d| match d {
            Declaration::Verb(v) if v.name == e.verb => Some(v),
            _ => None,
        }) else {
            reasons.insert("S_UNKNOWN_VERB".into());
            continue;
        };
        if !head && !profile.body_verbs.contains(&e.verb) {
            reasons.insert("P_BODY_VOCABULARY".into());
        }
        if e.polarity == Polarity::EvidentialNot
            && (head || !profile.evidential_not_verbs.contains(&e.verb))
        {
            reasons.insert("P_POLARITY".into());
        }
        if e.args.len() != v.args.len() {
            reasons.insert("S_ARITY".into());
        }
        for (t, a) in e.args.iter().zip(&v.args) {
            term_check(t, &a.ty, &scope, &cs, profile, &mut reasons);
        }
        term_check(
            &e.proposition.time.to_term(),
            &Type::PropositionTime,
            &scope,
            &cs,
            profile,
            &mut reasons,
        );
    }
    if profile.require_connected {
        let mut reached = identities(&clause.consequent);
        let mut remaining: Vec<_> = input.iter().map(|e| identities(e)).collect();
        loop {
            let before = remaining.len();
            remaining.retain(|xs| {
                if xs.is_disjoint(&reached) {
                    true
                } else {
                    reached.extend(xs.clone());
                    false
                }
            });
            if before == remaining.len() {
                break;
            }
        }
        if !remaining.is_empty() {
            reasons.insert("S_DISCONNECTED".into());
        }
    }
    reasons
}

/// Admission of a whole candidate configuration: at most `max_clauses`
/// rules, each admitted by `classify` where the candidate's own intermediate
/// heads count as body vocabulary, and no chain longer than one intermediate
/// clause feeding one final clause (`B_CLAUSES`).
pub fn classify_candidate(
    m: &Module,
    rules: &[Rule],
    profile: &Profile,
    max_clauses: usize,
) -> BTreeSet<String> {
    let mut reasons = BTreeSet::new();
    if rules.is_empty() || rules.len() > max_clauses {
        reasons.insert("B_CLAUSES".into());
    }
    let heads: Vec<Option<&str>> = rules
        .iter()
        .map(|r| match &r.body {
            RuleBody::HornClause(c) => Some(c.consequent.verb.as_str()),
            RuleBody::ResidualClause(_) => None,
        })
        .collect();
    let mut extended = profile.clone();
    for h in heads.iter().flatten() {
        if profile.heads.contains(*h) {
            extended.body_verbs.insert((*h).to_string());
        }
    }
    for r in rules {
        reasons.extend(classify(m, r, &extended));
    }
    let body_verbs: Vec<BTreeSet<&str>> = rules
        .iter()
        .map(|r| match &r.body {
            RuleBody::HornClause(c) => {
                let mut atoms_in = Vec::new();
                atoms(&c.antecedent, &mut atoms_in);
                atoms_in.iter().map(|e| e.verb.as_str()).collect()
            }
            RuleBody::ResidualClause(_) => BTreeSet::new(),
        })
        .collect();
    // feeds[i][j]: clause i's head appears in clause j's body.
    let feeds = |i: usize, j: usize| heads[i].is_some_and(|h| body_verbs[j].contains(h));
    let n = rules.len();
    for i in 0..n {
        for j in 0..n {
            if feeds(i, j) && (i == j || (0..n).any(|k| feeds(j, k))) {
                reasons.insert("B_CLAUSES".into());
            }
        }
    }
    reasons
}
