module OwnsProgram

entity Person
entity Thing
entity Conduct

const alice : Person
const bob : Person
const charlie : Person
const car : Thing
const molest : Conduct
const tau : PropositionTime

seeded verb transfer(
    subject   : Person,
    object    : Thing,
    recipient : Person
)

derived verb owns(
    subject : Person,
    object  : Thing
)

derived verb notDo(
    subject        : Person,
    conduct        : Conduct,
    indirectObject : Thing?,
    beneficiary    : Person
)

rule transferToOwnership(giver, object, receiver, t) =
    transfer(giver, object, receiver) @ t
    <= owns(receiver, object) @ after t

rule ownsProgram(subject, conduct, owner, object, t) =
    owns(owner, object) @ t
    <= notDo(subject, conduct, object, owner) @ after t

rule protectionFromCharlie = ownsProgram(charlie, molest)

experiment base {
    seeds {}
    query residual owns(bob, car) @ after tau
}

experiment delta {
    seeds { transfer(alice, car, bob) @ tau, }
    query residual owns(bob, car) @ after tau
}
