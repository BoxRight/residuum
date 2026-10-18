module HypThing

import "../cast.res"

rule hThing(s, c, o, b, t) =
    considered(s, c, o, b) @ t & thingTrait(o, animal) @ t
    <= prohibition(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
