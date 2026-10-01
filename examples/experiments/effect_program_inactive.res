module EffectProgramInactive

entity Person
const juan : Person
const tau : PropositionTime

seeded verb requests(subject : Person)
effect verb changes(subject : Person) = program onChange
derived verb observed(subject : Person)

rule requestChange(subject, t) =
    requests(subject) @ t <= changes(subject) @ after t

rule onChange(subject, t) =
    I <= observed(subject) @ t

experiment effectKnown {
    seeds { requests(juan) @ tau, }
    query residual observed(juan) @ after tau
}
