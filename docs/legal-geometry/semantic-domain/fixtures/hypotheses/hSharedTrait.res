module HypSharedTrait

import "../cast.res"

rule hSharedTrait(s, c, o, b, phi, t) =
    considered(s, c, o, b) @ t & thingTrait(o, phi) @ t & sensitiveTo(b, phi) @ t
    <= prohibition(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
