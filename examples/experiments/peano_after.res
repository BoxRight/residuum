// Finite unary addition. Input spines contain no addition table.
// In this experiment only: Z = zeroTime, S(x) = after(x).
module PeanoAfter

entity Natural
entity Cursor
entity Request

const zeroTime : PropositionTime
const nZero : Natural
const nOne : Natural
const nTwo : Natural
const nThree : Natural
const nFour : Natural
const cZero : Cursor
const cOne : Cursor
const cTwo : Cursor
const cThree : Cursor
const cFour : Cursor
const r00 : Request
const r01 : Request
const r02 : Request
const r03 : Request
const r04 : Request
const r10 : Request
const r11 : Request
const r12 : Request
const r13 : Request
const r14 : Request
const r20 : Request
const r21 : Request
const r22 : Request
const r23 : Request
const r24 : Request
const r30 : Request
const r31 : Request
const r32 : Request
const r33 : Request
const r34 : Request
const r40 : Request
const r41 : Request
const r42 : Request
const r43 : Request
const r44 : Request

seeded verb start(number : Natural, cursor : Cursor)
seeded verb successor(number : Natural, cursor : Cursor, next : Cursor)
seeded verb end(number : Natural, cursor : Cursor)
seeded verb add(left : Natural, right : Natural, request : Request) = program Addition

derived verb leftWalk(left : Natural, right : Natural, request : Request, cursor : Cursor) = program LeftWalk
derived verb rightWalk(right : Natural, request : Request, cursor : Cursor) = program RightWalk
derived verb sum(request : Request)
derived verb completed(request : Request)

rule initialize(left, right, request, cursor, t) =
    add(left, right, request) @ t
    & start(left, cursor) @ zeroTime
    <= leftWalk(left, right, request, cursor) @ t

rule leftSuccessor(left, right, request, cursor, next, t) =
    leftWalk(left, right, request, cursor) @ t
    & successor(left, cursor, next) @ zeroTime
    <= leftWalk(left, right, request, next) @ after t

rule changePhase(left, right, request, cursor, first, t) =
    leftWalk(left, right, request, cursor) @ t
    & end(left, cursor) @ zeroTime
    & start(right, first) @ zeroTime
    <= rightWalk(right, request, first) @ t

rule rightSuccessor(right, request, cursor, next, t) =
    rightWalk(right, request, cursor) @ t
    & successor(right, cursor, next) @ zeroTime
    <= rightWalk(right, request, next) @ after t

rule finish(right, request, cursor, t) =
    rightWalk(right, request, cursor) @ t
    & end(right, cursor) @ zeroTime
    <= sum(request) @ t

// Completion observation stays at zeroTime; the numeric result is sum's index.
rule observe(request, t) = sum(request) @ t <= completed(request) @ zeroTime

program Addition(left : Natural, right : Natural, request : Request, t : PropositionTime) {
    use initialize { left = left, right = right, request = request, t = t }
}
program LeftWalk(left : Natural, right : Natural, request : Request, cursor : Cursor, t : PropositionTime) {
    use leftSuccessor { left = left, right = right, request = request, cursor = cursor, t = t }
    use changePhase { left = left, right = right, request = request, cursor = cursor, t = t }
}
program RightWalk(right : Natural, request : Request, cursor : Cursor, t : PropositionTime) {
    use rightSuccessor { right = right, request = request, cursor = cursor, t = t }
    use finish { right = right, request = request, cursor = cursor, t = t }
}

// Actual partial application and temporal beta reduction, using v1 syntax.
program LeftTwo = LeftWalk(nTwo)
program LeftTwoThree = LeftTwo(nThree)
program LeftGround = LeftTwoThree(r23, cZero, zeroTime)
program LeftDirect = LeftWalk(nTwo, nThree, r23, cZero, zeroTime)

experiment allPairs {
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
        add(nZero, nOne, r01) @ zeroTime,
        add(nZero, nTwo, r02) @ zeroTime,
        add(nZero, nThree, r03) @ zeroTime,
        add(nZero, nFour, r04) @ zeroTime,
        add(nOne, nZero, r10) @ zeroTime,
        add(nOne, nOne, r11) @ zeroTime,
        add(nOne, nTwo, r12) @ zeroTime,
        add(nOne, nThree, r13) @ zeroTime,
        add(nOne, nFour, r14) @ zeroTime,
        add(nTwo, nZero, r20) @ zeroTime,
        add(nTwo, nOne, r21) @ zeroTime,
        add(nTwo, nTwo, r22) @ zeroTime,
        add(nTwo, nThree, r23) @ zeroTime,
        add(nTwo, nFour, r24) @ zeroTime,
        add(nThree, nZero, r30) @ zeroTime,
        add(nThree, nOne, r31) @ zeroTime,
        add(nThree, nTwo, r32) @ zeroTime,
        add(nThree, nThree, r33) @ zeroTime,
        add(nThree, nFour, r34) @ zeroTime,
        add(nFour, nZero, r40) @ zeroTime,
        add(nFour, nOne, r41) @ zeroTime,
        add(nFour, nTwo, r42) @ zeroTime,
        add(nFour, nThree, r43) @ zeroTime,
        add(nFour, nFour, r44) @ zeroTime,
    }
    query residual completed(r23) @ zeroTime
}

experiment absent {
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
    query residual completed(r23) @ zeroTime
}

experiment negative {
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
        ~add(nTwo, nThree, r23) @ zeroTime,
    }
    query residual completed(r23) @ zeroTime
}
