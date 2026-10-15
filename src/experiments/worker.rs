//! In-process worker for the exploration protocol
//! (`docs/llm-explorer-invocation-protocol.md`, §5, §8, §9): owns the Rust
//! sessions of one episode, validates every request against the frozen
//! manifest, assigns identifiers, enforces budgets and idempotency, and maps
//! engine outcomes to the contract's status table.
//!
//! No transport, no serialisation, no controller, no provider: requests and
//! responses are typed DTOs. The worker only calls existing APIs
//! (`ForwardSession`, `ResidualSession`, `ActiveResidualSession`, the
//! proposal stages and the exploration analysis layer); it implements no
//! matching, substitution, closure, search or frontier of its own.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::elab::Error;
use crate::residual::{Abducible, HornQueryScope, QueryBinding, SearchLimits, SearchMetrics};
use crate::typed::{Closure, Declaration, Event, Module, Rule, VerbKind};
use crate::{ActiveResidualSession, ForwardSession, ResidualSession};

use super::Clock;
use super::exploration::{
    self, Decision, Partition, Profile, Supports, covers, discriminants, frontiers, goal_index,
    install, region_pool, seeded_indices, signature,
};
use super::proposal::{
    Admitted, Episode, EvaluationSession, ProposalContext, ProposalReport, ProposalStatus,
    SourceProposal, admit, check_scope, status_label,
};

pub const PROTOCOL_VERSION: &str = "residuum-worker/0.1";

// ---------------------------------------------------------------------------
// Manifest and budgets (frozen before the first request).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budgets {
    pub max_actions: usize,
    pub max_steps_per_action: usize,
    pub max_steps_total: usize,
    pub max_sessions: usize,
    pub max_candidates: usize,
    pub max_clauses: usize,
    pub search_limits: SearchLimits,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub episode_id: String,
    /// Opaque, assigned by the controller; every envelope must repeat it.
    pub manifest_hash: String,
    pub imports: Vec<PathBuf>,
    pub workdir: PathBuf,
    pub profile: Profile,
    pub decision: Decision,
    /// Dossier identifier → seeds. Only these dossiers can be referenced.
    pub dossiers: BTreeMap<String, Vec<Event>>,
    pub budgets: Budgets,
}

// ---------------------------------------------------------------------------
// Requests.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub protocol_version: String,
    pub episode_id: String,
    pub action_id: String,
    pub manifest_hash: String,
    pub request: Request,
}

/// `theory` is `None` for the baseline theory or `Some(candidate_id)` for an
/// admitted candidate's private module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    InspectTheory,
    BeginForward {
        dossier: String,
        theory: Option<String>,
    },
    BeginResidual {
        dossier: String,
        theory: Option<String>,
        goal: Event,
        scope: Option<HornQueryScope>,
        bindings: Vec<QueryBinding>,
        allowed: Option<Vec<Abducible>>,
        limits: Option<SearchLimits>,
    },
    BeginActiveResidual {
        dossier: String,
        theory: Option<String>,
        goal: Event,
        scope: Option<HornQueryScope>,
        bindings: Vec<QueryBinding>,
        allowed: Option<Vec<Abducible>>,
        limits: Option<SearchLimits>,
    },
    AdvanceSession {
        session: String,
        max_steps: usize,
    },
    ProposeCandidate {
        text: String,
    },
    BeginCandidateEvaluation {
        candidate: String,
        dossier: String,
    },
    MinimalSupports {
        candidate: String,
        goal: Event,
    },
    ComparePartition {
        candidates: Vec<String>,
        goal: Event,
    },
}

// ---------------------------------------------------------------------------
// Responses.

