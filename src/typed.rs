#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub declarations: Vec<Declaration>,
}

impl Module {
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
    Default(Default),
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbKind {
    Seeded,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultBody {
    Supernormal { rule: RuleRef },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRef {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleBody {
    HornClause(HornClause),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HornClause {
    pub antecedent: Antecedent,
    pub consequent: Event,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Antecedent {
    Event(Event),
    And(Box<Antecedent>, Box<Antecedent>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    pub verb: String,
    pub args: Vec<Term>,
    pub proposition: PropositionType,
    pub transition: Option<StateTransitionType>,
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
    At(String),
    After(Box<TimeExpr>),
}
