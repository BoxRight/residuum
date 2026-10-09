// Bounded workloads: operand spines and requests only, never arithmetic outputs.
// The imported programs define the complete recursive computation.
module ProgramIncrementalScaling
import "./peano_multiplication.res"

experiment depth0 {
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
        add(nZero, nZero, r00) @ zeroTime,
    }
    query residual completed(r00) @ zeroTime
}

experiment depth2 {
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
        add(nTwo, nTwo, r22) @ zeroTime,
    }
    query residual completed(r22) @ zeroTime
}

experiment depth4 {
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
        add(nFour, nFour, r44) @ zeroTime,
    }
    query residual completed(r44) @ zeroTime
}

experiment jobs4 {
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
        add(nTwo, nTwo, r00) @ zeroTime,
        add(nTwo, nTwo, r01) @ zeroTime,
        add(nTwo, nTwo, r02) @ zeroTime,
        add(nTwo, nTwo, r03) @ zeroTime,
    }
    query residual completed(r00) @ zeroTime
}

experiment jobs9 {
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
        add(nTwo, nTwo, r00) @ zeroTime,
        add(nTwo, nTwo, r01) @ zeroTime,
        add(nTwo, nTwo, r02) @ zeroTime,
        add(nTwo, nTwo, r03) @ zeroTime,
        add(nTwo, nTwo, r04) @ zeroTime,
        add(nTwo, nTwo, r10) @ zeroTime,
        add(nTwo, nTwo, r11) @ zeroTime,
        add(nTwo, nTwo, r12) @ zeroTime,
        add(nTwo, nTwo, r13) @ zeroTime,
    }
    query residual completed(r00) @ zeroTime
}

experiment jobs25 {
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
        add(nTwo, nTwo, r00) @ zeroTime,
        add(nTwo, nTwo, r01) @ zeroTime,
        add(nTwo, nTwo, r02) @ zeroTime,
        add(nTwo, nTwo, r03) @ zeroTime,
        add(nTwo, nTwo, r04) @ zeroTime,
        add(nTwo, nTwo, r10) @ zeroTime,
        add(nTwo, nTwo, r11) @ zeroTime,
        add(nTwo, nTwo, r12) @ zeroTime,
        add(nTwo, nTwo, r13) @ zeroTime,
        add(nTwo, nTwo, r14) @ zeroTime,
        add(nTwo, nTwo, r20) @ zeroTime,
        add(nTwo, nTwo, r21) @ zeroTime,
        add(nTwo, nTwo, r22) @ zeroTime,
        add(nTwo, nTwo, r23) @ zeroTime,
        add(nTwo, nTwo, r24) @ zeroTime,
        add(nTwo, nTwo, r30) @ zeroTime,
        add(nTwo, nTwo, r31) @ zeroTime,
        add(nTwo, nTwo, r32) @ zeroTime,
        add(nTwo, nTwo, r33) @ zeroTime,
        add(nTwo, nTwo, r34) @ zeroTime,
        add(nTwo, nTwo, r40) @ zeroTime,
        add(nTwo, nTwo, r41) @ zeroTime,
        add(nTwo, nTwo, r42) @ zeroTime,
        add(nTwo, nTwo, r43) @ zeroTime,
        add(nTwo, nTwo, r44) @ zeroTime,
    }
    query residual completed(r00) @ zeroTime
}

experiment multiplications {
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
        multiply(nOne, nOne, r11) @ zeroTime,
        multiply(nTwo, nTwo, r22) @ zeroTime,
        multiply(nFour, nFour, r44) @ zeroTime,
    }
    query residual productCompleted(r44) @ zeroTime
}
