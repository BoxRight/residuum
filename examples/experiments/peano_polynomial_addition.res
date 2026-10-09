// Dense polynomial addition over an explicit, finite exponent domain.
// All numeric values are symbolic PropositionTime terms rooted at zeroTime.
module PeanoPolynomialAddition
import "./peano_after.res"

entity Polynomial
entity PeanoOperand <: Natural { value : PropositionTime }
entity PeanoPosition <: Cursor { value : PropositionTime }
entity PolynomialSumInvocation <: Request {
    left : Polynomial
    right : Polynomial
    result : Polynomial
    exponent : PropositionTime
    requestedAt : PropositionTime
}

const p : Polynomial
const q : Polynomial
const r : Polynomial
const zeroPolynomial : Polynomial
const preserved : Polynomial
const gapsLeft : Polynomial
const gapsRight : Polynomial
const gapsResult : Polynomial
const secondResult : Polynomial

seeded verb exponent(value : PropositionTime)
seeded verb inputCoefficient(polynomial : Polynomial, exponent : PropositionTime, value : PropositionTime)
derived verb coefficient(polynomial : Polynomial, exponent : PropositionTime, value : PropositionTime)
seeded verb addPolynomial(left : Polynomial, right : Polynomial, result : Polynomial) = program PolynomialAddition
derived verb polynomialRequest(left : Polynomial, right : Polynomial, result : Polynomial)
derived verb coefficientAddition(left : Polynomial, right : Polynomial, result : Polynomial, exponent : PropositionTime, a : PropositionTime, b : PropositionTime) = program CoefficientAddition

// Constructor-to-spine adapter: decomposition is structural, not arithmetic.
derived verb unfoldOperand(number : Natural, remaining : PropositionTime)
derived verb operandStep(number : Natural, current : PropositionTime, next : PropositionTime) = program OperandSuccessor

rule receiveCoefficient(polynomial, e, value) =
    inputCoefficient(polynomial, e, value) @ zeroTime
    <= coefficient(polynomial, e, value) @ zeroTime

// Only a known request activates this I-bodied member of PolynomialAddition.
rule exposePolynomialRequest(left, right, result, t) =
    I <= polynomialRequest(left, right, result) @ t
rule provideZeroStart(left, right, result, t) =
    polynomialRequest(left, right, result) @ t <= start(nZero, cZero) @ zeroTime
rule provideZeroEnd(left, right, result, t) =
    polynomialRequest(left, right, result) @ t <= end(nZero, cZero) @ zeroTime

rule beginCoefficient(left, right, result, e, a, b, t) =
    polynomialRequest(left, right, result) @ t
    & exponent(e) @ zeroTime
    & coefficient(left, e, a) @ zeroTime
    & coefficient(right, e, b) @ zeroTime
    <= coefficientAddition(left, right, result, e, a, b) @ t

rule collectCoefficient(left, right, result, e, a, b, t, total) =
    polynomialRequest(left, right, result) @ t
    & coefficientAddition(left, right, result, e, a, b) @ t
    & sum(PolynomialSumInvocation {
        left = left, right = right, result = result, exponent = e, requestedAt = t,
    }) @ total
    <= coefficient(result, e, total) @ zeroTime

rule beginOperand(left, right, result, e, a, b, t) =
    coefficientAddition(left, right, result, e, a, b) @ t
    <= start(PeanoOperand { value = b, }, PeanoPosition { value = b, }) @ zeroTime
rule unfoldCoefficient(left, right, result, e, a, b, t) =
    coefficientAddition(left, right, result, e, a, b) @ t
    <= unfoldOperand(PeanoOperand { value = b, }, b) @ zeroTime
rule calculateCoefficient(left, right, result, e, a, b, t) =
    coefficientAddition(left, right, result, e, a, b) @ t
    <= add(nZero, PeanoOperand { value = b, }, PolynomialSumInvocation {
        left = left, right = right, result = result, exponent = e, requestedAt = t,
    }) @ a

// Peel one constructor; zero terminates. No sum facts are inputs.
rule splitOperand(number, tail) =
    unfoldOperand(number, after(tail)) @ zeroTime
    <= operandStep(number, after(tail), tail) @ zeroTime
rule finishOperand(number) =
    unfoldOperand(number, zeroTime) @ zeroTime
    <= end(number, PeanoPosition { value = zeroTime, }) @ zeroTime
rule emitOperandSuccessor(number, current, next, t) =
    operandStep(number, current, next) @ t
    <= successor(number, PeanoPosition { value = current, }, PeanoPosition { value = next, }) @ zeroTime
rule continueOperand(number, current, next, t) =
    operandStep(number, current, next) @ t
    <= unfoldOperand(number, next) @ zeroTime

program PolynomialAddition(left : Polynomial, right : Polynomial, result : Polynomial, t : PropositionTime) {
    use exposePolynomialRequest { left = left, right = right, result = result, t = t }
    use provideZeroStart { left = left, right = right, result = result, t = t }
    use provideZeroEnd { left = left, right = right, result = result, t = t }
    use beginCoefficient { left = left, right = right, result = result, t = t }
    use collectCoefficient { left = left, right = right, result = result, t = t }
}
program CoefficientAddition(left : Polynomial, right : Polynomial, result : Polynomial, e : PropositionTime, a : PropositionTime, b : PropositionTime, t : PropositionTime) {
    use beginOperand { left = left, right = right, result = result, e = e, a = a, b = b, t = t }
    use unfoldCoefficient { left = left, right = right, result = result, e = e, a = a, b = b, t = t }
    use calculateCoefficient { left = left, right = right, result = result, e = e, a = a, b = b, t = t }
}
program OperandSuccessor(number : Natural, current : PropositionTime, next : PropositionTime, t : PropositionTime) {
    use emitOperandSuccessor { number = number, current = current, next = next, t = t }
    use continueOperand { number = number, current = current, next = next, t = t }
}

// Partial application is existing source syntax, not a polynomial primitive.
program AddFromP = PolynomialAddition(p)
program AddPAndQ = AddFromP(q)
program AddPQGround = AddPAndQ(r, zeroTime)
program AddPQDirect = PolynomialAddition(p, q, r, zeroTime)
seeded verb addPolynomialFromP(right : Polynomial, result : Polynomial) = program AddFromP
