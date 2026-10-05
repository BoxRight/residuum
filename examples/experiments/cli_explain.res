module CLIExplain

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

// Dos justificaciones del mismo hecho; explain debe conservar ambas.
experiment both {
    seeds { A() @ tau, B() @ tau, }
    query residual result() @ tau
}

experiment seedGoal {
    seeds { A() @ tau, }
    query residual A() @ tau
}

experiment unitGoal {
    seeds {}
    query residual ready() @ tau
}

experiment missing {
    seeds {}
    query residual result() @ tau
}
