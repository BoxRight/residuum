module Legal

entity Thing
entity Patrimony {
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
entity Conduct

effect verb records(
    subject   : Person,
    object    : Thing,
    recipient : Person
) =>
    subject.patrimony.obligations += Relation {
        debtor   = none,
        thing    = object,
        conduct  = none,
        creditor = recipient,
    }
