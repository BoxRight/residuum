module LegalPatrimonial

import "./legal_general.res"

const deliver : Conduct
const molest : Conduct

// Política patrimonial de este expediente: la prestación cumplida acredita
// el activo de la cosa a su beneficiario; la violación registra el pasivo del
// obligado. Son hechos Derived, no mutaciones de un Store ni valoración monetaria.
rule complianceAsset(norm, act, subject, object, beneficiary, authority, effective, t) =
    compliance(norm, act, subject, object, beneficiary, authority, effective) @ t
    <= patrimonialAsset(beneficiary, object, norm, authority, effective) @ t
