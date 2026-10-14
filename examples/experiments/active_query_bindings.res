module SourceActiveQueryBindings

entity Thing
entity Authority
const item : Thing
const private : Authority
const public : Authority
const tau : PropositionTime

seeded verb starts(object : Thing, authority : Authority) = program P
derived verb done(object : Thing)
rule finish(object, authority, t) = I <= done(object) @ after t

program P(object : Thing, authority : Authority, t : PropositionTime) {
    use finish { object = object, authority = authority, t = t }
    use finish { object = object, authority = authority, t = t }
}

experiment pending {
    seeds {}
    query active residual done(item) @ after tau
    using program P
    with { program P.authority = private }
}

experiment satisfied {
    seeds { starts(item, private) @ tau }
    query active residual done(item) @ after tau
    using rules { finish }
    with { rule finish.authority = private }
}

experiment otherActivation {
    seeds { starts(item, public) @ tau }
    query active residual done(item) @ after tau
    using program P
    with { program P.authority = private }
}
