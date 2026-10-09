// Contract data: no arithmetic or new language constructs.
module DynamicIncrementalPrograms

entity Token
entity Marker
entity Receipt {
    token : Token
    marker : Marker
    effective : PropositionTime
}
const alice : Token
const bob : Token
const stamp : Marker
const tau : PropositionTime

seeded verb oldSupport(token : Token)
seeded verb alternativeSupport(token : Token)
seeded verb lateSupport(token : Token)
seeded verb request(token : Token) = program Root
derived verb receipt(value : Receipt)
derived verb activated(token : Token) = program Branch
derived verb shared(token : Token)
derived verb finished(token : Token)
derived verb echo(token : Token)
derived verb branchReady(token : Token)
derived verb complemented(token : Token)

rule makeReceipt(marker, token, t) =
    I <= receipt(Receipt {
        token = token, marker = marker, effective = after(t),
    }) @ t
rule useOld(token, t) = oldSupport(token) @ t <= shared(token) @ t
rule echoRequest(token, t) = request(token) @ t <= echo(token) @ t
rule receiptActivation(token, marker, effective, t) =
    receipt(Receipt { token = token, marker = marker, effective = effective, }) @ t
    <= activated(token) @ t
rule deriveAlternative(token, t) = alternativeSupport(token) @ t <= shared(token) @ t
rule consequence(token, t) = shared(token) @ t <= finished(token) @ t
rule branchUnit(token, t) = I <= branchReady(token) @ t
rule branchOld(token, t) = oldSupport(token) @ t <= finished(token) @ t
rule waitLate(token, t) = lateSupport(token) @ t <= complemented(token) @ t

program RootOpen(marker : Marker, token : Token, t : PropositionTime) {
    use makeReceipt { marker = marker, token = token, t = t }
    use useOld { token = token, t = t }
    use echoRequest { token = token, t = t }
}
program Root = RootOpen(stamp)
program Branch(token : Token, t : PropositionTime) {
    use branchUnit { token = token, t = t }
    use branchOld { token = token, t = t }
    use waitLate { token = token, t = t }
}

experiment base {
    seeds { oldSupport(alice) @ tau, oldSupport(bob) @ tau, }
    query residual finished(alice) @ tau
}
experiment activate {
    seeds { request(alice) @ tau, }
    query residual finished(alice) @ tau
}
experiment late {
    seeds { lateSupport(alice) @ tau, }
    query residual complemented(alice) @ tau
}
experiment alternative {
    seeds { alternativeSupport(alice) @ tau, }
    query residual finished(alice) @ tau
}
experiment secondActivation {
    seeds { request(bob) @ tau, }
    query residual finished(bob) @ tau
}
experiment negativeTrigger {
    seeds { ~request(alice) @ tau, }
    query residual branchReady(alice) @ tau
}
// A preexisting snapshot for checking DeltaActivations with empty DeltaEvents.
// It is model input, not a claim that Derived facts may be seeded by the language.
experiment existingSnapshot {
    premises { request(alice) @ tau, oldSupport(alice) @ tau, alternativeSupport(alice) @ tau, }
    query fourFusion finished(alice) @ tau
}
