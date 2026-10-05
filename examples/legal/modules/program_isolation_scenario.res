module ProgramIsolationScenario

import "./sale_defaults.res"
import "./default_comparison/data.res"

const generalDuty : Fact
const pricePayment : Fact
const newDate : PropositionTime

// Sonda exclusiva de este experimento. Introduce un hecho intermedio contrario
// al paquete elegido para que las reglas NO seleccionadas tengan evidencia.
// No conecta reciprocity con selectedReciprocity ni cambia la política de defaults.
seeded verb testPackageEvidence(sale : Fact, package : ReciprocityPackage)
rule testExposePackage(saleId, package, t) =
    testPackageEvidence(saleId, package) @ t
    <= selectedReciprocity(saleId, package) @ t

experiment normalControl {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment normalWithRetentionEvidence {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
        testPackageEvidence(saleCase, Retention) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionUnpaidControl {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionUnpaidWithPlainEvidence {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        testPackageEvidence(saleCase, Plain) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionPaidControl {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment retentionPaidWithPlainEvidence {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        obligationFiled(generalDuty, juan, deliver, car, pedro, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
        paymentFor(saleCase, pricePayment) @ tau,
        Act(pricePayment, pedro, pay, price, juan, private, newDate) @ tau,
        testPackageEvidence(saleCase, Plain) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
