module SaleDefaultOppositionExperiment

import "./sale_defaults.res"

// Evidencia y reglas exclusivamente experimentales. No cambian la ontología
// ni convierten una oposición en bloqueo, derrota o inconsistencia automática.
const juan : Person
const pedro : Person
const charlie : Person
const car : Thing
const price : Thing
const oldDate : PropositionTime
const newDate : PropositionTime
const tau : PropositionTime
const saleCase : Fact
const saleTransferDuty : Fact
const saleTransferPermission : Fact
const pricePayment : Fact

// El documento negativo es Seeded; su interpretación negativa de la obligación
// es Derived por esta regla explícita, no por ausencia de evidencia positiva.
rule recordNegativeFiledObligation(id, subject, conduct, object, beneficiary, authority, effective, t) =
    ~obligationFiled(id, subject, conduct, object, beneficiary, authority, effective) @ t
    <= ~obligation(id, subject, conduct, object, beneficiary, authority, effective) @ t

// Control para observar una oposición nacida dentro del cierre hipotético.
// Sólo produce evidencia negativa cuando hay un paquete seleccionado conocido.
seeded verb negativeDutyOnSelectedPackage(sale : Fact)
rule conditionalNegativeDuty(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective, package, t) =
    sale(saleId, seller, object, buyer, priceObject, duty, permit, authority, effective) @ t
    & selectedReciprocity(saleId, package) @ t
    & negativeDutyOnSelectedPackage(saleId) @ t
    <= ~obligation(duty, seller, transfer, object, buyer, authority, effective) @ t

experiment clearSale {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment opposedNormal {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        ~obligationFiled(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment opposedRetentionUnpaid {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        ~obligationFiled(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment opposedRetentionPaid {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
        ~obligationFiled(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment alreadyOpposed {
    seeds {
        obligationFiled(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
        ~obligationFiled(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment differentAuthority {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        ~obligationFiled(saleTransferDuty, juan, transfer, car, pedro, judicial, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment differentBeneficiary {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        ~obligationFiled(saleTransferDuty, juan, transfer, car, charlie, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment differentTime {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        ~obligationFiled(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ after tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment internalOpposition {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        negativeDutyOnSelectedPackage(saleCase) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
