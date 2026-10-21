//! Pilot 004-R generative trajectory (`docs/pilot-004r-generative-contract.md`):
//! a learner proposes Formal rules over the fixed theory `O`, the harness
//! admits them through the existing proposal stages, certifies every
//! extension against `O`, applies the committed hypotheses to dossiers in a
//! fixed order, scores each dossier before its feedback (prequential), and
//! delivers documented discrepancies only in the feedback condition.
//!
//! Two pieces are new here and belong to the harness, not to the engine:
//! - `check_closure`: an independent naive checker of a completed closure
//!   (every justification is a valid, well-founded rule instance; every
//!   known event is supported; no rule instance over Known is missing);
//! - `certify_attributed`: the `attributed_extension` certificate, which
//!   requires both checked closures, monotonicity over `O`, and attribution
//!   of every new event and justification to the learned rules.
//!
//! The learner never receives `ReservedValuations`: it sees seeds, closures,
//! derivations, its own hypotheses and, only under `Condition::Feedback`,
//! the discrepancy view computed after each commitment.
//!
//! Navigation: the learner may also send the existing worker requests
//! (`worker::Request`: theory inventory, forward and residual sessions,
//! minimal supports, partition comparison) to a `worker::Worker` that the
//! harness rebuilds at each new dossier over the presented dossiers only and
//! over the admitted hypotheses (each as its own `O ∪ h` theory). The worker
//! never sees the reserved valuations. Hypothesis identifiers are translated
//! to and from the worker's candidate identifiers.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::elab::Error;
use crate::typed::{
    Antecedent, Closure, Declaration, Event, Module, Polarity, PropositionType, Rule, RuleBody,
    Substitution, SubstitutionValue, Term, TimeExpr,
};

use super::Clock;
use super::exploration::{Criterion, Decision, Profile, install, theory_preserved};
use super::proposal::{
    Episode, ProposalContext, SourceProposal, admit, compose, relative, status_label,
};
use super::worker;

// ---------------------------------------------------------------------------
// Independent checker of a completed closure.

/// Result of `check_closure`. Each flag is a separate obligation; `problems`
/// names the first offending items for the record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureCheck {
    /// Every justification instantiates its rule exactly under its recorded
    /// substitution, and is grounded in the seeds (well-founded).
    pub sound: bool,
    /// Every seed is known and every known non-seed event has a justification.
    pub supported: bool,
    /// No rule instance over Known yields an event outside Known.
    pub event_complete: bool,
    /// Every rule instance over Known appears as a justification.
    pub justification_complete: bool,
    pub problems: Vec<String>,
}

impl ClosureCheck {
    pub fn holds(&self) -> bool {
        self.sound && self.supported && self.event_complete && self.justification_complete
    }
}

type Binding = BTreeMap<String, SubstitutionValue>;
type Justification = (String, Vec<Event>, Event);

fn atoms<'a>(a: &'a Antecedent, out: &mut Vec<&'a Event>) {
    match a {
        Antecedent::Unit => {}
        Antecedent::Event(e) => out.push(e),
        Antecedent::And(l, r) => {
            atoms(l, out);
            atoms(r, out);
        }
    }
}

fn bind(b: &mut Binding, name: &str, value: SubstitutionValue) -> bool {
    match b.get(name) {
        Some(old) => *old == value,
        None => {
            b.insert(name.to_string(), value);
            true
        }
    }
}

fn match_term(p: &Term, f: &Term, b: &mut Binding) -> Result<bool, Error> {
    Ok(match (p, f) {
        (Term::Var(x), _) => bind(b, x, SubstitutionValue::Term(f.clone())),
        (Term::Const(a), Term::Const(c)) => a == c,
        (Term::None, Term::None) => true,
        (
            Term::Constructor {
                constructor: c1,
                argument: a1,
            },
            Term::Constructor {
                constructor: c2,
                argument: a2,
            },
        ) => c1 == c2 && match_term(a1, a2, b)?,
        (Term::RecordLiteral { .. }, _) => {
            return Err(Error::new(
                "checker: record literal patterns are outside the checked fragment",
            ));
        }
        _ => false,
    })
}

fn match_time(p: &TimeExpr, f: &TimeExpr, b: &mut Binding) -> bool {
    match (p, f) {
        // One representation for the checker's own bindings: a temporal value
        // is encoded as its term, so a parameter used in both positions agrees.
        (TimeExpr::Var(x), _) => bind(b, x, SubstitutionValue::Term(f.to_term())),
        (TimeExpr::At(a), TimeExpr::At(c)) => a == c,
        (TimeExpr::After(p1), TimeExpr::After(f1)) => match_time(p1, f1, b),
        _ => false,
    }
}

fn match_event(p: &Event, f: &Event, b: &mut Binding) -> Result<bool, Error> {
    if p.verb != f.verb || p.polarity != f.polarity || p.args.len() != f.args.len() {
        return Ok(false);
    }
    for (pt, ft) in p.args.iter().zip(&f.args) {
        if !match_term(pt, ft, b)? {
            return Ok(false);
        }
    }
    Ok(match_time(&p.proposition.time, &f.proposition.time, b))
}

fn instantiate_term(t: &Term, b: &Binding) -> Option<Term> {
    match t {
        // The engine stores a PropositionTime parameter as a time value even
        // where it fills an argument position; both encodings are accepted.
        Term::Var(x) => match b.get(x) {
            Some(SubstitutionValue::Term(v)) => Some(v.clone()),
            Some(SubstitutionValue::Time(v)) => Some(v.to_term()),
            None => None,
        },
        Term::Const(_) | Term::None => Some(t.clone()),
        Term::Constructor {
            constructor,
            argument,
        } => Some(Term::Constructor {
            constructor: *constructor,
            argument: Box::new(instantiate_term(argument, b)?),
        }),
        Term::RecordLiteral { .. } => None,
    }
}

fn instantiate_time(t: &TimeExpr, b: &Binding) -> Option<TimeExpr> {
    match t {
        TimeExpr::Var(x) => match b.get(x) {
            Some(SubstitutionValue::Time(v)) => Some(v.clone()),
            Some(SubstitutionValue::Term(v)) => TimeExpr::from_term(v),
            None => None,
        },
        TimeExpr::At(_) => Some(t.clone()),
        TimeExpr::After(inner) => Some(TimeExpr::After(Box::new(instantiate_time(inner, b)?))),
    }
}

fn instantiate_event(e: &Event, b: &Binding) -> Option<Event> {
    let args = e
        .args
        .iter()
        .map(|t| instantiate_term(t, b))
        .collect::<Option<Vec<_>>>()?;
    Some(Event {
        polarity: e.polarity,
        verb: e.verb.clone(),
        args,
        proposition: PropositionType {
            kind: e.proposition.kind.clone(),
            time: instantiate_time(&e.proposition.time, b)?,
        },
        transition: e.transition.clone(),
    })
}

fn substitution_binding(s: &Substitution) -> Binding {
    s.bindings
        .iter()
        .map(|b| (b.parameter.clone(), b.value.clone()))
        .collect()
}

/// Horn rules of the module, refusing fragments the checker does not cover.
fn horn_rules(module: &Module) -> Result<Vec<(&Rule, Vec<&Event>, &Event)>, Error> {
    let mut out = Vec::new();
    for d in &module.declarations {
        match d {
            Declaration::Default(_) => {
                return Err(Error::new(
                    "checker: defaults are outside the checked fragment",
                ));
            }
            Declaration::Verb(v) if v.program.is_some() => {
                return Err(Error::new(format!(
                    "checker: verb program `{}` is outside the checked fragment",
                    v.name
                )));
            }
            Declaration::Rule(r) => match &r.body {
                RuleBody::HornClause(c) => {
                    let mut body = Vec::new();
                    atoms(&c.antecedent, &mut body);
                    out.push((r, body, &c.consequent));
                }
                RuleBody::ResidualClause(_) => {
                    return Err(Error::new(format!(
                        "checker: residual clause `{}` is outside the checked fragment",
                        r.name
                    )));
                }
            },
            _ => {}
        }
    }
    Ok(out)
}

fn enumerate(
    body: &[&Event],
    by_verb: &BTreeMap<&str, Vec<&Event>>,
    binding: Binding,
    chosen: &mut Vec<Event>,
    out: &mut Vec<(Binding, Vec<Event>)>,
) -> Result<(), Error> {
    let Some((first, rest)) = body.split_first() else {
        out.push((binding, chosen.clone()));
        return Ok(());
    };
    for fact in by_verb.get(first.verb.as_str()).into_iter().flatten() {
        let mut b = binding.clone();
        if match_event(first, fact, &mut b)? {
            chosen.push((*fact).clone());
            enumerate(rest, by_verb, b, chosen, out)?;
            chosen.pop();
        }
    }
    Ok(())
}

