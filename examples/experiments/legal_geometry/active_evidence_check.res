module GeometryActiveEvidenceCheck

import "./general.res"

// Experimental invocation, not an additional legal norm. Referencing the
// imported clause through this association makes its availability conditional.
seeded verb inspectEvidence(
    id : Fact,
    subject : Person,
    conduct : Conduct,
    object : Thing,
    beneficiary : Person,
    authority : Authority,
    effective : PropositionTime
) = program EvidenceCheck

program EvidenceCheck(
    id : Fact,
    subject : Person,
    conduct : Conduct,
    object : Thing,
    beneficiary : Person,
    authority : Authority,
    effective : PropositionTime,
    t : PropositionTime
) {
    use conflictingActEvidence {
        act = id,
        subject = subject,
        conduct = conduct,
        object = object,
        beneficiary = beneficiary,
        authority = authority,
        effective = effective,
        t = t
    }
}

experiment exactEvidence {
    seeds {
        Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        ~Act(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
        inspectEvidence(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau,
    }
    query residual inconsistent(gAct, gSubject, gConduct, gObject, gBeneficiary, private, gDate) @ gTau
}
