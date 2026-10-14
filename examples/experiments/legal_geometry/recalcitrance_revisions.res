module HypotheticalJustificatoryRevisions

import "./recalcitrance_cases.res"

// A catalog of ground rule templates, NOT an executable proposed legal regime.
// The test selects a family in a private Module copy, and removes only the two
// compliance producers there. Loading all templates is not a candidate revision.
// No source declaration of LegalGeneral is changed.

rule justificatorySupport0() =
    I
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport1() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport2() =
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport3() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport4() =
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport5() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport6() =
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport7() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport8() =
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport9() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport10() =
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport11() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport12() =
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    &
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport13() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    &
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport14() =
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    &
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule justificatorySupport15() =
    obligationFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    right(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    &
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    &
    permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    <=
    compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

