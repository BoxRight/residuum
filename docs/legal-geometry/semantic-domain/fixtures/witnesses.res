module SharedSpacesWitnesses

import "./cast.res"

// Witness situations for the sufficiency check of the initial cardinalities.
// s0 is a base situation; every other experiment is one admissible move away
// from s0: a change of state of one atom, of the anchor conduct, or of the
// person in the subject or beneficiary role. No valuation is attached.

experiment s0 {
    seeds {
        considered(ana, enterWith, obj, bea) @ t0,
        thingTrait(obj, animal) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sCert {
    seeds {
        considered(ana, enterWith, obj, bea) @ t0,
        thingTrait(obj, animal) @ t0,
        thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sNoSens {
    seeds {
        considered(ana, enterWith, obj, bea) @ t0,
        thingTrait(obj, animal) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        bond(dan, cal, livesWith) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sNotAnimal {
    seeds {
        considered(ana, enterWith, obj, bea) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sRoleSubject {
    seeds {
        considered(dan, enterWith, obj, bea) @ t0,
        thingTrait(obj, animal) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sRoleBeneficiary {
    seeds {
        considered(ana, enterWith, obj, cal) @ t0,
        thingTrait(obj, animal) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sConduct {
    seeds {
        considered(ana, useInside, obj, bea) @ t0,
        thingTrait(obj, animal) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sMinor {
    seeds {
        considered(ana, enterWith, obj, bea) @ t0,
        thingTrait(obj, animal) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
        personTrait(bea, minor) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}

experiment sNeutral {
    seeds {
        considered(ana, enterWith, obj, bea) @ t0,
        thingTrait(obj, animal) @ t0,
        ~thingTrait(obj, certifiedAssistance) @ t0,
        custody(ana, obj) @ t0,
        bond(cal, bea, presentWith) @ t0,
        sensitiveTo(cal, animal) @ t0,
        bond(dan, cal, livesWith) @ t0,
        personTrait(dan, pregnant) @ t0,
    }
    query residual
    notDo(ana, enterWith, obj, bea) @ after t0
}
