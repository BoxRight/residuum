module PartialNormalSaleCompatibility

import "../legal_monotonic.res"

const juan : Person
const pedro : Person
const car : Thing
const tau : PropositionTime
const effectiveDate : PropositionTime
const duty : Fact

seeded verb saleBasis(seller : Person, object : Thing, buyer : Person)
derived verb normalChosen(seller : Person, object : Thing, buyer : Person)

// Dos relaciones del cuerpo compuesto propuesto, separadas en el AST actual.
// t va primero: la aplicación existente consume un prefijo de parámetros.
rule normalEntry(t, seller, object, buyer) =
    saleBasis(seller, object, buyer) @ t
    <= normalChosen(seller, object, buyer) @ after t

rule normalDuty(t, seller, object, buyer) =
    normalChosen(seller, object, buyer) @ after t
    <= obligation(duty, seller, transfer, object, buyer, private, effectiveDate) @ after t

// La misma sustitución se propaga independientemente a las dos relaciones.
rule entryForJuan = normalEntry(tau, juan)
rule dutyForJuan = normalDuty(tau, juan)

// El resultado parcial puede aplicarse de nuevo.
rule completedEntry = entryForJuan(car, pedro)
rule completedDuty = dutyForJuan(car, pedro)
rule directEntry = normalEntry(tau, juan, car, pedro)
rule directDuty = normalDuty(tau, juan, car, pedro)

// Consumidor observable; las reglas siguen separadas, no son un paquete nativo.
experiment partialNormal {
    seeds { saleBasis(juan, car, pedro) @ tau, }
    query residual obligation(duty, juan, transfer, car, pedro, private, effectiveDate) @ after tau
}
