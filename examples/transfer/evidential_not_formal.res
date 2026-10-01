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

effect verb transfers(
    subject   : Person,
    object    : Thing,
    recipient : Person
) =>
    subject.assets   -= object
    recipient.assets += object

effect verb acquires(
    subject : Person,
    object  : Thing
) =>
    subject.assets += object

rule transfer(giver, object, receiver, t) =
    delivers(giver, object, receiver) @ t
    <=
    transfers(giver, object, receiver) @ after t

rule deniedDelivery(giver, object, receiver, t) =
    ~delivers(giver, object, receiver) @ t
    <=
    ~transfers(giver, object, receiver) @ after t

rule deniedAcquisition(giver, object, receiver, t) =
    ~transfers(giver, object, receiver) @ t
    <=
    ~acquires(receiver, object) @ after t
