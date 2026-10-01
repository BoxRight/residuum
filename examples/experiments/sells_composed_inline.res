module SellsComposedInline

entity Thing
entity Conduct
entity Person

const juan : Person
const pedro : Person
const charlie : Person
const car : Thing
const molest : Conduct
const tau : PropositionTime
const sigma : PropositionTime

seeded verb sells(subject : Person, object : Thing, recipient : Person)
derived verb owns(owner : Person, object : Thing)

derived verb notDo(
    subject : Person,
    conduct : Conduct,
    indirectObject : Thing?,
    beneficiary : Person
)

rule sellsProgram(seller, object, buyer, t) =
    sells(seller, object, buyer) @ t
    <= owns(buyer, object) @ after t

rule protectionFromCharlie(owner, object, t) =
    owns(owner, object) @ t
    <= notDo(charlie, molest, object, owner) @ after t

experiment real {
    seeds { sells(juan, car, pedro) @ tau, }
    query residual owns(pedro, car) @ after tau
}

experiment absent {
    seeds {}
    query residual owns(pedro, car) @ after tau
}

experiment negativeOnly {
    seeds { ~sells(juan, car, pedro) @ tau, }
    query residual owns(pedro, car) @ after tau
}

experiment twoInstances {
    seeds {
        sells(juan, car, pedro) @ tau,
        sells(pedro, car, juan) @ sigma,
    }
    query residual owns(juan, car) @ after sigma
}
