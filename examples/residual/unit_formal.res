module Residual

entity Person
entity Thing

seeded verb requests(subject : Person, object : Thing)
seeded verb available(object : Thing)
derived verb eligible(subject : Person, object : Thing)

rule emptyContext(person, object, t) =
    I
    <= available(object) @ t
    -o eligible(person, object) @ after t

rule leftUnit(person, object, t) =
    I & requests(person, object) @ t
    <= available(object) @ t
    -o eligible(person, object) @ after t

rule rightUnit(person, object, t) =
    requests(person, object) @ t & I
    <= available(object) @ t
    -o eligible(person, object) @ after t

rule noRequirement(person, object, t) =
    requests(person, object) @ t
    <= I
    -o eligible(person, object) @ after t

rule unitHorn(person, object, t) =
    I & requests(person, object) @ t
    <= eligible(person, object) @ after t
