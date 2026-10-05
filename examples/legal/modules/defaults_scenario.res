module DefaultScenario

import "./sale_defaults.res"
// Dependencias repetidas deliberadamente: no duplican declaraciones.
import "./sales.res"
import "./legal_monotonic.res"

const juan : Person
const pedro : Person
const car : Thing
const price : Thing
const oldDate : PropositionTime
const newDate : PropositionTime
const tau : PropositionTime
const saleCase : Fact
const saleTransferDuty : Fact
const saleTransferPermission : Fact
const pricePayment : Fact

experiment noRetention {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionUnpaid {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    // No se deriva la obligación; tampoco se afirma ~obligation.
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionPaid {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment normalBlocked {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    // Mismo expediente que retentionUnpaid; aquí se observa la lambda ganadora.
    query residual selectedReciprocity(saleCase, Retention) @ tau
}

experiment sellsReal {
    seeds { sells(juan, car, pedro) @ tau, }
    query residual give(juan, car, pedro) @ after tau
}
experiment sellsAbsent {
    seeds {}
    query residual give(juan, car, pedro) @ after tau
}
experiment suppliedPlain {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Plain) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
experiment suppliedOpposition {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Retention) @ tau,
        retentionAgreed(saleCase) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        ~Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
