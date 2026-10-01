module AssociatedProgramWaits

entity Person
const juan : Person
const tau : PropositionTime
const sigma : PropositionTime

seeded verb starts(subject : Person) = program onStart
seeded verb signals(subject : Person) = program prepare
derived verb ready(subject : Person)
derived verb done(subject : Person)

rule onStart(subject, t) =
    ready(subject) @ t <= done(subject) @ after t

rule prepare(subject, t) =
    signals(subject) @ t <= ready(subject) @ t

experiment later {
    seeds { starts(juan) @ tau, signals(juan) @ tau, }
    query residual done(juan) @ after tau
}

experiment wrongTime {
    seeds { starts(juan) @ tau, signals(juan) @ sigma, }
    query residual done(juan) @ after tau
}

experiment noInstance {
    seeds { signals(juan) @ tau, }
    query residual done(juan) @ after tau
}
