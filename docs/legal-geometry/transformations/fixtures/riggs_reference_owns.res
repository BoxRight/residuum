module RiggsReferenceOwnsObservation

import "../../../../examples/experiments/legal_geometry/riggs_succession_reference.res"

// Bounded verification (transformations research). The six Riggs dossiers of
// riggs_cases.res, seeds unchanged; the goal is the beneficiary's ownership
// under the literal reference theory. No rule is added.

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
