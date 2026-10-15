module ConstantTime

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// F6. Rules whose proposition time is a constant, written directly or obtained
// by rule application (beta reduction of the time parameter).

const enterWith : Conduct
const ana : Person
const host : Person
const rex : Thing
const pinnedBan : Fact
const templateBan : Fact
const enact0 : PropositionTime
const t0 : PropositionTime
const t1 : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
derived verb pinnedProbe(owner : Person, dog : Thing)

// Written with the constant t0: fires only for evidence indexed at t0.
rule pinnedAtT0(owner, dog, venue) =
    accompanies(owner, dog, venue) @ t0
    <= pinnedProbe(owner, dog) @ t0

// A template whose first parameter is the time, and its specialization at t0.
rule banTemplate(t, owner, dog, venue) =
    accompanies(owner, dog, venue) @ t
    <= prohibition(templateBan, owner, enterWith, dog, venue, administrative, enact0) @ t

rule banAtT0 = banTemplate(t0)

experiment twoTimes {
    seeds {
        accompanies(ana, rex, host) @ t0,
        accompanies(ana, rex, host) @ t1,
    }
    query residual pinnedProbe(ana, rex) @ t1
}

experiment specializedAtT0 {
    seeds {
        accompanies(ana, rex, host) @ t0,
        accompanies(ana, rex, host) @ t1,
    }
    query residual prohibition(templateBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}
