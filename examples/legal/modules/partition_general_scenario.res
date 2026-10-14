module LegalGeneralPartitionScenario

import "./legal_general.res"

// Una conducta de uso, distinta de transfer, no atribuye la cosa por cumplimiento.
const juan : Person
const pedro : Person
const car : Thing
const useDuty : Fact
const useAct : Fact
const oldDate : PropositionTime
const newDate : PropositionTime
const tau : PropositionTime

experiment negativeUse {
    seeds {
        obligationFiled(useDuty, juan, use, car, pedro, private, oldDate) @ tau,
        ~Act(useAct, juan, use, car, pedro, private, newDate) @ tau,
    }
    query residual patrimonialLiability(juan, car, useDuty, private, oldDate) @ tau
}

experiment fulfilledUse {
    seeds {
        obligationFiled(useDuty, juan, use, car, pedro, private, oldDate) @ tau,
        Act(useAct, juan, use, car, pedro, private, newDate) @ tau,
    }
    query residual compliance(useDuty, useAct, juan, car, pedro, private, oldDate) @ tau
}
