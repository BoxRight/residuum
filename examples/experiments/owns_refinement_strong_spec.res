module OwnsRefinementStrongSpec

entity Person
entity Thing

derived verb owns(owner : Person, object : Thing)
derived verb protected(owner : Person, object : Thing)
derived verb insured(owner : Person, object : Thing)

rule protectionGuarantee(owner, object, t) =
    owns(owner, object) @ t
    <= protected(owner, object) @ after t

rule insuranceGuarantee(owner, object, t) =
    owns(owner, object) @ t
    <= insured(owner, object) @ after t
