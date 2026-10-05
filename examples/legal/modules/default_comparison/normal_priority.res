module SaleDefaultsNormalPriorityExperiment

import "../sales.res"

seeded verb preferPlain(sale : Fact)

// D1 = NormalSale; D2 = RetentionSale. La competencia entre ellos se declara
// sólo en el harness experimental: todavía no hay sintaxis para esa relación.
default RetentionSale =
    supernormal retentionSale

// Prioridad direccional existente: D1 > D2, independiente de la competencia.
default NormalSale =
    exception normalSale
    to RetentionSale
    when
        preferPlain(saleId) @ t
