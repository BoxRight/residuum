module HypotheticalComplianceCases

import "./general.res"

// Experimental theory snapshots, NOT new legal rules or actual case data.
// D is an explicitly supplied Derived premise, not an invented Seeded fact.
// Labels live in region_revision_labels.tsv and are external assumptions.
// Existing syntax requires premises + fourFusion. The harness uses only the
// elaborated input/goal and evaluates Horn closures, not FOUR entailment.

experiment catalog {
    premises {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        omissionProhibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment theoryViolation {
    premises {
        violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment baselineObligation {
    premises {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment baselineRight {
    premises {
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment assumedPositiveDispute {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment assumedPositiveOmission {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        omissionProhibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment empty {
    premises {
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment actOnly {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment permissionOnly {
    premises {
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment disputeOnly {
    premises {
        disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment omissionOnly {
    premises {
        omissionProhibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment permissionDispute {
    premises {
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment permissionOmission {
    premises {
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        omissionProhibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment obligationOnly {
    premises {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment rightOnly {
    premises {
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}
