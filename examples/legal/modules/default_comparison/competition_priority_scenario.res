module SaleCompetitionPriorityScenario

import "./normal_priority.res"
import "./data.res"

experiment normalHasPriority {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau,
        preferPlain(saleCase) @ tau,
    }
    query residual obligation(saleTransferDuty, juan, transfer, car, pedro, private, oldDate) @ tau
}
