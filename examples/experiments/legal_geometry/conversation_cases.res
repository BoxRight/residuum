module ConversationalJustificationCases

import "./recalcitrance_cases.res"

// Questions and challenges, not new norms. No external answers are derived.
// Labels and scenario selection live in conversation_valuations.tsv.

experiment candidateR {
    seeds {
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment candidateOR {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment candidateOP {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment candidateRP {
    seeds {
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment candidateORP {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment challengeO {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment outsideFeatures {
    seeds {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        ~Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment outsideTime {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gOtherTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gOtherTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gOtherTau
}

