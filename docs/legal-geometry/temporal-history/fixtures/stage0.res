module Stage0

import "formalization_r0.res"

// T0 = O ∪ I_{t0}(R0). The anchoring I_{t0} is one axiom: it produces the
// activation instance appliesR0() @ t0, which binds the program parameter t.
rule anchorR0AtT0() = I <= appliesR0() @ t0

experiment dossier0 {
    seeds {
        accompanies(ana, rex, host) @ t0,
        propertyA(rex) @ t0,
        accompanies(bea, fido, host) @ t0,
        normEffective(dogBan, enact0) @ t0,
        Act(visitAna0, ana, enterWith, rex, host, private, visitDate0) @ t0,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}
