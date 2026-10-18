module HypCustody

import "../cast.res"

rule hCustody(s, c, o, b, t) =
    considered(s, c, o, b) @ t & custody(s, o) @ t
    <= obligation(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
