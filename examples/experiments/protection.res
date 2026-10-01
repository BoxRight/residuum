module Protection

const alice : Person
const bob : Person
const charlie : Person
const car : Thing
const tau : PropositionTime

seeded verb transfer(subject : Person, object : Thing, recipient : Person)

rule transferToOwnership(giver, object, receiver, t) =
    transfer(giver, object, receiver) @ t
    <= owns(receiver, object) @ after t

rule protection = ownsProgram(charlie, molest)

experiment base {
    seeds {}
    query residual owns(bob, car) @ after tau
}

experiment delta {
    seeds { transfer(alice, car, bob) @ tau, }
    query residual owns(bob, car) @ after tau
}
