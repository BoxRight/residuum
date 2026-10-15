//! Source path for text proposals (exploration contract §2.6):
//!
//! ```text
//! Formal text → parse → private module importing the theory → candidate
//! rules → structural admission → certificate scope → evaluation
//! ```
//!
//! Every stage reports a distinct status. Tool misconfiguration (bad id,
//! missing work directory, unloadable theory, invalid `D`) is an `Err` of
//! `process`; it is never turned into a status about the proposal.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use crate::elab::Error;
use crate::typed::{Closure, Declaration, Event, Rule, RuleBody};

use super::Clock;
use super::exploration::{
    Conservation, Decision, Profile, certify_addition, classify_candidate, evaluate,
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
    /// Profile codes of §2.6, plus `S_DECLARATION` (the proposal declared
    /// something other than rules) and `S_ORIGIN` (theory not preserved).
    StructuralReject(BTreeSet<String>),
    /// Admitted, but some head is already produced by the reference closure:
    /// the addition certificate's proven scope does not cover it (§7, limit 3).
    OutOfCertificateScope(BTreeSet<String>),
    /// The evaluator refused or could not complete (activation cycle, runner
    /// limit). Not a refutation and not a compatibility verdict.
    EvaluationPending(String),
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

/// Run the source path. `Err` means the harness itself is misconfigured.
pub fn process(proposal: &SourceProposal, ctx: &ProposalContext) -> Result<ProposalReport, Error> {
    ctx.decision.validate()?;
    let name = module_name(&proposal.id)?;
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

    // Baseline: the same imports with no proposal. Its failure is a harness
    // error, not a statement about the proposal.
    let baseline_name = format!("{name}Baseline");
    let baseline_file = workdir.join(format!("{baseline_name}.res"));
    write(&baseline_file, &compose(&baseline_name, &imports, ""))?;
    let baseline = crate::load_formal_module(&baseline_file)
        .map_err(|e| Error::new(format!("theory imports do not elaborate: {e}")))?;

    let source = compose(&name, &imports, &proposal.text);
    let file = workdir.join(format!("{name}.res"));
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
    macro_rules! finish {
        ($status:expr) => {{
            report.status = $status;
            return Ok(report);
        }};
    }

    // 1. Syntax.
    if let Err(e) = crate::parse_formal(&source) {
        finish!(ProposalStatus::Syntax(e.to_string()));
    }
    report.stages.push("parsed");

    // 2. Elaboration in the private module with the theory imported.
    write(&file, &source)?;
    let module = match crate::load_formal_module(&file) {
        Ok(m) => m,
        Err(e) => finish!(ProposalStatus::Elaboration(e.to_string())),
    };
    report.stages.push("elaborated");

    // 3. Extraction of the candidate declarations by origin.
    let mut reasons = BTreeSet::new();
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
    if theory_preserved(&baseline, &module).is_err() {
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
        finish!(ProposalStatus::StructuralReject(reasons));
    }
    report.stages.push("admitted");

    // 5. Certificate scope against the reference closure.
    let reference: Closure =
        match crate::close_program(ctx.seeds.to_vec(), &baseline, &mut Clock::default()) {
            Ok(c) => c,
            Err(e) => finish!(ProposalStatus::EvaluationPending(format!(
                "reference closure: {e}"
            ))),
        };
    let heads: BTreeSet<String> = rules
        .iter()
        .filter_map(|r| match &r.body {
            RuleBody::HornClause(c) => Some(c.consequent.verb.clone()),
            RuleBody::ResidualClause(_) => None,
        })
        .collect();
    let produced: BTreeSet<String> = reference
        .known
        .iter()
        .filter(|e| heads.contains(&e.verb))
        .map(|e| e.verb.clone())
        .collect();
    if !produced.is_empty() {
        finish!(ProposalStatus::OutOfCertificateScope(produced));
    }
    report.stages.push("in_certificate_scope");

    // 6. Evaluation and certificate.
    let evaluation = match evaluate(&module, ctx.seeds, ctx.decision) {
        Ok(ev) => ev,
        Err(e) => finish!(ProposalStatus::EvaluationPending(e.to_string())),
    };
    let certificate = match certify_addition(&reference, &evaluation.closure, &heads) {
        Ok(c) => c,
        Err(e) => finish!(ProposalStatus::EvaluationPending(e.to_string())),
    };
    report.stages.push("evaluated");
    finish!(ProposalStatus::Evaluated {
        components: evaluation.components,
        compatible: evaluation.compatible,
        certificate,
    });
}

/// Convenience for logs: the status family as a stable label.
pub fn status_label(status: &ProposalStatus) -> &'static str {
    match status {
        ProposalStatus::Syntax(_) => "S_SYNTAX",
        ProposalStatus::Elaboration(_) => "S_ELABORATION",
        ProposalStatus::StructuralReject(_) => "structural_reject",
        ProposalStatus::OutOfCertificateScope(_) => "out_of_certificate_scope",
        ProposalStatus::EvaluationPending(_) => "evaluation_pending",
        ProposalStatus::Evaluated {
            compatible: true, ..
        } => "evaluated_compatible",
        ProposalStatus::Evaluated {
            compatible: false, ..
        } => "evaluated_incompatible",
    }
}
