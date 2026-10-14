module GeometryExplicitSubstitutions

import "./general.res"

// An explicitly chosen query scope, not a replacement for the entire ontology.
// Supplied values constrain the rules; they are not discovered existentially.
program ComplianceContext(conduct : Conduct, actAuthority : Authority, actEffective : PropositionTime, object : Thing, t : PropositionTime) {
    use obligationCompliance { conduct = conduct, actAuthority = actAuthority, actEffective = actEffective, object = object, t = t }
    use rightCompliance { conduct = conduct, actAuthority = actAuthority, actEffective = actEffective, object = object, t = t }
    use filedObligation { object = object, t = t }
}

program CompliancePartial = ComplianceContext(gConduct)
program ComplianceBound = CompliancePartial(private, gDate)
program ComplianceGround = ComplianceBound(gObject, gTau)

experiment suppliedContext {
    seeds { }
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
}
