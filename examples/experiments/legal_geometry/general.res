module GeometryGeneral

import "./ontology_reference/legal_general.res"

// Synthetic observations only: no case data or additional legal rules.
const gSubject : Person
const gBeneficiary : Person
const gLegislator : Person
const gObject : Thing
const gConduct : Conduct
const gNorm : Fact
const gAct : Fact
const gHigh : Fact
const gLow : Fact
const gGrant : Fact
const gEnact : Fact
const gStatute : Fact
const gTransfer : Fact
const gHolding : Fact
const gTau : PropositionTime
const gOtherTau : PropositionTime
const gDate : PropositionTime
const gEarlyDate : PropositionTime
const gLateDate : PropositionTime

experiment performance {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        prohibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        omissionProhibition(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        ~Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment authority {
    seeds {
        permissionFiled(gHigh, gSubject, gConduct, gObject, gBeneficiary, constitutional, gLateDate) @ gTau,
        obligationFiled(gHigh, gSubject, gConduct, gObject, gBeneficiary, constitutional, gLateDate) @ gTau,
        prohibition(gHigh, gSubject, gConduct, gObject, gBeneficiary, constitutional, gLateDate) @ gTau,
        permissionFiled(gLow, gSubject, gConduct, gObject, gBeneficiary, administrative, gEarlyDate) @ gTau,
        obligationFiled(gLow, gSubject, gConduct, gObject, gBeneficiary, administrative, gEarlyDate) @ gTau,
        prohibition(gLow, gSubject, gConduct, gObject, gBeneficiary, administrative, gEarlyDate) @ gTau,
        higherAuthority(constitutional, administrative) @ gTau,
        notEarlier(gLateDate, gEarlyDate) @ gTau,
    }
    query residual annulled(gLow, gHigh, constitutional, gLateDate) @ gTau
}

experiment legislation {
    seeds {
        powerGranted(gGrant, gLegislator, enact, gObject, legislative, gDate) @ gTau,
        Act(gEnact, gLegislator, enact, gObject, gBeneficiary, legislative, gDate) @ gTau,
        legislationText(gEnact, gStatute, gSubject, gConduct, gObject, gBeneficiary, legislative, gDate) @ gTau,
    }
    query residual law(gStatute, gSubject, gConduct, gObject, gBeneficiary, legislative, gDate) @ gTau
}

experiment transferCase {
    seeds {
        Act(gTransfer, gSubject, transfer, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual owns(gBeneficiary, gObject, gTransfer, private, gDate) @ gTau
}

experiment otherTime {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        ~Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gOtherTau,
    }
    query residual violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment otherAuthority {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        ~Act(gAct, gSubject, gConduct, gObject, gBeneficiary, judicial, gDate) @ gTau,
    }
    query residual violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment theoryCompliance {
    premises {
        compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment theoryAsset {
    premises {
        patrimonialAsset(gBeneficiary, gObject, gNorm, administrative, gDate) @ gTau,
    }
    query fourFusion patrimonialAsset(gBeneficiary, gObject, gNorm, administrative, gDate) @ gTau
}

experiment theoryObligation {
    premises {
        obligation(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
    }
    query fourFusion obligation(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
}

experiment holding {
    seeds {
        patrimonialHolding(gHolding, gBeneficiary, gObject, administrative, gDate) @ gTau,
    }
    query residual owns(gBeneficiary, gObject, gHolding, administrative, gDate) @ gTau
}

experiment extinction {
    seeds {
        obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau,
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        extinguished(gNorm, constitutional, gLateDate) @ gTau,
    }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}
