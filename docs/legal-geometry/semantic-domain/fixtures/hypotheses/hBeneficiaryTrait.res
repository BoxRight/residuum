module HypBeneficiaryTrait

import "../cast.res"

rule hBeneficiaryTrait(s, c, o, b, t) =
    considered(s, c, o, b) @ t & personTrait(b, minor) @ t
    <= permission(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
