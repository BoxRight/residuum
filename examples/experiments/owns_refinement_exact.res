module OwnsRefinementExact

rule ownsProgram(owner, object, t) =
    owns(owner, object) @ t
    <= protected(owner, object) @ after t