/// Check a completed closure of `module` over `seeds` without reusing the
/// engine's matcher or closure loop. An unsupported fragment is an error
/// (pending), never a failed check.
pub fn check_closure(
    module: &Module,
    seeds: &[Event],
    closure: &Closure,
) -> Result<ClosureCheck, Error> {
    let rules = horn_rules(module)?;
    let by_name: BTreeMap<&str, (&Rule, &Vec<&Event>, &Event)> = rules
        .iter()
        .map(|(r, body, head)| (r.name.as_str(), (*r, body, *head)))
        .collect();
    let known: BTreeSet<&Event> = closure.known.iter().collect();
    let seed_set: BTreeSet<&Event> = seeds.iter().collect();
    let mut problems = Vec::new();

    // Soundness: each record is an exact instance of its rule.
    let mut sound = true;
    for d in &closure.derivations {
        let Some((_, body, head)) = by_name.get(d.record.rule.as_str()) else {
            sound = false;
            problems.push(format!("justification by unknown rule `{}`", d.record.rule));
            continue;
        };
        let b = substitution_binding(&d.record.substitution);
        let instance: Option<Vec<Event>> = body.iter().map(|a| instantiate_event(a, &b)).collect();
        let consequent = instantiate_event(head, &b);
        if instance.as_ref() != Some(&d.record.antecedents) || consequent.as_ref() != Some(&d.event)
        {
            sound = false;
            problems.push(format!(
                "justification of {:?} by `{}` is not an instance under its substitution",
                d.event, d.record.rule
            ));
        }
        if !known.contains(&d.event) || d.record.antecedents.iter().any(|a| !known.contains(a)) {
            sound = false;
            problems.push(format!(
                "justification of {:?} mentions an unknown event",
                d.event
            ));
        }
    }
    // Well-foundedness, independent of the order of the records.
    let mut grounded: BTreeSet<&Event> = seed_set.clone();
    loop {
        let before = grounded.len();
        for d in &closure.derivations {
            if d.record.antecedents.iter().all(|a| grounded.contains(a)) {
                grounded.insert(&d.event);
            }
        }
        if grounded.len() == before {
            break;
        }
    }
    for d in &closure.derivations {
        if !grounded.contains(&d.event) {
            sound = false;
            problems.push(format!("{:?} has no well-founded justification", d.event));
        }
    }

    // Support.
    let justified: BTreeSet<&Event> = closure.derivations.iter().map(|d| &d.event).collect();
    let mut supported = true;
    for s in seeds {
        if !known.contains(s) {
            supported = false;
            problems.push(format!("seed {s:?} is not known"));
        }
    }
    for e in &closure.known {
        if !seed_set.contains(e) && !justified.contains(e) {
            supported = false;
            problems.push(format!("known {e:?} is neither a seed nor justified"));
        }
    }

    // Completeness: every rule instance over Known.
    let mut by_verb: BTreeMap<&str, Vec<&Event>> = BTreeMap::new();
    for e in &closure.known {
        by_verb.entry(e.verb.as_str()).or_default().push(e);
    }
    let recorded: BTreeSet<Justification> = closure
        .derivations
        .iter()
        .map(|d| {
            (
                d.record.rule.clone(),
                d.record.antecedents.clone(),
                d.event.clone(),
            )
        })
        .collect();
    let mut event_complete = true;
    let mut justification_complete = true;
    for (rule, body, head) in &rules {
        let mut found = Vec::new();
        enumerate(body, &by_verb, Binding::new(), &mut Vec::new(), &mut found)?;
        for (b, chosen) in found {
            let Some(consequent) = instantiate_event(head, &b) else {
                return Err(Error::new(format!(
                    "checker: head of `{}` is not ground under a body match",
                    rule.name
                )));
            };
            if !known.contains(&consequent) {
                event_complete = false;
                problems.push(format!(
                    "`{}` yields {consequent:?} outside Known",
                    rule.name
                ));
            }
            if !recorded.contains(&(rule.name.clone(), chosen, consequent.clone())) {
                justification_complete = false;
                problems.push(format!(
                    "instance of `{}` for {consequent:?} is not recorded",
                    rule.name
                ));
            }
        }
    }
    problems.truncate(20);
    Ok(ClosureCheck {
        sound,
        supported,
        event_complete,
        justification_complete,
        problems,
    })
}

/// Learned rules each event depends on: the union, over its justifications,
/// of the learned rule used and the dependencies of its antecedents.
pub fn learned_dependencies(
    closure: &Closure,
    learned: &BTreeSet<String>,
) -> BTreeMap<Event, BTreeSet<String>> {
    let mut deps: BTreeMap<Event, BTreeSet<String>> = BTreeMap::new();
    loop {
        let mut changed = false;
        for d in &closure.derivations {
            let mut s = BTreeSet::new();
            if learned.contains(&d.record.rule) {
                s.insert(d.record.rule.clone());
            }
            for a in &d.record.antecedents {
                if let Some(x) = deps.get(a) {
                    s.extend(x.iter().cloned());
                }
            }
            let entry = deps.entry(d.event.clone()).or_default();
            let before = entry.len();
            entry.extend(s);
            changed |= entry.len() != before;
        }
        if !changed {
            break;
        }
    }
    deps.retain(|_, s| !s.is_empty());
    deps
}

// ---------------------------------------------------------------------------
// `attributed_extension` certificate.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEvent {
    pub event: Event,
    /// Learned rules that justify the event directly.
    pub direct: BTreeSet<String>,
    /// Rules of `O` that justify it from new antecedents.
    pub chained: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributedCertificate {
    pub reference: ClosureCheck,
    pub extended: ClosureCheck,
    /// `O` is unchanged and the only additions are the learned rules.
    pub theory_preserved: bool,
    pub monotone_events: bool,
    pub monotone_justifications: bool,
    /// Every new justification uses a learned rule or a new antecedent.
    pub attribution: bool,
    pub new_events: Vec<NewEvent>,
    /// New justifications of events `O` already derives (redundant support).
    pub new_justifications_of_existing: Vec<(Event, String)>,
    /// Rules of `O` that fire on consequences of the learned rules.
    pub activated_base_rules: BTreeSet<String>,
    pub problems: Vec<String>,
}

impl AttributedCertificate {
    pub fn holds(&self) -> bool {
        self.reference.holds()
            && self.extended.holds()
            && self.theory_preserved
            && self.monotone_events
            && self.monotone_justifications
            && self.attribution
    }
}

fn close(module: &Module, seeds: &[Event]) -> Result<Closure, Error> {
    super::require_program_acyclic(module)?;
    crate::close_program(seeds.to_vec(), module, &mut Clock::default())
}

/// Certify `extended = reference ⊕ learned` over `seeds`. A closure that does
/// not complete, or a fragment outside the checker, is an error: a pending
/// status, never a refutation.
pub fn certify_attributed(
    reference_module: &Module,
    extended_module: &Module,
    learned: &BTreeSet<String>,
    seeds: &[Event],
) -> Result<(AttributedCertificate, Closure), Error> {
    let reference = close(reference_module, seeds)?;
    let extended = close(extended_module, seeds)?;
    let reference_check = check_closure(reference_module, seeds, &reference)?;
    let extended_check = check_closure(extended_module, seeds, &extended)?;
    let mut problems = Vec::new();

    let additions: BTreeSet<String> = extended_module
        .declarations
        .iter()
        .filter(|d| {
            !reference_module
                .declarations
                .iter()
                .any(|r| r.name() == d.name())
        })
        .map(|d| d.name().to_string())
        .collect();
    let only_rules = extended_module.declarations.iter().all(|d| {
        !additions.contains(d.name())
            || matches!(d, Declaration::Rule(r) if learned.contains(&r.name))
    });
    let preserved = theory_preserved(reference_module, extended_module).is_ok()
        && only_rules
        && additions == *learned;
    if !preserved {
        problems.push(format!(
            "additions {additions:?} differ from learned {learned:?}"
        ));
    }

    let ref_known: BTreeSet<&Event> = reference.known.iter().collect();
    let ext_known: BTreeSet<&Event> = extended.known.iter().collect();
    let monotone_events = ref_known.is_subset(&ext_known);
    let key = |d: &crate::typed::DerivedEvent| -> Justification {
        (
            d.record.rule.clone(),
            d.record.antecedents.clone(),
            d.event.clone(),
        )
    };
    let ref_just: BTreeSet<Justification> = reference.derivations.iter().map(key).collect();
    let ext_just: BTreeSet<Justification> = extended.derivations.iter().map(key).collect();
    let monotone_justifications = ref_just.is_subset(&ext_just);
    if !monotone_events || !monotone_justifications {
        problems.push("the extension loses events or justifications of O".into());
    }

    let mut attribution = true;
    let mut new_events: BTreeMap<Event, NewEvent> = BTreeMap::new();
    let mut redundant = Vec::new();
    let mut activated = BTreeSet::new();
    for (rule, antecedents, event) in ext_just.difference(&ref_just) {
        let direct = learned.contains(rule);
        let fresh_antecedent = antecedents.iter().any(|a| !ref_known.contains(a));
        if !direct && !fresh_antecedent {
            attribution = false;
            problems.push(format!(
                "unattributed justification of {event:?} by `{rule}`"
            ));
        }
        if !direct {
            activated.insert(rule.clone());
        }
        if ref_known.contains(event) {
            redundant.push((event.clone(), rule.clone()));
        } else {
            let entry = new_events.entry(event.clone()).or_insert_with(|| NewEvent {
                event: event.clone(),
                direct: BTreeSet::new(),
                chained: BTreeSet::new(),
            });
            if direct {
                entry.direct.insert(rule.clone());
            } else {
                entry.chained.insert(rule.clone());
            }
        }
    }
    problems.truncate(20);
    Ok((
        AttributedCertificate {
            reference: reference_check,
            extended: extended_check,
            theory_preserved: preserved,
            monotone_events,
            monotone_justifications,
            attribution,
            new_events: new_events.into_values().collect(),
            new_justifications_of_existing: redundant,
            activated_base_rules: activated,
            problems,
        },
        extended,
    ))
}

