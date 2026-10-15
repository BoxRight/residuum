//! Source path for text proposals (exploration contract §2.6, §6.3):
//!
//! ```text
//! Formal text → parse and frontier check → private module importing the
//! theory → candidate rules → structural admission → certificate scope →
//! evaluation (full closure, or stepwise through ForwardSession)
//! ```
//!
//! The stages are separate functions so that a tool which promises not to
//! evaluate can stop at admission, and so that evaluation can advance by
//! bounded steps. `process` is their composition and keeps its outcome.
//! Every stage reports a distinct status. Tool misconfiguration (bad id,
//! missing work directory, unloadable theory, invalid `D`) is an `Err`; it
//! is never turned into a status about the proposal.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use crate::ForwardSession;
use crate::elab::Error;
use crate::surface::FormalDeclaration;
use crate::typed::{Closure, Declaration, Event, Rule, RuleBody};

use super::Clock;
use super::exploration::{
    Conservation, Decision, Profile, certify_addition, classify_candidate, read_decision,
    theory_preserved,
};

/// A proposal as the explorer produces it: an identifier and Formal rule
/// declarations, without module header or imports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProposal {
    pub id: String,
    pub text: String,
}

/// Fixed per episode. `imports` are the theory and problem-manifest files the
/// private module imports; `workdir` receives the generated `.res` files.
#[derive(Debug, Clone)]
pub struct ProposalContext<'a> {
    pub imports: &'a [PathBuf],
    pub workdir: &'a Path,
    pub profile: &'a Profile,
    pub max_clauses: usize,
    pub seeds: &'a [Event],
    pub decision: &'a Decision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    /// `S_SYNTAX`: the composed source does not parse. The text is kept.
    Syntax(String),
    /// `S_ELABORATION`: references, types or bindings rejected by the
    /// elaborator or the module resolver.
    Elaboration(String),
    /// Profile codes of §2.6, plus `S_IMPORT` (the text contains an import),
    /// `S_DECLARATION` (it declares something other than rules) and
    /// `S_ORIGIN` (theory not preserved). `S_IMPORT` and `S_DECLARATION` are
    /// decided on the parsed AST, before any file is resolved.
    StructuralReject(BTreeSet<String>),
    /// Admitted, but some head is already produced by the reference closure:
    /// the addition certificate's proven scope does not cover it (§7, limit 3).
    OutOfCertificateScope(BTreeSet<String>),
    /// The evaluator refused, could not complete, or was stopped by its step
    /// budget. Not a refutation and not a compatibility verdict.
    EvaluationPending(String),
    /// The closure completed but the addition did not conserve the reference
    /// outside its declared heads (new events, new justifications or
    /// undeclared heads). A technical status: `D` is not read, no
    /// compatibility is reported.
    CertificateFailed(Conservation),
    /// Evaluated and certified; `compatible` reads `D` through its criterion.
    Evaluated {
        components: Vec<bool>,
        compatible: bool,
        certificate: Conservation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalReport {
    pub id: String,
    pub module_name: String,
    /// The proposal exactly as received.
    pub original_text: String,
    /// The composed private module source that was parsed.
    pub source: String,
    pub file: PathBuf,
    /// Candidate rules extracted after elaboration; empty before that stage.
    pub rules: Vec<Rule>,
    /// Stages completed, in order.
    pub stages: Vec<&'static str>,
    pub status: ProposalStatus,
}

impl ProposalReport {
    fn with(mut self, status: ProposalStatus) -> Self {
        self.status = status;
        self
    }
    /// Replace the status, keeping text, stages and rules.
    pub fn with_status(self, status: ProposalStatus) -> Self {
        self.with(status)
    }
}

fn module_name(id: &str) -> Result<String, Error> {
    if id.is_empty()
        || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        || !id.chars().next().unwrap().is_ascii_alphabetic()
    {
        return Err(Error::new(format!(
            "proposal id `{id}` must be an ASCII identifier starting with a letter"
        )));
    }
    Ok(format!("Candidate{id}"))
}

/// Relative path from directory `from` to file `to`, both canonical.
fn relative(from: &Path, to: &Path) -> Result<String, Error> {
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".into(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    if parts.is_empty() {
        return Err(Error::new("import path resolves to the work directory"));
    }
    let joined = parts.join("/");
    if joined.contains('"') || joined.contains('\\') {
        return Err(Error::new(format!(
            "import path `{joined}` is not expressible"
        )));
    }
    Ok(joined)
}

