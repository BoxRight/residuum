module RiggsBlockedEntitlementAsset

import "../../../../examples/experiments/legal_geometry/riggs_succession_reference.res"

// Bounded verification (transformations research). Same dossiers and goal.
// The reference rule literalEntitlementAsset is placed under a default whose
// blocker is the existing derived relation purposefulAcquisition. A default
// condition may only use its target's parameters (elab check_default_condition),
// so the rule is wrapped in a program whose parameters also bind act and death.
// No new verb, no new rule body.
program entitlementAssetUnit(
    will : Fact, testator : Person, beneficiary : Person, object : Thing,
    authority : Authority, effective : PropositionTime, act : Fact, death : Fact,
    t : PropositionTime
) {
    use literalEntitlementAsset {
        will = will, testator = testator, beneficiary = beneficiary, object = object,
        authority = authority, effective = effective, t = t,
    }
}

default LiteralEntitlementAsset =
    supernormal entitlementAssetUnit
    blocked when
        purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t

const francisPalmer : Person
const elmerPalmer : Person
const otherKiller : Person
const residuaryInterest : Thing
const palmerWill : Fact
const palmerDeath : Fact
const killingEvidence : Fact
const will1880 : PropositionTime
const death1882 : PropositionTime
const dossierTime : PropositionTime

experiment historicalRiggs {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
        deathCausedBy(killingEvidence, palmerDeath) @ dossierTime,
        acquisitionPurposeRecorded(killingEvidence, palmerWill, elmerPalmer) @ dossierTime,
    }
    query residual owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880) @ dossierTime
}

experiment noBeneficiaryKilling {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        ~intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
    }
    query residual owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880) @ dossierTime
}

// Explicit negative purpose, not an inference from missing purpose evidence.
experiment killingWithoutAcquisitionPurpose {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
        deathCausedBy(killingEvidence, palmerDeath) @ dossierTime,
        ~acquisitionPurposeRecorded(killingEvidence, palmerWill, elmerPalmer) @ dossierTime,
    }
    query residual owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880) @ dossierTime
}

experiment killingByThirdParty {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        intentionalKilling(killingEvidence, otherKiller, francisPalmer, judicial, death1882) @ dossierTime,
        ~intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
        deathCausedBy(killingEvidence, palmerDeath) @ dossierTime,
    }
    query residual owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880) @ dossierTime
}

experiment causalLinkUnestablished {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
    }
    query residual owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880) @ dossierTime
}

experiment survivalUnestablished {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
        deathCausedBy(killingEvidence, palmerDeath) @ dossierTime,
        acquisitionPurposeRecorded(killingEvidence, palmerWill, elmerPalmer) @ dossierTime,
    }
    query residual owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880) @ dossierTime
}