// ---------------------------------------------------------------------------
// Positions, reserved valuations and loss.

/// Normative heads with the seven-argument form whose projection is
/// `(head, subject, conduct, object, beneficiary)`.
pub const PROJECTED_HEADS: [&str; 5] = [
    "obligation",
    "permission",
    "prohibition",
    "omissionProhibition",
    "right",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub head: String,
    pub subject: String,
    pub conduct: String,
    pub object: String,
    pub beneficiary: String,
}

pub fn project(e: &Event) -> Option<Position> {
    if e.polarity != Polarity::Positive
        || !PROJECTED_HEADS.contains(&e.verb.as_str())
        || e.args.len() != 7
    {
        return None;
    }
    let c = |i: usize| match &e.args[i] {
        Term::Const(s) => Some(s.clone()),
        _ => None,
    };
    Some(Position {
        head: e.verb.clone(),
        subject: c(1)?,
        conduct: c(2)?,
        object: c(3)?,
        beneficiary: c(4)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    Required,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Valuation {
    pub dossier: String,
    pub id: String,
    pub position: Position,
    pub label: Label,
    pub status: String,
    pub source: String,
}

/// Held by the trajectory's evaluator only; never part of a learner view.
#[derive(Debug, Clone)]
pub struct ReservedValuations {
    valuations: Vec<Valuation>,
}

impl ReservedValuations {
    pub fn new(valuations: Vec<Valuation>) -> Result<Self, Error> {
        let mut seen = BTreeMap::new();
        for v in &valuations {
            if let Some(other) = seen.insert((v.dossier.clone(), v.position.clone()), v.label) {
                if other != v.label {
                    return Err(Error::new(format!(
                        "valuation {}: the same position is required and excluded",
                        v.id
                    )));
                }
            }
        }
        Ok(Self { valuations })
    }

    /// `dossier id head subject conduct object beneficiary label status source`.
    pub fn load_tsv(path: &std::path::Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::new(format!("cannot read `{}`: {e}", path.display())))?;
        let mut valuations = Vec::new();
        for (n, line) in text.lines().enumerate().skip(1) {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() != 10 {
                return Err(Error::new(format!(
                    "valuations line {}: 10 fields expected",
                    n + 1
                )));
            }
            let label = match f[7] {
                "required" => Label::Required,
                "excluded" => Label::Excluded,
                other => {
                    return Err(Error::new(format!(
                        "valuations line {}: label `{other}`",
                        n + 1
                    )));
                }
            };
            valuations.push(Valuation {
                dossier: f[0].into(),
                id: f[1].into(),
                position: Position {
                    head: f[2].into(),
                    subject: f[3].into(),
                    conduct: f[4].into(),
                    object: f[5].into(),
                    beneficiary: f[6].into(),
                },
                label,
                status: f[8].into(),
                source: f[9].into(),
            });
        }
        Self::new(valuations)
    }

    fn of(&self, dossier: &str) -> impl Iterator<Item = &Valuation> {
        self.valuations.iter().filter(move |v| v.dossier == dossier)
    }
}

/// One valued position on which a hypothesis contributed a derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exposure {
    pub dossier: String,
    pub valuation: String,
    /// Recorded at commitment, before the dossier's feedback.
    pub prequential: bool,
    /// Required and derived (true) or excluded and derived (false).
    pub correct: bool,
}

/// Evaluator-side record of one scoring of a dossier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scoring {
    pub dossier: String,
    pub committed: Vec<String>,
    pub loss: usize,
    pub missed: Vec<Valuation>,
    pub excess: Vec<(Valuation, BTreeSet<String>)>,
    pub valued: usize,
    pub unvalued_derived: usize,
    pub exposures: Vec<(String, Exposure)>,
}

// ---------------------------------------------------------------------------
// Learner interface.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    /// The discrepancy view of each dossier is delivered after commitment.
    Feedback,
    /// Same budgets and tools; only "dossier evaluated" is delivered.
    NoFeedback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Explore,
    Revise,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnerAction {
    Propose {
        text: String,
        parent: Option<String>,
        rationale: String,
    },
    Evaluate {
        hypothesis: String,
        dossier: String,
    },
    Commit {
        hypotheses: Vec<String>,
        /// Traceability only: recorded, never checked.
        reason: Option<String>,
    },
    /// Record that a hypothesis is set aside and why. Traceability only:
    /// it executes nothing and does not change the committed set.
    Discard {
        hypothesis: String,
        reason: String,
    },
    /// One existing worker request; hypothesis identifiers stand for the
    /// worker's candidates. Proposal and candidate evaluation go through
    /// `Propose` and `Evaluate` instead.
    Navigate {
        request: worker::Request,
    },
    /// An investigable question and the tool the learner plans to answer
    /// it with. Recorded; it executes nothing.
    Question {
        question: String,
        tool: String,
    },
    /// An action the transport could not decode; refused with its reason.
    Malformed {
        message: String,
    },
    /// An explore phase ends only after a proposal in that phase or with a
    /// stated abstention; a revise phase ends freely.
    EndPhase {
        abstention: Option<String>,
        /// A findings report (possibly empty); `None` when the protocol has
        /// no report.
        findings: Option<Vec<Finding>>,
    },
}

/// Reported by the provider for one learner turn; kept apart from attempts
/// and verified evaluations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub calls: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_microusd: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JustificationView {
    pub event: Event,
    pub rule: String,
    /// `O` or the hypothesis that owns the rule.
    pub origin: String,
    pub antecedents: Vec<Event>,
    pub substitution: Substitution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DossierView {
    pub id: String,
    pub seeds: Vec<Event>,
    /// Closure of `O ∪ R_k` (committed hypotheses).
    pub known: Vec<Event>,
    pub justifications: Vec<JustificationView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationView {
    /// `e1`, `e2`, … over the trajectory: the reference a finding cites.
    pub id: String,
    pub dossier: String,
    pub status: String,
    pub certified: bool,
    pub new_events: Vec<Event>,
    pub activated_base_rules: BTreeSet<String>,
    pub positions: Vec<Position>,
    /// Closure of the evaluated theory: the record findings are checked
    /// against. Not rendered in the learner's view.
    pub closure: Vec<Event>,
    /// Committed hypotheses other than the evaluated one: the baseline
    /// theory is `O` plus these.
    pub baseline: Vec<String>,
    /// Events of the evaluated theory absent from the baseline closure.
    pub added_events: Vec<Event>,
    /// Justifications of the evaluated theory absent from the baseline
    /// closure, on new or on already derived events.
    pub contributed: Vec<JustificationView>,
}

/// One evaluation shown in the learner's view when it committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationSeen {
    pub hypothesis: String,
    pub id: String,
    pub dossier: String,
    pub baseline: Vec<String>,
    pub status: String,
}

/// What the learner's view contained when it asked to commit: the state the
/// view is built from, recorded by the harness without the learner's report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitTrace {
    pub dossier: usize,
    pub phase: Phase,
    pub before: Vec<String>,
    pub requested: Vec<String>,
    pub result: String,
    pub reason: Option<String>,
    /// Dossiers whose feedback had been delivered.
    pub feedback_seen: Vec<String>,
    pub evaluations_seen: Vec<EvaluationSeen>,
    pub navigation_actions_seen: usize,
    pub findings_seen: usize,
}

