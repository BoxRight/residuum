use crate::typed::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalModule {
    pub name: String,
    pub declarations: Vec<FormalDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormalDeclaration {
    Entity(FormalEntity),
    Const(FormalConst),
    Verb(FormalVerb),
    Rule(FormalRule),
    Default(FormalDefault),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalConst {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalEntity {
    pub name: String,
    pub supertypes: Vec<String>,
    pub fields: Vec<FormalField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalVerb {
    pub name: String,
    pub kind: SurfaceVerbKind,
    pub args: Vec<FormalArg>,
    pub effect: Option<SurfaceEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalArg {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceVerbKind {
    Seeded,
    Effect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceEffect {
    pub operations: Vec<SurfaceStateOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceStateOperation {
    AddToSet {
        target: SurfaceFieldAccess,
        value: SurfaceTerm,
    },
    RemoveFromSet {
        target: SurfaceFieldAccess,
        value: SurfaceTerm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceTerm {
    Var(String),
    None,
    RecordLiteral {
        entity: String,
        fields: Vec<SurfaceRecordFieldValue>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceRecordFieldValue {
    pub field: String,
    pub value: SurfaceTerm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceFieldAccess {
    pub base: String,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalRule {
    pub name: String,
    pub body: SurfaceRuleBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceRuleBody {
    Horn {
        params: Vec<String>,
        premise: SurfaceAntecedent,
        conclusion: SurfaceEvent,
    },
    Application {
        rule: String,
        args: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceAntecedent {
    Event(SurfaceEvent),
    And(Box<SurfaceAntecedent>, Box<SurfaceAntecedent>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalDefault {
    pub name: String,
    pub body: SurfaceDefaultBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceDefaultBody {
    Supernormal { rule: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceEvent {
    pub verb: String,
    pub args: Vec<String>,
    pub time: Option<SurfaceTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceTime {
    At(String),
    After(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnlModule {
    pub name: String,
    pub declarations: Vec<CnlDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CnlDeclaration {
    Entity(CnlEntity),
    Const(CnlConst),
    Field(CnlField),
    Verb(CnlVerb),
    Rule(CnlRule),
    Default(CnlDefault),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnlConst {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnlEntity {
    pub name: String,
    pub supertypes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnlField {
    pub owner: String,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnlVerb {
    pub name: String,
    pub kind: SurfaceVerbKind,
    pub args: Vec<FormalArg>,
    pub effect: Option<SurfaceEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnlRule {
    pub name: String,
    pub body: SurfaceRuleBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnlDefault {
    pub name: String,
    pub body: SurfaceDefaultBody,
}
