// Multiplication requests and operand spines are inputs, never products.
module PeanoMultiplication
import "./peano_after.res"

// Composite invocation identities prevent different recursive sums from mixing.
entity AdditionInvocation <: Request {
    operation : Request
    position : Cursor
}

seeded verb multiply(a : Natural, b : Natural, request : Request) = program Multiply
derived verb productSoFar(a : Natural, b : Natural, request : Request, cursor : Cursor) = program MultiplyStep
derived verb awaitingSum(a : Natural, b : Natural, request : Request, next : Cursor, call : Request) = program AdditionContinuation
derived verb product(request : Request)
derived verb productCompleted(request : Request)

rule multiplicationStart(a, b, request, cursor, t) =
    multiply(a, b, request) @ t
    & start(b, cursor) @ zeroTime
    <= productSoFar(a, b, request, cursor) @ t

rule multiplicationNext(a, b, request, cursor, next, call, t) =
    productSoFar(a, b, request, cursor) @ t
    & successor(b, cursor, next) @ zeroTime
    <= awaitingSum(a, b, request, next, call) @ t

// The existing Addition adds its two operand spines to the supplied base index.
// Here it computes accumulated + a as accumulated + (zero + a).
rule requestAddition(a, b, request, next, call, t) =
    awaitingSum(a, b, request, next, call) @ t
    <= add(nZero, a, call) @ t

rule acceptAddition(a, b, request, next, call, t, resultTime) =
    awaitingSum(a, b, request, next, call) @ t
    & sum(call) @ resultTime
    <= productSoFar(a, b, request, next) @ resultTime

rule multiplicationEnd(a, b, request, cursor, t) =
    productSoFar(a, b, request, cursor) @ t
    & end(b, cursor) @ zeroTime
    <= product(request) @ t

rule observeProduct(request, t) =
    product(request) @ t <= productCompleted(request) @ zeroTime

program Multiply(a : Natural, b : Natural, request : Request, t : PropositionTime) {
    use multiplicationStart { a = a, b = b, request = request, t = t }
}
program MultiplyStep(a : Natural, b : Natural, request : Request, cursor : Cursor, t : PropositionTime) {
    use multiplicationNext {
        a = a, b = b, request = request, cursor = cursor, t = t,
        call = AdditionInvocation { operation = request, position = cursor, },
    }
    use multiplicationEnd { a = a, b = b, request = request, cursor = cursor, t = t }
}
program AdditionContinuation(a : Natural, b : Natural, request : Request, next : Cursor, call : Request, t : PropositionTime) {
    use requestAddition { a = a, b = b, request = request, next = next, call = call, t = t }
    use acceptAddition { a = a, b = b, request = request, next = next, call = call, t = t }
}

program MultiplyByFour = Multiply(nFour)
program ProductStepForFour = MultiplyStep(nFour)
program ProductStepPartial = ProductStepForFour(nFour, r44)
program ProductStepGround = ProductStepPartial(cZero, zeroTime)
program ProductStepDirect = MultiplyStep(nFour, nFour, r44, cZero, zeroTime)

experiment product00 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nZero, nZero, r00) @ zeroTime,
    }
    query residual productCompleted(r00) @ zeroTime
}

experiment product01 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nZero, nOne, r01) @ zeroTime,
    }
    query residual productCompleted(r01) @ zeroTime
}

experiment product02 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nZero, nTwo, r02) @ zeroTime,
    }
    query residual productCompleted(r02) @ zeroTime
}

experiment product03 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nZero, nThree, r03) @ zeroTime,
    }
    query residual productCompleted(r03) @ zeroTime
}

experiment product04 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nZero, nFour, r04) @ zeroTime,
    }
    query residual productCompleted(r04) @ zeroTime
}

experiment product10 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nOne, nZero, r10) @ zeroTime,
    }
    query residual productCompleted(r10) @ zeroTime
}

experiment product11 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nOne, nOne, r11) @ zeroTime,
    }
    query residual productCompleted(r11) @ zeroTime
}

