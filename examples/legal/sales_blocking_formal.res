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

seeded verb agreesPrice(subject : Person, object : Thing, recipient : Person)
seeded verb agreesObject(subject : Person, object : Thing, recipient : Person)
seeded verb retentionAgreed(subject : Person, object : Thing, recipient : Person)

derived verb inconsistentSale(subject : Person, object : Thing, recipient : Person)
derived verb inconsistentRetention(subject : Person, object : Thing, recipient : Person)

rule saleConflict(debtor, object, creditor, t) =
    agreesPrice(debtor, object, creditor) @ t
    & ~agreesPrice(debtor, object, creditor) @ t
    <= inconsistentSale(debtor, object, creditor) @ t

rule retentionConflict(debtor, object, creditor, t) =
    retentionAgreed(debtor, object, creditor) @ t
    & ~retentionAgreed(debtor, object, creditor) @ t
    <= inconsistentRetention(debtor, object, creditor) @ t

rule sales(debtor, object, creditor, t) =
    agreesPrice(debtor, object, creditor) @ t
    & agreesObject(creditor, object, debtor) @ t
    <= give(debtor, object, creditor) @ after t

rule retention(debtor, object, creditor, t) =
    agreesPrice(debtor, object, creditor) @ t
    & agreesObject(creditor, object, debtor) @ t
    <= do(debtor, deliver, object, creditor) @ after t

default NormalSale =
    supernormal sales
    blocked when
        inconsistentSale(debtor, object, creditor) @ t

default RetentionSale =
    exception retention
    to NormalSale
    when
        retentionAgreed(debtor, object, creditor) @ t
    blocked when
        inconsistentRetention(debtor, object, creditor) @ t
