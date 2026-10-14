module RiggsHistoricalExperiment

import "./riggs_succession_reference.res"

// Names from the opinion; estate represents only the interest under discussion.
// Support charges, contingent gifts and the entire estate distribution are out
// of this delimited reference. Counterfactuals are experimental stipulations.
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
    query residual testamentaryEntitlement(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime
}

experiment noBeneficiaryKilling {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        ~intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
    }
    query residual testamentaryEntitlement(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime
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
    query residual testamentaryEntitlement(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime
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
    query residual testamentaryEntitlement(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime
}

experiment causalLinkUnestablished {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        survivorRecorded(elmerPalmer, francisPalmer) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
    }
    query residual testamentaryEntitlement(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime
}

experiment survivalUnestablished {
    seeds {
        operativeDisposition(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime,
        deathRecorded(palmerDeath, francisPalmer, judicial, death1882) @ dossierTime,
        intentionalKilling(killingEvidence, elmerPalmer, francisPalmer, judicial, death1882) @ dossierTime,
        deathCausedBy(killingEvidence, palmerDeath) @ dossierTime,
        acquisitionPurposeRecorded(killingEvidence, palmerWill, elmerPalmer) @ dossierTime,
    }
    query residual testamentaryEntitlement(palmerWill, francisPalmer, elmerPalmer, residuaryInterest, private, will1880) @ dossierTime
}
