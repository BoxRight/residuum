module ForwardProvenance

const tau : PropositionTime
seeded verb A()
seeded verb B()
derived verb shared()
derived verb result()
derived verb ready()

rule fromA(t) = A() @ t <= shared() @ t
rule fromB(t) = B() @ t <= shared() @ t
rule consequence(t) = shared() @ t <= result() @ t
rule axiom() = I <= ready() @ tau

experiment base {
    seeds { A() @ tau, }
    query residual result() @ tau
}

experiment delta {
    seeds { B() @ tau, }
    query residual result() @ tau
}
