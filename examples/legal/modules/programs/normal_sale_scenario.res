module NormalSaleProgramScenario

import "./normal_sale.res"
import "../default_comparison/data.res"

program NormalSaleForJuan = NormalSale(saleCase, juan)
program NormalSaleGround = NormalSaleForJuan(car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate, tau)
program NormalSaleDirect = NormalSale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate, tau)

experiment plain {
    seeds {
        sale(saleCase, juan, car, pedro, price, saleTransferDuty, saleTransferPermission, private, oldDate) @ tau
    }
    query residual do(juan, transfer, car, pedro) @ after tau
}
