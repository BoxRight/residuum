module DoubleAnchor

import "../stage0.res"
import "../formalization_r1.res"

// R1 anchored at t0 as well: two formalizations evaluate the same dossier
// moment inside one theory, the premise of the F4 notDo finding.
rule anchorR1AtT0() = I <= appliesR1() @ t0

experiment history {
    seeds {
        accompanies(ana, rex, host) @ t0,
        propertyA(rex) @ t0,
        accompanies(bea, fido, host) @ t0,
        normEffective(dogBan, enact0) @ t0,
        Act(visitAna0, ana, enterWith, rex, host, private, visitDate0) @ t0,
    }
    query residual notDo(ana, enterWith, rex, host) @ after t0
}
