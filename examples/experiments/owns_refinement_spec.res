module OwnsRefinementSpec

entity Person
entity Thing

derived verb owns(owner : Person, object : Thing)
derived verb protected(owner : Person, object : Thing)
derived verb insured(owner : Person, object : Thing)

rule protectionGuarantee(owner, object, t) =
    owns(owner, object) @ t
    <= protected(owner, object) @ after t
