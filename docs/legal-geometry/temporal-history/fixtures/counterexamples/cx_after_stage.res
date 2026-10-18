module AfterStage

import "../formalization_r1.res"

// A dossier whose moment is after t0, a term in the orbit of t0.
rule anchorR1AfterT0() = I <= appliesR1() @ after t0

experiment dossierAfter {
    seeds {
        accompanies(ana, rex, host) @ after t0,
        propertyA(rex) @ after t0,
        normEffective(regDuty, enact1) @ after t0,
    }
    query residual obligation(regDuty, ana, register, rex, host, administrative, enact1) @ after t0
}
