module CumulativeRightsSlotExperiment

import "./data.res"

const bike : Thing
seeded verb benefitBasis(person : Person)
derived verb entitledTo(person : Person, thing : Thing)

// Dos derechos acumulativos sobre cosas distintas. La coincidencia del
// beneficiario y del tiempo no los convierte en alternativas excluyentes.
rule carBenefit(person, t) =
    benefitBasis(person) @ t
    <= entitledTo(person, car) @ t

rule bikeBenefit(person, t) =
    benefitBasis(person) @ t
    <= entitledTo(person, bike) @ t

default CarBenefit =
    supernormal carBenefit

default BikeBenefit =
    supernormal bikeBenefit

experiment cumulativeRights {
    seeds { benefitBasis(juan) @ tau, }
    query residual entitledTo(juan, car) @ tau
}
