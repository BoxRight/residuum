module SpecializedHistory

import "../generalizations.res"

// Alternative B. The templates of Generalizations are passive definitions
// (captured by the never-activated Program Templates). Native partial
// application fixes their first parameter, the proposition time; each
// specialization is a new ordinary rule.
rule b0OrdinaryBan = ordinaryBan(t0)
rule b1RefinedBan = refinedBan(t1)
rule b1RegistrationDuty = registrationDuty(t1)
rule b2RefinedBan = refinedBan(t2)
rule b2RegistrationDuty = registrationDuty(t2)

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
        accompanies(bea, fido, host) @ t2,
        ~propertyA(fido) @ t2,
        normEffective(dogBan, enact0) @ t2,
        permissionFiled(permitFido, bea, enterWith, fido, host, legislative, enact1) @ t2,
        higherAuthority(legislative, administrative) @ t2,
        notEarlier(enact1, enact0) @ t2,
    }
    query residual prohibition(dogBan, ana, enterWith, rex, host, administrative, enact0) @ t0
}
