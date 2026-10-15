//! Episode manifest at the boundary: a versioned, validated DTO converted
//! once into the worker's `Manifest`. Requests never reach it; the worker
//! owns the converted value for the life of the process.

use std::collections::BTreeMap;
use std::path::PathBuf;

use residuum::SearchLimits;
use residuum::experiments::exploration::{Criterion, Decision, Profile};
use residuum::experiments::worker::{Budgets, Manifest};
use residuum::typed::Event;
use serde::{Deserialize, Serialize};

use crate::transport::TransportLimits;
use crate::wire::{WIRE_VERSION, WireError, WireEvent, WireLimits, WireSearchLimits};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireProfile {
    pub heads: Vec<String>,
    pub body_verbs: Vec<String>,
    pub evidential_not_verbs: Vec<String>,
    pub max_atoms: usize,
    pub max_binders: usize,
    pub max_after_depth: usize,
    pub max_unit_nodes: usize,
    pub require_connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireCriterion {
    Component(usize),
    Exact(Vec<bool>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDecision {
    pub components: Vec<WireEvent>,
    pub criterion: WireCriterion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireBudgets {
    pub max_actions: usize,
    pub max_steps_per_action: usize,
    pub max_steps_total: usize,
    pub max_sessions: usize,
    pub max_candidates: usize,
    pub max_clauses: usize,
    pub search_limits: WireSearchLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireTransportLimits {
    pub max_line_bytes: usize,
    pub max_output_bytes: usize,
    pub max_wire_bytes: usize,
    pub max_depth: usize,
}

impl From<&TransportLimits> for WireTransportLimits {
    fn from(l: &TransportLimits) -> Self {
        Self {
            max_line_bytes: l.max_line_bytes,
            max_output_bytes: l.max_output_bytes,
            max_wire_bytes: l.wire.max_bytes,
            max_depth: l.wire.max_depth,
        }
    }
}

impl From<&WireTransportLimits> for TransportLimits {
    fn from(w: &WireTransportLimits) -> Self {
        Self {
            max_line_bytes: w.max_line_bytes,
            max_output_bytes: w.max_output_bytes,
            wire: WireLimits {
                max_bytes: w.max_wire_bytes,
                max_depth: w.max_depth,
            },
        }
    }
}

/// The file the binary loads. Paths are relative to the manifest file's
/// directory unless absolute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireManifest {
    pub wire_version: String,
    pub episode_id: String,
    pub manifest_hash: String,
    pub imports: Vec<String>,
    pub workdir: String,
    pub profile: WireProfile,
    pub decision: WireDecision,
    pub dossiers: BTreeMap<String, Vec<WireEvent>>,
    pub budgets: WireBudgets,
    /// Optional; the transport defaults apply when absent.
    pub transport: Option<WireTransportLimits>,
}

fn ident(value: &str, what: &str) -> Result<String, WireError> {
    if value.is_empty()
        || value.len() > 128
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(WireError::Validation(format!(
            "{what} `{value}` is not a valid identifier"
        )));
    }
    Ok(value.to_string())
}

fn idents(values: &[String], what: &str) -> Result<Vec<String>, WireError> {
    values.iter().map(|v| ident(v, what)).collect()
}

/// Resolve a manifest path relative to `base` (the manifest's directory).
fn resolve(base: &std::path::Path, value: &str) -> Result<PathBuf, WireError> {
    if value.is_empty() {
        return Err(WireError::Validation("empty path in manifest".into()));
    }
    let p = PathBuf::from(value);
    Ok(if p.is_absolute() { p } else { base.join(p) })
}

/// Validate and convert. `base` is the directory the manifest was read from.
pub fn to_manifest(
    w: &WireManifest,
    base: &std::path::Path,
) -> Result<(Manifest, TransportLimits), WireError> {
    if w.wire_version != WIRE_VERSION {
        return Err(WireError::Validation(format!(
            "manifest wire version `{}` is not `{WIRE_VERSION}`",
            w.wire_version
        )));
    }
    if w.imports.is_empty() {
        return Err(WireError::Validation(
            "manifest declares no theory imports".into(),
        ));
    }
    if w.dossiers.is_empty() {
        return Err(WireError::Validation(
            "manifest declares no dossiers".into(),
        ));
    }
    let mut dossiers = BTreeMap::new();
    for (name, events) in &w.dossiers {
        let seeds = events
            .iter()
            .cloned()
            .map(Event::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        dossiers.insert(ident(name, "dossier")?, seeds);
    }
    let decision = Decision {
        components: w
            .decision
            .components
            .iter()
            .cloned()
            .map(Event::try_from)
            .collect::<Result<_, _>>()?,
        criterion: match &w.decision.criterion {
            WireCriterion::Component(i) => Criterion::Component(*i),
            WireCriterion::Exact(v) => Criterion::Exact(v.clone()),
        },
    };
    decision
        .validate()
        .map_err(|e| WireError::Validation(e.to_string()))?;
    let profile = Profile {
        heads: idents(&w.profile.heads, "head verb")?.into_iter().collect(),
        body_verbs: idents(&w.profile.body_verbs, "body verb")?
            .into_iter()
            .collect(),
        evidential_not_verbs: idents(&w.profile.evidential_not_verbs, "verb")?
            .into_iter()
            .collect(),
        max_atoms: w.profile.max_atoms,
        max_binders: w.profile.max_binders,
        max_after_depth: w.profile.max_after_depth,
        max_unit_nodes: w.profile.max_unit_nodes,
        require_connected: w.profile.require_connected,
    };
    let b = &w.budgets;
    if b.max_actions == 0 || b.max_steps_per_action == 0 || b.max_steps_total == 0 {
        return Err(WireError::Validation(
            "budgets must allow at least one action and one step".into(),
        ));
    }
    let manifest = Manifest {
        episode_id: ident(&w.episode_id, "episode")?,
        manifest_hash: ident(&w.manifest_hash, "manifest hash")?,
        imports: w
            .imports
            .iter()
            .map(|p| resolve(base, p))
            .collect::<Result<_, _>>()?,
        workdir: resolve(base, &w.workdir)?,
        profile,
        decision,
        dossiers,
        budgets: Budgets {
            max_actions: b.max_actions,
            max_steps_per_action: b.max_steps_per_action,
            max_steps_total: b.max_steps_total,
            max_sessions: b.max_sessions,
            max_candidates: b.max_candidates,
            max_clauses: b.max_clauses,
            search_limits: SearchLimits::from(b.search_limits),
        },
    };
    let transport = w
        .transport
        .as_ref()
        .map(TransportLimits::from)
        .unwrap_or_default();
    Ok((manifest, transport))
}

/// Encode a worker `Manifest` for a manifest file (tests and controllers).
/// Paths are written as given; callers decide whether they are absolute.
pub fn from_manifest(m: &Manifest, transport: Option<&TransportLimits>) -> WireManifest {
    WireManifest {
        wire_version: WIRE_VERSION.into(),
        episode_id: m.episode_id.clone(),
        manifest_hash: m.manifest_hash.clone(),
        imports: m
            .imports
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
        workdir: m.workdir.to_string_lossy().into_owned(),
        profile: WireProfile {
            heads: m.profile.heads.iter().cloned().collect(),
            body_verbs: m.profile.body_verbs.iter().cloned().collect(),
            evidential_not_verbs: m.profile.evidential_not_verbs.iter().cloned().collect(),
            max_atoms: m.profile.max_atoms,
            max_binders: m.profile.max_binders,
            max_after_depth: m.profile.max_after_depth,
            max_unit_nodes: m.profile.max_unit_nodes,
            require_connected: m.profile.require_connected,
        },
        decision: WireDecision {
            components: m.decision.components.iter().map(WireEvent::from).collect(),
            criterion: match &m.decision.criterion {
                Criterion::Component(i) => WireCriterion::Component(*i),
                Criterion::Exact(v) => WireCriterion::Exact(v.clone()),
            },
        },
        dossiers: m
            .dossiers
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().map(WireEvent::from).collect()))
            .collect(),
        budgets: WireBudgets {
            max_actions: m.budgets.max_actions,
            max_steps_per_action: m.budgets.max_steps_per_action,
            max_steps_total: m.budgets.max_steps_total,
            max_sessions: m.budgets.max_sessions,
            max_candidates: m.budgets.max_candidates,
            max_clauses: m.budgets.max_clauses,
            search_limits: WireSearchLimits::from(m.budgets.search_limits),
        },
        transport: transport.map(WireTransportLimits::from),
    }
}

/// Parse a manifest file's bytes. Size is bounded before parsing.
pub fn parse_manifest(bytes: &[u8], max_bytes: usize) -> Result<WireManifest, WireError> {
    if bytes.len() > max_bytes {
        return Err(WireError::Transport(format!(
            "manifest of {} bytes exceeds the limit of {max_bytes}",
            bytes.len()
        )));
    }
    serde_json::from_slice(bytes).map_err(|e| match e.classify() {
        serde_json::error::Category::Data => WireError::Validation(e.to_string()),
        _ => WireError::Transport(e.to_string()),
    })
}
