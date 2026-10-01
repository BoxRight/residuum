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

derived verb inconsistent(
    subject   : Person,
    object    : Thing,
    recipient : Person
)

rule evidentialConflict(giver, object, receiver, t) =
    delivers(giver, object, receiver) @ t
    &
    ~delivers(giver, object, receiver) @ t
    <=
    inconsistent(giver, object, receiver) @ after t
