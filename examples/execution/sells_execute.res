module SellsExecute

entity Thing
entity Person {
    assets : Set Thing
}

const juan : Person
const pedro : Person
const charlie : Person
const car : Thing
const tau : PropositionTime

seeded verb sells(subject : Person, object : Thing, recipient : Person)

effect verb give(subject : Person, object : Thing, recipient : Person) =>
    subject.assets -= object
    recipient.assets += object

derived verb owns(owner : Person, object : Thing)

rule sellsProgram(seller, object, buyer, t) =
    sells(seller, object, buyer) @ t
    <= give(seller, object, buyer) @ after t

rule ownershipFromTransfer(seller, object, buyer, t) =
    give(seller, object, buyer) @ t
    <= owns(buyer, object) @ t

// One sale: the car moves from juan to pedro.
experiment sale {
    seeds { sells(juan, car, pedro) @ tau, }
    state @ tau { juan.assets = {car}, pedro.assets = {}, }
    query residual give(juan, car, pedro) @ after tau
}

// A resale one step later: the second give reads the state the first wrote.
experiment resale {
    seeds { sells(juan, car, pedro) @ tau, sells(pedro, car, charlie) @ after tau, }
    state @ tau { juan.assets = {car}, pedro.assets = {}, charlie.assets = {} }
    query residual give(juan, car, pedro) @ after tau
}

// Two sales in opposite directions at the same time: juan.assets both loses and
// gains car, so the result depends on the order and is reported as a conflict.
experiment crossed {
    seeds { sells(juan, car, pedro) @ tau, sells(pedro, car, juan) @ tau, }
    state @ tau { juan.assets = {car}, pedro.assets = {car} }
    query residual give(juan, car, pedro) @ after tau
}

// Without a state the closure is the same as `run`; nothing is executed.
experiment stateless {
    seeds { sells(juan, car, pedro) @ tau, }
    query residual give(juan, car, pedro) @ after tau
}
