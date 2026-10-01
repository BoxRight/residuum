module OwnsRefinementMissing

rule ownsProgram(owner, object, t) =
    owns(owner, object) @ t
    <= insured(owner, object) @ after t
