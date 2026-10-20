module Pilot004RContrastPool

import "../vocabulary.res"

// Counterfactual pool for the h1/h2 contrast (run-003). NOT documented
// facts and NOT part of the corpus: no valuation is attached and none may be
// inferred from these atoms. The pool offers, for one Brookside stipulation,
// the filed obligation plus a performance and a documented non-performance
// by each party; the minimal-support analysis selects the minimal
// configurations from it. actJ and actU are counterfactual act identifiers.

const actJ : Fact
const actU : Fact

experiment pool {
    seeds {
        obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB,
        Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB,
        ~Act(actJ, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB,
        Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB,
        ~Act(actU, usa, adoptPolicy, policyA, jant, private, d20210216) @ tB,
    }
    query residual
    obligationFiled(saB_j, jant, adoptPolicy, policyA, usa, private, d20210216) @ tB
}
