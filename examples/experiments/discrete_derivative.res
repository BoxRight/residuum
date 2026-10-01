module DiscreteDerivative

const tau : PropositionTime
derived verb P()
derived verb Q()

rule axiom() = I <= P() @ tau
rule forward() = P() @ tau <= Q() @ tau
rule cycle() = Q() @ tau <= P() @ tau
rule conjunction() = P() @ tau & ~P() @ tau <= ~Q() @ tau
rule negative() = ~Q() @ tau <= ~P() @ tau

experiment universe {
    premises {
        P() @ tau,
        ~P() @ tau,
        Q() @ tau,
        ~Q() @ tau,
    }
    query fourFusion Q() @ tau
}
