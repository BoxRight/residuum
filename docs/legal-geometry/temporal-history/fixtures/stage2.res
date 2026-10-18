module Stage2

import "formalization_r1.res"

// T2 = O ∪ I_{t2}(R1): the same formalization anchored again, at a new moment.
// A legislative permission with a not-earlier legal date annuls the
// administrative prohibition by the LegalGeneral rule, from data alone.
rule anchorR1AtT2() = I <= appliesR1() @ t2

experiment dossier2 {
    seeds {
        accompanies(bea, fido, host) @ t2,
        ~propertyA(fido) @ t2,
        normEffective(dogBan, enact0) @ t2,
        permissionFiled(permitFido, bea, enterWith, fido, host, legislative, enact1) @ t2,
        higherAuthority(legislative, administrative) @ t2,
        notEarlier(enact1, enact0) @ t2,
    }
    query residual annulled(dogBan, permitFido, legislative, enact1) @ t2
}
