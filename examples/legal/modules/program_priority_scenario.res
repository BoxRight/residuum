module ProgramPriorityScenario

import "./sales.res"
import "./default_comparison/data.res"

seeded verb retentionAgreed(sale : Fact)

// Mismos programas, sin blocker: se aísla la prioridad explícita existente.
default NormalSale =
    supernormal normalSaleProgram

default RetentionSale =
    exception retentionSaleProgram
    to NormalSale
    when
        retentionAgreed(saleId) @ t

experiment priorityWithoutBlocking {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    query residual selectedReciprocity(saleCase, Retention) @ tau
}
