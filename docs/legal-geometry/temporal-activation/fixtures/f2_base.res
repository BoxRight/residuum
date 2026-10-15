module MonotoneBase

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// F2, theory T0: the general prohibition G0 only. f2_extended.res imports this
// module and adds a class-level generalization G1; both run the same seeds.

const enterWith : Conduct
const ana : Person
const bea : Person
const host : Person
const rex : Thing
const fido : Thing
const dogBan : Fact
const exemptA : Fact
const visitAna : Fact
const visitBea : Fact
const enact0 : PropositionTime
const enact1 : PropositionTime
const t0 : PropositionTime
const t1 : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
// Property A of the control case; its legal relevance is not presupposed.
seeded verb propertyA(dog : Thing)

rule generalDogBan(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, enact0) @ t

experiment history {
    seeds {
        accompanies(ana, rex, host) @ t0,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}

// Two dogs in one dossier at t1: rex has A, fido does not.
experiment twoDogs {
    seeds {
        accompanies(ana, rex, host) @ t1,
        propertyA(rex) @ t1,
        accompanies(bea, fido, host) @ t1,
        Act(visitAna, ana, enterWith, rex, host, private, t1) @ t1,
        Act(visitBea, bea, enterWith, fido, host, private, t1) @ t1,
        higherAuthority(legislative, administrative) @ t1,
        notEarlier(enact1, enact0) @ t1,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t1
}
