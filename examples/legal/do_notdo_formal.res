module Legal

entity Fungible <: Thing
entity Immovable <: Thing
entity Money <: Fungible
entity Movable <: Thing
entity Thing
entity Conduct
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
