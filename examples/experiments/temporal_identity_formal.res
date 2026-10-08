// Captured constant t and remaining parameter t must retain distinct identities.
module TemporalIdentity

entity Item
const item : Item
const t : PropositionTime
const tau : PropositionTime

seeded verb A(object : Item)
seeded verb B(object : Item)
derived verb C(object : Item)
seeded verb general(first : PropositionTime, object : Item) = program template
seeded verb activateRule(object : Item) = program partial
seeded verb activateProgram(object : Item) = program CapturedTime
seeded verb inactive(object : Item) = program InactiveDefinitions

rule template(first, object, t) =
    A(object) @ first
    & B(object) @ t
    <= C(object) @ after t
rule partial = template(t)
rule fixed = partial(item, tau)
rule direct = template(t, item, tau)

program WithCapturedTime(first : PropositionTime, object : Item, t : PropositionTime) {
    use template { first = first, object = object, t = t }
}
program CapturedTime = WithCapturedTime(t)
program ProgramGround = CapturedTime(item, tau)
program ProgramDirect = WithCapturedTime(t, item, tau)
// These definitions are not mandatory rules; inactive is never seeded.
program InactiveDefinitions(object : Item, t : PropositionTime) {
    use fixed {}
    use direct {}
}

experiment correct {
    seeds {
        A(item) @ t,
        B(item) @ tau,
        activateRule(item) @ tau,
        activateProgram(item) @ tau,
    }
    query residual C(item) @ after tau
}
experiment wrongConstant {
    seeds {
        A(item) @ tau,
        B(item) @ tau,
        activateRule(item) @ tau,
        activateProgram(item) @ tau,
    }
    query residual C(item) @ after tau
}
experiment wrongParameter {
    seeds {
        A(item) @ t,
        B(item) @ t,
        activateRule(item) @ tau,
        activateProgram(item) @ tau,
    }
    query residual C(item) @ after tau
}
experiment absentTriggers {
    seeds { A(item) @ t, B(item) @ tau }
    query residual C(item) @ after tau
}
