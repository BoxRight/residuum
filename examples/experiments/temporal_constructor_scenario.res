module TemporalConstructorScenario
import "./temporal_constructor_rules.res"

experiment computed {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        add(nTwo, nThree, r23) @ zeroTime,
    }
    query residual verified(polynomial, after(after(after(after(after(after(after(zeroTime)))))))) @ zeroTime
}
experiment constructedInput {
    seeds { inspect(after(after(zeroTime))) @ zeroTime }
    query residual observed(after(after(after(after(zeroTime))))) @ zeroTime
}
experiment empty {
    seeds { }
    query residual observed(zeroTime) @ zeroTime
}

experiment alignedValue {
    seeds { aligned(after(zeroTime)) @ after zeroTime }
    query residual alignedResult(after(zeroTime)) @ after zeroTime
}
experiment mismatchedValue {
    seeds { aligned(after(after(zeroTime))) @ after zeroTime }
    query residual alignedResult(after(zeroTime)) @ after zeroTime
}
