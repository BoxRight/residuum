// Boundary probe, not an implementation of polynomial addition.
module PeanoPolynomialTimeBoundary
import "./peano_after.res"

entity Polynomial
const p : Polynomial

derived verb coefficient(polynomial : Polynomial, exponent : PropositionTime, value : PropositionTime)
seeded verb suppliedTimeValue(value : PropositionTime)
derived verb copiedTimeValue(value : PropositionTime)

// Static types agree; runtime substitution must still cross TimeExpr -> Term.
rule exportCoefficient(request, t) =
    sum(request) @ t <= coefficient(p, zeroTime, t) @ zeroTime
rule exportExponent(request, t) =
    sum(request) @ t <= coefficient(p, t, zeroTime) @ zeroTime

// Positive control: this named time value is supplied as an argument, not computed.
rule copyTimeValue(value) =
    suppliedTimeValue(value) @ zeroTime <= copiedTimeValue(value) @ zeroTime

experiment computedIndex {
    seeds {
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
        suppliedTimeValue(zeroTime) @ zeroTime,
    }
    query residual coefficient(p, zeroTime, zeroTime) @ zeroTime
}
