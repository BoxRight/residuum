module History1

import "stage0.res"
import "stage1.res"

// H = O ∪ ⋃ I_tk(Rk): the stage modules contribute their formalizations and
// anchoring axioms; their experiments are not imported. The seeds are the
// disjoint union of the dossiers, in stage order.
experiment history {
    seeds {
        accompanies(ana, rex, host) @ t0,
        propertyA(rex) @ t0,
        accompanies(bea, fido, host) @ t0,
        normEffective(dogBan, enact0) @ t0,
        Act(visitAna0, ana, enterWith, rex, host, private, visitDate0) @ t0,
        accompanies(ana, rex, host) @ t1,
        propertyA(rex) @ t1,
        accompanies(bea, fido, host) @ t1,
        ~propertyA(fido) @ t1,
        normEffective(dogBan, enact0) @ t1,
        normEffective(regDuty, enact1) @ t1,
        Act(sale1, ana, transfer, rex, cal, private, saleDate1) @ t1,
        obligationFiled(regDuty, ana, register, rex, host, administrative, enact1) @ t1,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}