/// One finding of a phase report, as checked by the driver with the
/// existing claims verifier (`tools/normative_explorer/claims.py`). The
/// harness records it; it never scores it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: String,
    pub text: String,
    /// The cited record (`n…` navigation, `e…` evaluation), if any.
    pub reference: Option<String>,
    /// `attribute=value` of the assertion, if any.
    pub assertion: Option<String>,
    pub verdict: String,
    pub detail: String,
    /// Diagnostic only, never a certified property.
    pub novelty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingView {
    pub dossier: String,
    pub phase: Phase,
    pub finding: Finding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypothesisView {
    pub id: String,
    pub parent: Option<String>,
    pub text: String,
    pub rules: Vec<String>,
    pub admission: String,
    pub evaluations: Vec<EvaluationView>,
    pub committed: bool,
    /// Reason of the latest `Discard`, if any.
    pub discarded: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackView {
    Discrepancies {
        dossier: String,
        missed: Vec<Position>,
        excess: Vec<(Position, BTreeSet<String>)>,
        valued_total: usize,
        discrepant: usize,
    },
    Evaluated {
        dossier: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnerView {
    pub phase: Phase,
    pub dossier: String,
    pub dossiers: Vec<DossierView>,
    pub hypotheses: Vec<HypothesisView>,
    pub committed: Vec<String>,
    pub feedback: Vec<FeedbackView>,
    pub verified_evaluations_left: usize,
    pub proposals_left: usize,
    pub turns_left: usize,
    pub last_result: Option<String>,
    pub navigation_actions_left: usize,
    pub navigation_steps_left: usize,
    /// Every finding reported so far, with its verdict and reference.
    pub findings: Vec<FindingView>,
    /// The worker's complete response to the previous navigation request.
    pub last_navigation: Option<worker::Response>,
}

pub trait Learner {
    fn act(&mut self, view: &LearnerView) -> (LearnerAction, Usage);
}

// ---------------------------------------------------------------------------
// Trajectory.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrajectoryBudgets {
    /// Certified learner evaluations per trajectory (contract: 8).
    pub max_verified_evaluations: usize,
    pub max_proposals: usize,
    pub max_turns_per_phase: usize,
    pub max_clauses: usize,
    /// Worker requests per trajectory; 0 disables navigation.
    pub max_navigation_actions: usize,
    /// Engine steps over all navigation sessions of the trajectory.
    pub max_navigation_steps: usize,
}

#[derive(Debug, Clone)]
pub struct DossierInput {
    pub id: String,
    pub seeds: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct TrajectoryManifest {
    pub trajectory_id: String,
    pub condition: Condition,
    /// Theory imports of every private module: `O` plus instance constants.
    pub imports: Vec<PathBuf>,
    pub workdir: PathBuf,
    pub profile: Profile,
    /// Fixed before the run; never reordered.
    pub order: Vec<DossierInput>,
    pub budgets: TrajectoryBudgets,
    /// Module name of `O` in the resolved baseline (`LegalGeneral`).
    pub base_module: String,
    /// Explore protocol: before proposing or abstaining, pose a question
    /// and consult Residuum (a navigation request or a verified evaluation)
    /// after it. Never forces a proposal. Requires navigation.
    pub question_first: bool,
}

/// The admissible language of contract §2: A3 heads, a body vocabulary over
/// `O`, `~Act` as the only negative literal.
const NAVIGATION_HASH: &str = "pilot004r-navigation";
const NAVIGATION_STEPS_PER_ACTION: usize = 64;
const NAVIGATION_SESSIONS: usize = 64;

fn navigation_kind(r: &worker::Request) -> &'static str {
    use worker::Request as R;
    match r {
        R::InspectTheory => "inspect_theory",
        R::BeginForward { .. } => "begin_forward",
        R::BeginResidual { .. } => "begin_residual",
        R::BeginActiveResidual { .. } => "begin_active_residual",
        R::AdvanceSession { .. } => "advance_session",
        R::ProposeCandidate { .. } => "propose_candidate",
        R::BeginCandidateEvaluation { .. } => "begin_candidate_evaluation",
        R::MinimalSupports { .. } => "minimal_supports",
        R::ComparePartition { .. } => "compare_partition",
    }
}

/// Elaborate one event written in Formal (`verb(args) @ time`, `~Act(...)`)
/// against the trajectory imports, as the goal of a navigation request. The
/// event is the query of a throwaway experiment with no seeds.
pub fn parse_event(
    imports: &[PathBuf],
    workdir: &std::path::Path,
    serial: usize,
    text: &str,
) -> Result<Event, Error> {
    let dir = workdir.join("goals");
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(format!("cannot create `{}`: {e}", dir.display())))?;
    let dir = dir
        .canonicalize()
        .map_err(|e| Error::new(format!("cannot resolve `{}`: {e}", dir.display())))?;
    let mut relative_imports = Vec::new();
    for import in imports {
        let target = import.canonicalize().map_err(|e| {
            Error::new(format!("cannot resolve import `{}`: {e}", import.display()))
        })?;
        relative_imports.push(relative(&dir, &target)?);
    }
    if text.contains('}') || text.contains('{') {
        return Err(Error::new("an event contains no braces"));
    }
    let name = format!("Goal{serial}");
    let body = format!(
        "experiment goal {{\n    seeds {{\n    }}\n    query residual\n    {}\n}}\n",
        text.trim()
    );
    let file = dir.join(format!("{name}.res"));
    std::fs::write(&file, compose(&name, &relative_imports, &body))
        .map_err(|e| Error::new(format!("cannot write `{}`: {e}", file.display())))?;
    let module = crate::load_formal_module(&file)?;
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == "goal" => Some(e.goal.clone()),
            _ => None,
        })
        .ok_or_else(|| Error::new("the goal did not elaborate to an event"))
}

