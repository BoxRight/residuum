#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub declarations: Vec<Declaration>,
    /// Resolution metadata, outside logical identity and derivation records.
    pub origins: std::collections::BTreeMap<String, DeclarationOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationOrigin {
    pub module: String,
    pub file: Option<std::path::PathBuf>,
}

impl Declaration {
    pub fn name(&self) -> &str {
        match self {
            Self::Entity(d) => &d.name,
            Self::Const(d) => &d.name,
            Self::Verb(d) => &d.name,
            Self::Rule(d) => &d.name,
            Self::Program(d) => &d.identity,
            Self::Default(d) => &d.name,
            Self::Experiment(d) => &d.name,
        }
    }
}

impl Module {
    /// Also resolves the origin of RuleRef and Verb.program by their unique name.
    pub fn origin(&self, name: &str) -> Option<&DeclarationOrigin> {
        self.origins.get(name)
    }
    pub fn is_subtype(&self, subtype: &str, supertype: &str) -> bool {
        if subtype == supertype {
            return true;
        }

        self.direct_supertypes(subtype)
            .iter()
            .any(|parent| self.is_subtype(parent, supertype))
    }

    fn direct_supertypes(&self, subtype: &str) -> Vec<&str> {
        self.declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Entity(entity) if entity.name == subtype => Some(entity),
                _ => None,
            })
            .flat_map(|entity| entity.supertypes.iter().map(String::as_str))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Entity(Entity),
    Const(Constant),
    Verb(Verb),
    Rule(Rule),
    Program(Program),
    Default(Default),
    Experiment(Experiment),
}

