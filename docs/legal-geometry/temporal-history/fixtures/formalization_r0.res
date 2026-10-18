module FormalizationR0

import "generalizations.res"

// R0: the general prohibition. A time-free formalization: a Program over the
// generalization space, parameterised only by the proposition time.
program R0(t : PropositionTime) {
    use ordinaryBan { t = t, }
}

derived verb appliesR0() = program R0
