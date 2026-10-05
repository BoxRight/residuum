module SalePriorityScenario

import "./priority.res"
import "./data.res"

// Known vacío no implica defaults inaplicables: NormalSale es incondicional.
experiment emptyKnown {
    seeds {}
    query residual selectedReciprocity(saleCase, Plain) @ tau
}

experiment normalOnly {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}

experiment explicitPriority {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        retentionAgreed(saleCase) @ tau,
    }
    query residual selectedReciprocity(saleCase, Retention) @ tau
}
