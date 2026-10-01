module OwnsRefinementDomain

const alice : Person
const bob : Person
const car : Thing
const house : Thing
const tau : PropositionTime
const sigma : PropositionTime

seeded verb transfer(subject : Person, object : Thing, recipient : Person)

rule probeOwnership(giver, object, receiver, t) =
    transfer(giver, object, receiver) @ t
    <= owns(receiver, object) @ t

experiment probes {
    seeds {
        transfer(alice, car, alice) @ tau,
        transfer(alice, house, alice) @ tau,
        transfer(alice, car, bob) @ tau,
        transfer(alice, house, bob) @ tau,
        transfer(alice, car, alice) @ sigma,
        transfer(alice, house, alice) @ sigma,
        transfer(alice, car, bob) @ sigma,
        transfer(alice, house, bob) @ sigma,
    }
    query residual owns(bob, car) @ tau
}
