module OwnsSpecification

entity Thing
entity Person
entity Conduct

const charlie : Person
const molest : Conduct

derived verb owns(owner : Person, object : Thing)

derived verb notDo(
    subject        : Person,
    conduct        : Conduct,
    indirectObject : Thing?,
    beneficiary    : Person
)

rule ownsGuarantee(owner, object, t) =
    owns(owner, object) @ t
    <= notDo(charlie, molest, object, owner) @ after t
