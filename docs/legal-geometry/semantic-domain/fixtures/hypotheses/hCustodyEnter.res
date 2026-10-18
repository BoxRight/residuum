module HypCustodyEnter

import "../cast.res"

rule hCustodyEnter(s, o, b, t) =
    considered(s, enterWith, o, b) @ t & custody(s, o) @ t
    <= obligation(hypNorm, s, enterWith, o, b, administrative, hypEnacted) @ t
