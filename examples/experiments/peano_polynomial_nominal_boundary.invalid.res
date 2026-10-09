// Deliberately invalid type-boundary probe.
module PeanoPolynomialNominalBoundary
import "./peano_after.res"
entity Polynomial
const p : Polynomial
derived verb nominalCoefficient(polynomial : Polynomial, value : Natural)
rule exportNatural(request, t) =
    sum(request) @ t <= nominalCoefficient(p, t) @ zeroTime
