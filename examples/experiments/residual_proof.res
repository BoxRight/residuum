module ResidualProof

entity Thing
entity Person {
    assets : Set Thing
}

const alice : Person
const bob : Person
const car : Thing
const tau : PropositionTime

seeded verb agreesPrice(subject : Person, object : Thing, recipient : Person)
seeded verb agreesObject(subject : Person, object : Thing, recipient : Person)

effect verb give(subject : Person, object : Thing, recipient : Person) =>
    subject.assets -= object
    recipient.assets += object

rule sales(debtor, object, creditor, t) =
    agreesPrice(debtor, object, creditor) @ t
    & agreesObject(creditor, object, debtor) @ t
    <= give(debtor, object, creditor) @ after t

experiment empty {
    seeds {}
    query residual give(alice, car, bob) @ after tau
}

experiment priceOnly {
    seeds {
        agreesPrice(alice, car, bob) @ tau,
    }
    query residual give(alice, car, bob) @ after tau
}

experiment both {
    seeds {
        agreesPrice(alice, car, bob) @ tau,
        agreesObject(bob, car, alice) @ tau,
    }
    query residual give(alice, car, bob) @ after tau
}
