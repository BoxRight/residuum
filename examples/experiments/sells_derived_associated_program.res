module SellsDerivedAssociatedProgram

entity Thing
entity Conduct
entity Person { assets : Set Thing }

const juan : Person
const pedro : Person
const charlie : Person
const car : Thing
const molest : Conduct
const tau : PropositionTime

seeded verb confirmedSale(subject : Person, object : Thing, recipient : Person)
derived verb sells(subject : Person, object : Thing, recipient : Person) =
    program sellsProgram

effect verb give(subject : Person, object : Thing, recipient : Person) =>
    subject.assets -= object
    recipient.assets += object

derived verb owns(owner : Person, object : Thing)
derived verb notDo(subject : Person, conduct : Conduct, indirectObject : Thing?, beneficiary : Person)

rule confirm(seller, object, buyer, t) =
    confirmedSale(seller, object, buyer) @ t
    <= sells(seller, object, buyer) @ t

rule sellsProgram(seller, object, buyer, t) =
    sells(seller, object, buyer) @ t
    <= give(seller, object, buyer) @ after t

rule juanSale = sellsProgram(juan)

rule ownershipFromTransfer(seller, object, buyer, t) =
    give(seller, object, buyer) @ t
    <= owns(buyer, object) @ t

rule ownsProgram(thirdParty, conduct, owner, object, t) =
    owns(owner, object) @ t
    <= notDo(thirdParty, conduct, object, owner) @ after t

rule protectionFromCharlie = ownsProgram(charlie, molest)

experiment real {
    seeds { confirmedSale(juan, car, pedro) @ tau, }
    query residual give(juan, car, pedro) @ after tau
}

experiment absent {
    seeds {}
    query residual give(juan, car, pedro) @ after tau
}
