module SaleDefaultsUnorderedComparison

import "../sales.res"

seeded verb retentionAgreed(sale : Fact)

// Ambos son supernormales incondicionales, sin excepción ni bloqueo.
// El lenguaje no permite when en supernormal: no se simula esa capacidad.
default NormalSale =
    supernormal normalSale

default RetentionSale =
    supernormal retentionSale
