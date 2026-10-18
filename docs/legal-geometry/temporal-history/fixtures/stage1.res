module Stage1

import "formalization_r1.res"

// T1 = O ∪ I_{t1}(R1). Participants ana, rex, bea and fido are shared with
// dossier 0; rex still has A, now fido has explicit not-A. The filed
// registration duty gives obligation a second, ontological justification.
rule anchorR1AtT1() = I <= appliesR1() @ t1

experiment dossier1 {
    seeds {
        accompanies(ana, rex, host) @ t1,
        propertyA(rex) @ t1,
        accompanies(bea, fido, host) @ t1,
        ~propertyA(fido) @ t1,
        normEffective(dogBan, enact0) @ t1,
        normEffective(regDuty, enact1) @ t1,
        Act(sale1, ana, transfer, rex, cal, private, saleDate1) @ t1,
        obligationFiled(regDuty, ana, register, rex, host, administrative, enact1) @ t1,
    }
    query residual obligation(regDuty, ana, register, rex, host, administrative, enact1) @ t1
}
