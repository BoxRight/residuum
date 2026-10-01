module Residual

entity Person
entity Thing

seeded verb requests(subject : Person, object : Thing)
seeded verb available(object : Thing)
derived verb eligible(subject : Person, object : Thing)

rule eligibility(person, object, t) =
    requests(person, object) @ t
    <= available(object) @ t
    -o eligible(person, object) @ after t
