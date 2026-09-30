module Transfer

entity Asset

entity Person {
    assets : Set Asset
}

effect verb transfers(
    subject   : Person,
    object    : Asset,
    recipient : Person
) =>
    subject.assets -= object
    recipient.assets += object

seeded verb delivers(
    subject   : Person,
    object    : Asset,
    recipient : Person
)

rule transfer(
    giver,
    asset,
    receiver,
    t
) =
    delivers(giver, asset, receiver) @ t
    ≤
    transfers(giver, asset, receiver) @ after t

default transferNormally =
    supernormal transfer
