module SellsScenario

import "./sales.res"

const juan : Person
const pedro : Person
const car : Thing
const tau : PropositionTime

experiment sellsReal {
    seeds { sells(juan, car, pedro) @ tau, }
    query residual give(juan, car, pedro) @ after tau
}

experiment sellsAbsent {
    seeds {}
    query residual give(juan, car, pedro) @ after tau
}

experiment sellsNegative {
    seeds { ~sells(juan, car, pedro) @ tau, }
    query residual give(juan, car, pedro) @ after tau
}
