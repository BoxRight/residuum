module FormalizationR1

import "generalizations.res"

// R1: a different formalization, adopted after the valuation of dossier 0:
// the prohibition requires explicit evidence of not-A, and dogs with A give a
// registration duty.
program R1(t : PropositionTime) {
    use refinedBan { t = t, }
    use registrationDuty { t = t, }
}

derived verb appliesR1() = program R1
