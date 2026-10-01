module OwnsRefinementExtra

derived verb entitlement(owner : Person, object : Thing)

rule ownsProgram(owner, object, t) =
    owns(owner, object) @ t
    <= entitlement(owner, object) @ t

rule completeProtection(owner, object, t) =
    entitlement(owner, object) @ t
    <= protected(owner, object) @ after t

rule insurance(owner, object, t) =
    owns(owner, object) @ t
    <= insured(owner, object) @ after t
