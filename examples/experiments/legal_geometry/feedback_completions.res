module HypotheticalComplianceGuardCompletion

import "./region_revision.res"

// Mathematical controls only, NOT proposed legal justifications.
// These rules are never added to legal_general.res or to a real scenario.
// V is absent from the original 64-case catalog. The harness adds one control
// to a COPY of a hypothetical revision to distinguish observational agreement
// from equality of the entire closure operator on the expanded universe.

rule hypotheticalGuardCompletionAV() =
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    & violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
    <= compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau

rule hypotheticalGuardCompletionAPV() =
    Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
    & permissionFiled(gNorm, gSubject, gConduct, gObject, gBeneficiary, administrative, gDate) @ gTau
    & violation(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
    <= compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
