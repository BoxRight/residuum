module BetaScenario

import "./associated_program.res"

const juan : Person
const pedro : Person
const car : Thing
const tau : PropositionTime

// β-reduction en el consumidor de una lambda importada.
rule juanSale = givingProgram(juan)
seeded verb sellsFromJuan(object : Thing, recipient : Person) = program juanSale

experiment specializedSale {
    seeds { sellsFromJuan(car, pedro) @ tau, }
    query residual give(juan, car, pedro) @ after tau
}
