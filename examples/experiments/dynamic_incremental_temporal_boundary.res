// Minimal provenance-identity counterexample, not a new temporal semantics.
module DynamicIncrementalTemporalBoundary
entity TimeBox {
    value : PropositionTime
}
const tau : PropositionTime
seeded verb run() = program Timed
seeded verb evidence(value : TimeBox)
derived verb copied()

rule copyShared(t, observed) =
    evidence(TimeBox { value = t, }) @ observed <= copied() @ t
program Timed(t : PropositionTime) {
    use copyShared { t = t }
}
experiment base {
    seeds { run() @ tau, }
    query residual copied() @ tau
}
experiment delta {
    seeds { evidence(TimeBox { value = tau, }) @ tau, }
    query residual copied() @ tau
}
