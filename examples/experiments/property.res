module Property

entity Thing
entity Person
entity Conduct

const molest : Conduct

derived verb owns(owner : Person, object : Thing)

derived verb notDo(
    subject        : Person,
    conduct        : Conduct,
    indirectObject : Thing?,
    beneficiary    : Person
)

rule ownsProgram(thirdParty, conduct, owner, object, t) =
    owns(owner, object) @ t
    <= notDo(thirdParty, conduct, object, owner) @ after t
