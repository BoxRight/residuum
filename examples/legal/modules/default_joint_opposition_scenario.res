module JointDefaultOppositionExperiment

import "./legal_monotonic.res"

// Dos lambdas independientes sobre la misma evidencia. No hay prioridad,
// excepción, bloqueo ni regla que derive inconsistent automáticamente.
rule positiveDuty(id, subject, conduct, object, beneficiary, authority, effective, t) =
    right(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t

rule negativeDuty(id, subject, conduct, object, beneficiary, authority, effective, t) =
    right(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= ~obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t

default PositiveDuty =
    supernormal positiveDuty

default NegativeDuty =
    supernormal negativeDuty

const juan : Person
const pedro : Person
const car : Thing
const duty : Fact
const effectiveDate : PropositionTime
const tau : PropositionTime

experiment jointCandidates {
    seeds {
        right(duty, juan, transfer, car, pedro, private, effectiveDate) @ tau,
    }
    query residual obligation(duty, juan, transfer, car, pedro, private, effectiveDate) @ tau
}
