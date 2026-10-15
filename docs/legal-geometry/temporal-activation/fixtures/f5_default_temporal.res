module DefaultTemporal

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// F5. The existing default mechanism with temporal evidence. G0 is put under a
// supernormal default blocked by property A at the same proposition index.
// Question: does a blocker at t1 for one dog leave G0's consequences at t0,
// and for another dog at t1?

const enterWith : Conduct
const ana : Person
const bea : Person
const host : Person
const rex : Thing
const fido : Thing
const dogBan : Fact
const enact0 : PropositionTime
const t0 : PropositionTime
const t1 : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
seeded verb propertyA(dog : Thing)

rule generalDogBan(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, enact0) @ t

default GeneralDogBan =
    supernormal generalDogBan
    blocked when
        propertyA(dog) @ t

experiment blockerLater {
    seeds {
        accompanies(ana, rex, host) @ t0,
        accompanies(ana, rex, host) @ t1,
        propertyA(rex) @ t1,
        accompanies(bea, fido, host) @ t1,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}

experiment noBlocker {
    seeds {
        accompanies(ana, rex, host) @ t0,
        accompanies(ana, rex, host) @ t1,
        accompanies(bea, fido, host) @ t1,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}
