module Transfer

entity Thing

entity Person {
    assets : Set Thing
}

seeded verb delivers(
    subject   : Person,
    object    : Thing,
    recipient : Person
)

effect verb acquires(
    subject : Person,
    object  : Thing
) =>
    subject.assets += object

rule missingDeliveryTime(
    giver,
    object,
    receiver,
    t
) =
    delivers(giver, object, receiver)
    ≤
    acquires(receiver, object) @ after t
