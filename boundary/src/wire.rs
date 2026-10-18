//! Versioned wire DTOs at the worker boundary (invocation protocol §9),
//! in the boundary crate so that serialisation never enters `residuum`.
//!
//! Explicit mirrors of the worker's request and response types and of the
//! engine structures they carry. Serialisation lives only here: nothing in
//! `typed`, `residual` or `derive` derives `Serialize`/`Deserialize`, and
//! `Debug` is never a representation. Encoding is total; decoding validates
//! the wire version, rejects unknown fields and variants, and checks
//! identifiers. Input size is checked before parsing and nesting depth
//! before any work is dispatched. Transport, validation and execution errors
//! stay distinct: the first two are `WireError`, the third is a `Response`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use residuum::residual::{
    Abducible, AbductionFailure, ActiveProgramResidualQuery, CertifiedActiveExplanation,
    HornQueryScope, ProgramResidualQuery, QueryBinding, QueryParameterScope, RequirementFormula,
    ResidualActivation, ResidualExplanation, ResidualProof, ResidualSearchTree, SearchLimits,
    SearchMetrics,
};
use residuum::typed::{
    Antecedent, Arg, BehaviorKind, BehaviorRef, Closure, Constant, Constructor, DeclarationOrigin,
    DerivationRecord, DerivationTime, DerivedEvent, Entity, Event, Field, HornClause, Polarity,
    ProgramRef, ProgramRuleOrigin, PropositionKind, PropositionType, RecordFieldValue, Residual,
    ResidualClause, Rule, RuleBody, RuleRef, StateTime, StateTransitionType, Substitution,
    SubstitutionBinding, SubstitutionValue, Term, TimeExpr, Type, VerbKind,
};

use residuum::experiments::exploration::{Conservation, Partition};
use residuum::experiments::proposal::{ProposalReport, ProposalStatus};
use residuum::experiments::worker::{
    BudgetState, Envelope, ForwardRound, Payload, Request, ResidualOutcome, ResidualRound,
    Response, SessionKind, Status, TheoryInventory, VerbSignature,
};

pub const WIRE_VERSION: &str = "residuum-wire/1";

/// Transport: bytes that are not a message (size, syntax). Validation: a
/// message that is not acceptable (version, unknown field or variant,
/// invalid identifier, depth). Execution outcomes never appear here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    Transport(String),
    Validation(String),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(m) => write!(f, "transport: {m}"),
            Self::Validation(m) => write!(f, "validation: {m}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireLimits {
    pub max_bytes: usize,
    pub max_depth: usize,
}

impl Default for WireLimits {
    fn default() -> Self {
        Self {
            max_bytes: 1024 * 1024,
            max_depth: 64,
        }
    }
}

/// Maximum nesting of arrays and objects, scanning bytes without building
/// a value. Strings and escapes are skipped; a malformed string is a
/// transport error.
pub fn json_depth(bytes: &[u8]) -> Result<usize, WireError> {
    let mut depth = 0usize;
    let mut max = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for &b in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'[' | b'{' => {
                depth += 1;
                max = max.max(depth);
            }
            b']' | b'}' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| WireError::Transport("unbalanced closing bracket".into()))?;
            }
            _ => {}
        }
    }
    if in_string {
        return Err(WireError::Transport("unterminated string".into()));
    }
    if depth != 0 {
        return Err(WireError::Transport("unbalanced brackets".into()));
    }
    Ok(max)
}

fn precheck(bytes: &[u8], limits: &WireLimits) -> Result<(), WireError> {
    if bytes.len() > limits.max_bytes {
        return Err(WireError::Transport(format!(
            "message of {} bytes exceeds the limit of {}",
            bytes.len(),
            limits.max_bytes
        )));
    }
    let depth = json_depth(bytes)?;
    if depth > limits.max_depth {
        return Err(WireError::Validation(format!(
            "nesting depth {depth} exceeds the limit of {}",
            limits.max_depth
        )));
    }
    Ok(())
}

