module ProgramActivation

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// F3. Positive selection by Verb.program: a generalization packaged as a
// Program is activated only by a known instance of its verb, with the
// instance's arguments and proposition time as bindings.

const enterWith : Conduct
const ana : Person
const bea : Person
const host : Person
const rex : Thing
const fido : Thing
const dogBan : Fact
const serviceEntry : Fact
const enact0 : PropositionTime
const enact3 : PropositionTime
const t0 : PropositionTime
const t3 : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
// Property C of the control case.
seeded verb serviceDog(dog : Thing, owner : Person)

rule generalDogBan(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, enact0) @ t

// The generalization G3 as a passive definition: a member rule and a Program.
rule serviceRight(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t
    <= right(serviceEntry, owner, enterWith, dog, venue, legislative, enact3) @ t

program ServiceRegime(owner : Person, dog : Thing, venue : Person, t : PropositionTime) {
    use serviceRight { owner = owner, dog = dog, venue = venue, t = t, }
}

// Its activation is a relational property of the dossier, derived by a rule.
derived verb serviceContext(owner : Person, dog : Thing, venue : Person) = program ServiceRegime

rule detectService(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & serviceDog(dog, owner) @ t
    <= serviceContext(owner, dog, venue) @ t

// rex is a service dog at t3, fido is not; rex at t0 is not marked.
experiment selection {
    seeds {
        accompanies(ana, rex, host) @ t0,
        accompanies(ana, rex, host) @ t3,
        serviceDog(rex, ana) @ t3,
        accompanies(bea, fido, host) @ t3,
    }
    query residual right(serviceEntry, ana, enterWith, rex, host, legislative, enact3) @ t3
}

// Control: without any activating instance the definition stays passive.
experiment passive {
    seeds {
        accompanies(ana, rex, host) @ t3,
        accompanies(bea, fido, host) @ t3,
    }
    query residual right(serviceEntry, ana, enterWith, rex, host, legislative, enact3) @ t3
}
