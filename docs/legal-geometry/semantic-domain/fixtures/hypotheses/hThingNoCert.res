module HypThingNoCert

import "../cast.res"

rule hThingNoCert(s, c, o, b, t) =
    considered(s, c, o, b) @ t & thingTrait(o, animal) @ t & ~thingTrait(o, certifiedAssistance) @ t
    <= prohibition(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
