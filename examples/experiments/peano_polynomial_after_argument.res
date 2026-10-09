// Former v1 source boundary, now valid on experiment-peano-after.
module PeanoPolynomialAfterArgument
entity Polynomial
const p : Polynomial
const zeroTime : PropositionTime
derived verb coefficient(polynomial : Polynomial, exponent : PropositionTime, value : PropositionTime)
rule constructedArgument(t) =
    I <= coefficient(p, zeroTime, after(t)) @ zeroTime
