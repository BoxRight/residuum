module AssociatedMultiRuleProgram

entity Thing
entity Box { value : Thing }
entity Envelope { payload : Box }
const item : Thing
const other : Thing
const tau : PropositionTime
const later : PropositionTime

seeded verb starts(object : Thing) = program Chain
seeded verb requests(object : Thing)
derived verb activated(object : Thing) = program Chain
derived verb middle(object : Thing)
derived verb completed(object : Thing)
derived verb remembered(object : Thing, effective : PropositionTime)
derived verb sealed(value : Envelope)

rule first(local, when) = I <= middle(local) @ after when
rule second(local, when) = middle(local) @ after when <= completed(local) @ after when
rule remember(local, effective, when) = I <= remembered(local, effective) @ after when
rule seal(record) = I <= sealed(record) @ tau
rule request(local, when) = requests(local) @ when <= activated(local) @ when

program Chain(object : Thing, t : PropositionTime) {
    use first { local = object, when = t }
    use second { local = object, when = t }
}
program Partial = Chain(item)
program Ground = Partial(tau)
program Direct = Chain(item, tau)
program WithLocals(object : Thing) {
    use second { local = object }
}
program Temporal(object : Thing, effective : PropositionTime, t : PropositionTime) {
    use remember { local = object, effective = effective, when = t }
    use seal { record = Envelope { payload = Box { value = object, }, } }
}
program TemporalPartial = Temporal(item, later)
program TemporalGround = TemporalPartial(tau)

experiment real {
    seeds { starts(item) @ tau }
    query residual completed(item) @ after tau
}
experiment derived {
    seeds { requests(item) @ tau }
    query residual completed(item) @ after tau
}
experiment absent {
    seeds { }
    query residual completed(item) @ after tau
}
experiment negative {
    seeds { ~starts(item) @ tau }
    query residual completed(item) @ after tau
}
