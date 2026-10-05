module TemporalBetaArguments

entity Person
const juan : Person
const oldDate : PropositionTime
const newDate : PropositionTime
const tau : PropositionTime
const sigma : PropositionTime

seeded verb observed(subject : Person, effective : PropositionTime)
derived verb recorded(subject : Person, effective : PropositionTime)
effect verb registered(subject : Person, effective : PropositionTime)

rule remember(effective, t, subject) =
    observed(subject, effective) @ t <= recorded(subject, effective) @ after t
rule partiallyRemember = remember(oldDate)
rule fixedRemember = partiallyRemember(tau, juan)
rule directRemember = remember(oldDate, tau, juan)
rule inlineRemember() =
    observed(juan, oldDate) @ tau <= recorded(juan, oldDate) @ after tau

// El mismo parámetro temporal puede ocupar ambas posiciones simultáneamente.
rule register(effective, subject) =
    observed(subject, effective) @ effective <= I -o registered(subject, effective) @ after effective
rule partialRegister = register(oldDate)
rule fixedRegister = partialRegister(juan)
rule directRegister = register(oldDate, juan)
rule inlineRegister() =
    observed(juan, oldDate) @ oldDate <= I -o registered(juan, oldDate) @ after oldDate

experiment right {
    seeds { observed(juan, oldDate) @ tau, }
    query residual recorded(juan, oldDate) @ after tau
}
experiment wrongArgument {
    seeds { observed(juan, newDate) @ tau, }
    query residual recorded(juan, oldDate) @ after tau
}
experiment wrongTime {
    seeds { observed(juan, oldDate) @ sigma, }
    query residual recorded(juan, oldDate) @ after tau
}
experiment sameBinderBothPositions {
    seeds { observed(juan, oldDate) @ oldDate, }
    query residual registered(juan, oldDate) @ after oldDate
}
