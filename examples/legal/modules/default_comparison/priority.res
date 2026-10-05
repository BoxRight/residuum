module SaleDefaultsPriorityComparison

import "../sales.res"

seeded verb retentionAgreed(sale : Fact)

// Variante experimental sin bloqueo, para observar exclusivamente prioridad.
// Supernormal es incondicional; una venta habilita su cuerpo, no su selección.
default NormalSale =
    supernormal normalSale

default RetentionSale =
    exception retentionSale
    to NormalSale
    when
        retentionAgreed(saleId) @ t
