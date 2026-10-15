module WrappedWithVigencia

import "f2_base.res"

// F8. As F7, with a second activating route by legal time: before the revision
// date rev1 the generalization applies to every dog. Order between dates is
// explicit positive data (precedes), as notEarlier is in LegalGeneral; it is not
// computed from after and not inferred from the absence of evidence.
const rev1 : PropositionTime

seeded verb precedes(earlier : PropositionTime, later : PropositionTime)

program OrdinaryEntry(owner : Person, dog : Thing, venue : Person, t : PropositionTime) {
    use generalDogBan { owner = owner, dog = dog, venue = venue, t = t, }
}

derived verb ordinaryEntry(owner : Person, dog : Thing, venue : Person) = program OrdinaryEntry

rule entryBeforeRevision(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & precedes(t, rev1) @ t
    <= ordinaryEntry(owner, dog, venue) @ t

rule entryWithoutA(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & ~propertyA(dog) @ t
    <= ordinaryEntry(owner, dog, venue) @ t

experiment history {
    seeds {
        accompanies(ana, rex, host) @ t0,
        precedes(t0, rev1) @ t0,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}

// Without the order datum the historical dossier is not covered.
experiment historyWithoutOrder {
    seeds {
        accompanies(ana, rex, host) @ t0,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}

experiment twoDogs {
    seeds {
        accompanies(ana, rex, host) @ t1,
        propertyA(rex) @ t1,
        accompanies(bea, fido, host) @ t1,
        ~propertyA(fido) @ t1,
    }
    query residual prohibition(dogBan, bea, enterWith, fido, host, administrative, enact0) @ t1
}

// A dog with A before the revision: still prohibited (legal-time vigencia).
experiment aBeforeRevision {
    seeds {
        accompanies(ana, rex, host) @ t0,
        propertyA(rex) @ t0,
        precedes(t0, rev1) @ t0,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}
