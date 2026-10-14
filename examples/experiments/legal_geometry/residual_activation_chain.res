module ResidualActivationChain

entity Thing
const item : Thing
const other : Thing
const tau : PropositionTime
const sigma : PropositionTime

seeded verb starts(object : Thing) = program Prepare
derived verb middle(object : Thing)
derived verb handoff(object : Thing) = program Finish
derived verb processed(object : Thing)
derived verb completed(object : Thing)

// I is the empty product of requirements. These definitions still need
// their associated verb instance before the executable evaluator activates them.
rule prepareFirst(local, when) = I <= middle(local) @ when
rule prepareSecond(local, when) = middle(local) @ when <= handoff(local) @ when
rule finishFirst(local, when) = I <= processed(local) @ after when
rule finishSecond(local, when) = processed(local) @ after when <= completed(local) @ after when

program Prepare(object : Thing, t : PropositionTime) {
    use prepareFirst { local = object, when = t }
    use prepareSecond { local = object, when = t }
}
program Finish(object : Thing, t : PropositionTime) {
    use finishFirst { local = object, when = t }
    use finishSecond { local = object, when = t }
}
