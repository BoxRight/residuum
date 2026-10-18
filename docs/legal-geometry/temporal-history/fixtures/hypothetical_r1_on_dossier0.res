module HypotheticalR1OnDossier0

import "formalization_r1.res"

// Hypothetical, isolated: R1 anchored at the moment of dossier 0, with the
// seeds of dossier 0. It is not part of any history module.
rule anchorR1AtT0() = I <= appliesR1() @ t0

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
