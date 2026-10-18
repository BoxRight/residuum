module SharedSpacesConflation

import "./cast.res"

// Counterexample for the projection: two positions that differ only in
// authority are legally distinct. Dropping authority would identify them;
// the projection is admissible only where authority is constant by
// construction, and the check must detect this module.

rule hAdministrative(s, c, o, b, t) =
    considered(s, c, o, b) @ t & thingTrait(o, animal) @ t
    <= prohibition(hypNorm, s, c, o, b, administrative, hypEnacted) @ t

rule hLegislative(s, c, o, b, t) =
    considered(s, c, o, b) @ t & thingTrait(o, animal) @ t
    <= prohibition(hypNorm, s, c, o, b, legislative, hypEnacted) @ t
