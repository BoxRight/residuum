module PendingDiscriminatingCases
import "./region_revision.res"

// Predictions are not external labels. No answer is supplied here.
// Existing premises/fourFusion syntax is used for typed input only;
// the harness compares Horn closures, not FOUR entailment.

experiment case_0005 {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        disputedPerformance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment case_0065 {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        omissionProhibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment case_0513 {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment case_2049 {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment case_2561 {
    premises {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}
