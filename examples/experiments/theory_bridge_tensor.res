module TheoryBridgeTensor

const tau : PropositionTime
derived verb A()
derived verb B()
derived verb C()

rule combine() = A() @ tau & B() @ tau <= C() @ tau

experiment inconsistentConclusion {
    premises { A() @ tau, B() @ tau, ~C() @ tau, }
    query fourFusion C() @ tau
}
