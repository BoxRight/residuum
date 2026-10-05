module SaleDefaults

import "./sales.res"

seeded verb retentionAgreed(sale : Fact)

// El bloqueo es explícito. La excepción además conserva su relación de
// prioridad; blocked y defeated son registros distintos del evaluador actual.
default NormalSale =
    supernormal normalSaleProgram
    blocked when
        retentionAgreed(saleId) @ t

default RetentionSale =
    exception retentionSaleProgram
    to NormalSale
    when
        retentionAgreed(saleId) @ t
