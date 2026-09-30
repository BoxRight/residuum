module Legal


// ============================================================================
// DOMAIN
// ============================================================================

entity Thing

entity Movable   <: Thing
entity Immovable <: Thing
entity Fungible  <: Thing
entity Money     <: Fungible


// ============================================================================
// CONDUCT
// ============================================================================

entity Conduct {
    conduct        : Thing
    indirectObject : Thing?
}


// ============================================================================
// LEGAL RELATION
// ============================================================================

entity Relation {
    debtor   : Person
    thing    : Thing?
    conduct  : Conduct?
    creditor : Person
}


// ============================================================================
// PATRIMONY
// ============================================================================

entity Patrimony {
    assets      : Set Thing
    rights      : Set Relation
    obligations : Set Relation
}

entity Person {
    patrimony : Patrimony
}


// ============================================================================
// LEGAL EFFECTS
//
// Effect <: Derived <: Prop
//
// Once derived, an Effect may itself occur as an antecedent.
// ============================================================================

effect verb give(
    subject   : Person,
    object    : Thing,
    recipient : Person
) =>
    subject.patrimony.obligations += Relation {
        debtor   = subject,
        thing    = object,
        conduct  = none,
        creditor = recipient,
    }

    recipient.patrimony.rights += Relation {
        debtor   = subject,
        thing    = object,
        conduct  = none,
        creditor = recipient,
    }


effect verb do(
    subject        : Person,
    conduct        : Conduct,
    indirectObject : Thing?,
    beneficiary    : Person
) =>
    subject.patrimony.obligations += Relation {
        debtor   = subject,
        thing    = indirectObject,
        conduct  = conduct,
        creditor = beneficiary,
    }

    beneficiary.patrimony.rights += Relation {
        debtor   = subject,
        thing    = indirectObject,
        conduct  = conduct,
        creditor = beneficiary,
    }


effect verb notDo(
    subject        : Person,
    conduct        : Conduct,
    indirectObject : Thing?,
    beneficiary    : Person
) =>
    subject.patrimony.obligations += Relation {
        debtor   = subject,
        thing    = indirectObject,
        conduct  = conduct,
        creditor = beneficiary,
    }

    beneficiary.patrimony.rights += Relation {
        debtor   = subject,
        thing    = indirectObject,
        conduct  = conduct,
        creditor = beneficiary,
    }


// ============================================================================
// SALE FACTS
// ============================================================================

seeded verb agreesPrice(
    subject   : Person,
    object    : Thing,
    recipient : Person
)

seeded verb agreesObject(
    subject   : Person,
    object    : Thing,
    recipient : Person
)


// ============================================================================
// SALE
// ============================================================================

rule sales(
    debtor,
    object,
    creditor,
    t
) =
    agreesPrice(debtor, object, creditor) @ t

    ⊗

    agreesObject(creditor, object, debtor) @ t

    ≤

    give(debtor, object, creditor) @ after t


// ============================================================================
// TRADITIO
//
// `give` is Derived/Effect, but may be an antecedent.
// ============================================================================

rule traditio(
    debtor,
    conduct,
    object,
    creditor,
    t
) =
    give(debtor, object, creditor) @ t

    ≤

    do(
        debtor,
        conduct,
        object,
        creditor
    ) @ after t


// ============================================================================
// RETENTION OF TITLE
//
// Same relevant sale conditions; competing consequence.
// The default structure determines precedence.
//
// TODO: rewrite this using stratified negation-as-failure in antecedent position.
// Negation belongs in antecedents, e.g. A & !B <= C, not in consequents.
// ============================================================================

// ============================================================================
// DEFAULTS
// ============================================================================

default NormalSale =
    supernormal sales


default RetentionOfTitleSale =
    exception retention
    to NormalSale
