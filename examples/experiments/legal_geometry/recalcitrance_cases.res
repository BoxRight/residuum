module RecalcitranceCases

import "./general.res"

// Synthetic Seeded cases. External labels are in recalcitrance_labels.tsv.
// No label or justification is inferred by these experiments.

experiment catalog {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment geometryCatalog {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        ~Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment empty {
    seeds {
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment rightAct {
    seeds {
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment obligationAct {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment actPermission {
    seeds {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment actOnly {
    seeds {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment permissionOnly {
    seeds {
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

