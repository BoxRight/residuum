module EpochT0

import "../../../../examples/experiments/legal_geometry/ontology_reference/legal_general.res"

// F4, theory T0. Learning epochs e0..e3 are PropositionTime constants used as
// ARGUMENTS; legal dossier times sA..sF are proposition indices. They are kept
// distinct. A dossier is evaluated under an epoch by an explicit seed
// evaluatedAt(e). Every generalization is written, from T0 on, with the same
// guard unrevisedThrough(itself, e): the theory states for which epochs it
// keeps that generalization. A later theory only ADDS rules and certificates.
// The epoch is carried in the effective argument of the consequence, so the
// assessments under different epochs are distinct logical facts.

const enterWith : Conduct
const ana : Person
const bea : Person
const cal : Person
const dan : Person
const eva : Person
const host : Person
const rex : Thing
const fido : Thing
const kim : Thing
const max : Thing
const pip : Thing
const g0 : Fact
const g0a : Fact
const g3 : Fact
const e0 : PropositionTime
const e1 : PropositionTime
const e2 : PropositionTime
const e3 : PropositionTime
const sA : PropositionTime
const sB : PropositionTime
const sC : PropositionTime
const sD : PropositionTime
const sE : PropositionTime
const sF : PropositionTime

seeded verb accompanies(owner : Person, dog : Thing, venue : Person)
seeded verb propertyA(dog : Thing)
seeded verb allergicPresent(person : Person, venue : Person)
seeded verb serviceDog(dog : Thing, owner : Person)
seeded verb evaluatedAt(epoch : PropositionTime)
derived verb unrevisedThrough(generalization : Fact, epoch : PropositionTime)

// G0: the general prohibition, with the epoch guard.
rule g0Ban(owner, dog, venue, e, t) =
    accompanies(owner, dog, venue) @ t & evaluatedAt(e) @ t & unrevisedThrough(g0, e) @ t
    <= prohibition(g0, owner, enterWith, dog, venue, administrative, e) @ t

rule certG0E0(t) = evaluatedAt(e0) @ t <= unrevisedThrough(g0, e0) @ t

// Shared panel: identical seeds in every theory T0..T3.
experiment s1Dossier {
    seeds {
        accompanies(ana, rex, host) @ sA,
        propertyA(rex) @ sA,
        evaluatedAt(e0) @ sA,
        evaluatedAt(e1) @ sA,
        evaluatedAt(e3) @ sA,
    }
    query residual prohibition(g0, ana, enterWith, rex, host, administrative, e0) @ sA
}

experiment plainDog {
    seeds {
        accompanies(bea, fido, host) @ sB,
        ~propertyA(fido) @ sB,
        evaluatedAt(e0) @ sB,
        evaluatedAt(e1) @ sB,
        evaluatedAt(e2) @ sB,
        evaluatedAt(e3) @ sB,
    }
    query residual prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sB
}

experiment unknownA {
    seeds {
        accompanies(bea, fido, host) @ sC,
        evaluatedAt(e0) @ sC,
        evaluatedAt(e1) @ sC,
    }
    query residual prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sC
}

experiment s2Dossier {
    seeds {
        accompanies(cal, kim, host) @ sD,
        ~propertyA(kim) @ sD,
        allergicPresent(dan, host) @ sD,
        evaluatedAt(e2) @ sD,
        evaluatedAt(e3) @ sD,
    }
    query residual right(g3, cal, enterWith, kim, host, legislative, e3) @ sD
}

experiment s3Dossier {
    seeds {
        accompanies(eva, max, host) @ sE,
        ~propertyA(max) @ sE,
        serviceDog(max, eva) @ sE,
        evaluatedAt(e2) @ sE,
        evaluatedAt(e3) @ sE,
    }
    query residual right(g3, eva, enterWith, max, host, legislative, e3) @ sE
}

experiment s4Question {
    seeds {
        accompanies(eva, pip, host) @ sF,
        ~propertyA(pip) @ sF,
        serviceDog(pip, eva) @ sF,
        allergicPresent(dan, host) @ sF,
        evaluatedAt(e3) @ sF,
    }
    query residual right(g3, eva, enterWith, pip, host, legislative, e3) @ sF
}

experiment twoDogs {
    seeds {
        accompanies(ana, rex, host) @ sA,
        propertyA(rex) @ sA,
        accompanies(bea, fido, host) @ sA,
        ~propertyA(fido) @ sA,
        evaluatedAt(e1) @ sA,
    }
    query residual prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sA
}

