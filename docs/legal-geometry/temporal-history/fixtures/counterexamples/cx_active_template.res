module ActiveTemplate

import "../vocabulary.res"

// Alternative B without capture (the F6 premise): the template looseBan is an
// ordinary rule, so specializing it at t0 does not confine it to t0.
rule looseBan(t, owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t & normEffective(dogBan, e) @ t
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, e) @ t
rule looseBanAtT0 = looseBan(t0)

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
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t1
}
