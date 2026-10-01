module ForwardIncremental

entity Person
entity Thing

const alice : Person
const bob : Person
const car : Thing
const tau : PropositionTime

seeded verb agreesPrice(subject : Person, object : Thing, recipient : Person)
seeded verb agreesObject(subject : Person, object : Thing, recipient : Person)
derived verb saleEstablished(subject : Person, object : Thing, recipient : Person)
derived verb deliverable(subject : Person, object : Thing, recipient : Person)

rule sales(debtor, object, creditor, t) =
    agreesPrice(debtor, object, creditor) @ t
    & agreesObject(creditor, object, debtor) @ t
    <= saleEstablished(debtor, object, creditor) @ t

rule consequence(debtor, object, creditor, t) =
    saleEstablished(debtor, object, creditor) @ t
    <= deliverable(debtor, object, creditor) @ t

experiment base {
    seeds { agreesPrice(alice, car, bob) @ tau, }
    query residual deliverable(alice, car, bob) @ tau
}

experiment delta {
    seeds { agreesObject(bob, car, alice) @ tau, }
    query residual deliverable(alice, car, bob) @ tau
}
