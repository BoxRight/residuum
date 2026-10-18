module AfterHistory

import "../stage0.res"
import "cx_after_stage.res"

// notDo(ana, enterWith, rex, host) @ after t0, produced in dossier 0, has the
// moment of dossierAfter: the two orbits intersect.
experiment history {
    seeds {
        accompanies(ana, rex, host) @ t0,
        propertyA(rex) @ t0,
        accompanies(bea, fido, host) @ t0,
        normEffective(dogBan, enact0) @ t0,
        Act(visitAna0, ana, enterWith, rex, host, private, visitDate0) @ t0,
        accompanies(ana, rex, host) @ after t0,
        propertyA(rex) @ after t0,
        normEffective(regDuty, enact1) @ after t0,
    }
    query residual notDo(ana, enterWith, rex, host) @ after t0
}
