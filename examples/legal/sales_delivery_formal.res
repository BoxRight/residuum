module Legal

entity Thing
entity Conduct
entity Relation {
    debtor   : Person
    thing    : Thing?
    conduct  : Conduct?
    creditor : Person
}
entity Patrimony {
    assets      : Set Thing
    rights      : Set Relation
    obligations : Set Relation
}
entity Person {
    patrimony : Patrimony
}

const deliver : Conduct

effect verb give(
    subject   : Person,
    object    : Thing,
    recipient : Person
) =>
    subject.patrimony.assets   -= object
    recipient.patrimony.assets += object

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

rule traditio(conduct, debtor, object, creditor, t) =
    give(debtor, object, creditor) @ t
    <=
    do(debtor, conduct, object, creditor) @ after t

rule delivery = traditio(deliver)
