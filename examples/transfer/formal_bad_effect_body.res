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
    subject.assets -= recipient

rule transfer(
    giver,
    asset,
    receiver,
    t
) =
    transfers(giver, asset, receiver) @ t
    ≤
    transfers(giver, asset, receiver) @ after t
