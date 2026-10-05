module SaleUnorderedScenario

import "./unordered.res"
import "./data.res"

// Observador de consecuencias declarado por este experimento. Ningún nombre
// ni polaridad concede prioridad o bloqueo a un default.
seeded verb observeOpposition(sale : Fact)
derived verb comparisonP(sale : Fact)

rule plainConsequence(saleId, t) =
    selectedReciprocity(saleId, Plain) @ t
    & observeOpposition(saleId) @ t
    <= comparisonP(saleId) @ t

rule retentionConsequence(saleId, t) =
    selectedReciprocity(saleId, Retention) @ t
    & observeOpposition(saleId) @ t
    <= ~comparisonP(saleId) @ t

experiment noPriority {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment opposedConsequences {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        observeOpposition(saleCase) @ tau,
    }
    query residual comparisonP(saleCase) @ tau
}

// Mismo expediente, objetivo negativo: explain expone también esta cadena.
experiment opposedNegativeConsequences {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        observeOpposition(saleCase) @ tau,
    }
    query residual ~comparisonP(saleCase) @ tau
}
