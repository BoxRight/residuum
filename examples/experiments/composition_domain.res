module CompositionDomain

entity Person
entity Thing
entity Conduct

const juan : Person
const pedro : Person
const charlie : Person
const car : Thing
const molest : Conduct
const tau : PropositionTime
const sigma : PropositionTime

seeded verb sells(subject : Person, object : Thing, recipient : Person)
derived verb owns(owner : Person, object : Thing)
derived verb registered(owner : Person, object : Thing)
derived verb notDo(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person)
derived verb protected(owner : Person, object : Thing)

experiment seedDomain {
    seeds {
        sells(juan, car, pedro) @ tau,
        ~sells(juan, car, pedro) @ tau,
        sells(pedro, car, juan) @ sigma,
        ~sells(pedro, car, juan) @ sigma,
    }
    query residual owns(pedro, car) @ after tau
}

experiment knowledgeDomain {
    premises {
        owns(pedro, car) @ after tau,
        ~owns(pedro, car) @ after tau,
        owns(juan, car) @ after sigma,
        ~owns(juan, car) @ after sigma,
    }
    query fourFusion owns(pedro, car) @ after tau
}

experiment competingSource {
    seeds {
        sells(charlie, car, pedro) @ tau,
        ~sells(charlie, car, pedro) @ tau,
    }
    query residual owns(pedro, car) @ after tau
}
