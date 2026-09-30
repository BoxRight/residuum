module Legal

entity Fungible <: Thing
entity Immovable <: Thing
entity Money <: Fungible
entity Movable <: Thing
entity Thing
entity Conduct {
    conduct        : Thing
    indirectObject : Thing?
}
entity Patrimony {
    assets      : Set Thing
    rights      : Set Relation
    obligations : Set Relation
}
entity Person {
    patrimony : Patrimony
}
entity Relation {
    debtor   : Person
    thing    : Thing?
    conduct  : Conduct?
    creditor : Person
}

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

rule sales(debtor, object, creditor, t) =
    agreesPrice(debtor, object, creditor) @ t
    &
    agreesObject(creditor, object, debtor) @ t
    <=
    give(debtor, object, creditor) @ after t
