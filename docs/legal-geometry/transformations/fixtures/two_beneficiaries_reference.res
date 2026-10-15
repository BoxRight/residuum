module TwoBeneficiariesReference

import "../../../../examples/experiments/legal_geometry/riggs_succession_reference.res"

// Bounded verification (transformations research): one will, two beneficiaries;
// only elmerPalmer killed the testator with a recorded acquisition purpose.
// Literal reference theory, no rule added.
const francisPalmer : Person
const elmerPalmer : Person
const innocentHeir : Person
const residuaryInterest : Thing
const legacyInterest : Thing
const palmerWill : Fact
const palmerDeath : Fact
const killingEvidence : Fact
const will1880 : PropositionTime
const death1882 : PropositionTime
const dossierTime : PropositionTime

experiment twoBeneficiaries_innocentHeir {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        operativeDisposition(palmerWill, francisPalmer, innocentHeir, legacyInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        survivorRecorded(innocentHeir, francisPalmer) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
        deathCausedBy(killingEvidence, palmerDeath) @ dossierTime,
        acquisitionPurposeRecorded(killingEvidence, palmerWill, elmerPalmer) @ dossierTime,
    }
    query residual owns(innocentHeir, legacyInterest, palmerWill, private, will1880) @ dossierTime
}

experiment twoBeneficiaries_elmerPalmer {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        operativeDisposition(palmerWill, francisPalmer, innocentHeir, legacyInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        survivorRecorded(innocentHeir, francisPalmer) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
        deathCausedBy(killingEvidence, palmerDeath) @ dossierTime,
        acquisitionPurposeRecorded(killingEvidence, palmerWill, elmerPalmer) @ dossierTime,
    }
    query residual owns(elmerPalmer, residuaryInterest, palmerWill, private, will1880) @ dossierTime
}