/// The contract's status table (§3). `runner_limit` belongs to the process
/// supervisor, not to the worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Complete,
    StepLimit,
    SearchLimit,
    Cyclic,
    UnresolvedParameters,
    BindingError,
    StructuralReject,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheoryInventory {
    pub module: String,
    pub rules: Vec<(String, String)>,
    pub seeded_verbs: Vec<String>,
    pub derived_verbs: Vec<String>,
    pub effect_verbs: Vec<String>,
    pub dossiers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionKind {
    Forward,
    Residual,
    ActiveResidual,
    Evaluation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Theory(TheoryInventory),
    SessionOpened {
        session: String,
        kind: SessionKind,
    },
    Forward {
        session: String,
        round: usize,
        added: Vec<Event>,
        known: usize,
        complete: bool,
    },
    Residual {
        session: String,
        kind: SessionKind,
        round: usize,
        pending: Vec<Event>,
        branches: usize,
        metrics: SearchMetrics,
        complete: bool,
        /// Rendered requirement formula once complete; `None` before
        /// completion or when there is no explanation.
        requirement: Option<String>,
        explanations: usize,
    },
    Candidate {
        candidate: Option<String>,
        report: ProposalReport,
    },
    Evaluation {
        session: String,
        candidate: String,
        dossier: String,
        round: usize,
        complete: bool,
        report: Option<ProposalReport>,
    },
    Supports {
        candidate: String,
        goal_in_universe: bool,
        frontier: Vec<Vec<Event>>,
    },
    Partition {
        pool: Vec<Vec<Event>>,
        partition: Partition,
        signatures: Vec<Vec<bool>>,
        discriminants: Vec<usize>,
        /// `(general, specific)` pairs whose regions are ordered by inclusion.
        covers: Vec<(String, String)>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetState {
    pub actions_remaining: usize,
    pub steps_remaining: usize,
    pub sessions_remaining: usize,
    pub candidates_remaining: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub action_id: String,
    pub status: Status,
    pub payload: Payload,
    pub steps_consumed: usize,
    pub budget: BudgetState,
}

// ---------------------------------------------------------------------------
// Worker state.

enum Session {
    Forward {
        forward: ForwardSession,
        clock: Clock,
        round: usize,
    },
    Residual {
        session: ResidualSession,
        round: usize,
        failed: Option<Status>,
    },
    Active {
        session: ActiveResidualSession,
        clock: Clock,
        round: usize,
        failed: Option<Status>,
    },
    Evaluation {
        candidate: String,
        dossier: String,
        session: Option<EvaluationSession>,
        round: usize,
        report: Option<ProposalReport>,
    },
}

struct Candidate {
    admitted: Admitted,
    rules: Vec<Rule>,
}

pub struct Worker {
    manifest: Manifest,
    episode: Episode,
    references: BTreeMap<String, Closure>,
    sessions: BTreeMap<String, Session>,
    candidates: BTreeMap<String, Candidate>,
    /// `action_id` → (request fingerprint, response) for idempotent replay.
    done: BTreeMap<String, (String, Response)>,
    actions: usize,
    steps_total: usize,
    proposals: usize,
    next_session: usize,
    next_candidate: usize,
}

/// Map an engine `Error` to the status table by its documented message.
/// The engine's `Error` carries no kind; this mapping is by text and is
/// listed as a limitation of the worker.
fn classify_error(e: &Error) -> Status {
    let m = e.to_string();
    if m.contains("limit exceeded") {
        Status::SearchLimit
    } else if m.contains("cyclic residual search") {
        Status::Cyclic
    } else if m.contains("unresolved query parameters") {
        Status::UnresolvedParameters
    } else if m.contains("search scope") {
        Status::BindingError
    } else if m.contains("query binding")
        || m.contains("query scope")
        || m.contains("query parameter")
        || m.contains("query rule")
        || m.contains("query program")
        || m.contains("query record")
        || m.contains("temporal query")
    {
        Status::BindingError
    } else {
        Status::Error
    }
}

impl Worker {
    pub fn new(manifest: Manifest) -> Result<Self, Error> {
        if manifest.dossiers.is_empty() {
            return Err(Error::new("manifest declares no dossiers"));
        }
        let first = manifest.dossiers.values().next().unwrap();
        let ctx = ProposalContext {
            imports: &manifest.imports,
            workdir: &manifest.workdir,
            profile: &manifest.profile,
            max_clauses: manifest.budgets.max_clauses,
            seeds: first,
            decision: &manifest.decision,
        };
        let episode = Episode::prepare(&ctx)?;
        Ok(Self {
            manifest,
            episode,
            references: BTreeMap::new(),
            sessions: BTreeMap::new(),
            candidates: BTreeMap::new(),
            done: BTreeMap::new(),
            actions: 0,
            steps_total: 0,
            proposals: 0,
            next_session: 0,
            next_candidate: 0,
        })
    }

    pub fn budget(&self) -> BudgetState {
        let b = &self.manifest.budgets;
        BudgetState {
            actions_remaining: b.max_actions.saturating_sub(self.actions),
            steps_remaining: b.max_steps_total.saturating_sub(self.steps_total),
            sessions_remaining: b.max_sessions.saturating_sub(self.sessions.len()),
            candidates_remaining: b.max_candidates.saturating_sub(self.proposals),
        }
    }

    fn respond(&self, action_id: &str, status: Status, payload: Payload, steps: usize) -> Response {
        Response {
            action_id: action_id.into(),
            status,
            payload,
            steps_consumed: steps,
            budget: self.budget(),
        }
    }

    fn error(&self, action_id: &str, message: impl Into<String>) -> Response {
        self.respond(
            action_id,
            Status::Error,
            Payload::Error {
                message: message.into(),
            },
            0,
        )
    }

    /// Validate the envelope, replay or reject repeated identifiers, charge
    /// the action budget, dispatch, and record the response.
    pub fn handle(&mut self, envelope: Envelope) -> Response {
        let id = envelope.action_id.clone();
        if envelope.protocol_version != PROTOCOL_VERSION {
            return self.error(
                &id,
                format!(
                    "protocol: version `{}` unsupported",
                    envelope.protocol_version
                ),
            );
        }
        if envelope.episode_id != self.manifest.episode_id {
            return self.error(&id, "protocol: episode identifier does not match");
        }
        if envelope.manifest_hash != self.manifest.manifest_hash {
            return self.error(
                &id,
                "harness: manifest hash changed; episode must not continue",
            );
        }
        let fingerprint = format!("{:?}", envelope.request);
        if let Some((previous, response)) = self.done.get(&id) {
            if *previous == fingerprint {
                return response.clone();
            }
            return self.error(
                &id,
                "protocol: action identifier reused with different content",
            );
        }
        if self.actions >= self.manifest.budgets.max_actions {
            return self.error(&id, "budget: actions exhausted");
        }
        self.actions += 1;
        let response = self.dispatch(&id, envelope.request);
        self.done.insert(id, (fingerprint, response.clone()));
        response
    }

    fn dispatch(&mut self, id: &str, request: Request) -> Response {
        match request {
            Request::InspectTheory => self.inspect(id),
            Request::BeginForward { dossier, theory } => {
                self.begin_forward(id, &dossier, theory.as_deref())
            }
            Request::BeginResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => self.begin_residual(
                id,
                &dossier,
                theory.as_deref(),
                &goal,
                scope,
                &bindings,
                allowed,
                limits,
                false,
            ),
            Request::BeginActiveResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => self.begin_residual(
                id,
                &dossier,
                theory.as_deref(),
                &goal,
                scope,
                &bindings,
                allowed,
                limits,
                true,
            ),
            Request::AdvanceSession { session, max_steps } => self.advance(id, &session, max_steps),
            Request::ProposeCandidate { text } => self.propose(id, text),
            Request::BeginCandidateEvaluation { candidate, dossier } => {
                self.begin_evaluation(id, &candidate, &dossier)
            }
            Request::MinimalSupports { candidate, goal } => {
                self.minimal_supports(id, &candidate, &goal)
            }
            Request::ComparePartition { candidates, goal } => {
                self.compare_partition(id, &candidates, &goal)
            }
        }
    }

    // -- references -------------------------------------------------------

    fn seeds(&self, dossier: &str) -> Result<&Vec<Event>, String> {
        self.manifest
            .dossiers
            .get(dossier)
            .ok_or_else(|| format!("reference: unknown dossier `{dossier}`"))
    }

    fn theory(&self, theory: Option<&str>) -> Result<&Module, String> {
        match theory {
            None => Ok(&self.episode.baseline),
            Some(c) => self
                .candidates
                .get(c)
                .map(|slot| &slot.admitted.module)
                .ok_or_else(|| format!("reference: unknown candidate `{c}`")),
        }
    }

    fn reference(&mut self, dossier: &str) -> Result<Closure, Error> {
        if let Some(c) = self.references.get(dossier) {
            return Ok(c.clone());
        }
        let seeds = self.seeds(dossier).map_err(Error::new)?.clone();
        let closure = self.episode.reference(&seeds)?;
        self.references.insert(dossier.into(), closure.clone());
        Ok(closure)
    }

    fn new_session_id(&mut self) -> Result<String, String> {
        if self.sessions.len() >= self.manifest.budgets.max_sessions {
            return Err("budget: sessions exhausted".into());
        }
        self.next_session += 1;
        Ok(format!("s{}", self.next_session))
    }

    fn sigma(&self) -> Vec<Event> {
        self.manifest
            .dossiers
            .values()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    // -- operations -------------------------------------------------------

    fn inspect(&self, id: &str) -> Response {
        let m = &self.episode.baseline;
        let mut inventory = TheoryInventory {
            module: m.name.clone(),
            rules: Vec::new(),
            seeded_verbs: Vec::new(),
            derived_verbs: Vec::new(),
            effect_verbs: Vec::new(),
            dossiers: self.manifest.dossiers.keys().cloned().collect(),
        };
        for d in &m.declarations {
            match d {
                Declaration::Rule(r) => inventory.rules.push((
                    r.name.clone(),
                    m.origin(&r.name)
                        .map(|o| o.module.clone())
                        .unwrap_or_default(),
                )),
                Declaration::Verb(v) => match v.kind {
                    VerbKind::Seeded => inventory.seeded_verbs.push(v.name.clone()),
                    VerbKind::Derived => inventory.derived_verbs.push(v.name.clone()),
                    VerbKind::Effect => inventory.effect_verbs.push(v.name.clone()),
                },
                _ => {}
            }
        }
        self.respond(id, Status::Complete, Payload::Theory(inventory), 0)
    }

    fn begin_forward(&mut self, id: &str, dossier: &str, theory: Option<&str>) -> Response {
        let (seeds, module) = match (self.seeds(dossier), self.theory(theory)) {
            (Ok(s), Ok(m)) => (s.clone(), m.clone()),
            (Err(e), _) | (_, Err(e)) => return self.error(id, e),
        };
        if let Err(e) = super::require_program_acyclic(&module) {
            return self.error(id, e.to_string());
        }
        let forward = match ForwardSession::for_module(seeds, &module) {
            Ok(f) => f,
            Err(e) => return self.error(id, e.to_string()),
        };
        let sid = match self.new_session_id() {
            Ok(s) => s,
            Err(e) => return self.error(id, e),
        };
        self.sessions.insert(
            sid.clone(),
            Session::Forward {
                forward,
                clock: Clock::default(),
                round: 0,
            },
        );
        self.respond(
            id,
            Status::Complete,
            Payload::SessionOpened {
                session: sid,
                kind: SessionKind::Forward,
            },
            0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_residual(
        &mut self,
        id: &str,
        dossier: &str,
        theory: Option<&str>,
        goal: &Event,
        scope: Option<HornQueryScope>,
        bindings: &[QueryBinding],
        allowed: Option<Vec<Abducible>>,
        limits: Option<SearchLimits>,
        active: bool,
    ) -> Response {
        let (seeds, module) = match (self.seeds(dossier), self.theory(theory)) {
            (Ok(s), Ok(m)) => (s.clone(), m.clone()),
            (Err(e), _) | (_, Err(e)) => return self.error(id, e),
        };
        let limits = limits.unwrap_or(self.manifest.budgets.search_limits);
        let allowed = allowed.as_deref();
        let built: Result<Session, Error> = if active {
            if let Err(e) = super::require_program_acyclic(&module) {
                return self.error(id, e.to_string());
            }
            let session = match &scope {
                Some(scope) => crate::begin_active_residual_query_in_scope_with_bindings(
                    &module, scope, &seeds, goal, bindings, limits, allowed,
                )
                .map(|q| q.query),
                None if !bindings.is_empty() => crate::begin_active_residual_query_with_bindings(
                    &module, &seeds, goal, bindings, limits, allowed,
                )
                .map(|q| q.query),
                None => ActiveResidualSession::new(&module, &seeds, goal, limits, allowed),
            };
            session.map(|session| Session::Active {
                session,
                clock: Clock::default(),
                round: 0,
                failed: None,
            })
        } else {
            let session = match &scope {
                Some(scope) => crate::begin_residual_query_with_bindings(
                    &module, scope, &seeds, goal, bindings, limits, allowed,
                )
                .map(|q| q.query),
                None if !bindings.is_empty() => {
                    Err(Error::new("scoped residual query requires a search scope"))
                }
                None => ResidualSession::new(&module, &seeds, goal, limits, allowed),
            };
            session.map(|session| Session::Residual {
                session,
                round: 0,
                failed: None,
            })
        };
        let session = match built {
            Ok(s) => s,
            Err(e) => {
                let status = classify_error(&e);
                return self.respond(
                    id,
                    status,
                    Payload::Error {
                        message: e.to_string(),
                    },
                    0,
                );
            }
        };
        let sid = match self.new_session_id() {
            Ok(s) => s,
            Err(e) => return self.error(id, e),
        };
        let kind = if active {
            SessionKind::ActiveResidual
        } else {
            SessionKind::Residual
        };
        self.sessions.insert(sid.clone(), session);
        self.respond(
            id,
            Status::Complete,
            Payload::SessionOpened { session: sid, kind },
            0,
        )
    }

    fn step_allowance(&self, requested: usize) -> usize {
        requested
            .min(self.manifest.budgets.max_steps_per_action)
            .min(
                self.manifest
                    .budgets
                    .max_steps_total
                    .saturating_sub(self.steps_total),
            )
    }

    fn advance(&mut self, id: &str, sid: &str, max_steps: usize) -> Response {
        if max_steps == 0 {
            return self.error(id, "protocol: max_steps must be at least 1");
        }
        let allowance = self.step_allowance(max_steps);
        if allowance == 0 {
            return self.error(id, "budget: steps exhausted");
        }
        let advanced = match self.sessions.get_mut(sid) {
            None => return self.error(id, format!("reference: unknown session `{sid}`")),
            Some(session) => advance_session(sid, session, allowance),
        };
        self.steps_total += advanced.consumed;
        self.respond(id, advanced.status, advanced.payload, advanced.consumed)
    }

    fn propose(&mut self, id: &str, text: String) -> Response {
        if self.proposals >= self.manifest.budgets.max_candidates {
            return self.error(id, "budget: candidates exhausted");
        }
        self.proposals += 1;
        self.next_candidate += 1;
        let cid = format!("c{}", self.next_candidate);
        let first = self.manifest.dossiers.values().next().unwrap().clone();
        let ctx = ProposalContext {
            imports: &self.manifest.imports,
            workdir: &self.manifest.workdir,
            profile: &self.manifest.profile,
            max_clauses: self.manifest.budgets.max_clauses,
            seeds: &first,
            decision: &self.manifest.decision,
        };
        let proposal = SourceProposal {
            id: cid.clone(),
            text,
        };
        match admit(&proposal, &ctx, &self.episode) {
            Err(e) => self.error(id, format!("harness: {e}")),
            Ok(Err(report)) => self.respond(
                id,
                Status::StructuralReject,
                Payload::Candidate {
                    candidate: None,
                    report,
                },
                0,
            ),
            Ok(Ok(admitted)) => {
                let report = admitted.report.clone();
                let rules = admitted
                    .module
                    .declarations
                    .iter()
                    .filter_map(|d| match d {
                        Declaration::Rule(r)
                            if admitted.module.origin(&r.name).map(|o| o.module.as_str())
                                == Some(admitted.report.module_name.as_str()) =>
                        {
                            Some(r.clone())
                        }
                        _ => None,
                    })
                    .collect();
                self.candidates
                    .insert(cid.clone(), Candidate { admitted, rules });
                self.respond(
                    id,
                    Status::Complete,
                    Payload::Candidate {
                        candidate: Some(cid),
                        report,
                    },
                    0,
                )
            }
        }
    }

    fn begin_evaluation(&mut self, id: &str, cid: &str, dossier: &str) -> Response {
        let seeds = match self.seeds(dossier) {
            Ok(s) => s.clone(),
            Err(e) => return self.error(id, e),
        };
        let Some(slot) = self.candidates.get(cid) else {
            return self.error(id, format!("reference: unknown candidate `{cid}`"));
        };
        let admitted = slot.admitted.clone();
        let reference = match self.reference(dossier) {
            Ok(c) => c,
            Err(e) => {
                return self.respond(
                    id,
                    Status::Error,
                    Payload::Candidate {
                        candidate: Some(cid.into()),
                        report: admitted.report.clone().with_status(
                            ProposalStatus::EvaluationPending(format!("reference closure: {e}")),
                        ),
                    },
                    0,
                );
            }
        };
        let admitted = match check_scope(admitted, &reference) {
            Ok(a) => a,
            Err(report) => {
                return self.respond(
                    id,
                    Status::Complete,
                    Payload::Candidate {
                        candidate: Some(cid.into()),
                        report,
                    },
                    0,
                );
            }
        };
        let ctx = ProposalContext {
            imports: &self.manifest.imports,
            workdir: &self.manifest.workdir,
            profile: &self.manifest.profile,
            max_clauses: self.manifest.budgets.max_clauses,
            seeds: &seeds,
            decision: &self.manifest.decision,
        };
        let session = match EvaluationSession::begin(admitted, &ctx, reference) {
            Ok(s) => s,
            Err(report) => {
                return self.respond(
                    id,
                    Status::Error,
                    Payload::Candidate {
                        candidate: Some(cid.into()),
                        report,
                    },
                    0,
                );
            }
        };
        let sid = match self.new_session_id() {
            Ok(s) => s,
            Err(e) => return self.error(id, e),
        };
        self.sessions.insert(
            sid.clone(),
            Session::Evaluation {
                candidate: cid.into(),
                dossier: dossier.into(),
                session: Some(session),
                round: 0,
                report: None,
            },
        );
        self.respond(
            id,
            Status::Complete,
            Payload::SessionOpened {
                session: sid,
                kind: SessionKind::Evaluation,
            },
            0,
        )
    }

    /// Joint universe of the baseline with every admitted candidate's rules.
    fn joint(&self) -> Result<(Module, Vec<Event>, BTreeSet<usize>), Error> {
        let rules: Vec<Rule> = self
            .candidates
            .values()
            .flat_map(|c| c.rules.clone())
            .collect();
        let (joint, e) =
            exploration::joint_universe(&self.episode.baseline, &[], &rules, &self.sigma())?;
        let allowed = seeded_indices(&joint, &e);
        Ok((joint, e, allowed))
    }

    fn candidate_frontier(
        &self,
        cid: &str,
        e: &[Event],
        allowed: &BTreeSet<usize>,
        goal: &Event,
    ) -> Result<(bool, Supports), Error> {
        let slot = self
            .candidates
            .get(cid)
            .ok_or_else(|| Error::new(format!("reference: unknown candidate `{cid}`")))?;
        let module = install(
            &self.episode.baseline,
            &[],
            &slot.rules,
            &slot.admitted.report.module_name,
        )?;
        let f = frontiers(&module, e, allowed)?;
        Ok(match goal_index(e, goal) {
            Some(q) => (true, f.supports[q].clone()),
            None => (false, Supports::new()),
        })
    }

    fn minimal_supports(&mut self, id: &str, cid: &str, goal: &Event) -> Response {
        let (_, e, allowed) = match self.joint() {
            Ok(j) => j,
            Err(err) => return self.error(id, err.to_string()),
        };
        match self.candidate_frontier(cid, &e, &allowed, goal) {
            Err(err) => self.error(id, err.to_string()),
            Ok((goal_in_universe, frontier)) => self.respond(
                id,
                Status::Complete,
                Payload::Supports {
                    candidate: cid.into(),
                    goal_in_universe,
                    frontier: frontier
                        .iter()
                        .map(|s| exploration::state_events(&e, s))
                        .collect(),
                },
                0,
            ),
        }
    }

    fn compare_partition(&mut self, id: &str, cids: &[String], goal: &Event) -> Response {
        if cids.len() < 2 {
            return self.error(id, "protocol: a partition needs at least two candidates");
        }
        let (_, e, allowed) = match self.joint() {
            Ok(j) => j,
            Err(err) => return self.error(id, err.to_string()),
        };
        let mut fronts = Vec::new();
        for cid in cids {
            match self.candidate_frontier(cid, &e, &allowed, goal) {
                Ok((_, f)) => fronts.push(f),
                Err(err) => return self.error(id, err.to_string()),
            }
        }
        let pool = region_pool(fronts.iter());
        let signatures: Vec<Vec<bool>> = fronts.iter().map(|f| signature(f, &pool)).collect();
        let partition = match Partition::new(cids.to_vec(), signatures.clone()) {
            Ok(p) => p,
            Err(err) => return self.error(id, err.to_string()),
        };
        let split = discriminants(&signatures);
        let mut order = Vec::new();
        for (i, a) in cids.iter().enumerate() {
            for (j, b) in cids.iter().enumerate() {
                if i != j && covers(&fronts[i], &fronts[j]) {
                    order.push((a.clone(), b.clone()));
                }
            }
        }
        self.respond(
            id,
            Status::Complete,
            Payload::Partition {
                pool: pool
                    .iter()
                    .map(|s| exploration::state_events(&e, s))
                    .collect(),
                partition,
                signatures,
                discriminants: split,
                covers: order,
            },
            0,
        )
    }
}

/// Stable label of a candidate report for logs.
pub fn report_label(report: &ProposalReport) -> &'static str {
    status_label(&report.status)
}

struct Advanced {
    status: Status,
    payload: Payload,
    consumed: usize,
}

const TERMINAL: &str = "session failed earlier; errors are terminal";

fn residual_payload(
    sid: &str,
    kind: SessionKind,
    round: usize,
    pending: Vec<Event>,
    branches: usize,
    metrics: SearchMetrics,
    complete: bool,
    requirement: Option<String>,
    explanations: usize,
) -> Payload {
    Payload::Residual {
        session: sid.into(),
        kind,
        round,
        pending,
        branches,
        metrics,
        complete,
        requirement,
        explanations,
    }
}

fn completion(complete: bool) -> Status {
    if complete {
        Status::Complete
    } else {
        Status::StepLimit
    }
}

/// Advance one session by at most `allowance` transitions. Borrows only the
/// session, so the worker can account the steps afterwards.
fn advance_session(sid: &str, session: &mut Session, allowance: usize) -> Advanced {
    let mut consumed = 0;
    match session {
        Session::Forward {
            forward,
            clock,
            round,
        } => {
            let mut last_added = Vec::new();
            let mut complete = forward.is_complete();
            while !complete && consumed < allowance {
                let step = forward.step(clock);
                consumed += 1;
                *round = step.round;
                last_added = step.added;
                complete = step.complete;
            }
            Advanced {
                status: completion(complete),
                payload: Payload::Forward {
                    session: sid.into(),
                    round: *round,
                    added: last_added,
                    known: forward.closure().known.len(),
                    complete,
                },
                consumed,
            }
        }
        Session::Residual {
            session,
            round,
            failed,
        } => {
            if let Some(status) = *failed {
                return Advanced {
                    status,
                    payload: Payload::Error {
                        message: TERMINAL.into(),
                    },
                    consumed: 0,
                };
            }
            if let Some(r) = session.result() {
                return Advanced {
                    status: Status::Complete,
                    payload: residual_payload(
                        sid,
                        SessionKind::Residual,
                        *round,
                        Vec::new(),
                        0,
                        r.metrics.clone(),
                        true,
                        r.required.as_ref().map(super::render_requirement),
                        r.explanations.len(),
                    ),
                    consumed: 0,
                };
            }
            let mut last = None;
            while consumed < allowance {
                match session.step() {
                    Ok(step) => {
                        consumed += 1;
                        *round = step.round;
                        let complete = step.complete;
                        last = Some(step);
                        if complete {
                            break;
                        }
                    }
                    Err(e) => {
                        let status = classify_error(&e);
                        *failed = Some(status);
                        return Advanced {
                            status,
                            payload: Payload::Error {
                                message: e.to_string(),
                            },
                            consumed,
                        };
                    }
                }
            }
            let step = last.expect("allowance is at least one");
            let (requirement, explanations) = match session.result() {
                Some(r) => (
                    r.required.as_ref().map(super::render_requirement),
                    r.explanations.len(),
                ),
                None => (None, 0),
            };
            Advanced {
                status: completion(step.complete),
                payload: residual_payload(
                    sid,
                    SessionKind::Residual,
                    step.round,
                    step.pending,
                    step.branches.len(),
                    step.metrics,
                    step.complete,
                    requirement,
                    explanations,
                ),
                consumed,
            }
        }
        Session::Active {
            session,
            clock,
            round,
            failed,
        } => {
            if let Some(status) = *failed {
                return Advanced {
                    status,
                    payload: Payload::Error {
                        message: TERMINAL.into(),
                    },
                    consumed: 0,
                };
            }
            if let Some(r) = session.result() {
                return Advanced {
                    status: Status::Complete,
                    payload: residual_payload(
                        sid,
                        SessionKind::ActiveResidual,
                        *round,
                        Vec::new(),
                        0,
                        r.metrics.clone(),
                        true,
                        r.required.as_ref().map(super::render_requirement),
                        r.explanations.len(),
                    ),
                    consumed: 0,
                };
            }
            let mut last = None;
            while consumed < allowance {
                match session.step(clock) {
                    Ok(step) => {
                        consumed += 1;
                        *round = step.round;
                        let complete = step.complete;
                        last = Some(step);
                        if complete {
                            break;
                        }
                    }
                    Err(e) => {
                        let status = classify_error(&e);
                        *failed = Some(status);
                        return Advanced {
                            status,
                            payload: Payload::Error {
                                message: e.to_string(),
                            },
                            consumed,
                        };
                    }
                }
            }
            let step = last.expect("allowance is at least one");
            let (requirement, explanations) = match session.result() {
                Some(r) => (
                    r.required.as_ref().map(super::render_requirement),
                    r.explanations.len(),
                ),
                None => (None, 0),
            };
            Advanced {
                status: completion(step.complete),
                payload: residual_payload(
                    sid,
                    SessionKind::ActiveResidual,
                    step.round,
                    step.pending,
                    step.branches.len(),
                    step.metrics,
                    step.complete,
                    requirement,
                    explanations,
                ),
                consumed,
            }
        }
        Session::Evaluation {
            candidate,
            dossier,
            session: slot,
            round,
            report,
        } => {
            if let Some(report) = report {
                return Advanced {
                    status: Status::Complete,
                    payload: Payload::Evaluation {
                        session: sid.into(),
                        candidate: candidate.clone(),
                        dossier: dossier.clone(),
                        round: *round,
                        complete: true,
                        report: Some(report.clone()),
                    },
                    consumed: 0,
                };
            }
            let open = slot.as_mut().expect("open evaluation session");
            let mut complete = open.is_complete();
            while !complete && consumed < allowance {
                let step = open.step();
                consumed += 1;
                *round = step.round;
                complete = step.complete;
            }
            let finished = if complete {
                let finished = slot.take().expect("open evaluation session").finish();
                *report = Some(finished.clone());
                Some(finished)
            } else {
                None
            };
            Advanced {
                status: completion(complete),
                payload: Payload::Evaluation {
                    session: sid.into(),
                    candidate: candidate.clone(),
                    dossier: dossier.clone(),
                    round: *round,
                    complete,
                    report: finished,
                },
                consumed,
            }
        }
    }
}
