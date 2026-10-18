module PinnedHistory

import "../vocabulary.res"

// Alternative A (reference behaviour). Each formalization is rewritten by hand
// with the dossier moment as a constant; nothing is a template or a Program.
// R0 at t0
rule a0OrdinaryBan(owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t0 & normEffective(dogBan, e) @ t0
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, e) @ t0
// R1 at t1
rule a1RefinedBan(owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t1 & ~propertyA(dog) @ t1 & normEffective(dogBan, e) @ t1
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, e) @ t1
rule a1RegistrationDuty(owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t1 & propertyA(dog) @ t1 & normEffective(regDuty, e) @ t1
    <= obligation(regDuty, owner, register, dog, venue, administrative, e) @ t1
// R1 again at t2: the same formalization has to be copied with new names.
rule a2RefinedBan(owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t2 & ~propertyA(dog) @ t2 & normEffective(dogBan, e) @ t2
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, e) @ t2
rule a2RegistrationDuty(owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t2 & propertyA(dog) @ t2 & normEffective(regDuty, e) @ t2
    <= obligation(regDuty, owner, register, dog, venue, administrative, e) @ t2

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
