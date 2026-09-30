module Transfer

entity Thing

entity Person {
    assets : Set Thing
}

effect verb acquires(
    subject : Person,
    object  : Thing
) =>
    subject.assets += object

seeded verb delivers(
    subject   : Person,
    object    : Thing,
    recipient : Person
)

rule badDelivery(
    thing,
    person,
    t
) =
    delivers(thing, person, person) @ t
    ≤
    acquires(person, thing) @ after t
