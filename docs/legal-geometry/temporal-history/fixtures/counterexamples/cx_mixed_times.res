module MixedTimes

import "../stage0.res"

// A formalization whose member rule has a second, unbound time variable u.
// Activation binds t only; u matches any moment, so the vigencia datum of
// dossier 0 (t0) licenses a prohibition in dossier 1 (t1).
rule mixedBan(t, u, owner, dog, venue, e) =
    accompanies(owner, dog, venue) @ t & normEffective(dogBan, e) @ u
    <= prohibition(dogBan, owner, enterWith, dog, venue, administrative, e) @ t

program RMixed(t : PropositionTime) {
    use mixedBan { t = t, }
}

derived verb appliesRMixed() = program RMixed

rule anchorRMixedAtT1() = I <= appliesRMixed() @ t1

// Isolated: dossier 1 has no vigencia datum for dogBan, so no prohibition.
experiment isolated {
    seeds {
        accompanies(bea, fido, host) @ t1,
    }
    query residual prohibition(dogBan, bea, enterWith, fido, host, administrative, enact0) @ t1
}

// History with dossier 0: the datum at t0 crosses into t1.
experiment history {
    seeds {
        accompanies(ana, rex, host) @ t0,
        propertyA(rex) @ t0,
        accompanies(bea, fido, host) @ t0,
        normEffective(dogBan, enact0) @ t0,
        Act(visitAna0, ana, enterWith, rex, host, private, visitDate0) @ t0,
        accompanies(bea, fido, host) @ t1,
    }
    query residual prohibition(dogBan, bea, enterWith, fido, host, administrative, enact0) @ t1
}
