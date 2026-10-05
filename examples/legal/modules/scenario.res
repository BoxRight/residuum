module LegalMonotonicScenario

import "./sales_monotonic_baseline.res"

// Datos concretos y los 16 experiments de case_monotonic_formal.res.
// No incorpora defaults ni ejecuta Effects.

const juan : Person
const pedro : Person
const legislator : Person
const car : Thing
const boat : Thing
const house : Thing
const book : Thing
const bicycle : Thing
const legislativeSeat : Thing
const price : Thing

const oldDate : PropositionTime
const newDate : PropositionTime
const tau : PropositionTime

const transferDuty : Fact
const transferRight : Fact
const transferAct : Fact
const deliveryDuty : Fact
const deliveryRight : Fact
const deliveryAct : Fact
const noMolesting : Fact
const molestingAct : Fact
const noOmission : Fact
const holding : Fact
const grantedPower : Fact
const enactment : Fact
const statutoryDuty : Fact
const carPermission : Fact
const carProhibition : Fact
const boatDuty : Fact
const boatProhibition : Fact
const housePermission : Fact
const houseProhibition : Fact
const seatDuty : Fact
const seatProhibition : Fact
const olderPermission : Fact
const newerProhibition : Fact
const sameDatePermission : Fact
const sameDateProhibition : Fact
const saleCase : Fact
const saleTransferDuty : Fact
const saleTransferPermission : Fact
const pricePayment : Fact

// ~Act usa exclusivamente el EvidentialNot ya existente: evidencia explícita
// negativa del mismo Act, no ausencia de evidencia ni retractación.
// Act y ~Act pueden coexistir, junto con compliance y violation.
// Fact identifica el documento/hecho; no reifica una proposición del core.
// La autoridad y la vigencia se conservan en los argumentos y en provenance.
experiment fulfilledTransfer {
    seeds {
        obligationFiled(transferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
        right(transferRight, juan, transfer, car, pedro, private, oldDate) @ tau,
        Act(transferAct, juan, transfer, car, pedro, private, newDate) @ tau,
    }
    query residual owns(pedro, car, transferDuty, private, oldDate) @ tau
}

experiment negativeActOnly {
    seeds {
        obligationFiled(deliveryDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        right(deliveryRight, juan, deliver, car, pedro, private, oldDate) @ tau,
        omissionProhibition(noOmission, juan, deliver, car, pedro, judicial, oldDate) @ tau,
        ~Act(deliveryAct, juan, deliver, car, pedro, private, newDate) @ tau,
    }
    query residual violation(deliveryDuty, deliveryAct, juan, car, pedro, private, oldDate) @ tau
}

experiment actAndEvidentialNot {
    // Conserva el expediente antes llamado actAndCounter. La query observa la
    // disputa ordinaria; el cierre conserva además ambos actos y ambos resultados.
    seeds {
        obligationFiled(deliveryDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        Act(deliveryAct, juan, deliver, car, pedro, private, newDate) @ tau,
        ~Act(deliveryAct, juan, deliver, car, pedro, private, newDate) @ tau,
        extinguished(deliveryDuty, judicial, newDate) @ tau,
    }
    query residual disputedPerformance(deliveryDuty, deliveryAct, juan, car, pedro, private, oldDate) @ tau
}

experiment prohibitedOccurrence {
    seeds {
        prohibition(noMolesting, juan, molest, car, pedro, administrative, oldDate) @ tau,
        Act(molestingAct, juan, molest, car, pedro, private, newDate) @ tau,
    }
    query residual violation(noMolesting, molestingAct, juan, car, pedro, administrative, oldDate) @ tau
}

experiment legislation {
    seeds {
        patrimonialHolding(holding, legislator, legislativeSeat, constitutional, oldDate) @ tau,
        powerGranted(grantedPower, legislator, enact, legislativeSeat, constitutional, oldDate) @ tau,
        Act(enactment, legislator, enact, car, pedro, legislative, newDate) @ tau,
        legislationText(enactment, statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau,
    }
    query residual obligation(statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau
}

experiment authorityOverrides {
    seeds {
        higherAuthority(constitutional, private) @ tau,
        higherAuthority(legislative, administrative) @ tau,
        higherAuthority(judicial, private) @ tau,
        notEarlier(newDate, oldDate) @ tau,
        notEarlier(oldDate, oldDate) @ tau,
        permissionFiled(carPermission, juan, transfer, car, pedro, constitutional, newDate) @ tau,
        prohibition(carProhibition, juan, transfer, car, pedro, private, oldDate) @ tau,
        obligationFiled(boatDuty, juan, deliver, boat, pedro, legislative, newDate) @ tau,
        prohibition(boatProhibition, juan, deliver, boat, pedro, administrative, oldDate) @ tau,
        prohibition(houseProhibition, juan, molest, house, pedro, judicial, newDate) @ tau,
        permissionFiled(housePermission, juan, molest, house, pedro, private, oldDate) @ tau,
        prohibition(seatProhibition, juan, transfer, legislativeSeat, pedro, constitutional, newDate) @ tau,
        obligationFiled(seatDuty, juan, transfer, legislativeSeat, pedro, private, oldDate) @ tau,
        permissionFiled(olderPermission, juan, transfer, book, pedro, judicial, oldDate) @ tau,
        prohibition(newerProhibition, juan, transfer, book, pedro, private, newDate) @ tau,
        permissionFiled(sameDatePermission, juan, transfer, bicycle, pedro, constitutional, oldDate) @ tau,
        prohibition(sameDateProhibition, juan, transfer, bicycle, pedro, private, oldDate) @ tau,
    }
    query residual annulled(carProhibition, carPermission, constitutional, newDate) @ tau
}

experiment unperformed {
    seeds {
        obligationFiled(deliveryDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
    }
    query residual violation(deliveryDuty, deliveryAct, juan, car, pedro, private, oldDate) @ tau
}

experiment enactmentWithoutPower {
    seeds {
        Act(enactment, legislator, enact, car, pedro, legislative, newDate) @ tau,
        legislationText(enactment, statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau,
    }
    query residual obligation(statutoryDuty, juan, deliver, car, pedro, legislative, newDate) @ tau
}

// Mismos términos de venta en cada expediente. Cambian exclusivamente los
// paquetes y la evidencia de pago. La query sólo observa el deber de transmitir.
experiment plainWithoutPayment {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Plain) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionWithoutPayment {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Retention) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionPaid {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Retention) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionNegativePayment {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Retention) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        ~Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionPaidAndNegative {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Retention) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
        ~Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment bothReciprocities {
    // Al aportar ambos paquetes, las reglas acumulan consecuencias y pruebas.
    // Ningún paquete derrota al otro; esa selección corresponde a defaults.
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Plain) @ tau,
        reciprocity(saleCase, Retention) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionWrongPayment {
    // El documento refiere a la venta, pero el destinatario del pago no es
    // el vendedor. Esa evidencia no satisface la condición compartida.
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        reciprocity(saleCase, Retention) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, pedro, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment noReciprocityPackage {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
