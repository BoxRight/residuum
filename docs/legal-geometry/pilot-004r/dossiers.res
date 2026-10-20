module Pilot004RDossiers

import "./vocabulary.res"

// Established configurations S_i only. Allegations stay in the documentary
// record (docs/pilot-004-documentary-corpus.md) and never enter here.
// No valuation is attached; the reserved valuations live in reserved/.
// Flowers has no established representable atom, so it has no experiment:
// its seeds are the empty set. Each query is one of the dossier's own seeds.

// Brookside: the signed agreement (documentary input). The obligation
// positions are derived from obligationFiled by LegalGeneral, not seeded.
experiment brookside {
    seeds {
        Act(saB_j, jant, agree, saDocB, usa, private, d20210216) @ tB,
        Act(saB_u, usa, agree, saDocB, jant, private, d20210216) @ tB,
        obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB,
        obligationFiled(saB_j, jant, distribute, policyA, usa, private, d20210216) @ tB,
        obligationFiled(saB_j, jant, post, policyA, usa, private, d20210216) @ tB,
        obligationFiled(saB_j, jant, keepAvailable, policyA, usa, private, d20210216) @ tB,
        obligationFiled(saB_j, jant, post, signB, usa, private, d20210216) @ tB,
        prohibition(saB_u, usa, sue, premisesB, jant, private, d20210216) @ tB,
    }
    query residual
    Act(saB_j, jant, agree, saDocB, usa, private, d20210216) @ tB
}

// Casa Fiesta: statements of the federal party (grade R). The beneficiary
// usa of cfA1/cfA2 rests on "as part of the settlement" (reservation).
experiment casa_fiesta {
    seeds {
        Act(saCF_c, casaFiesta, agree, saDocCF, usa, private, dCF) @ tCF,
        Act(saCF_u, usa, agree, saDocCF, casaFiesta, private, dCF) @ tCF,
        Act(cfA1, casaFiesta, adoptPolicy, policyCF, usa, private, dCF) @ tCF,
        Act(cfA2, casaFiesta, post, signCF, usa, private, dCF) @ tCF,
    }
    query residual
    Act(saCF_c, casaFiesta, agree, saDocCF, usa, private, dCF) @ tCF
}