/// A source-defined experiment, distinct from asserted or derived events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Experiment {
    pub name: String,
    pub input_kind: ExperimentInputKind,
    pub input: Vec<Event>,
    pub query_kind: ExperimentQueryKind,
    pub goal: Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentInputKind {
    Seeds,
    Premises,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentQueryKind {
    Residual,
    FourFusion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub name: String,
    pub supertypes: Vec<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Entity(String),
    Set(Box<Type>),
    Optional(Box<Type>),
    PropositionTime,
    DerivationTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    pub name: String,
    pub kind: VerbKind,
    pub args: Vec<Arg>,
    pub effect: Option<Effect>,
    /// Association activated for Seeded/Derived instances by `close_program`.
    /// The rules-only `close` API does not consult verb declarations.
    pub program: Option<BehaviorRef>,
}

impl Verb {
    /// Seeds may be proposed as explicit hypotheses by default. Derived and
    /// Effect events require a rule proof and are never invented as hypotheses.
    pub fn is_abducible(&self) -> bool {
        self.kind == VerbKind::Seeded
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbKind {
    Seeded,
    Derived,
    Effect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    StateTransform {
        transition: StateTransitionType,
        operations: Vec<StateOperation>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateTransitionType {
    pub input: StateTime,
    pub output: StateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateTime {
    Unresolved,
    At(TimeExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateOperation {
    AddToSet { target: FieldAccess, value: Term },
    RemoveFromSet { target: FieldAccess, value: Term },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldAccess {
    pub base: Term,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Constructor {
        constructor: Constructor,
        argument: Box<Term>,
    },
    Var(String),
    Const(String),
    None,
    RecordLiteral {
        entity: String,
        fields: Vec<RecordFieldValue>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordFieldValue {
    pub field: String,
    pub value: Term,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedEvent {
    pub event: Event,
    pub derived_at: DerivationTime,
    pub record: DerivationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnstampedDerivation {
    pub event: Event,
    pub record: DerivationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closure {
    pub known: Vec<Event>,
    pub derivations: Vec<DerivedEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationTime {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationRecord {
    pub rule: String,
    pub antecedents: Vec<Event>,
    pub substitution: Substitution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution {
    pub bindings: Vec<SubstitutionBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionBinding {
    pub parameter: String,
    pub value: SubstitutionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstitutionValue {
    Term(Term),
    Time(TimeExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub parameters: Vec<Arg>,
    pub body: RuleBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Default {
    pub name: String,
    pub parameters: Vec<Arg>,
    pub body: DefaultBody,
    pub condition: Option<Antecedent>,
    pub blocking: Option<Antecedent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultBody {
    Supernormal { rule: RuleRef },
    Exception { rule: RuleRef, to: DefaultRef },
    SupernormalProgram { program: ProgramRef },
    ExceptionProgram { program: ProgramRef, to: DefaultRef },
}

impl DefaultBody {
    pub fn target_name(&self) -> &str {
        match self {
            Self::Supernormal { rule } | Self::Exception { rule, .. } => &rule.name,
            Self::SupernormalProgram { program } | Self::ExceptionProgram { program, .. } => {
                &program.name
            }
        }
    }
    pub fn exception_parent(&self) -> Option<&DefaultRef> {
        match self {
            Self::Exception { to, .. } | Self::ExceptionProgram { to, .. } => Some(to),
            _ => None,
        }
    }
    pub fn is_program(&self) -> bool {
        matches!(
            self,
            Self::SupernormalProgram { .. } | Self::ExceptionProgram { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramRef {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultRef {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRef {
    pub name: String,
}

/// A resolved association; legacy single-rule associations remain supported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorRef {
    pub name: String,
    pub kind: BehaviorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorKind {
    Rule,
    Program,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub identity: String,
    pub parameters: Vec<Arg>,
    pub rule_uses: Vec<RuleUse>,
    pub defining_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleUse {
    pub rule: RuleRef,
    /// Missing entries are local binders of the referenced Rule.
    pub bindings: Vec<ProgramBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramBinding {
    pub parameter: String,
    pub value: Term,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramRuleOrigin {
    pub program: String,
    pub defining_module: String,
    pub rule: String,
    pub rule_origin: DeclarationOrigin,
    pub rule_use: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleBody {
    HornClause(HornClause),
    ResidualClause(ResidualClause),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HornClause {
    pub antecedent: Antecedent,
    pub consequent: Event,
}

/// A <= (B multimap C). This is a clause under a Rule's existing lambda binders,
/// not an Event to insert into Known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualClause {
    pub antecedent: Antecedent,
    pub consequent: Residual,
}

/// The first residual fragment retains a required antecedent and an event result.
/// Their proposition times remain independent; this is not a StateTransform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Residual {
    pub required: Antecedent,
    pub consequent: Event,
}

/// A pending condition for one residual rule and target, not derived evidence.
/// `goal_substitution` records constraints from the ground goal, not evidence.
/// `substitution` combines those bindings with matches of actual evidence.
/// `parameters` retain the scope and types of still unbound lambda parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualRequirement {
    pub rule: RuleRef,
    pub evidence: Vec<Event>,
    pub goal_substitution: Substitution,
    pub substitution: Substitution,
    pub goal: Event,
    pub required: Antecedent,
    pub parameters: Vec<Arg>,
}

impl ResidualClause {
    pub fn unresiduate(&self) -> HornClause {
        HornClause {
            antecedent: Antecedent::And(
                Box::new(self.antecedent.clone()),
                Box::new(self.consequent.required.clone()),
            ),
            consequent: self.consequent.consequent.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Antecedent {
    /// Monoidal identity I: one neutral witness independently of Known.
    /// This is neither empty knowledge nor a timed Event.
    Unit,
    Event(Event),
    And(Box<Antecedent>, Box<Antecedent>),
}

/// A logical proposition with an associated operational interpretation.
/// Equality and ordering use verb, arguments, proposition time, and polarity.
/// The declared proposition kind is checked by M2, not part of logical identity.
/// Compare `transition` explicitly when checking operational metadata.
#[derive(Debug, Clone)]
pub struct Event {
    pub polarity: Polarity,
    pub verb: String,
    pub args: Vec<Term>,
    pub proposition: PropositionType,
    pub transition: Option<StateTransitionType>,
}

impl Event {
    pub fn logical_key(&self) -> (&str, &[Term], &TimeExpr, Polarity) {
        (
            &self.verb,
            &self.args,
            &self.proposition.time,
            self.polarity,
        )
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.logical_key() == other.logical_key()
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.logical_key().cmp(&other.logical_key())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Polarity {
    Positive,
    EvidentialNot,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PropositionType {
    pub kind: PropositionKind,
    pub time: TimeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropositionKind {
    Prop,
    Derived,
    Seeded,
    Effect,
}

impl PropositionType {
    pub fn is_subtype_of(&self, expected: &Self) -> bool {
        self.time == expected.time && self.kind.is_subkind_of(&expected.kind)
    }
}

impl PropositionKind {
    fn is_subkind_of(&self, expected: &Self) -> bool {
        use PropositionKind::{Derived, Effect, Prop, Seeded};

        matches!(
            (self, expected),
            (Prop, Prop)
                | (Derived, Derived | Prop)
                | (Effect, Effect | Derived | Prop)
                | (Seeded, Seeded | Prop)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeExpr {
    /// A resolved temporal constant; never a substitution binder.
    At(String),
    /// A temporal parameter, distinguished even from a homonymous constant.
    Var(String),
    After(Box<TimeExpr>),
}

/// The only term constructor in this experimental extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constructor {
    After,
}
impl Constructor {
    pub fn signature(self) -> (Type, Type) {
        match self {
            Self::After => (Type::PropositionTime, Type::PropositionTime),
        }
    }
}
impl TimeExpr {
    /// Structural encoding; no evaluation or fresh identities.
    pub fn to_term(&self) -> Term {
        match self {
            Self::At(name) => Term::Const(name.clone()),
            Self::Var(name) => Term::Var(name.clone()),
            Self::After(inner) => Term::Constructor {
                constructor: Constructor::After,
                argument: Box::new(inner.to_term()),
            },
        }
    }
    /// Structural decoding. The caller must already know the term has type
    /// PropositionTime; a bare constant's declared type is checked by elaboration.
    pub fn from_term(term: &Term) -> Option<Self> {
        match term {
            Term::Const(name) => Some(Self::At(name.clone())),
            Term::Var(name) => Some(Self::Var(name.clone())),
            Term::Constructor {
                constructor: Constructor::After,
                argument,
            } => Some(Self::After(Box::new(Self::from_term(argument)?))),
            _ => None,
        }
    }
}
impl SubstitutionValue {
    pub(crate) fn as_term(&self) -> Term {
        match self {
            Self::Term(term) => term.clone(),
            Self::Time(time) => time.to_term(),
        }
    }
    pub(crate) fn as_time(&self) -> Option<TimeExpr> {
        match self {
            Self::Time(time) => Some(time.clone()),
            Self::Term(term) => TimeExpr::from_term(term),
        }
    }
    pub(crate) fn compatible(&self, other: &Self) -> bool {
        self.as_term() == other.as_term()
    }
}

impl Term {
    pub fn is_ground(&self) -> bool {
        match self {
            Self::Var(_) => false,
            Self::Constructor { argument, .. } => argument.is_ground(),
            Self::RecordLiteral { fields, .. } => fields.iter().all(|f| f.value.is_ground()),
            Self::Const(_) | Self::None => true,
        }
    }
}
