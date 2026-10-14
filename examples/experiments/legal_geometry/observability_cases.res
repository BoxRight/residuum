module DiscriminatingCaseObservability

import "./region_revision.res"

// Synthetic Seeded inputs only. No new hypothesis or external legal answer.
// The harness checks paths with the original general ontology, then separately
// compares the nine already established hypothetical modules.
// Reaching D/V may require evidence beyond the original discriminating snapshot.

experiment directAP {
    seeds {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment directAM {
    seeds {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        omissionProhibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment reachableAD {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        ~Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment reachableAV {
    seeds {
        prohibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment reachableAPV {
    seeds {
        prohibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}