pub fn profile_004r() -> Profile {
    let heads = [
        "obligation",
        "permission",
        "prohibition",
        "omissionProhibition",
        "right",
        "compliance",
        "violation",
        "enforceability",
        "patrimonialLiability",
    ];
    let body = [
        "Act",
        "right",
        "obligationFiled",
        "prohibition",
        "omissionProhibition",
        "permissionFiled",
        "patrimonialHolding",
        "powerGranted",
        "legislationText",
        "obligation",
        "permission",
        "compliance",
        "violation",
        "enforceability",
        "patrimonialAsset",
        "patrimonialLiability",
        "transferred",
        "owns",
        "capacity",
        "law",
    ];
    Profile {
        heads: heads.iter().map(|s| s.to_string()).collect(),
        body_verbs: body.iter().map(|s| s.to_string()).collect(),
        evidential_not_verbs: BTreeSet::from(["Act".to_string()]),
        max_atoms: 4,
        max_binders: 16,
        max_after_depth: 1,
        max_unit_nodes: 1,
        require_connected: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypothesisRecord {
    pub id: String,
    pub parent: Option<String>,
    pub text: String,
    pub rationale: String,
    pub origin: (usize, Phase),
    pub rules: Vec<Rule>,
    pub admission: String,
    pub admitted: bool,
    pub case_specific: bool,
    pub certified_evaluations: usize,
    pub committed_ok: bool,
    pub retired_at: Option<usize>,
    pub evaluations: Vec<EvaluationView>,
    /// Reason of the latest `Discard`, if any (traceability only).
    pub discarded: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypothesisStatus {
    Valid,
    Compatible {
        dossiers: usize,
        exposures: usize,
    },
    Prospective {
        dossiers: usize,
        exposures: usize,
    },
    Refuted {
        dossier: String,
        valuations: Vec<String>,
    },
    Unexposed,
    CaseSpecific,
    SupersededBy(String),
    Pending(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counters {
    pub turns: usize,
    pub attempts: usize,
    pub admitted: usize,
    pub verified_evaluations: usize,
    pub refused_actions: usize,
    pub abstentions: usize,
    pub questions: usize,
    pub discards: usize,
    /// Phase reports received (an empty report counts).
    pub reports: usize,
    pub findings: usize,
    /// Findings whose assertion the verifier matched.
    pub findings_verified: usize,
    pub navigation_actions: usize,
    pub navigation_steps: usize,
    pub commits: usize,
    pub commit_rejections: usize,
    pub usage: Usage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub seq: usize,
    pub dossier: String,
    pub phase: Option<Phase>,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrajectoryReport {
    pub trajectory_id: String,
    pub condition: Condition,
    pub order: Vec<String>,
    pub prequential: Vec<Scoring>,
    /// After each revision phase, every seen dossier rescored (not prospective).
    pub compatibility: Vec<Vec<Scoring>>,
    pub hypotheses: Vec<HypothesisRecord>,
    pub statuses: BTreeMap<String, Vec<HypothesisStatus>>,
    pub counters: Counters,
    pub ledger: Vec<LedgerEntry>,
    pub findings: Vec<FindingView>,
    pub commit_traces: Vec<CommitTrace>,
}

impl TrajectoryReport {
    pub fn prequential_loss(&self) -> usize {
        self.prequential.iter().map(|s| s.loss).sum()
    }

    /// Index of the first dossier whose prequential scoring had a discrepancy.
    pub fn first_discrepancy(&self) -> Option<usize> {
        self.prequential.iter().position(|s| s.loss > 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionEffect {
    NotMeasurable(String),
    /// Control minus feedback prequential loss on the dossiers after the
    /// first discrepancy of the feedback trajectory.
    Measured {
        dossiers: Vec<String>,
        delta: i64,
    },
}

pub fn revision_effect(feedback: &TrajectoryReport, control: &TrajectoryReport) -> RevisionEffect {
    if feedback.condition != Condition::Feedback || control.condition != Condition::NoFeedback {
        return RevisionEffect::NotMeasurable("conditions are not feedback/control".into());
    }
    if feedback.order != control.order {
        return RevisionEffect::NotMeasurable("the trajectories use different orders".into());
    }
    let Some(first) = feedback.first_discrepancy() else {
        return RevisionEffect::NotMeasurable("no discrepancy occurred".into());
    };
    if first + 1 >= feedback.order.len() {
        return RevisionEffect::NotMeasurable(
            "the first discrepancy occurred on the last dossier".into(),
        );
    }
    let later = |r: &TrajectoryReport| -> i64 {
        r.prequential[first + 1..]
            .iter()
            .map(|s| s.loss as i64)
            .sum()
    };
    RevisionEffect::Measured {
        dossiers: feedback.order[first + 1..].to_vec(),
        delta: later(control) - later(feedback),
    }
}

struct State<'a> {
    manifest: &'a TrajectoryManifest,
    reserved: &'a ReservedValuations,
    episode: Episode,
    placeholder: Decision,
    baseline_constants: BTreeSet<String>,
    hypotheses: Vec<HypothesisRecord>,
    committed: Vec<String>,
    feedback: Vec<FeedbackView>,
    counters: Counters,
    ledger: Vec<LedgerEntry>,
    prequential: Vec<Scoring>,
    compatibility: Vec<Vec<Scoring>>,
    last_result: Option<String>,
    navigator: Option<worker::Worker>,
    /// Hypothesis id → worker candidate id, for the current navigator.
    navigation_ids: BTreeMap<String, String>,
    navigation_serial: usize,
    last_navigation: Option<worker::Response>,
    findings: Vec<FindingView>,
    commit_traces: Vec<CommitTrace>,
}

fn event_constants(e: &Event, out: &mut BTreeSet<String>) {
    fn term(t: &Term, out: &mut BTreeSet<String>) {
        match t {
            Term::Const(c) => {
                out.insert(c.clone());
            }
            Term::Constructor { argument, .. } => term(argument, out),
            _ => {}
        }
    }
    fn time(t: &TimeExpr, out: &mut BTreeSet<String>) {
        match t {
            TimeExpr::At(c) => {
                out.insert(c.clone());
            }
            TimeExpr::After(inner) => time(inner, out),
            TimeExpr::Var(_) => {}
        }
    }
    for t in &e.args {
        term(t, out);
    }
    time(&e.proposition.time, out);
}

fn rule_constants(r: &Rule) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let RuleBody::HornClause(c) = &r.body {
        let mut body = Vec::new();
        atoms(&c.antecedent, &mut body);
        for e in body {
            event_constants(e, &mut out);
        }
        event_constants(&c.consequent, &mut out);
    }
    out
}

impl<'a> State<'a> {
    fn log(&mut self, dossier: usize, phase: Option<Phase>, kind: &str, detail: String) {
        let seq = self.ledger.len();
        self.ledger.push(LedgerEntry {
            seq,
            dossier: self.manifest.order[dossier].id.clone(),
            phase,
            kind: kind.into(),
            detail,
        });
    }

    fn hypothesis(&self, id: &str) -> Option<&HypothesisRecord> {
        self.hypotheses.iter().find(|h| h.id == id)
    }

    fn rule_origins(&self, ids: &[String]) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for id in ids {
            if let Some(h) = self.hypothesis(id) {
                for r in &h.rules {
                    out.insert(r.name.clone(), id.clone());
                }
            }
        }
        out
    }

    fn theory(&self, ids: &[String]) -> Result<(Module, BTreeSet<String>), Error> {
        let mut rules = Vec::new();
        for id in ids {
            let h = self
                .hypothesis(id)
                .ok_or_else(|| Error::new(format!("unknown hypothesis `{id}`")))?;
            rules.extend(h.rules.iter().cloned());
        }
        let learned = rules.iter().map(|r| r.name.clone()).collect();
        let module = install(
            &self.episode.baseline,
            &[],
            &rules,
            &self.manifest.trajectory_id,
        )?;
        Ok((module, learned))
    }

    fn seen_constants(&self, upto: usize) -> BTreeMap<String, BTreeSet<usize>> {
        let mut out: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for (i, d) in self.manifest.order.iter().enumerate().take(upto + 1) {
            let mut cs = BTreeSet::new();
            for e in &d.seeds {
                event_constants(e, &mut cs);
            }
            for c in cs {
                out.entry(c).or_default().insert(i);
            }
        }
        out
    }

    fn view(&self, index: usize, phase: Phase, turns_left: usize) -> LearnerView {
        let origins = self.rule_origins(&self.committed);
        let theory = self.theory(&self.committed);
        let dossiers = self.manifest.order[..=index]
            .iter()
            .map(|d| {
                let closure = theory
                    .as_ref()
                    .ok()
                    .and_then(|(m, _)| close(m, &d.seeds).ok());
                let (known, justifications) = match closure {
                    Some(c) => {
                        let js = c
                            .derivations
                            .iter()
                            .map(|j| JustificationView {
                                event: j.event.clone(),
                                rule: j.record.rule.clone(),
                                origin: origins
                                    .get(&j.record.rule)
                                    .cloned()
                                    .unwrap_or_else(|| "O".into()),
                                antecedents: j.record.antecedents.clone(),
                                substitution: j.record.substitution.clone(),
                            })
                            .collect();
                        (c.known, js)
                    }
                    None => (Vec::new(), Vec::new()),
                };
                DossierView {
                    id: d.id.clone(),
                    seeds: d.seeds.clone(),
                    known,
                    justifications,
                }
            })
            .collect();
        let b = &self.manifest.budgets;
        LearnerView {
            phase,
            dossier: self.manifest.order[index].id.clone(),
            dossiers,
            hypotheses: self
                .hypotheses
                .iter()
                .map(|h| HypothesisView {
                    id: h.id.clone(),
                    parent: h.parent.clone(),
                    text: h.text.clone(),
                    rules: h.rules.iter().map(|r| r.name.clone()).collect(),
                    admission: h.admission.clone(),
                    evaluations: h.evaluations.clone(),
                    committed: self.committed.contains(&h.id),
                    discarded: h.discarded.clone(),
                })
                .collect(),
            committed: self.committed.clone(),
            feedback: self.feedback.clone(),
            verified_evaluations_left: b
                .max_verified_evaluations
                .saturating_sub(self.counters.verified_evaluations),
            proposals_left: b.max_proposals.saturating_sub(self.counters.attempts),
            turns_left,
            last_result: self.last_result.clone(),
            navigation_actions_left: b
                .max_navigation_actions
                .saturating_sub(self.counters.navigation_actions),
            navigation_steps_left: b
                .max_navigation_steps
                .saturating_sub(self.counters.navigation_steps),
            last_navigation: self.last_navigation.clone(),
            findings: self.findings.clone(),
        }
    }

    fn propose(
        &mut self,
        index: usize,
        phase: Phase,
        text: String,
        parent: Option<String>,
        rationale: String,
    ) -> String {
        if self.counters.attempts >= self.manifest.budgets.max_proposals {
            self.counters.refused_actions += 1;
            return "refused: proposal budget exhausted".into();
        }
        self.counters.attempts += 1;
        let id = format!("h{}", self.hypotheses.len() + 1);
        let ctx = ProposalContext {
            imports: &self.manifest.imports,
            workdir: &self.manifest.workdir,
            profile: &self.manifest.profile,
            max_clauses: self.manifest.budgets.max_clauses,
            seeds: &[],
            decision: &self.placeholder,
        };
        let proposal = SourceProposal {
            id: id.clone(),
            text: text.clone(),
        };
        let (admission, rules, admitted) = match admit(&proposal, &ctx, &self.episode) {
            Err(e) => (format!("harness: {e}"), Vec::new(), false),
            Ok(Err(report)) => (
                format!("{} {:?}", status_label(&report.status), report.status),
                Vec::new(),
                false,
            ),
            Ok(Ok(a)) => ("admitted".to_string(), a.report.rules, true),
        };
        let mut record = HypothesisRecord {
            id: id.clone(),
            parent: parent.filter(|p| self.hypothesis(p).is_some()),
            text,
            rationale,
            origin: (index, phase),
            rules,
            admission,
            admitted,
            case_specific: false,
            certified_evaluations: 0,
            committed_ok: false,
            retired_at: None,
            evaluations: Vec::new(),
            discarded: None,
        };
        if record.admitted {
            let taken: BTreeSet<&str> = self
                .hypotheses
                .iter()
                .flat_map(|h| h.rules.iter().map(|r| r.name.as_str()))
                .collect();
            let seen = self.seen_constants(index);
            let mut foreign = BTreeSet::new();
            let mut sources = BTreeSet::new();
            for r in &record.rules {
                for c in rule_constants(r) {
                    if self.baseline_constants.contains(&c) {
                        continue;
                    }
                    match seen.get(&c) {
                        Some(ds) => sources.extend(ds.iter().copied()),
                        None => {
                            foreign.insert(c);
                        }
                    }
                }
            }
            if record.rules.iter().any(|r| taken.contains(r.name.as_str())) {
                record.admitted = false;
                record.admission = "S_NAME_COLLISION".into();
            } else if !foreign.is_empty() {
                record.admitted = false;
                record.admission = format!("A6_UNSEEN_CONSTANT {foreign:?}");
            } else {
                record.case_specific = sources.len() == 1;
            }
        }
        if record.admitted {
            self.counters.admitted += 1;
        }
        let registered = if record.admitted {
            self.register(&record.id, &record.text)
        } else {
            Ok(())
        };
        let mut result = format!("{id}: {}", record.admission);
        if let Err(e) = registered {
            result.push_str(&format!(" (not available to navigation: {e})"));
        }
        self.log(index, Some(phase), "propose", result.clone());
        self.hypotheses.push(record);
        result
    }

    // -- navigation over the existing worker -------------------------------

    /// Rebuild the navigator over the dossiers presented so far, registering
    /// every admitted hypothesis as a worker candidate. Sessions do not
    /// survive a new dossier; hypothesis identifiers do.
    fn open_navigator(&mut self, index: usize) -> Result<(), Error> {
        self.navigator = None;
        self.navigation_ids.clear();
        if self.manifest.budgets.max_navigation_actions == 0 {
            return Ok(());
        }
        let dossiers: BTreeMap<String, Vec<Event>> = self.manifest.order[..=index]
            .iter()
            .map(|d| (d.id.clone(), d.seeds.clone()))
            .collect();
        let navigator = worker::Worker::new(worker::Manifest {
            episode_id: self.manifest.trajectory_id.clone(),
            manifest_hash: NAVIGATION_HASH.into(),
            imports: self.manifest.imports.clone(),
            workdir: self.manifest.workdir.join(format!("navigator-{index}")),
            profile: self.manifest.profile.clone(),
            decision: self.placeholder.clone(),
            dossiers,
            budgets: worker::Budgets {
                // The trajectory budgets are enforced here, not by the worker.
                max_actions: usize::MAX,
                max_steps_per_action: NAVIGATION_STEPS_PER_ACTION,
                max_steps_total: usize::MAX,
                max_sessions: NAVIGATION_SESSIONS,
                max_candidates: usize::MAX,
                max_clauses: self.manifest.budgets.max_clauses,
                search_limits: crate::SearchLimits::default(),
            },
        })?;
        self.navigator = Some(navigator);
        let admitted: Vec<(String, String)> = self
            .hypotheses
            .iter()
            .filter(|h| h.admitted)
            .map(|h| (h.id.clone(), h.text.clone()))
            .collect();
        for (id, text) in admitted {
            if let Err(e) = self.register(&id, &text) {
                self.log(
                    index,
                    None,
                    "navigator",
                    format!("{id} not registered: {e}"),
                );
            }
        }
        Ok(())
    }

    fn send(&mut self, request: worker::Request) -> Option<worker::Response> {
        let navigator = self.navigator.as_mut()?;
        self.navigation_serial += 1;
        Some(navigator.handle(worker::Envelope {
            protocol_version: worker::PROTOCOL_VERSION.into(),
            episode_id: self.manifest.trajectory_id.clone(),
            action_id: format!("n{}", self.navigation_serial),
            manifest_hash: NAVIGATION_HASH.into(),
            request,
        }))
    }

    fn register(&mut self, id: &str, text: &str) -> Result<(), String> {
        if self.navigator.is_none() {
            return Ok(());
        }
        match self.send(worker::Request::ProposeCandidate { text: text.into() }) {
            Some(worker::Response {
                payload:
                    worker::Payload::Candidate {
                        candidate: Some(cid),
                        ..
                    },
                ..
            }) => {
                self.navigation_ids.insert(id.into(), cid);
                Ok(())
            }
            Some(other) => Err(format!("{:?}: {:?}", other.status, other.payload)),
            None => Ok(()),
        }
    }

    fn candidate(&self, id: &str) -> Result<String, String> {
        self.navigation_ids
            .get(id)
            .cloned()
            .ok_or_else(|| format!("`{id}` is not an admitted hypothesis"))
    }

    fn navigate(&mut self, index: usize, phase: Phase, request: worker::Request) -> String {
        use worker::Request as R;
        if self.navigator.is_none() {
            self.counters.refused_actions += 1;
            return "refused: navigation is not available in this trajectory".into();
        }
        if self.counters.navigation_actions >= self.manifest.budgets.max_navigation_actions {
            self.counters.refused_actions += 1;
            return "refused: navigation budget exhausted".into();
        }
        let theory = |s: &Self, t: Option<String>| t.map(|h| s.candidate(&h)).transpose();
        let translated = match request {
            R::ProposeCandidate { .. } | R::BeginCandidateEvaluation { .. } => Err(
                "proposals and their evaluation go through propose_rule and evaluate_rule".into(),
            ),
            R::InspectTheory => Ok(R::InspectTheory),
            R::BeginForward { dossier, theory: t } => {
                theory(self, t).map(|theory| R::BeginForward { dossier, theory })
            }
            R::BeginResidual {
                dossier,
                theory: t,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => theory(self, t).map(|theory| R::BeginResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            }),
            R::BeginActiveResidual {
                dossier,
                theory: t,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => theory(self, t).map(|theory| R::BeginActiveResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            }),
            R::AdvanceSession { session, max_steps } => {
                let left = self
                    .manifest
                    .budgets
                    .max_navigation_steps
                    .saturating_sub(self.counters.navigation_steps);
                if left == 0 {
                    Err("navigation step budget exhausted".into())
                } else {
                    Ok(R::AdvanceSession {
                        session,
                        max_steps: max_steps.min(left),
                    })
                }
            }
            R::MinimalSupports { candidate, goal } => self
                .candidate(&candidate)
                .map(|candidate| R::MinimalSupports { candidate, goal }),
            R::ComparePartition { candidates, goal } => candidates
                .iter()
                .map(|h| self.candidate(h))
                .collect::<Result<Vec<_>, _>>()
                .map(|candidates| R::ComparePartition { candidates, goal }),
        };
        let request = match translated {
            Ok(r) => r,
            Err(reason) => {
                self.counters.refused_actions += 1;
                let refusal = format!("refused: {reason}");
                self.log(index, Some(phase), "navigate_refused", refusal.clone());
                return refusal;
            }
        };
        let kind = navigation_kind(&request);
        let Some(mut response) = self.send(request) else {
            self.counters.refused_actions += 1;
            return "refused: navigation is not available in this trajectory".into();
        };
        self.counters.navigation_actions += 1;
        self.counters.navigation_steps += response.steps_consumed;
        // The worker's own budgets are unbounded here; report the trajectory's.
        let b = &self.manifest.budgets;
        response.budget.actions_remaining = b
            .max_navigation_actions
            .saturating_sub(self.counters.navigation_actions);
        response.budget.steps_remaining = b
            .max_navigation_steps
            .saturating_sub(self.counters.navigation_steps);
        response.budget.candidates_remaining =
            b.max_proposals.saturating_sub(self.counters.attempts);
        // Back to hypothesis identifiers.
        let back: BTreeMap<String, String> = self
            .navigation_ids
            .iter()
            .map(|(h, c)| (c.clone(), h.clone()))
            .collect();
        let name = |c: &String| back.get(c).cloned().unwrap_or_else(|| c.clone());
        match &mut response.payload {
            worker::Payload::Supports { candidate, .. } => *candidate = name(candidate),
            worker::Payload::Partition {
                partition, covers, ..
            } => {
                partition.hypotheses = partition.hypotheses.iter().map(name).collect();
                *covers = covers.iter().map(|(a, b)| (name(a), name(b))).collect();
            }
            _ => {}
        }
        let result = format!(
            "{kind}: {:?} (steps {})",
            response.status, response.steps_consumed
        );
        self.log(index, Some(phase), "navigate", result.clone());
        self.last_navigation = Some(response);
        result
    }

    fn evaluate(
        &mut self,
        index: usize,
        phase: Phase,
        hypothesis: String,
        dossier: String,
    ) -> String {
        let Some(position) = self.manifest.order.iter().position(|d| d.id == dossier) else {
            self.counters.refused_actions += 1;
            return format!("refused: unknown dossier `{dossier}`");
        };
        if position > index {
            self.counters.refused_actions += 1;
            return format!("refused: `{dossier}` has not been presented");
        }
        match self.hypothesis(&hypothesis) {
            Some(h) if h.admitted => {}
            _ => {
                self.counters.refused_actions += 1;
                return format!("refused: `{hypothesis}` is not an admitted hypothesis");
            }
        }
        if self.counters.verified_evaluations >= self.manifest.budgets.max_verified_evaluations {
            self.counters.refused_actions += 1;
            return "refused: verified evaluation budget exhausted".into();
        }
        let mut ids = self.committed.clone();
        if !ids.contains(&hypothesis) {
            ids.push(hypothesis.clone());
        }
        let seeds = self.manifest.order[position].seeds.clone();
        let outcome = self.theory(&ids).and_then(|(m, learned)| {
            certify_attributed(&self.episode.baseline, &m, &learned, &seeds)
        });
        // Attribution against the theory the learner already holds: `O` plus
        // the other committed hypotheses.
        let baseline: Vec<String> = ids
            .iter()
            .filter(|id| **id != hypothesis)
            .cloned()
            .collect();
        let base_closure = self
            .theory(&baseline)
            .and_then(|(m, _)| close(&m, &seeds))
            .ok();
        let origins = self.rule_origins(&ids);
        let own: BTreeSet<String> = self
            .hypothesis(&hypothesis)
            .map(|h| h.rules.iter().map(|r| r.name.clone()).collect())
            .unwrap_or_default();
        let evaluation_id = format!(
            "e{}",
            self.hypotheses
                .iter()
                .map(|h| h.evaluations.len())
                .sum::<usize>()
                + 1
        );
        let view = match outcome {
            Err(e) => EvaluationView {
                id: evaluation_id.clone(),
                dossier: dossier.clone(),
                status: format!("pending: {e}"),
                certified: false,
                new_events: Vec::new(),
                activated_base_rules: BTreeSet::new(),
                positions: Vec::new(),
                closure: Vec::new(),
                baseline: baseline.clone(),
                added_events: Vec::new(),
                contributed: Vec::new(),
            },
            Ok((certificate, closure)) => {
                self.counters.verified_evaluations += 1;
                let deps = learned_dependencies(&closure, &own);
                let mut positions: Vec<Position> = closure
                    .known
                    .iter()
                    .filter(|e| deps.contains_key(*e))
                    .filter_map(project)
                    .collect();
                positions.sort();
                positions.dedup();
                let (added_events, contributed) = match &base_closure {
                    Some(base) => {
                        let key = |d: &crate::typed::DerivedEvent| -> Justification {
                            (
                                d.record.rule.clone(),
                                d.record.antecedents.clone(),
                                d.event.clone(),
                            )
                        };
                        let base_known: BTreeSet<&Event> = base.known.iter().collect();
                        let base_just: BTreeSet<Justification> =
                            base.derivations.iter().map(key).collect();
                        let added = closure
                            .known
                            .iter()
                            .filter(|e| !base_known.contains(e))
                            .cloned()
                            .collect();
                        let contributed = closure
                            .derivations
                            .iter()
                            .filter(|d| !base_just.contains(&key(d)))
                            .map(|d| JustificationView {
                                event: d.event.clone(),
                                rule: d.record.rule.clone(),
                                origin: origins
                                    .get(&d.record.rule)
                                    .cloned()
                                    .unwrap_or_else(|| "O".into()),
                                antecedents: d.record.antecedents.clone(),
                                substitution: d.record.substitution.clone(),
                            })
                            .collect();
                        (added, contributed)
                    }
                    None => (Vec::new(), Vec::new()),
                };
                EvaluationView {
                    id: evaluation_id.clone(),
                    dossier: dossier.clone(),
                    status: if certificate.holds() {
                        "certified".into()
                    } else {
                        format!("certificate_failed {:?}", certificate.problems)
                    },
                    certified: certificate.holds(),
                    new_events: certificate
                        .new_events
                        .iter()
                        .map(|n| n.event.clone())
                        .collect(),
                    activated_base_rules: certificate.activated_base_rules.clone(),
                    positions,
                    closure: closure.known.clone(),
                    baseline: baseline.clone(),
                    added_events,
                    contributed,
                }
            }
        };
        let result = format!("{hypothesis} on {dossier}: {}", view.status);
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.id == hypothesis) {
            if view.certified {
                h.certified_evaluations += 1;
            }
            h.evaluations.push(view);
        }
        self.log(index, Some(phase), "evaluate", result.clone());
        result
    }

    /// Keep a phase report: each finding with its verdict, reference and
    /// novelty diagnostic; an empty report is recorded as such.
    fn record_findings(&mut self, index: usize, phase: Phase, findings: Vec<Finding>) {
        self.counters.reports += 1;
        if findings.is_empty() {
            self.log(index, Some(phase), "findings", "empty report".into());
        }
        for f in findings {
            self.counters.findings += 1;
            if f.verdict == "match" {
                self.counters.findings_verified += 1;
            }
            self.log(
                index,
                Some(phase),
                "finding",
                format!(
                    "[{}] ref={} assertion={} verdict={} ({}) novelty={}: {}",
                    f.kind,
                    f.reference.as_deref().unwrap_or("-"),
                    f.assertion.as_deref().unwrap_or("-"),
                    f.verdict,
                    f.detail,
                    f.novelty,
                    f.text
                ),
            );
            self.findings.push(FindingView {
                dossier: self.manifest.order[index].id.clone(),
                phase,
                finding: f,
            });
        }
    }

    fn discard(&mut self, index: usize, phase: Phase, id: String, reason: String) -> String {
        let Some(h) = self.hypotheses.iter_mut().find(|h| h.id == id) else {
            self.counters.refused_actions += 1;
            return format!("refused: `{id}` is not a hypothesis of this trajectory");
        };
        h.discarded = Some(reason.clone());
        self.counters.discards += 1;
        let detail = format!("{id}: {reason}");
        self.log(index, Some(phase), "discard", detail.clone());
        format!("discarded {detail}")
    }

    fn commit(
        &mut self,
        index: usize,
        phase: Phase,
        ids: Vec<String>,
        reason: Option<String>,
    ) -> String {
        let before = self.committed.clone();
        let mut unique = Vec::new();
        for id in ids {
            if !unique.contains(&id) {
                unique.push(id);
            }
        }
        if let Some(bad) = unique
            .iter()
            .find(|id| !self.hypothesis(id).is_some_and(|h| h.admitted))
        {
            self.counters.commit_rejections += 1;
            return format!("commit rejected: `{bad}` is not admitted");
        }
        // Mandatory gate, outside the learner's evaluation budget: the new
        // theory must certify on every presented dossier.
        let checked = self.theory(&unique).and_then(|(m, learned)| {
            let mut failures = Vec::new();
            for d in &self.manifest.order[..=index] {
                let (c, _) = certify_attributed(&self.episode.baseline, &m, &learned, &d.seeds)?;
                if !c.holds() {
                    failures.push(format!("{}: {:?}", d.id, c.problems));
                }
            }
            Ok(failures)
        });
        let result = match checked {
            Err(e) => {
                self.counters.commit_rejections += 1;
                format!("commit rejected: pending {e}")
            }
            Ok(failures) if !failures.is_empty() => {
                self.counters.commit_rejections += 1;
                format!("commit rejected: certificate {failures:?}")
            }
            Ok(_) => {
                for h in self.hypotheses.iter_mut() {
                    if self.committed.contains(&h.id) && !unique.contains(&h.id) {
                        h.retired_at = Some(index);
                    }
                    if unique.contains(&h.id) {
                        h.committed_ok = true;
                    }
                }
                self.committed = unique.clone();
                self.counters.commits += 1;
                format!("committed {unique:?}")
            }
        };
        let detail = match &reason {
            Some(r) => format!("{result} reason: {r}"),
            None => result.clone(),
        };
        self.log(index, Some(phase), "commit", detail);
        self.commit_traces.push(CommitTrace {
            dossier: index,
            phase,
            before,
            requested: unique,
            result: result.clone(),
            reason,
            feedback_seen: self
                .feedback
                .iter()
                .map(|f| match f {
                    FeedbackView::Discrepancies { dossier, .. }
                    | FeedbackView::Evaluated { dossier } => dossier.clone(),
                })
                .collect(),
            evaluations_seen: self
                .hypotheses
                .iter()
                .flat_map(|h| {
                    h.evaluations.iter().map(|e| EvaluationSeen {
                        hypothesis: h.id.clone(),
                        id: e.id.clone(),
                        dossier: e.dossier.clone(),
                        baseline: e.baseline.clone(),
                        status: e.status.clone(),
                    })
                })
                .collect(),
            navigation_actions_seen: self.counters.navigation_actions,
            findings_seen: self.findings.len(),
        });
        result
    }

    fn phase(&mut self, learner: &mut dyn Learner, index: usize, phase: Phase) {
        let max = self.manifest.budgets.max_turns_per_phase;
        let attempts_before = self.counters.attempts;
        // Question-first explore: consultations counted since the latest
        // question of this phase; `None` until a question is posed.
        let gated = self.manifest.question_first && phase == Phase::Explore;
        let consultations =
            |s: &Self| s.counters.navigation_actions + s.counters.verified_evaluations;
        let mut since_question: Option<usize> = None;
        let gate_open = |s: &Self, since: Option<usize>| {
            !gated || since.is_some_and(|at| consultations(s) > at)
        };
        const GATE: &str = "refused: in explore, first pose a question (pose_question) and consult \
                            Residuum about it (a navigation tool or evaluate_rule); then propose a \
                            hypothesis or state an abstention";
        for turn in 0..max {
            let view = self.view(index, phase, max - turn);
            let (action, usage) = learner.act(&view);
            self.counters.turns += 1;
            self.counters.usage.calls += usage.calls;
            self.counters.usage.input_tokens += usage.input_tokens;
            self.counters.usage.output_tokens += usage.output_tokens;
            self.counters.usage.cost_microusd += usage.cost_microusd;
            self.log(
                index,
                Some(phase),
                "usage",
                format!(
                    "calls={} input={} output={} cost_microusd={}",
                    usage.calls, usage.input_tokens, usage.output_tokens, usage.cost_microusd
                ),
            );
            // A navigation response is shown once, right after its request.
            self.last_navigation = None;
            let gate_closed = !gate_open(self, since_question);
            let result = match action {
                LearnerAction::EndPhase { .. } | LearnerAction::Propose { .. } if gate_closed => {
                    self.counters.refused_actions += 1;
                    self.log(index, Some(phase), "gate_refused", GATE.into());
                    GATE.into()
                }
                LearnerAction::Question { question, tool } => {
                    self.counters.questions += 1;
                    since_question = Some(consultations(self));
                    let detail = format!(
                        "q{}: {} [tool: {}]",
                        self.counters.questions, question, tool
                    );
                    self.log(index, Some(phase), "question", detail.clone());
                    format!("recorded {detail}")
                }
                LearnerAction::EndPhase {
                    abstention,
                    findings,
                } => {
                    let proposed = self.counters.attempts > attempts_before;
                    let stated = abstention
                        .map(|a| a.trim().to_string())
                        .filter(|a| !a.is_empty());
                    if phase == Phase::Explore && !proposed {
                        if let Some(reason) = stated {
                            self.counters.abstentions += 1;
                            self.log(index, Some(phase), "abstention", reason);
                        } else {
                            self.counters.refused_actions += 1;
                            let refusal = "refused: an explore phase ends only after a proposal \
                                           in this phase or with a stated abstention"
                                .to_string();
                            self.log(index, Some(phase), "end_refused", refusal.clone());
                            self.last_result = Some(refusal);
                            continue;
                        }
                    }
                    if let Some(findings) = findings {
                        self.record_findings(index, phase, findings);
                    }
                    self.last_result = None;
                    return;
                }
                LearnerAction::Propose {
                    text,
                    parent,
                    rationale,
                } => self.propose(index, phase, text, parent, rationale),
                LearnerAction::Evaluate {
                    hypothesis,
                    dossier,
                } => self.evaluate(index, phase, hypothesis, dossier),
                LearnerAction::Commit { hypotheses, reason } => {
                    self.commit(index, phase, hypotheses, reason)
                }
                LearnerAction::Discard { hypothesis, reason } => {
                    self.discard(index, phase, hypothesis, reason)
                }
                LearnerAction::Navigate { request } => self.navigate(index, phase, request),
                LearnerAction::Malformed { message } => {
                    self.counters.refused_actions += 1;
                    let refusal = format!("refused: malformed action: {message}");
                    self.log(index, Some(phase), "malformed", refusal.clone());
                    refusal
                }
            };
            self.last_result = Some(result);
        }
        let detail = if gated && since_question.is_none() {
            "turn limit reached without a question"
        } else if gated && !gate_open(self, since_question) {
            "turn limit reached without a consultation after the question"
        } else if phase == Phase::Explore && self.counters.attempts == attempts_before {
            "turn limit reached without proposal or abstention"
        } else {
            "turn limit reached"
        };
        self.log(index, Some(phase), "phase", detail.into());
        self.last_result = None;
    }

    /// Evaluator only: score the committed theory on one dossier.
    fn score(&self, index: usize, prequential: bool) -> Result<Scoring, Error> {
        let d = &self.manifest.order[index];
        let (module, learned) = self.theory(&self.committed)?;
        let closure = close(&module, &d.seeds)?;
        let origins = self.rule_origins(&self.committed);
        let deps = learned_dependencies(&closure, &learned);
        let derived: BTreeMap<Position, &Event> = closure
            .known
            .iter()
            .filter_map(|e| project(e).map(|p| (p, e)))
            .collect();
        let mut missed = Vec::new();
        let mut excess = Vec::new();
        let mut exposures = Vec::new();
        let mut valued = 0;
        let mut valued_positions = BTreeSet::new();
        for v in self.reserved.of(&d.id) {
            valued += 1;
            valued_positions.insert(v.position.clone());
            let event = derived.get(&v.position);
            let owners: BTreeSet<String> = event
                .and_then(|e| deps.get(*e))
                .map(|rules| {
                    rules
                        .iter()
                        .filter_map(|r| origins.get(r).cloned())
                        .collect()
                })
                .unwrap_or_default();
            match (v.label, event.is_some()) {
                (Label::Required, false) => missed.push(v.clone()),
                (Label::Excluded, true) => excess.push((v.clone(), owners.clone())),
                _ => {}
            }
            for h in owners {
                exposures.push((
                    h,
                    Exposure {
                        dossier: d.id.clone(),
                        valuation: v.id.clone(),
                        prequential,
                        correct: v.label == Label::Required,
                    },
                ));
            }
        }
        let unvalued_derived = derived
            .keys()
            .filter(|p| !valued_positions.contains(*p))
            .count();
        Ok(Scoring {
            dossier: d.id.clone(),
            committed: self.committed.clone(),
            loss: missed.len() + excess.len(),
            missed,
            excess,
            valued,
            unvalued_derived,
            exposures,
        })
    }
}

fn statuses(
    hypotheses: &[HypothesisRecord],
    prequential: &[Scoring],
    compatibility: &[Vec<Scoring>],
) -> BTreeMap<String, Vec<HypothesisStatus>> {
    let mut out = BTreeMap::new();
    let latest: BTreeMap<&str, &Scoring> = compatibility
        .iter()
        .flatten()
        .map(|s| (s.dossier.as_str(), s))
        .collect();
    for h in hypotheses {
        let mut st = Vec::new();
        if !h.admitted {
            st.push(HypothesisStatus::Pending(h.admission.clone()));
            out.insert(h.id.clone(), st);
            continue;
        }
        if h.certified_evaluations > 0 || h.committed_ok {
            st.push(HypothesisStatus::Valid);
        } else {
            st.push(HypothesisStatus::Pending("not certified".into()));
        }
        let mine = |s: &Scoring| -> Vec<Exposure> {
            s.exposures
                .iter()
                .filter(|(id, _)| *id == h.id)
                .map(|(_, e)| e.clone())
                .collect()
        };
        let pre: Vec<Exposure> = prequential.iter().flat_map(|s| mine(s)).collect();
        let compat: Vec<Exposure> = latest.values().flat_map(|s| mine(s)).collect();
        let mut refuted: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for e in pre.iter().chain(&compat).filter(|e| !e.correct) {
            refuted
                .entry(e.dossier.clone())
                .or_default()
                .push(e.valuation.clone());
        }
        for (dossier, mut valuations) in refuted.clone() {
            valuations.sort();
            valuations.dedup();
            st.push(HypothesisStatus::Refuted {
                dossier,
                valuations,
            });
        }
        if refuted.is_empty() {
            let count = |es: &[Exposure]| {
                let ds: BTreeSet<&str> = es.iter().map(|e| e.dossier.as_str()).collect();
                (ds.len(), es.len())
            };
            if !pre.is_empty() {
                let (dossiers, exposures) = count(&pre);
                st.push(HypothesisStatus::Prospective {
                    dossiers,
                    exposures,
                });
            }
            if !compat.is_empty() {
                let (dossiers, exposures) = count(&compat);
                st.push(HypothesisStatus::Compatible {
                    dossiers,
                    exposures,
                });
            }
        }
        if pre.is_empty() && compat.is_empty() {
            st.push(HypothesisStatus::Unexposed);
        }
        if h.case_specific {
            st.push(HypothesisStatus::CaseSpecific);
        }
        let child = hypotheses
            .iter()
            .find(|c| c.parent.as_deref() == Some(h.id.as_str()) && c.committed_ok);
        if let (Some(_), Some(child)) = (h.retired_at, child) {
            st.push(HypothesisStatus::SupersededBy(child.id.clone()));
        }
        out.insert(h.id.clone(), st);
    }
    out
}

/// Run one trajectory. The learner sees only `LearnerView`; the reserved
/// valuations are read by `score` after each commitment.
pub fn run_trajectory(
    manifest: &TrajectoryManifest,
    reserved: &ReservedValuations,
    learner: &mut dyn Learner,
) -> Result<TrajectoryReport, Error> {
    if manifest.order.is_empty() {
        return Err(Error::new("trajectory: empty dossier order"));
    }
    if manifest.question_first && manifest.budgets.max_navigation_actions == 0 {
        return Err(Error::new(
            "trajectory: question_first needs navigation to consult Residuum",
        ));
    }
    // `admit` does not read D; `Episode::prepare` only validates one. The
    // placeholder observes an arbitrary seed and is never evaluated.
    let anchor = manifest
        .order
        .iter()
        .flat_map(|d| d.seeds.iter())
        .next()
        .cloned()
        .ok_or_else(|| Error::new("trajectory: no seed to anchor the placeholder decision"))?;
    let placeholder = Decision {
        components: vec![anchor],
        criterion: Criterion::Component(0),
    };
    let ctx = ProposalContext {
        imports: &manifest.imports,
        workdir: &manifest.workdir,
        profile: &manifest.profile,
        max_clauses: manifest.budgets.max_clauses,
        seeds: &[],
        decision: &placeholder,
    };
    let episode = Episode::prepare(&ctx)?;
    let baseline_constants = episode
        .baseline
        .declarations
        .iter()
        .filter(|d| matches!(d, Declaration::Const(_)))
        .filter(|d| {
            episode.baseline.origin(d.name()).map(|o| o.module.as_str())
                == Some(manifest.base_module.as_str())
        })
        .map(|d| d.name().to_string())
        .collect();
    let mut state = State {
        manifest,
        reserved,
        episode,
        placeholder,
        baseline_constants,
        hypotheses: Vec::new(),
        committed: Vec::new(),
        feedback: Vec::new(),
        counters: Counters::default(),
        ledger: Vec::new(),
        prequential: Vec::new(),
        compatibility: Vec::new(),
        last_result: None,
        navigator: None,
        navigation_ids: BTreeMap::new(),
        navigation_serial: 0,
        last_navigation: None,
        findings: Vec::new(),
        commit_traces: Vec::new(),
    };
    for index in 0..manifest.order.len() {
        state.open_navigator(index)?;
        state.phase(learner, index, Phase::Explore);
        let scoring = state.score(index, true)?;
        state.log(
            index,
            None,
            "prequential",
            format!("committed={:?} loss={}", scoring.committed, scoring.loss),
        );
        let delivered = match manifest.condition {
            Condition::Feedback => FeedbackView::Discrepancies {
                dossier: scoring.dossier.clone(),
                missed: scoring.missed.iter().map(|v| v.position.clone()).collect(),
                excess: scoring
                    .excess
                    .iter()
                    .map(|(v, owners)| (v.position.clone(), owners.clone()))
                    .collect(),
                valued_total: scoring.valued,
                discrepant: scoring.loss,
            },
            Condition::NoFeedback => FeedbackView::Evaluated {
                dossier: scoring.dossier.clone(),
            },
        };
        state.feedback.push(delivered);
        state.prequential.push(scoring);
        state.phase(learner, index, Phase::Revise);
        let mut rescored = Vec::new();
        for seen in 0..=index {
            rescored.push(state.score(seen, false)?);
        }
        state.compatibility.push(rescored);
    }
    let statuses = statuses(&state.hypotheses, &state.prequential, &state.compatibility);
    Ok(TrajectoryReport {
        trajectory_id: manifest.trajectory_id.clone(),
        condition: manifest.condition,
        order: manifest.order.iter().map(|d| d.id.clone()).collect(),
        prequential: state.prequential,
        compatibility: state.compatibility,
        hypotheses: state.hypotheses,
        statuses,
        counters: state.counters,
        ledger: state.ledger,
        findings: state.findings,
        commit_traces: state.commit_traces,
    })
}
