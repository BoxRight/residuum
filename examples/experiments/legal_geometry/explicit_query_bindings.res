module GeometrySourceQueryBindings

import "./explicit_substitutions.res"

experiment boundRules {
    seeds {}
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
    using rules { obligationCompliance, rightCompliance, filedObligation }
    with {
        rule obligationCompliance.conduct = gConduct,
        rule obligationCompliance.actAuthority = private,
        rule obligationCompliance.actEffective = gDate,
        rule rightCompliance.conduct = gConduct,
        rule rightCompliance.actAuthority = private,
        rule rightCompliance.actEffective = gDate
    }
}

experiment boundProgram {
    seeds {}
    query residual compliance(gNorm, gAct, gSubject, gObject, gBeneficiary, administrative, gDate) @ gTau
    using program ComplianceContext
    with {
        program ComplianceContext.conduct = gConduct,
        program ComplianceContext.actAuthority = private,
        program ComplianceContext.actEffective = gDate
    }
}
