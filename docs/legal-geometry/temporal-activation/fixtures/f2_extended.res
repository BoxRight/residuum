module MonotoneExtended

import "f2_base.res"

// F2, theory T0 + G1. G1 is a class-level generalization over property A,
// written with LegalGeneral's own derogation route: a permission of higher
// authority and not earlier vigencia yields annulled(...). Nothing else.
rule exemptionA(owner, dog, venue, t) =
    accompanies(owner, dog, venue) @ t & propertyA(dog) @ t
    <= permission(exemptA, owner, enterWith, dog, venue, legislative, enact1) @ t

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
        Act(visitAna, ana, enterWith, rex, host, private, t1) @ t1,
        Act(visitBea, bea, enterWith, fido, host, private, t1) @ t1,
        higherAuthority(legislative, administrative) @ t1,
        notEarlier(enact1, enact0) @ t1,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t1
}
