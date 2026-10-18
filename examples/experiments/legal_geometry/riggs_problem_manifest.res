module RiggsProblemManifest

import "./riggs_succession_reference.res"

// Problem manifest for the LLM-guided exploration contract (§2.2). It adds
// only the experimental output head to the reference theory. No rule, no
// catalogue, no valuation: candidates are proposed elsewhere and installed
// in private modules that import this file.
derived verb hypotheticalSuccessionImpediment(will : Fact, testator : Person, beneficiary : Person, object : Thing, authority : Authority, effective : PropositionTime)