fn compose(name: &str, imports: &[String], text: &str) -> String {
    let mut source = format!("module {name}\n\n");
    for import in imports {
        source.push_str(&format!("import \"{import}\"\n"));
    }
    source.push('\n');
    source.push_str(text);
    if !text.ends_with('\n') {
        source.push('\n');
    }
    source
}

fn write(path: &Path, source: &str) -> Result<(), Error> {
    std::fs::write(path, source)
        .map_err(|e| Error::new(format!("cannot write `{}`: {e}", path.display())))
}

// ---------------------------------------------------------------------------
// Episode preparation: resolved imports and the baseline theory.

/// Episode-level state shared by every proposal: canonical work directory,
/// import paths relative to it and the baseline module (same imports, no
/// proposal). Its failure is a harness error.
#[derive(Debug, Clone)]
pub struct Episode {
    workdir: PathBuf,
    imports: Vec<String>,
    pub baseline: crate::typed::Module,
}

impl Episode {
    pub fn prepare(ctx: &ProposalContext) -> Result<Self, Error> {
        ctx.decision.validate()?;
        std::fs::create_dir_all(ctx.workdir)
            .map_err(|e| Error::new(format!("cannot create `{}`: {e}", ctx.workdir.display())))?;
        let workdir = ctx
            .workdir
            .canonicalize()
            .map_err(|e| Error::new(format!("cannot resolve `{}`: {e}", ctx.workdir.display())))?;
        let mut imports = Vec::new();
        for import in ctx.imports {
            let target = import.canonicalize().map_err(|e| {
                Error::new(format!("cannot resolve import `{}`: {e}", import.display()))
            })?;
            imports.push(relative(&workdir, &target)?);
        }
        if imports.is_empty() {
            return Err(Error::new(
                "a proposal context needs at least one theory import",
            ));
        }
        let baseline_file = workdir.join("Baseline.res");
        write(&baseline_file, &compose("Baseline", &imports, ""))?;
        let baseline = crate::load_formal_module(&baseline_file)
            .map_err(|e| Error::new(format!("theory imports do not elaborate: {e}")))?;
        Ok(Self {
            workdir,
            imports,
            baseline,
        })
    }

    /// Reference closure of the baseline on the episode seeds. Computed by
    /// the verifier, once per dossier; a failure is reported as pending.
    pub fn reference(&self, seeds: &[Event]) -> Result<Closure, Error> {
        super::require_program_acyclic(&self.baseline)?;
        crate::close_program(seeds.to_vec(), &self.baseline, &mut Clock::default())
    }
}

// ---------------------------------------------------------------------------
// Admission: parse, frontier check, elaborate, extract, structural profile.

/// A proposal that passed admission and is ready for scope check and
/// evaluation. Holds the elaborated private module.
#[derive(Debug, Clone)]
pub struct Admitted {
    pub report: ProposalReport,
    pub module: crate::typed::Module,
    pub heads: BTreeSet<String>,
}

