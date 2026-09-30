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
