module TheoryBridge

const tau : PropositionTime
derived verb P()
derived verb Q()

rule implication() = P() @ tau <= Q() @ tau

experiment empty {
    premises {}
    query fourFusion P() @ tau
}

experiment positive {
    premises { P() @ tau, }
    query fourFusion Q() @ tau
}

experiment negative {
    premises { ~Q() @ tau, }
    query fourFusion ~P() @ tau
}

experiment both {
    premises { P() @ tau, ~P() @ tau, }
    query fourFusion Q() @ tau
}

experiment closedButNotModel {
    premises { P() @ tau, Q() @ tau, ~Q() @ tau, }
    query fourFusion ~P() @ tau
}