/// Stages 1-4. `Ok(Err(report))` is a rejected proposal with its status;
/// `Err` is a harness failure.
pub fn admit(
    proposal: &SourceProposal,
    ctx: &ProposalContext,
    episode: &Episode,
) -> Result<Result<Admitted, ProposalReport>, Error> {
    let name = module_name(&proposal.id)?;
    let source = compose(&name, &episode.imports, &proposal.text);
    let file = episode.workdir.join(format!("{name}.res"));
    let mut report = ProposalReport {
        id: proposal.id.clone(),
        module_name: name.clone(),
        original_text: proposal.text.clone(),
        source: source.clone(),
        file: file.clone(),
        rules: Vec::new(),
        stages: Vec::new(),
        status: ProposalStatus::Syntax(String::new()),
    };

    // A rejected proposal leaves no stale private module under its id.
    if file.exists() {
        std::fs::remove_file(&file)
            .map_err(|e| Error::new(format!("cannot remove `{}`: {e}", file.display())))?;
    }

    // 1. Syntax, on the composed source, before anything touches the disk.
    let surface = match crate::parse_formal(&source) {
        Ok(s) => s,
        Err(e) => return Ok(Err(report.with(ProposalStatus::Syntax(e.to_string())))),
    };
    report.stages.push("parsed");

    // 1b. Frontier: beyond the controller's imports, the text may contain
    // rules only. Decided on the AST, before any import is resolved.
    let mut reasons = BTreeSet::new();
    let mut imports_seen = 0;
    for d in &surface.declarations {
        match d {
            FormalDeclaration::Import(_) => {
                imports_seen += 1;
                if imports_seen > episode.imports.len() {
                    reasons.insert("S_IMPORT".into());
                }
            }
            FormalDeclaration::Rule(_) => {}
            _ => {
                reasons.insert("S_DECLARATION".into());
            }
        }
    }
    if !reasons.is_empty() {
        return Ok(Err(report.with(ProposalStatus::StructuralReject(reasons))));
    }

    // 2. Elaboration in the private module with the theory imported.
    write(&file, &source)?;
    let module = match crate::load_formal_module(&file) {
        Ok(m) => m,
        Err(e) => {
            return Ok(Err(report.with(ProposalStatus::Elaboration(e.to_string()))));
        }
    };
    report.stages.push("elaborated");

    // 3. Extraction of the candidate declarations by origin.
    let mut rules = Vec::new();
    for d in &module.declarations {
        if module.origin(d.name()).map(|o| o.module.as_str()) != Some(name.as_str()) {
            continue;
        }
        match d {
            Declaration::Rule(r) => rules.push(r.clone()),
            _ => {
                reasons.insert("S_DECLARATION".into());
            }
        }
    }
    report.rules = rules.clone();
    report.stages.push("extracted");
    if theory_preserved(&episode.baseline, &module).is_err() {
        reasons.insert("S_ORIGIN".into());
    }

    // 4. Structural admissibility.
    reasons.extend(classify_candidate(
        &module,
        &rules,
        ctx.profile,
        ctx.max_clauses,
    ));
    if !reasons.is_empty() {
        return Ok(Err(report.with(ProposalStatus::StructuralReject(reasons))));
    }
    report.stages.push("admitted");
    let heads = rules
        .iter()
        .filter_map(|r| match &r.body {
            RuleBody::HornClause(c) => Some(c.consequent.verb.clone()),
            RuleBody::ResidualClause(_) => None,
        })
        .collect();
    Ok(Ok(Admitted {
        report,
        module,
        heads,
    }))
}

// ---------------------------------------------------------------------------
// Certificate scope.

/// Stage 5: the candidate heads must be absent from the reference closure.
pub fn check_scope(
    mut admitted: Admitted,
    reference: &Closure,
) -> Result<Admitted, ProposalReport> {
    let produced: BTreeSet<String> = reference
        .known
        .iter()
        .filter(|e| admitted.heads.contains(&e.verb))
        .map(|e| e.verb.clone())
        .collect();
    if !produced.is_empty() {
        return Err(admitted
            .report
            .with(ProposalStatus::OutOfCertificateScope(produced)));
    }
    admitted.report.stages.push("in_certificate_scope");
    Ok(admitted)
}

// ---------------------------------------------------------------------------
// Evaluation: shared reading of a completed closure.

fn conclude(
    mut report: ProposalReport,
    closure: &Closure,
    reference: &Closure,
    heads: &BTreeSet<String>,
    decision: &Decision,
) -> ProposalReport {
    // Acceptance requires, jointly: completed closure, valid certificate,
    // and only then the reading of D. A failed certificate is a technical
    // status, never a normative verdict.
    let certificate = match certify_addition(reference, closure, heads) {
        Ok(c) => c,
        Err(e) => return report.with(ProposalStatus::EvaluationPending(e.to_string())),
    };
    report.stages.push("closed");
    if !certificate.holds() {
        return report.with(ProposalStatus::CertificateFailed(certificate));
    }
    report.stages.push("certified");
    let (components, compatible) = match read_decision(closure, decision) {
        Ok(read) => read,
        Err(e) => return report.with(ProposalStatus::EvaluationPending(e.to_string())),
    };
    report.stages.push("evaluated");
    report.with(ProposalStatus::Evaluated {
        components,
        compatible,
        certificate,
    })
}

/// Stage 6 by full closure: `close_program` of the private module.
pub fn evaluate_admitted(
    admitted: Admitted,
    ctx: &ProposalContext,
    reference: &Closure,
) -> ProposalReport {
    let Admitted {
        report,
        module,
        heads,
    } = admitted;
    if let Err(e) = super::require_program_acyclic(&module) {
        return report.with(ProposalStatus::EvaluationPending(e.to_string()));
    }
    let closure = match crate::close_program(ctx.seeds.to_vec(), &module, &mut Clock::default()) {
        Ok(c) => c,
        Err(e) => return report.with(ProposalStatus::EvaluationPending(e.to_string())),
    };
    conclude(report, &closure, reference, &heads, ctx.decision)
}

