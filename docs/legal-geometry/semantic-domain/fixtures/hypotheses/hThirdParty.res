module HypThirdParty

import "../cast.res"

rule hThirdParty(s, c, o, b, x, t) =
    considered(s, c, o, b) @ t & bond(x, b, presentWith) @ t & sensitiveTo(x, animal) @ t
    <= prohibition(hypNorm, s, c, o, b, administrative, hypEnacted) @ t
