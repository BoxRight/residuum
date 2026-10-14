module TypedExplanatoryCases

import "./ontology_reference/legal_general.res"

// Synthetic alternative dossiers, not real administrative instruments.
// Same people, performance, object, Act, validity and proposition time.
// No new legal rules or purposes. External questions are stored separately.
const ana : Person
const instituto : Person
const informeAnual : Thing
const presentarInforme : Conduct
const instrumentoInforme : Fact
const presentacionInforme : Fact
const vigenciaInforme : PropositionTime
const observacionInforme : PropositionTime

experiment owedAndPerformed {
    seeds {
        obligationFiled(instrumentoInforme, ana, presentarInforme, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme,
        Act(presentacionInforme, ana, presentarInforme, informeAnual, instituto, private, vigenciaInforme) @ observacionInforme,
    }
    query residual compliance(instrumentoInforme, presentacionInforme, ana, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme
}

experiment permittedAndPerformed {
    seeds {
        permissionFiled(instrumentoInforme, ana, presentarInforme, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme,
        Act(presentacionInforme, ana, presentarInforme, informeAnual, instituto, private, vigenciaInforme) @ observacionInforme,
    }
    query residual compliance(instrumentoInforme, presentacionInforme, ana, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme
}

// Proposed discriminating case: a claim to that performance is recorded.
// No obligationFiled is supplied, but that absence is not ~obligation.
experiment claimedAndPerformed {
    seeds {
        right(instrumentoInforme, ana, presentarInforme, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme,
        Act(presentacionInforme, ana, presentarInforme, informeAnual, instituto, private, vigenciaInforme) @ observacionInforme,
    }
    query residual compliance(instrumentoInforme, presentacionInforme, ana, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme
}

// Missing-evidence control, not a negative valuation of the performance.
experiment owedUnobserved {
    seeds {
        obligationFiled(instrumentoInforme, ana, presentarInforme, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme,
    }
    query residual compliance(instrumentoInforme, presentacionInforme, ana, informeAnual, instituto, administrative, vigenciaInforme) @ observacionInforme
}
