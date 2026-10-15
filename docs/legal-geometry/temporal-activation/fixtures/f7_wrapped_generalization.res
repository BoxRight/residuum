module WrappedGeneralization

import "f2_base.res"

// F7. T0 of f2_base plus two declarations only: a Program whose member is the
// existing rule generalDogBan, associated to a derived verb, and the rule that
// derives that verb. prepare_programs then treats generalDogBan as a definition:
// it no longer fires as an ordinary rule, only under an activating instance.
program OrdinaryEntry(owner : Person, dog : Thing, venue : Person, t : PropositionTime) {
    use generalDogBan { owner = owner, dog = dog, venue = venue, t = t, }
}

derived verb ordinaryEntry(owner : Person, dog : Thing, venue : Person) = program OrdinaryEntry

rule detectOrdinary(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & ~propertyA(dog) @ t
    <= ordinaryEntry(owner, dog, venue) @ t

experiment history {
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
        Act(visitAna, ana, enterWith, rex, host, private, t1) @ t1,
        Act(visitBea, bea, enterWith, fido, host, private, t1) @ t1,
    }
    query residual prohibition(dogBan, bea, enterWith, fido, host, administrative, enact0) @ t1
}