fn parse<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, WireError> {
    serde_json::from_slice(bytes).map_err(|e| match e.classify() {
        serde_json::error::Category::Syntax
        | serde_json::error::Category::Eof
        | serde_json::error::Category::Io => WireError::Transport(e.to_string()),
        serde_json::error::Category::Data => WireError::Validation(e.to_string()),
    })
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

fn version(found: &str) -> Result<(), WireError> {
    if found != WIRE_VERSION {
        return Err(WireError::Validation(format!(
            "wire version `{found}` is not `{WIRE_VERSION}`"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Public entry points.

pub fn encode_envelope(envelope: &Envelope) -> Result<String, WireError> {
    serde_json::to_string(&WireEnvelope::from(envelope))
        .map_err(|e| WireError::Transport(e.to_string()))
}

pub fn decode_envelope(bytes: &[u8], limits: &WireLimits) -> Result<Envelope, WireError> {
    precheck(bytes, limits)?;
    let wire: WireEnvelope = parse(bytes)?;
    Envelope::try_from(wire)
}

pub fn encode_response(response: &Response) -> Result<String, WireError> {
    serde_json::to_string(&WireResponse::from(response))
        .map_err(|e| WireError::Transport(e.to_string()))
}

pub fn decode_response(bytes: &[u8], limits: &WireLimits) -> Result<Response, WireError> {
    precheck(bytes, limits)?;
    let wire: WireResponse = parse(bytes)?;
    Response::try_from(wire)
}

// ---------------------------------------------------------------------------
// Terms, times, events.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireTime {
    At(String),
    Var(String),
    After(Box<WireTime>),
}

impl From<&TimeExpr> for WireTime {
    fn from(t: &TimeExpr) -> Self {
        match t {
            TimeExpr::At(n) => Self::At(n.clone()),
            TimeExpr::Var(n) => Self::Var(n.clone()),
            TimeExpr::After(inner) => Self::After(Box::new(Self::from(inner.as_ref()))),
        }
    }
}

impl TryFrom<WireTime> for TimeExpr {
    type Error = WireError;
    fn try_from(w: WireTime) -> Result<Self, WireError> {
        Ok(match w {
            WireTime::At(n) => Self::At(ident(&n, "time constant")?),
            WireTime::Var(n) => Self::Var(ident(&n, "time variable")?),
            WireTime::After(inner) => Self::After(Box::new(Self::try_from(*inner)?)),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct WireField {
    pub field: String,
    pub value: WireTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireTerm {
    Var(String),
    Const(String),
    None,
    After(Box<WireTerm>),
    Record {
        entity: String,
        fields: Vec<WireField>,
    },
}

impl From<&Term> for WireTerm {
    fn from(t: &Term) -> Self {
        match t {
            Term::Var(n) => Self::Var(n.clone()),
            Term::Const(n) => Self::Const(n.clone()),
            Term::None => Self::None,
            Term::Constructor {
                constructor: Constructor::After,
                argument,
            } => Self::After(Box::new(Self::from(argument.as_ref()))),
            Term::RecordLiteral { entity, fields } => Self::Record {
                entity: entity.clone(),
                fields: fields
                    .iter()
                    .map(|f| WireField {
                        field: f.field.clone(),
                        value: Self::from(&f.value),
                    })
                    .collect(),
            },
        }
    }
}

impl TryFrom<WireTerm> for Term {
    type Error = WireError;
    fn try_from(w: WireTerm) -> Result<Self, WireError> {
        Ok(match w {
            WireTerm::Var(n) => Self::Var(ident(&n, "variable")?),
            WireTerm::Const(n) => Self::Const(ident(&n, "constant")?),
            WireTerm::None => Self::None,
            WireTerm::After(inner) => Self::Constructor {
                constructor: Constructor::After,
                argument: Box::new(Self::try_from(*inner)?),
            },
            WireTerm::Record { entity, fields } => Self::RecordLiteral {
                entity: ident(&entity, "record entity")?,
                fields: fields
                    .into_iter()
                    .map(|f| {
                        Ok(RecordFieldValue {
                            field: ident(&f.field, "record field")?,
                            value: Self::try_from(f.value)?,
                        })
                    })
                    .collect::<Result<_, WireError>>()?,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WirePolarity {
    Positive,
    EvidentialNot,
}

impl From<Polarity> for WirePolarity {
    fn from(p: Polarity) -> Self {
        match p {
            Polarity::Positive => Self::Positive,
            Polarity::EvidentialNot => Self::EvidentialNot,
        }
    }
}

impl From<WirePolarity> for Polarity {
    fn from(p: WirePolarity) -> Self {
        match p {
            WirePolarity::Positive => Self::Positive,
            WirePolarity::EvidentialNot => Self::EvidentialNot,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WirePropositionKind {
    Prop,
    Derived,
    Seeded,
    Effect,
}

impl From<&PropositionKind> for WirePropositionKind {
    fn from(k: &PropositionKind) -> Self {
        match k {
            PropositionKind::Prop => Self::Prop,
            PropositionKind::Derived => Self::Derived,
            PropositionKind::Seeded => Self::Seeded,
            PropositionKind::Effect => Self::Effect,
        }
    }
}

impl From<WirePropositionKind> for PropositionKind {
    fn from(k: WirePropositionKind) -> Self {
        match k {
            WirePropositionKind::Prop => Self::Prop,
            WirePropositionKind::Derived => Self::Derived,
            WirePropositionKind::Seeded => Self::Seeded,
            WirePropositionKind::Effect => Self::Effect,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireStateTime {
    Unresolved,
    At(WireTime),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireTransition {
    pub input: WireStateTime,
    pub output: WireStateTime,
}

fn state_time_to_wire(s: &StateTime) -> WireStateTime {
    match s {
        StateTime::Unresolved => WireStateTime::Unresolved,
        StateTime::At(t) => WireStateTime::At(WireTime::from(t)),
    }
}

fn state_time_from_wire(w: WireStateTime) -> Result<StateTime, WireError> {
    Ok(match w {
        WireStateTime::Unresolved => StateTime::Unresolved,
        WireStateTime::At(t) => StateTime::At(TimeExpr::try_from(t)?),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireEvent {
    pub polarity: WirePolarity,
    pub verb: String,
    pub args: Vec<WireTerm>,
    pub kind: WirePropositionKind,
    pub time: WireTime,
    pub transition: Option<WireTransition>,
}

impl From<&Event> for WireEvent {
    fn from(e: &Event) -> Self {
        Self {
            polarity: e.polarity.into(),
            verb: e.verb.clone(),
            args: e.args.iter().map(WireTerm::from).collect(),
            kind: (&e.proposition.kind).into(),
            time: WireTime::from(&e.proposition.time),
            transition: e.transition.as_ref().map(|t| WireTransition {
                input: state_time_to_wire(&t.input),
                output: state_time_to_wire(&t.output),
            }),
        }
    }
}

impl TryFrom<WireEvent> for Event {
    type Error = WireError;
    fn try_from(w: WireEvent) -> Result<Self, WireError> {
        Ok(Self {
            polarity: w.polarity.into(),
            verb: ident(&w.verb, "verb")?,
            args: w
                .args
                .into_iter()
                .map(Term::try_from)
                .collect::<Result<_, _>>()?,
            proposition: PropositionType {
                kind: w.kind.into(),
                time: TimeExpr::try_from(w.time)?,
            },
            transition: match w.transition {
                None => None,
                Some(t) => Some(StateTransitionType {
                    input: state_time_from_wire(t.input)?,
                    output: state_time_from_wire(t.output)?,
                }),
            },
        })
    }
}

fn events_to_wire(events: &[Event]) -> Vec<WireEvent> {
    events.iter().map(WireEvent::from).collect()
}

fn events_from_wire(events: Vec<WireEvent>) -> Result<Vec<Event>, WireError> {
    events.into_iter().map(Event::try_from).collect()
}

// ---------------------------------------------------------------------------
// Types, substitutions, records, closures.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireType {
    Entity(String),
    Set(Box<WireType>),
    Optional(Box<WireType>),
    PropositionTime,
    DerivationTime,
}

impl From<&Type> for WireType {
    fn from(t: &Type) -> Self {
        match t {
            Type::Entity(n) => Self::Entity(n.clone()),
            Type::Set(inner) => Self::Set(Box::new(Self::from(inner.as_ref()))),
            Type::Optional(inner) => Self::Optional(Box::new(Self::from(inner.as_ref()))),
            Type::PropositionTime => Self::PropositionTime,
            Type::DerivationTime => Self::DerivationTime,
        }
    }
}

impl TryFrom<WireType> for Type {
    type Error = WireError;
    fn try_from(w: WireType) -> Result<Self, WireError> {
        Ok(match w {
            WireType::Entity(n) => Self::Entity(ident(&n, "entity")?),
            WireType::Set(inner) => Self::Set(Box::new(Self::try_from(*inner)?)),
            WireType::Optional(inner) => Self::Optional(Box::new(Self::try_from(*inner)?)),
            WireType::PropositionTime => Self::PropositionTime,
            WireType::DerivationTime => Self::DerivationTime,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireArg {
    pub name: String,
    pub ty: WireType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireSubstitutionValue {
    Term(WireTerm),
    Time(WireTime),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireSubstitutionBinding {
    pub parameter: String,
    pub value: WireSubstitutionValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireSubstitution {
    pub bindings: Vec<WireSubstitutionBinding>,
}

fn value_to_wire(v: &SubstitutionValue) -> WireSubstitutionValue {
    match v {
        SubstitutionValue::Term(t) => WireSubstitutionValue::Term(WireTerm::from(t)),
        SubstitutionValue::Time(t) => WireSubstitutionValue::Time(WireTime::from(t)),
    }
}

fn value_from_wire(w: WireSubstitutionValue) -> Result<SubstitutionValue, WireError> {
    Ok(match w {
        WireSubstitutionValue::Term(t) => SubstitutionValue::Term(Term::try_from(t)?),
        WireSubstitutionValue::Time(t) => SubstitutionValue::Time(TimeExpr::try_from(t)?),
    })
}

impl From<&Substitution> for WireSubstitution {
    fn from(s: &Substitution) -> Self {
        Self {
            bindings: s
                .bindings
                .iter()
                .map(|b| WireSubstitutionBinding {
                    parameter: b.parameter.clone(),
                    value: value_to_wire(&b.value),
                })
                .collect(),
        }
    }
}

impl TryFrom<WireSubstitution> for Substitution {
    type Error = WireError;
    fn try_from(w: WireSubstitution) -> Result<Self, WireError> {
        Ok(Self {
            bindings: w
                .bindings
                .into_iter()
                .map(|b| {
                    Ok(SubstitutionBinding {
                        parameter: ident(&b.parameter, "parameter")?,
                        value: value_from_wire(b.value)?,
                    })
                })
                .collect::<Result<_, WireError>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRecord {
    pub rule: String,
    /// Ordered as the rule matched them.
    pub antecedents: Vec<WireEvent>,
    pub substitution: WireSubstitution,
}

impl From<&DerivationRecord> for WireRecord {
    fn from(r: &DerivationRecord) -> Self {
        Self {
            rule: r.rule.clone(),
            antecedents: events_to_wire(&r.antecedents),
            substitution: WireSubstitution::from(&r.substitution),
        }
    }
}

impl TryFrom<WireRecord> for DerivationRecord {
    type Error = WireError;
    fn try_from(w: WireRecord) -> Result<Self, WireError> {
        Ok(Self {
            rule: ident(&w.rule, "rule")?,
            antecedents: events_from_wire(w.antecedents)?,
            substitution: Substitution::try_from(w.substitution)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDerived {
    pub event: WireEvent,
    pub derived_at: String,
    pub record: WireRecord,
}

impl From<&DerivedEvent> for WireDerived {
    fn from(d: &DerivedEvent) -> Self {
        Self {
            event: WireEvent::from(&d.event),
            derived_at: d.derived_at.name.clone(),
            record: WireRecord::from(&d.record),
        }
    }
}

impl TryFrom<WireDerived> for DerivedEvent {
    type Error = WireError;
    fn try_from(w: WireDerived) -> Result<Self, WireError> {
        Ok(Self {
            event: Event::try_from(w.event)?,
            derived_at: DerivationTime {
                name: ident(&w.derived_at, "derivation time")?,
            },
            record: DerivationRecord::try_from(w.record)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireClosure {
    pub known: Vec<WireEvent>,
    /// Every justification, with multiplicity.
    pub derivations: Vec<WireDerived>,
}

impl From<&Closure> for WireClosure {
    fn from(c: &Closure) -> Self {
        Self {
            known: events_to_wire(&c.known),
            derivations: c.derivations.iter().map(WireDerived::from).collect(),
        }
    }
}

impl TryFrom<WireClosure> for Closure {
    type Error = WireError;
    fn try_from(w: WireClosure) -> Result<Self, WireError> {
        Ok(Self {
            known: events_from_wire(w.known)?,
            derivations: w
                .derivations
                .into_iter()
                .map(DerivedEvent::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

// ---------------------------------------------------------------------------
// Rules.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireAntecedent {
    Unit,
    Event(WireEvent),
    And(Box<WireAntecedent>, Box<WireAntecedent>),
}

impl From<&Antecedent> for WireAntecedent {
    fn from(a: &Antecedent) -> Self {
        match a {
            Antecedent::Unit => Self::Unit,
            Antecedent::Event(e) => Self::Event(WireEvent::from(e)),
            Antecedent::And(l, r) => Self::And(
                Box::new(Self::from(l.as_ref())),
                Box::new(Self::from(r.as_ref())),
            ),
        }
    }
}

impl TryFrom<WireAntecedent> for Antecedent {
    type Error = WireError;
    fn try_from(w: WireAntecedent) -> Result<Self, WireError> {
        Ok(match w {
            WireAntecedent::Unit => Self::Unit,
            WireAntecedent::Event(e) => Self::Event(Event::try_from(e)?),
            WireAntecedent::And(l, r) => {
                Self::And(Box::new(Self::try_from(*l)?), Box::new(Self::try_from(*r)?))
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireRuleBody {
    Horn {
        antecedent: WireAntecedent,
        consequent: WireEvent,
    },
    Residual {
        antecedent: WireAntecedent,
        required: WireAntecedent,
        consequent: WireEvent,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRule {
    pub name: String,
    pub parameters: Vec<WireArg>,
    pub body: WireRuleBody,
}

impl From<&Rule> for WireRule {
    fn from(r: &Rule) -> Self {
        Self {
            name: r.name.clone(),
            parameters: r
                .parameters
                .iter()
                .map(|a| WireArg {
                    name: a.name.clone(),
                    ty: WireType::from(&a.ty),
                })
                .collect(),
            body: match &r.body {
                RuleBody::HornClause(c) => WireRuleBody::Horn {
                    antecedent: WireAntecedent::from(&c.antecedent),
                    consequent: WireEvent::from(&c.consequent),
                },
                RuleBody::ResidualClause(c) => WireRuleBody::Residual {
                    antecedent: WireAntecedent::from(&c.antecedent),
                    required: WireAntecedent::from(&c.consequent.required),
                    consequent: WireEvent::from(&c.consequent.consequent),
                },
            },
        }
    }
}

impl TryFrom<WireRule> for Rule {
    type Error = WireError;
    fn try_from(w: WireRule) -> Result<Self, WireError> {
        Ok(Self {
            name: ident(&w.name, "rule")?,
            parameters: w
                .parameters
                .into_iter()
                .map(|a| {
                    Ok(Arg {
                        name: ident(&a.name, "parameter")?,
                        ty: Type::try_from(a.ty)?,
                    })
                })
                .collect::<Result<_, WireError>>()?,
            body: match w.body {
                WireRuleBody::Horn {
                    antecedent,
                    consequent,
                } => RuleBody::HornClause(HornClause {
                    antecedent: Antecedent::try_from(antecedent)?,
                    consequent: Event::try_from(consequent)?,
                }),
                WireRuleBody::Residual {
                    antecedent,
                    required,
                    consequent,
                } => RuleBody::ResidualClause(ResidualClause {
                    antecedent: Antecedent::try_from(antecedent)?,
                    consequent: Residual {
                        required: Antecedent::try_from(required)?,
                        consequent: Event::try_from(consequent)?,
                    },
                }),
            },
        })
    }
}

// ---------------------------------------------------------------------------
// Query options.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireScope {
    Rules(Vec<String>),
    Program(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireParameterScope {
    Rule(String),
    Program(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireQueryBinding {
    pub scope: WireParameterScope,
    pub parameter: String,
    pub ty: WireType,
    pub value: WireSubstitutionValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireAbducible {
    pub verb: String,
    pub polarity: WirePolarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireSearchLimits {
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_branches: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireMetrics {
    pub nodes: usize,
    pub rules_examined: usize,
    pub head_matches: usize,
    pub branches_generated: usize,
    pub rejected_not_forward_supported: usize,
}

fn scope_to_wire(s: &HornQueryScope) -> WireScope {
    match s {
        HornQueryScope::Rules(rules) => {
            WireScope::Rules(rules.iter().map(|r| r.name.clone()).collect())
        }
        HornQueryScope::Program(p) => WireScope::Program(p.name.clone()),
    }
}

fn scope_from_wire(w: WireScope) -> Result<HornQueryScope, WireError> {
    Ok(match w {
        WireScope::Rules(rules) => HornQueryScope::Rules(
            rules
                .iter()
                .map(|n| {
                    Ok(RuleRef {
                        name: ident(n, "rule")?,
                    })
                })
                .collect::<Result<_, WireError>>()?,
        ),
        WireScope::Program(p) => HornQueryScope::Program(ProgramRef {
            name: ident(&p, "program")?,
        }),
    })
}

fn binding_to_wire(b: &QueryBinding) -> WireQueryBinding {
    WireQueryBinding {
        scope: match &b.scope {
            QueryParameterScope::Rule(r) => WireParameterScope::Rule(r.name.clone()),
            QueryParameterScope::Program(p) => WireParameterScope::Program(p.name.clone()),
        },
        parameter: b.parameter.clone(),
        ty: WireType::from(&b.ty),
        value: value_to_wire(&b.value),
    }
}

fn binding_from_wire(w: WireQueryBinding) -> Result<QueryBinding, WireError> {
    Ok(QueryBinding {
        scope: match w.scope {
            WireParameterScope::Rule(r) => QueryParameterScope::Rule(RuleRef {
                name: ident(&r, "rule")?,
            }),
            WireParameterScope::Program(p) => QueryParameterScope::Program(ProgramRef {
                name: ident(&p, "program")?,
            }),
        },
        parameter: ident(&w.parameter, "parameter")?,
        ty: Type::try_from(w.ty)?,
        value: value_from_wire(w.value)?,
    })
}

fn abducible_to_wire(a: &Abducible) -> WireAbducible {
    WireAbducible {
        verb: a.verb.clone(),
        polarity: a.polarity.into(),
    }
}

fn abducible_from_wire(w: WireAbducible) -> Result<Abducible, WireError> {
    Ok(Abducible {
        verb: ident(&w.verb, "verb")?,
        polarity: w.polarity.into(),
    })
}

impl From<SearchLimits> for WireSearchLimits {
    fn from(l: SearchLimits) -> Self {
        Self {
            max_depth: l.max_depth,
            max_nodes: l.max_nodes,
            max_branches: l.max_branches,
        }
    }
}

impl From<WireSearchLimits> for SearchLimits {
    fn from(w: WireSearchLimits) -> Self {
        Self {
            max_depth: w.max_depth,
            max_nodes: w.max_nodes,
            max_branches: w.max_branches,
        }
    }
}

impl From<&SearchMetrics> for WireMetrics {
    fn from(m: &SearchMetrics) -> Self {
        Self {
            nodes: m.nodes,
            rules_examined: m.rules_examined,
            head_matches: m.head_matches,
            branches_generated: m.branches_generated,
            rejected_not_forward_supported: m.rejected_not_forward_supported,
        }
    }
}

impl From<WireMetrics> for SearchMetrics {
    fn from(w: WireMetrics) -> Self {
        Self {
            nodes: w.nodes,
            rules_examined: w.rules_examined,
            head_matches: w.head_matches,
            branches_generated: w.branches_generated,
            rejected_not_forward_supported: w.rejected_not_forward_supported,
        }
    }
}

// ---------------------------------------------------------------------------
// Requests.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireRequest {
    InspectTheory,
    BeginForward {
        dossier: String,
        theory: Option<String>,
    },
    BeginResidual {
        dossier: String,
        theory: Option<String>,
        goal: WireEvent,
        scope: Option<WireScope>,
        bindings: Vec<WireQueryBinding>,
        allowed: Option<Vec<WireAbducible>>,
        limits: Option<WireSearchLimits>,
    },
    BeginActiveResidual {
        dossier: String,
        theory: Option<String>,
        goal: WireEvent,
        scope: Option<WireScope>,
        bindings: Vec<WireQueryBinding>,
        allowed: Option<Vec<WireAbducible>>,
        limits: Option<WireSearchLimits>,
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
        goal: WireEvent,
    },
    ComparePartition {
        candidates: Vec<String>,
        goal: WireEvent,
    },
}

#[allow(clippy::type_complexity)]
fn query_to_wire(
    goal: &Event,
    scope: &Option<HornQueryScope>,
    bindings: &[QueryBinding],
    allowed: &Option<Vec<Abducible>>,
    limits: &Option<SearchLimits>,
) -> (
    WireEvent,
    Option<WireScope>,
    Vec<WireQueryBinding>,
    Option<Vec<WireAbducible>>,
    Option<WireSearchLimits>,
) {
    (
        WireEvent::from(goal),
        scope.as_ref().map(scope_to_wire),
        bindings.iter().map(binding_to_wire).collect(),
        allowed
            .as_ref()
            .map(|a| a.iter().map(abducible_to_wire).collect()),
        limits.map(WireSearchLimits::from),
    )
}

#[allow(clippy::type_complexity)]
fn query_from_wire(
    goal: WireEvent,
    scope: Option<WireScope>,
    bindings: Vec<WireQueryBinding>,
    allowed: Option<Vec<WireAbducible>>,
    limits: Option<WireSearchLimits>,
) -> Result<
    (
        Event,
        Option<HornQueryScope>,
        Vec<QueryBinding>,
        Option<Vec<Abducible>>,
        Option<SearchLimits>,
    ),
    WireError,
> {
    Ok((
        Event::try_from(goal)?,
        scope.map(scope_from_wire).transpose()?,
        bindings
            .into_iter()
            .map(binding_from_wire)
            .collect::<Result<_, _>>()?,
        allowed
            .map(|a| {
                a.into_iter()
                    .map(abducible_from_wire)
                    .collect::<Result<_, _>>()
            })
            .transpose()?,
        limits.map(SearchLimits::from),
    ))
}

impl From<&Request> for WireRequest {
    fn from(r: &Request) -> Self {
        match r {
            Request::InspectTheory => Self::InspectTheory,
            Request::BeginForward { dossier, theory } => Self::BeginForward {
                dossier: dossier.clone(),
                theory: theory.clone(),
            },
            Request::BeginResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => {
                let (goal, scope, bindings, allowed, limits) =
                    query_to_wire(goal, scope, bindings, allowed, limits);
                Self::BeginResidual {
                    dossier: dossier.clone(),
                    theory: theory.clone(),
                    goal,
                    scope,
                    bindings,
                    allowed,
                    limits,
                }
            }
            Request::BeginActiveResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => {
                let (goal, scope, bindings, allowed, limits) =
                    query_to_wire(goal, scope, bindings, allowed, limits);
                Self::BeginActiveResidual {
                    dossier: dossier.clone(),
                    theory: theory.clone(),
                    goal,
                    scope,
                    bindings,
                    allowed,
                    limits,
                }
            }
            Request::AdvanceSession { session, max_steps } => Self::AdvanceSession {
                session: session.clone(),
                max_steps: *max_steps,
            },
            Request::ProposeCandidate { text } => Self::ProposeCandidate { text: text.clone() },
            Request::BeginCandidateEvaluation { candidate, dossier } => {
                Self::BeginCandidateEvaluation {
                    candidate: candidate.clone(),
                    dossier: dossier.clone(),
                }
            }
            Request::MinimalSupports { candidate, goal } => Self::MinimalSupports {
                candidate: candidate.clone(),
                goal: WireEvent::from(goal),
            },
            Request::ComparePartition { candidates, goal } => Self::ComparePartition {
                candidates: candidates.clone(),
                goal: WireEvent::from(goal),
            },
        }
    }
}

fn opt_ident(value: Option<String>, what: &str) -> Result<Option<String>, WireError> {
    value.map(|v| ident(&v, what)).transpose()
}

impl TryFrom<WireRequest> for Request {
    type Error = WireError;
    fn try_from(w: WireRequest) -> Result<Self, WireError> {
        Ok(match w {
            WireRequest::InspectTheory => Self::InspectTheory,
            WireRequest::BeginForward { dossier, theory } => Self::BeginForward {
                dossier: ident(&dossier, "dossier")?,
                theory: opt_ident(theory, "candidate")?,
            },
            WireRequest::BeginResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => {
                let (goal, scope, bindings, allowed, limits) =
                    query_from_wire(goal, scope, bindings, allowed, limits)?;
                Self::BeginResidual {
                    dossier: ident(&dossier, "dossier")?,
                    theory: opt_ident(theory, "candidate")?,
                    goal,
                    scope,
                    bindings,
                    allowed,
                    limits,
                }
            }
            WireRequest::BeginActiveResidual {
                dossier,
                theory,
                goal,
                scope,
                bindings,
                allowed,
                limits,
            } => {
                let (goal, scope, bindings, allowed, limits) =
                    query_from_wire(goal, scope, bindings, allowed, limits)?;
                Self::BeginActiveResidual {
                    dossier: ident(&dossier, "dossier")?,
                    theory: opt_ident(theory, "candidate")?,
                    goal,
                    scope,
                    bindings,
                    allowed,
                    limits,
                }
            }
            WireRequest::AdvanceSession { session, max_steps } => Self::AdvanceSession {
                session: ident(&session, "session")?,
                max_steps,
            },
            WireRequest::ProposeCandidate { text } => Self::ProposeCandidate { text },
            WireRequest::BeginCandidateEvaluation { candidate, dossier } => {
                Self::BeginCandidateEvaluation {
                    candidate: ident(&candidate, "candidate")?,
                    dossier: ident(&dossier, "dossier")?,
                }
            }
            WireRequest::MinimalSupports { candidate, goal } => Self::MinimalSupports {
                candidate: ident(&candidate, "candidate")?,
                goal: Event::try_from(goal)?,
            },
            WireRequest::ComparePartition { candidates, goal } => Self::ComparePartition {
                candidates: candidates
                    .iter()
                    .map(|c| ident(c, "candidate"))
                    .collect::<Result<_, _>>()?,
                goal: Event::try_from(goal)?,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireEnvelope {
    pub wire_version: String,
    pub protocol_version: String,
    pub episode_id: String,
    pub action_id: String,
    pub manifest_hash: String,
    pub request: WireRequest,
}

impl From<&Envelope> for WireEnvelope {
    fn from(e: &Envelope) -> Self {
        Self {
            wire_version: WIRE_VERSION.into(),
            protocol_version: e.protocol_version.clone(),
            episode_id: e.episode_id.clone(),
            action_id: e.action_id.clone(),
            manifest_hash: e.manifest_hash.clone(),
            request: WireRequest::from(&e.request),
        }
    }
}

impl TryFrom<WireEnvelope> for Envelope {
    type Error = WireError;
    fn try_from(w: WireEnvelope) -> Result<Self, WireError> {
        version(&w.wire_version)?;
        Ok(Self {
            protocol_version: ident(&w.protocol_version, "protocol version")?,
            episode_id: ident(&w.episode_id, "episode")?,
            action_id: ident(&w.action_id, "action")?,
            manifest_hash: ident(&w.manifest_hash, "manifest hash")?,
            request: Request::try_from(w.request)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Residual structures.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireTree {
    Pending {
        goal: WireEvent,
        depth: usize,
        ancestors: Vec<WireEvent>,
    },
    Unit,
    Evidence(WireEvent),
    Hypothesis(WireEvent),
    Tensor(Box<WireTree>, Box<WireTree>),
    Rule {
        rule: String,
        goal: WireEvent,
        substitution: WireSubstitution,
        antecedent: Box<WireTree>,
    },
}

impl From<&ResidualSearchTree> for WireTree {
    fn from(t: &ResidualSearchTree) -> Self {
        match t {
            ResidualSearchTree::Pending {
                goal,
                depth,
                ancestors,
            } => Self::Pending {
                goal: WireEvent::from(goal),
                depth: *depth,
                ancestors: events_to_wire(ancestors),
            },
            ResidualSearchTree::Unit => Self::Unit,
            ResidualSearchTree::Evidence(e) => Self::Evidence(WireEvent::from(e)),
            ResidualSearchTree::Hypothesis(e) => Self::Hypothesis(WireEvent::from(e)),
            ResidualSearchTree::Tensor(l, r) => Self::Tensor(
                Box::new(Self::from(l.as_ref())),
                Box::new(Self::from(r.as_ref())),
            ),
            ResidualSearchTree::Rule {
                rule,
                goal,
                substitution,
                antecedent,
            } => Self::Rule {
                rule: rule.name.clone(),
                goal: WireEvent::from(goal),
                substitution: WireSubstitution::from(substitution),
                antecedent: Box::new(Self::from(antecedent.as_ref())),
            },
        }
    }
}

impl TryFrom<WireTree> for ResidualSearchTree {
    type Error = WireError;
    fn try_from(w: WireTree) -> Result<Self, WireError> {
        Ok(match w {
            WireTree::Pending {
                goal,
                depth,
                ancestors,
            } => Self::Pending {
                goal: Event::try_from(goal)?,
                depth,
                ancestors: events_from_wire(ancestors)?,
            },
            WireTree::Unit => Self::Unit,
            WireTree::Evidence(e) => Self::Evidence(Event::try_from(e)?),
            WireTree::Hypothesis(e) => Self::Hypothesis(Event::try_from(e)?),
            WireTree::Tensor(l, r) => {
                Self::Tensor(Box::new(Self::try_from(*l)?), Box::new(Self::try_from(*r)?))
            }
            WireTree::Rule {
                rule,
                goal,
                substitution,
                antecedent,
            } => Self::Rule {
                rule: RuleRef {
                    name: ident(&rule, "rule")?,
                },
                goal: Event::try_from(goal)?,
                substitution: Substitution::try_from(substitution)?,
                antecedent: Box::new(Self::try_from(*antecedent)?),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireFormula {
    Unit,
    Event(WireEvent),
    Tensor(Box<WireFormula>, Box<WireFormula>),
    Join(Box<WireFormula>, Box<WireFormula>),
}

impl From<&RequirementFormula> for WireFormula {
    fn from(f: &RequirementFormula) -> Self {
        match f {
            RequirementFormula::Unit => Self::Unit,
            RequirementFormula::Event(e) => Self::Event(WireEvent::from(e)),
            RequirementFormula::Tensor(l, r) => Self::Tensor(
                Box::new(Self::from(l.as_ref())),
                Box::new(Self::from(r.as_ref())),
            ),
            RequirementFormula::Join(l, r) => Self::Join(
                Box::new(Self::from(l.as_ref())),
                Box::new(Self::from(r.as_ref())),
            ),
        }
    }
}

impl TryFrom<WireFormula> for RequirementFormula {
    type Error = WireError;
    fn try_from(w: WireFormula) -> Result<Self, WireError> {
        Ok(match w {
            WireFormula::Unit => Self::Unit,
            WireFormula::Event(e) => Self::Event(Event::try_from(e)?),
            WireFormula::Tensor(l, r) => {
                Self::Tensor(Box::new(Self::try_from(*l)?), Box::new(Self::try_from(*r)?))
            }
            WireFormula::Join(l, r) => {
                Self::Join(Box::new(Self::try_from(*l)?), Box::new(Self::try_from(*r)?))
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireProof {
    Unit,
    Evidence(WireEvent),
    Hypothesis(WireEvent),
    Tensor(Box<WireProof>, Box<WireProof>),
    Rule {
        rule: String,
        goal: WireEvent,
        substitution: WireSubstitution,
        antecedent: Box<WireProof>,
    },
}

impl From<&ResidualProof> for WireProof {
    fn from(p: &ResidualProof) -> Self {
        match p {
            ResidualProof::Unit => Self::Unit,
            ResidualProof::Evidence(e) => Self::Evidence(WireEvent::from(e)),
            ResidualProof::Hypothesis(e) => Self::Hypothesis(WireEvent::from(e)),
            ResidualProof::Tensor(l, r) => Self::Tensor(
                Box::new(Self::from(l.as_ref())),
                Box::new(Self::from(r.as_ref())),
            ),
            ResidualProof::Rule {
                rule,
                goal,
                substitution,
                antecedent,
            } => Self::Rule {
                rule: rule.name.clone(),
                goal: WireEvent::from(goal),
                substitution: WireSubstitution::from(substitution),
                antecedent: Box::new(Self::from(antecedent.as_ref())),
            },
        }
    }
}

impl TryFrom<WireProof> for ResidualProof {
    type Error = WireError;
    fn try_from(w: WireProof) -> Result<Self, WireError> {
        Ok(match w {
            WireProof::Unit => Self::Unit,
            WireProof::Evidence(e) => Self::Evidence(Event::try_from(e)?),
            WireProof::Hypothesis(e) => Self::Hypothesis(Event::try_from(e)?),
            WireProof::Tensor(l, r) => {
                Self::Tensor(Box::new(Self::try_from(*l)?), Box::new(Self::try_from(*r)?))
            }
            WireProof::Rule {
                rule,
                goal,
                substitution,
                antecedent,
            } => Self::Rule {
                rule: RuleRef {
                    name: ident(&rule, "rule")?,
                },
                goal: Event::try_from(goal)?,
                substitution: Substitution::try_from(substitution)?,
                antecedent: Box::new(Self::try_from(*antecedent)?),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireExplanation {
    pub required: WireAntecedent,
    pub proof: WireProof,
}

impl From<&ResidualExplanation> for WireExplanation {
    fn from(e: &ResidualExplanation) -> Self {
        Self {
            required: WireAntecedent::from(&e.required),
            proof: WireProof::from(&e.proof),
        }
    }
}

impl TryFrom<WireExplanation> for ResidualExplanation {
    type Error = WireError;
    fn try_from(w: WireExplanation) -> Result<Self, WireError> {
        Ok(Self {
            required: Antecedent::try_from(w.required)?,
            proof: ResidualProof::try_from(w.proof)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireFailure {
    NotAbducible,
    NotForwardSupported,
}

fn failure_to_wire(f: &Option<AbductionFailure>) -> Option<WireFailure> {
    f.as_ref().map(|f| match f {
        AbductionFailure::NotAbducible => WireFailure::NotAbducible,
        AbductionFailure::NotForwardSupported => WireFailure::NotForwardSupported,
    })
}

fn failure_from_wire(w: Option<WireFailure>) -> Option<AbductionFailure> {
    w.map(|f| match f {
        WireFailure::NotAbducible => AbductionFailure::NotAbducible,
        WireFailure::NotForwardSupported => AbductionFailure::NotForwardSupported,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireHornQuery {
    pub goal: WireEvent,
    pub required: Option<WireFormula>,
    pub explanations: Vec<WireExplanation>,
    pub metrics: WireMetrics,
    pub failure: Option<WireFailure>,
}

impl From<&ProgramResidualQuery> for WireHornQuery {
    fn from(q: &ProgramResidualQuery) -> Self {
        Self {
            goal: WireEvent::from(&q.goal),
            required: q.required.as_ref().map(WireFormula::from),
            explanations: q.explanations.iter().map(WireExplanation::from).collect(),
            metrics: WireMetrics::from(&q.metrics),
            failure: failure_to_wire(&q.failure),
        }
    }
}

impl TryFrom<WireHornQuery> for ProgramResidualQuery {
    type Error = WireError;
    fn try_from(w: WireHornQuery) -> Result<Self, WireError> {
        Ok(Self {
            goal: Event::try_from(w.goal)?,
            required: w.required.map(RequirementFormula::try_from).transpose()?,
            explanations: w
                .explanations
                .into_iter()
                .map(ResidualExplanation::try_from)
                .collect::<Result<_, _>>()?,
            metrics: SearchMetrics::from(w.metrics),
            failure: failure_from_wire(w.failure),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireOrigin {
    pub module: String,
    pub file: Option<String>,
}

fn origin_to_wire(o: &DeclarationOrigin) -> WireOrigin {
    WireOrigin {
        module: o.module.clone(),
        file: o.file.as_ref().map(|p| p.to_string_lossy().into_owned()),
    }
}

fn origin_from_wire(w: WireOrigin) -> Result<DeclarationOrigin, WireError> {
    Ok(DeclarationOrigin {
        module: ident(&w.module, "module")?,
        file: w.file.map(PathBuf::from),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireBehaviorKind {
    Rule,
    Program,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireBehavior {
    pub name: String,
    pub kind: WireBehaviorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireProgramRuleOrigin {
    pub program: String,
    pub defining_module: String,
    pub rule: String,
    pub rule_origin: WireOrigin,
    pub rule_use: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireActivation {
    pub verb: String,
    pub behavior: WireBehavior,
    pub rule: String,
    pub trigger: WireEvent,
    pub substitution: WireSubstitution,
    pub rule_origin: Option<WireOrigin>,
    pub program_origin: Option<WireProgramRuleOrigin>,
    pub search_rule: String,
}

impl From<&ResidualActivation> for WireActivation {
    fn from(a: &ResidualActivation) -> Self {
        Self {
            verb: a.verb.clone(),
            behavior: WireBehavior {
                name: a.behavior.name.clone(),
                kind: match a.behavior.kind {
                    BehaviorKind::Rule => WireBehaviorKind::Rule,
                    BehaviorKind::Program => WireBehaviorKind::Program,
                },
            },
            rule: a.rule.name.clone(),
            trigger: WireEvent::from(&a.trigger),
            substitution: WireSubstitution::from(&a.substitution),
            rule_origin: a.rule_origin.as_ref().map(origin_to_wire),
            program_origin: a.program_origin.as_ref().map(|p| WireProgramRuleOrigin {
                program: p.program.clone(),
                defining_module: p.defining_module.clone(),
                rule: p.rule.clone(),
                rule_origin: origin_to_wire(&p.rule_origin),
                rule_use: p.rule_use,
            }),
            search_rule: a.search_rule.name.clone(),
        }
    }
}

impl TryFrom<WireActivation> for ResidualActivation {
    type Error = WireError;
    fn try_from(w: WireActivation) -> Result<Self, WireError> {
        Ok(Self {
            verb: ident(&w.verb, "verb")?,
            behavior: BehaviorRef {
                name: ident(&w.behavior.name, "behavior")?,
                kind: match w.behavior.kind {
                    WireBehaviorKind::Rule => BehaviorKind::Rule,
                    WireBehaviorKind::Program => BehaviorKind::Program,
                },
            },
            rule: RuleRef {
                name: ident(&w.rule, "rule")?,
            },
            trigger: Event::try_from(w.trigger)?,
            substitution: Substitution::try_from(w.substitution)?,
            rule_origin: w.rule_origin.map(origin_from_wire).transpose()?,
            program_origin: w
                .program_origin
                .map(|p| {
                    Ok::<_, WireError>(ProgramRuleOrigin {
                        program: ident(&p.program, "program")?,
                        defining_module: ident(&p.defining_module, "module")?,
                        rule: ident(&p.rule, "rule")?,
                        rule_origin: origin_from_wire(p.rule_origin)?,
                        rule_use: p.rule_use,
                    })
                })
                .transpose()?,
            search_rule: RuleRef {
                name: ident(&w.search_rule, "rule")?,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireCertified {
    pub candidate: WireExplanation,
    pub activations: Vec<WireActivation>,
    pub hypothetical: WireClosure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireActiveQuery {
    pub goal: WireEvent,
    pub required: Option<WireFormula>,
    pub explanations: Vec<WireCertified>,
    pub metrics: WireMetrics,
    pub failure: Option<WireFailure>,
}

impl From<&ActiveProgramResidualQuery> for WireActiveQuery {
    fn from(q: &ActiveProgramResidualQuery) -> Self {
        Self {
            goal: WireEvent::from(&q.goal),
            required: q.required.as_ref().map(WireFormula::from),
            explanations: q
                .explanations
                .iter()
                .map(|c| WireCertified {
                    candidate: WireExplanation::from(&c.candidate),
                    activations: c.activations.iter().map(WireActivation::from).collect(),
                    hypothetical: WireClosure::from(&c.hypothetical),
                })
                .collect(),
            metrics: WireMetrics::from(&q.metrics),
            failure: failure_to_wire(&q.failure),
        }
    }
}

impl TryFrom<WireActiveQuery> for ActiveProgramResidualQuery {
    type Error = WireError;
    fn try_from(w: WireActiveQuery) -> Result<Self, WireError> {
        Ok(Self {
            goal: Event::try_from(w.goal)?,
            required: w.required.map(RequirementFormula::try_from).transpose()?,
            explanations: w
                .explanations
                .into_iter()
                .map(|c| {
                    Ok::<_, WireError>(CertifiedActiveExplanation {
                        candidate: ResidualExplanation::try_from(c.candidate)?,
                        activations: c
                            .activations
                            .into_iter()
                            .map(ResidualActivation::try_from)
                            .collect::<Result<_, _>>()?,
                        hypothetical: Closure::try_from(c.hypothetical)?,
                    })
                })
                .collect::<Result<_, _>>()?,
            metrics: SearchMetrics::from(w.metrics),
            failure: failure_from_wire(w.failure),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireOutcome {
    Horn(WireHornQuery),
    Active(WireActiveQuery),
}

// ---------------------------------------------------------------------------
// Reports, certificates, partitions, inventory.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireConservation {
    pub events_preserved: bool,
    pub proofs_preserved: bool,
    pub added_heads: Vec<String>,
    pub foreign_heads: Vec<String>,
}

impl From<&Conservation> for WireConservation {
    fn from(c: &Conservation) -> Self {
        Self {
            events_preserved: c.events_preserved,
            proofs_preserved: c.proofs_preserved,
            added_heads: c.added_heads.iter().cloned().collect(),
            foreign_heads: c.foreign_heads.iter().cloned().collect(),
        }
    }
}

fn set_from_wire(values: Vec<String>, what: &str) -> Result<BTreeSet<String>, WireError> {
    values.iter().map(|v| ident(v, what)).collect()
}

impl TryFrom<WireConservation> for Conservation {
    type Error = WireError;
    fn try_from(w: WireConservation) -> Result<Self, WireError> {
        Ok(Self {
            events_preserved: w.events_preserved,
            proofs_preserved: w.proofs_preserved,
            added_heads: set_from_wire(w.added_heads, "verb")?,
            foreign_heads: set_from_wire(w.foreign_heads, "verb")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WireProposalStatus {
    Syntax(String),
    Elaboration(String),
    StructuralReject(Vec<String>),
    OutOfCertificateScope(Vec<String>),
    EvaluationPending(String),
    CertificateFailed(WireConservation),
    Evaluated {
        components: Vec<bool>,
        compatible: bool,
        certificate: WireConservation,
    },
}

const STAGES: [&str; 8] = [
    "parsed",
    "elaborated",
    "extracted",
    "admitted",
    "in_certificate_scope",
    "closed",
    "certified",
    "evaluated",
];

fn stage_from_wire(s: &str) -> Result<&'static str, WireError> {
    STAGES
        .iter()
        .copied()
        .find(|k| *k == s)
        .ok_or_else(|| WireError::Validation(format!("unknown stage `{s}`")))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireReport {
    pub id: String,
    pub module_name: String,
    pub original_text: String,
    pub source: String,
    pub file: String,
    pub rules: Vec<WireRule>,
    pub stages: Vec<String>,
    pub status: WireProposalStatus,
}

impl From<&ProposalReport> for WireReport {
    fn from(r: &ProposalReport) -> Self {
        Self {
            id: r.id.clone(),
            module_name: r.module_name.clone(),
            original_text: r.original_text.clone(),
            source: r.source.clone(),
            file: r.file.to_string_lossy().into_owned(),
            rules: r.rules.iter().map(WireRule::from).collect(),
            stages: r.stages.iter().map(|s| s.to_string()).collect(),
            status: match &r.status {
                ProposalStatus::Syntax(m) => WireProposalStatus::Syntax(m.clone()),
                ProposalStatus::Elaboration(m) => WireProposalStatus::Elaboration(m.clone()),
                ProposalStatus::StructuralReject(codes) => {
                    WireProposalStatus::StructuralReject(codes.iter().cloned().collect())
                }
                ProposalStatus::OutOfCertificateScope(heads) => {
                    WireProposalStatus::OutOfCertificateScope(heads.iter().cloned().collect())
                }
                ProposalStatus::EvaluationPending(m) => {
                    WireProposalStatus::EvaluationPending(m.clone())
                }
                ProposalStatus::CertificateFailed(c) => {
                    WireProposalStatus::CertificateFailed(WireConservation::from(c))
                }
                ProposalStatus::Evaluated {
                    components,
                    compatible,
                    certificate,
                } => WireProposalStatus::Evaluated {
                    components: components.clone(),
                    compatible: *compatible,
                    certificate: WireConservation::from(certificate),
                },
            },
        }
    }
}

impl TryFrom<WireReport> for ProposalReport {
    type Error = WireError;
    fn try_from(w: WireReport) -> Result<Self, WireError> {
        Ok(Self {
            id: ident(&w.id, "proposal")?,
            module_name: ident(&w.module_name, "module")?,
            original_text: w.original_text,
            source: w.source,
            file: PathBuf::from(w.file),
            rules: w
                .rules
                .into_iter()
                .map(Rule::try_from)
                .collect::<Result<_, _>>()?,
            stages: w
                .stages
                .iter()
                .map(|s| stage_from_wire(s))
                .collect::<Result<_, _>>()?,
            status: match w.status {
                WireProposalStatus::Syntax(m) => ProposalStatus::Syntax(m),
                WireProposalStatus::Elaboration(m) => ProposalStatus::Elaboration(m),
                WireProposalStatus::StructuralReject(codes) => {
                    ProposalStatus::StructuralReject(set_from_wire(codes, "code")?)
                }
                WireProposalStatus::OutOfCertificateScope(heads) => {
                    ProposalStatus::OutOfCertificateScope(set_from_wire(heads, "verb")?)
                }
                WireProposalStatus::EvaluationPending(m) => ProposalStatus::EvaluationPending(m),
                WireProposalStatus::CertificateFailed(c) => {
                    ProposalStatus::CertificateFailed(Conservation::try_from(c)?)
                }
                WireProposalStatus::Evaluated {
                    components,
                    compatible,
                    certificate,
                } => ProposalStatus::Evaluated {
                    components,
                    compatible,
                    certificate: Conservation::try_from(certificate)?,
                },
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WirePartition {
    pub hypotheses: Vec<String>,
    pub kernel: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRuleOrigin {
    pub name: String,
    pub module: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireInventory {
    pub module: String,
    pub rules: Vec<WireRuleOrigin>,
    pub seeded_verbs: Vec<String>,
    pub derived_verbs: Vec<String>,
    pub effect_verbs: Vec<String>,
    pub dossiers: Vec<String>,
    pub verbs: Vec<WireVerb>,
    pub entities: Vec<WireEntity>,
    pub constants: Vec<WireArg>,
    pub rule_bodies: Vec<WireRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireVerb {
    pub name: String,
    pub kind: WireVerbKind,
    pub args: Vec<WireArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireVerbKind {
    Seeded,
    Derived,
    Effect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireEntity {
    pub name: String,
    pub supertypes: Vec<String>,
    pub fields: Vec<WireArg>,
}

fn args_to_wire(args: &[Arg]) -> Vec<WireArg> {
    args.iter()
        .map(|a| WireArg {
            name: a.name.clone(),
            ty: WireType::from(&a.ty),
        })
        .collect()
}

fn args_from_wire(args: Vec<WireArg>, what: &str) -> Result<Vec<Arg>, WireError> {
    args.into_iter()
        .map(|a| {
            Ok(Arg {
                name: ident(&a.name, what)?,
                ty: Type::try_from(a.ty)?,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Responses.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireStatus {
    Complete,
    StepLimit,
    SearchLimit,
    Cyclic,
    UnresolvedParameters,
    BindingError,
    StructuralReject,
    Error,
}

impl From<Status> for WireStatus {
    fn from(s: Status) -> Self {
        match s {
            Status::Complete => Self::Complete,
            Status::StepLimit => Self::StepLimit,
            Status::SearchLimit => Self::SearchLimit,
            Status::Cyclic => Self::Cyclic,
            Status::UnresolvedParameters => Self::UnresolvedParameters,
            Status::BindingError => Self::BindingError,
            Status::StructuralReject => Self::StructuralReject,
            Status::Error => Self::Error,
        }
    }
}

impl From<WireStatus> for Status {
    fn from(s: WireStatus) -> Self {
        match s {
            WireStatus::Complete => Self::Complete,
            WireStatus::StepLimit => Self::StepLimit,
            WireStatus::SearchLimit => Self::SearchLimit,
            WireStatus::Cyclic => Self::Cyclic,
            WireStatus::UnresolvedParameters => Self::UnresolvedParameters,
            WireStatus::BindingError => Self::BindingError,
            WireStatus::StructuralReject => Self::StructuralReject,
            WireStatus::Error => Self::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireSessionKind {
    Forward,
    Residual,
    ActiveResidual,
    Evaluation,
}

impl From<&SessionKind> for WireSessionKind {
    fn from(k: &SessionKind) -> Self {
        match k {
            SessionKind::Forward => Self::Forward,
            SessionKind::Residual => Self::Residual,
            SessionKind::ActiveResidual => Self::ActiveResidual,
            SessionKind::Evaluation => Self::Evaluation,
        }
    }
}

impl From<WireSessionKind> for SessionKind {
    fn from(k: WireSessionKind) -> Self {
        match k {
            WireSessionKind::Forward => Self::Forward,
            WireSessionKind::Residual => Self::Residual,
            WireSessionKind::ActiveResidual => Self::ActiveResidual,
            WireSessionKind::Evaluation => Self::Evaluation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireForwardRound {
    pub round: usize,
    pub added: Vec<WireEvent>,
    pub derivations: Vec<WireDerived>,
    pub witnesses: usize,
    pub complete: bool,
}

impl From<&ForwardRound> for WireForwardRound {
    fn from(r: &ForwardRound) -> Self {
        Self {
            round: r.round,
            added: events_to_wire(&r.added),
            derivations: r.derivations.iter().map(WireDerived::from).collect(),
            witnesses: r.witnesses,
            complete: r.complete,
        }
    }
}

impl TryFrom<WireForwardRound> for ForwardRound {
    type Error = WireError;
    fn try_from(w: WireForwardRound) -> Result<Self, WireError> {
        Ok(Self {
            round: w.round,
            added: events_from_wire(w.added)?,
            derivations: w
                .derivations
                .into_iter()
                .map(DerivedEvent::try_from)
                .collect::<Result<_, _>>()?,
            witnesses: w.witnesses,
            complete: w.complete,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireResidualRound {
    pub round: usize,
    pub pending: Vec<WireEvent>,
    pub branches: Vec<WireTree>,
    pub metrics: WireMetrics,
    pub complete: bool,
}

impl From<&ResidualRound> for WireResidualRound {
    fn from(r: &ResidualRound) -> Self {
        Self {
            round: r.round,
            pending: events_to_wire(&r.pending),
            branches: r.branches.iter().map(WireTree::from).collect(),
            metrics: WireMetrics::from(&r.metrics),
            complete: r.complete,
        }
    }
}

impl TryFrom<WireResidualRound> for ResidualRound {
    type Error = WireError;
    fn try_from(w: WireResidualRound) -> Result<Self, WireError> {
        Ok(Self {
            round: w.round,
            pending: events_from_wire(w.pending)?,
            branches: w
                .branches
                .into_iter()
                .map(ResidualSearchTree::try_from)
                .collect::<Result<_, _>>()?,
            metrics: SearchMetrics::from(w.metrics),
            complete: w.complete,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WirePayload {
    Theory(WireInventory),
    SessionOpened {
        session: String,
        kind: WireSessionKind,
    },
    Forward {
        session: String,
        rounds: Vec<WireForwardRound>,
        known: Vec<WireEvent>,
        complete: bool,
    },
    Residual {
        session: String,
        kind: WireSessionKind,
        rounds: Vec<WireResidualRound>,
        complete: bool,
        result: Option<WireOutcome>,
        error: Option<String>,
    },
    Candidate {
        candidate: Option<String>,
        report: WireReport,
    },
    Evaluation {
        session: String,
        candidate: String,
        dossier: String,
        rounds: Vec<WireForwardRound>,
        complete: bool,
        closure: Option<WireClosure>,
        report: Option<WireReport>,
    },
    Supports {
        candidate: String,
        goal_in_universe: bool,
        frontier: Vec<Vec<WireEvent>>,
    },
    Partition {
        pool: Vec<Vec<WireEvent>>,
        partition: WirePartition,
        signatures: Vec<Vec<bool>>,
        discriminants: Vec<usize>,
        covers: Vec<(String, String)>,
    },
    Error {
        message: String,
    },
}

fn states_to_wire(states: &[Vec<Event>]) -> Vec<Vec<WireEvent>> {
    states.iter().map(|s| events_to_wire(s)).collect()
}

fn states_from_wire(states: Vec<Vec<WireEvent>>) -> Result<Vec<Vec<Event>>, WireError> {
    states.into_iter().map(events_from_wire).collect()
}

impl From<&Payload> for WirePayload {
    fn from(p: &Payload) -> Self {
        match p {
            Payload::Theory(t) => Self::Theory(WireInventory {
                module: t.module.clone(),
                rules: t
                    .rules
                    .iter()
                    .map(|(name, module)| WireRuleOrigin {
                        name: name.clone(),
                        module: module.clone(),
                    })
                    .collect(),
                seeded_verbs: t.seeded_verbs.clone(),
                derived_verbs: t.derived_verbs.clone(),
                effect_verbs: t.effect_verbs.clone(),
                dossiers: t.dossiers.clone(),
                verbs: t
                    .verbs
                    .iter()
                    .map(|v| WireVerb {
                        name: v.name.clone(),
                        kind: match v.kind {
                            VerbKind::Seeded => WireVerbKind::Seeded,
                            VerbKind::Derived => WireVerbKind::Derived,
                            VerbKind::Effect => WireVerbKind::Effect,
                        },
                        args: args_to_wire(&v.args),
                    })
                    .collect(),
                entities: t
                    .entities
                    .iter()
                    .map(|e| WireEntity {
                        name: e.name.clone(),
                        supertypes: e.supertypes.clone(),
                        fields: e
                            .fields
                            .iter()
                            .map(|f| WireArg {
                                name: f.name.clone(),
                                ty: WireType::from(&f.ty),
                            })
                            .collect(),
                    })
                    .collect(),
                constants: t
                    .constants
                    .iter()
                    .map(|c| WireArg {
                        name: c.name.clone(),
                        ty: WireType::from(&c.ty),
                    })
                    .collect(),
                rule_bodies: t.rule_bodies.iter().map(WireRule::from).collect(),
            }),
            Payload::SessionOpened { session, kind } => Self::SessionOpened {
                session: session.clone(),
                kind: kind.into(),
            },
            Payload::Forward {
                session,
                rounds,
                known,
                complete,
            } => Self::Forward {
                session: session.clone(),
                rounds: rounds.iter().map(WireForwardRound::from).collect(),
                known: events_to_wire(known),
                complete: *complete,
            },
            Payload::Residual {
                session,
                kind,
                rounds,
                complete,
                result,
                error,
            } => Self::Residual {
                session: session.clone(),
                kind: kind.into(),
                rounds: rounds.iter().map(WireResidualRound::from).collect(),
                complete: *complete,
                result: result.as_ref().map(|r| match r {
                    ResidualOutcome::Horn(q) => WireOutcome::Horn(WireHornQuery::from(q)),
                    ResidualOutcome::Active(q) => WireOutcome::Active(WireActiveQuery::from(q)),
                }),
                error: error.clone(),
            },
            Payload::Candidate { candidate, report } => Self::Candidate {
                candidate: candidate.clone(),
                report: WireReport::from(report),
            },
            Payload::Evaluation {
                session,
                candidate,
                dossier,
                rounds,
                complete,
                closure,
                report,
            } => Self::Evaluation {
                session: session.clone(),
                candidate: candidate.clone(),
                dossier: dossier.clone(),
                rounds: rounds.iter().map(WireForwardRound::from).collect(),
                complete: *complete,
                closure: closure.as_ref().map(WireClosure::from),
                report: report.as_ref().map(WireReport::from),
            },
            Payload::Supports {
                candidate,
                goal_in_universe,
                frontier,
            } => Self::Supports {
                candidate: candidate.clone(),
                goal_in_universe: *goal_in_universe,
                frontier: states_to_wire(frontier),
            },
            Payload::Partition {
                pool,
                partition,
                signatures,
                discriminants,
                covers,
            } => Self::Partition {
                pool: states_to_wire(pool),
                partition: WirePartition {
                    hypotheses: partition.hypotheses.clone(),
                    kernel: partition.kernel.clone(),
                },
                signatures: signatures.clone(),
                discriminants: discriminants.clone(),
                covers: covers.clone(),
            },
            Payload::Error { message } => Self::Error {
                message: message.clone(),
            },
        }
    }
}

fn idents(values: Vec<String>, what: &str) -> Result<Vec<String>, WireError> {
    values.iter().map(|v| ident(v, what)).collect()
}

impl TryFrom<WirePayload> for Payload {
    type Error = WireError;
    fn try_from(w: WirePayload) -> Result<Self, WireError> {
        Ok(match w {
            WirePayload::Theory(t) => Self::Theory(TheoryInventory {
                module: ident(&t.module, "module")?,
                rules: t
                    .rules
                    .into_iter()
                    .map(|r| Ok((ident(&r.name, "rule")?, ident(&r.module, "module")?)))
                    .collect::<Result<_, WireError>>()?,
                seeded_verbs: idents(t.seeded_verbs, "verb")?,
                derived_verbs: idents(t.derived_verbs, "verb")?,
                effect_verbs: idents(t.effect_verbs, "verb")?,
                dossiers: idents(t.dossiers, "dossier")?,
                verbs: t
                    .verbs
                    .into_iter()
                    .map(|v| {
                        Ok(VerbSignature {
                            name: ident(&v.name, "verb")?,
                            kind: match v.kind {
                                WireVerbKind::Seeded => VerbKind::Seeded,
                                WireVerbKind::Derived => VerbKind::Derived,
                                WireVerbKind::Effect => VerbKind::Effect,
                            },
                            args: args_from_wire(v.args, "argument")?,
                        })
                    })
                    .collect::<Result<_, WireError>>()?,
                entities: t
                    .entities
                    .into_iter()
                    .map(|e| {
                        Ok(Entity {
                            name: ident(&e.name, "entity")?,
                            supertypes: idents(e.supertypes, "entity")?,
                            fields: args_from_wire(e.fields, "field")?
                                .into_iter()
                                .map(|a| Field {
                                    name: a.name,
                                    ty: a.ty,
                                })
                                .collect(),
                        })
                    })
                    .collect::<Result<_, WireError>>()?,
                constants: args_from_wire(t.constants, "constant")?
                    .into_iter()
                    .map(|a| Constant {
                        name: a.name,
                        ty: a.ty,
                    })
                    .collect(),
                rule_bodies: t
                    .rule_bodies
                    .into_iter()
                    .map(Rule::try_from)
                    .collect::<Result<_, _>>()?,
            }),
            WirePayload::SessionOpened { session, kind } => Self::SessionOpened {
                session: ident(&session, "session")?,
                kind: kind.into(),
            },
            WirePayload::Forward {
                session,
                rounds,
                known,
                complete,
            } => Self::Forward {
                session: ident(&session, "session")?,
                rounds: rounds
                    .into_iter()
                    .map(ForwardRound::try_from)
                    .collect::<Result<_, _>>()?,
                known: events_from_wire(known)?,
                complete,
            },
            WirePayload::Residual {
                session,
                kind,
                rounds,
                complete,
                result,
                error,
            } => Self::Residual {
                session: ident(&session, "session")?,
                kind: kind.into(),
                rounds: rounds
                    .into_iter()
                    .map(ResidualRound::try_from)
                    .collect::<Result<_, _>>()?,
                complete,
                result: match result {
                    None => None,
                    Some(WireOutcome::Horn(q)) => {
                        Some(ResidualOutcome::Horn(ProgramResidualQuery::try_from(q)?))
                    }
                    Some(WireOutcome::Active(q)) => Some(ResidualOutcome::Active(
                        ActiveProgramResidualQuery::try_from(q)?,
                    )),
                },
                error,
            },
            WirePayload::Candidate { candidate, report } => Self::Candidate {
                candidate: opt_ident(candidate, "candidate")?,
                report: ProposalReport::try_from(report)?,
            },
            WirePayload::Evaluation {
                session,
                candidate,
                dossier,
                rounds,
                complete,
                closure,
                report,
            } => Self::Evaluation {
                session: ident(&session, "session")?,
                candidate: ident(&candidate, "candidate")?,
                dossier: ident(&dossier, "dossier")?,
                rounds: rounds
                    .into_iter()
                    .map(ForwardRound::try_from)
                    .collect::<Result<_, _>>()?,
                complete,
                closure: closure.map(Closure::try_from).transpose()?,
                report: report.map(ProposalReport::try_from).transpose()?,
            },
            WirePayload::Supports {
                candidate,
                goal_in_universe,
                frontier,
            } => Self::Supports {
                candidate: ident(&candidate, "candidate")?,
                goal_in_universe,
                frontier: states_from_wire(frontier)?,
            },
            WirePayload::Partition {
                pool,
                partition,
                signatures,
                discriminants,
                covers,
            } => {
                let hypotheses = idents(partition.hypotheses, "candidate")?;
                if hypotheses.len() != partition.kernel.len() {
                    return Err(WireError::Validation(
                        "partition kernel and hypotheses differ in length".into(),
                    ));
                }
                Self::Partition {
                    pool: states_from_wire(pool)?,
                    partition: Partition {
                        hypotheses,
                        kernel: partition.kernel,
                    },
                    signatures,
                    discriminants,
                    covers: covers
                        .into_iter()
                        .map(|(a, b)| Ok((ident(&a, "candidate")?, ident(&b, "candidate")?)))
                        .collect::<Result<_, WireError>>()?,
                }
            }
            WirePayload::Error { message } => Self::Error { message },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireBudget {
    pub actions_remaining: usize,
    pub steps_remaining: usize,
    pub sessions_remaining: usize,
    pub candidates_remaining: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireResponse {
    pub wire_version: String,
    pub action_id: String,
    pub status: WireStatus,
    pub payload: WirePayload,
    pub steps_consumed: usize,
    pub budget: WireBudget,
}

impl From<&Response> for WireResponse {
    fn from(r: &Response) -> Self {
        Self {
            wire_version: WIRE_VERSION.into(),
            action_id: r.action_id.clone(),
            status: r.status.into(),
            payload: WirePayload::from(&r.payload),
            steps_consumed: r.steps_consumed,
            budget: WireBudget {
                actions_remaining: r.budget.actions_remaining,
                steps_remaining: r.budget.steps_remaining,
                sessions_remaining: r.budget.sessions_remaining,
                candidates_remaining: r.budget.candidates_remaining,
            },
        }
    }
}

impl TryFrom<WireResponse> for Response {
    type Error = WireError;
    fn try_from(w: WireResponse) -> Result<Self, WireError> {
        version(&w.wire_version)?;
        Ok(Self {
            action_id: ident(&w.action_id, "action")?,
            status: w.status.into(),
            payload: Payload::try_from(w.payload)?,
            steps_consumed: w.steps_consumed,
            budget: BudgetState {
                actions_remaining: w.budget.actions_remaining,
                steps_remaining: w.budget.steps_remaining,
                sessions_remaining: w.budget.sessions_remaining,
                candidates_remaining: w.budget.candidates_remaining,
            },
        })
    }
}