experiment product12 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nOne, nTwo, r12) @ zeroTime,
    }
    query residual productCompleted(r12) @ zeroTime
}

experiment product13 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nOne, nThree, r13) @ zeroTime,
    }
    query residual productCompleted(r13) @ zeroTime
}

experiment product14 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nOne, nFour, r14) @ zeroTime,
    }
    query residual productCompleted(r14) @ zeroTime
}

experiment product20 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nTwo, nZero, r20) @ zeroTime,
    }
    query residual productCompleted(r20) @ zeroTime
}

experiment product21 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nTwo, nOne, r21) @ zeroTime,
    }
    query residual productCompleted(r21) @ zeroTime
}

experiment product22 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nTwo, nTwo, r22) @ zeroTime,
    }
    query residual productCompleted(r22) @ zeroTime
}

experiment product23 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nTwo, nThree, r23) @ zeroTime,
    }
    query residual productCompleted(r23) @ zeroTime
}

experiment product24 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nTwo, nFour, r24) @ zeroTime,
    }
    query residual productCompleted(r24) @ zeroTime
}

experiment product30 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nThree, nZero, r30) @ zeroTime,
    }
    query residual productCompleted(r30) @ zeroTime
}

experiment product31 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nThree, nOne, r31) @ zeroTime,
    }
    query residual productCompleted(r31) @ zeroTime
}

experiment product32 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nThree, nTwo, r32) @ zeroTime,
    }
    query residual productCompleted(r32) @ zeroTime
}

experiment product33 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nThree, nThree, r33) @ zeroTime,
    }
    query residual productCompleted(r33) @ zeroTime
}

experiment product34 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nThree, nFour, r34) @ zeroTime,
    }
    query residual productCompleted(r34) @ zeroTime
}

experiment product40 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nFour, nZero, r40) @ zeroTime,
    }
    query residual productCompleted(r40) @ zeroTime
}

experiment product41 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nFour, nOne, r41) @ zeroTime,
    }
    query residual productCompleted(r41) @ zeroTime
}

experiment product42 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nFour, nTwo, r42) @ zeroTime,
    }
    query residual productCompleted(r42) @ zeroTime
}

experiment product43 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nFour, nThree, r43) @ zeroTime,
    }
    query residual productCompleted(r43) @ zeroTime
}

experiment product44 {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nFour, nFour, r44) @ zeroTime,
    }
    query residual productCompleted(r44) @ zeroTime
}

experiment absentProduct {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
    }
    query residual productCompleted(r44) @ zeroTime
}

experiment negativeProduct {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        ~multiply(nFour, nFour, r44) @ zeroTime,
    }
    query residual productCompleted(r44) @ zeroTime
}

experiment concurrentProducts {
    seeds {
        start(nZero, cZero) @ zeroTime,
        end(nZero, cZero) @ zeroTime,
        start(nOne, cZero) @ zeroTime,
        successor(nOne, cZero, cOne) @ zeroTime,
        end(nOne, cOne) @ zeroTime,
        start(nTwo, cZero) @ zeroTime,
        successor(nTwo, cZero, cOne) @ zeroTime,
        successor(nTwo, cOne, cTwo) @ zeroTime,
        end(nTwo, cTwo) @ zeroTime,
        start(nThree, cZero) @ zeroTime,
        successor(nThree, cZero, cOne) @ zeroTime,
        successor(nThree, cOne, cTwo) @ zeroTime,
        successor(nThree, cTwo, cThree) @ zeroTime,
        end(nThree, cThree) @ zeroTime,
        start(nFour, cZero) @ zeroTime,
        successor(nFour, cZero, cOne) @ zeroTime,
        successor(nFour, cOne, cTwo) @ zeroTime,
        successor(nFour, cTwo, cThree) @ zeroTime,
        successor(nFour, cThree, cFour) @ zeroTime,
        end(nFour, cFour) @ zeroTime,
        multiply(nFour, nFour, r44) @ zeroTime,
        multiply(nTwo, nThree, r23) @ zeroTime,
    }
    query residual productCompleted(r44) @ zeroTime
}