/// Stage 6 by bounded steps: the existing `ForwardSession::for_module` over
/// the private module. Each `step` is one synchronous round `T(X)`; only a
/// completed session is read through `D` and certified. Stopping early
/// yields `EvaluationPending`, never a verdict.
pub struct EvaluationSession {
    report: ProposalReport,
    heads: BTreeSet<String>,
    reference: Closure,
    decision: Decision,
    forward: ForwardSession,
    clock: Clock,
    pub rounds: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationStep {
    pub round: usize,
    pub added: usize,
    pub derivations: usize,
    pub complete: bool,
}

impl EvaluationSession {
    pub fn begin(
        admitted: Admitted,
        ctx: &ProposalContext,
        reference: Closure,
    ) -> Result<Self, ProposalReport> {
        let Admitted {
            report,
            module,
            heads,
        } = admitted;
        if let Err(e) = super::require_program_acyclic(&module) {
            return Err(report.with(ProposalStatus::EvaluationPending(e.to_string())));
        }
        let forward = match ForwardSession::for_module(ctx.seeds.to_vec(), &module) {
            Ok(f) => f,
            Err(e) => return Err(report.with(ProposalStatus::EvaluationPending(e.to_string()))),
        };
        Ok(Self {
            report,
            heads,
            reference,
            decision: ctx.decision.clone(),
            forward,
            clock: Clock::default(),
            rounds: 0,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.forward.is_complete()
    }

    pub fn report(&self) -> &ProposalReport {
        &self.report
    }

    /// The closure built so far; complete only when `is_complete()`.
    pub fn closure(&self) -> &Closure {
        self.forward.closure()
    }

    pub fn step(&mut self) -> EvaluationStep {
        let step = self.forward.step(&mut self.clock);
        self.rounds = step.round;
        EvaluationStep {
            round: step.round,
            added: step.added.len(),
            derivations: step.derivations.len(),
            complete: step.complete,
        }
    }

    /// Read `D` and certify if complete; otherwise pending with the rounds
    /// consumed, so a step budget is never mistaken for a result.
    pub fn finish(self) -> ProposalReport {
        if !self.forward.is_complete() {
            return self.report.with(ProposalStatus::EvaluationPending(format!(
                "forward session stopped after {} rounds without reaching its fixed point",
                self.rounds
            )));
        }
        conclude(
            self.report,
            self.forward.closure(),
            &self.reference,
            &self.heads,
            &self.decision,
        )
    }
}

// ---------------------------------------------------------------------------
// Composition.

/// Run the whole source path with full-closure evaluation. `Err` means the
/// harness itself is misconfigured.
pub fn process(proposal: &SourceProposal, ctx: &ProposalContext) -> Result<ProposalReport, Error> {
    let episode = Episode::prepare(ctx)?;
    let admitted = match admit(proposal, ctx, &episode)? {
        Ok(a) => a,
        Err(report) => return Ok(report),
    };
    let reference = match episode.reference(ctx.seeds) {
        Ok(c) => c,
        Err(e) => {
            return Ok(admitted
                .report
                .with(ProposalStatus::EvaluationPending(format!(
                    "reference closure: {e}"
                ))));
        }
    };
    let admitted = match check_scope(admitted, &reference) {
        Ok(a) => a,
        Err(report) => return Ok(report),
    };
    Ok(evaluate_admitted(admitted, ctx, &reference))
}

/// Convenience for logs: the status family as a stable label.
pub fn status_label(status: &ProposalStatus) -> &'static str {
    match status {
        ProposalStatus::Syntax(_) => "S_SYNTAX",
        ProposalStatus::Elaboration(_) => "S_ELABORATION",
        ProposalStatus::StructuralReject(_) => "structural_reject",
        ProposalStatus::OutOfCertificateScope(_) => "out_of_certificate_scope",
        ProposalStatus::EvaluationPending(_) => "evaluation_pending",
        ProposalStatus::CertificateFailed(_) => "certificate_failed",
        ProposalStatus::Evaluated {
            compatible: true, ..
        } => "evaluated_compatible",
        ProposalStatus::Evaluated {
            compatible: false, ..
        } => "evaluated_incompatible",
    }
}
