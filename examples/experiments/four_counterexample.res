module FourCounterexample

const tau : PropositionTime

derived verb P()
derived verb Q()

rule implication() =
    P() @ tau <= Q() @ tau

experiment contraposition {
    premises {
        ~Q() @ tau,
    }
    query fourFusion ~P() @ tau
}
