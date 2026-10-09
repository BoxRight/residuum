// Finite dense convolution. Numeric work is delegated to imported Peano programs.
module PeanoPolynomialProduct
import "./peano_polynomial_addition.res"
import "./peano_multiplication.res"

entity PolynomialProductJob {
  left : Polynomial
  right : Polynomial
  result : Polynomial
  requestedAt : PropositionTime
}
entity PairDegreeCall <: Request {
  job : PolynomialProductJob
  i : PropositionTime
  j : PropositionTime
}
entity PairProductCall <: Request {
  job : PolynomialProductJob
  i : PropositionTime
  j : PropositionTime
}
entity RowSumCall <: Request {
  job : PolynomialProductJob
  k : PropositionTime
  i : PropositionTime
}
const unitPolynomial : Polynomial

// The finite input domain is a contiguous successor chain starting at zeroTime.
seeded verb firstFactorExponent(e : PropositionTime)
seeded verb nextFactorExponent(e : PropositionTime, next : PropositionTime)
seeded verb lastFactorExponent(e : PropositionTime)
derived verb factorExponent(e : PropositionTime)
seeded verb multiplyPolynomial(left : Polynomial, right : Polynomial, result : Polynomial) = program PolynomialProduct
seeded verb multiplyPolynomialFromP(right : Polynomial, result : Polynomial) = program ProductFromP
derived verb productRequest(left : Polynomial, right : Polynomial, result : Polynomial)
derived verb pairWork(job : PolynomialProductJob, i : PropositionTime, j : PropositionTime, a : PropositionTime, b : PropositionTime) = program PairWork
derived verb pairValue(job : PolynomialProductJob, i : PropositionTime, j : PropositionTime, degree : PropositionTime, value : PropositionTime)
derived verb rowCandidate(job : PolynomialProductJob, k : PropositionTime, i : PropositionTime)
derived verb rowContribution(job : PolynomialProductJob, k : PropositionTime, i : PropositionTime, value : PropositionTime)
derived verb comparePeano(a : PropositionTime, b : PropositionTime)
derived verb lessPeano(a : PropositionTime, b : PropositionTime)
derived verb productFold(job : PolynomialProductJob, k : PropositionTime, i : PropositionTime, total : PropositionTime)
derived verb foldAddition(job : PolynomialProductJob, k : PropositionTime, i : PropositionTime, total : PropositionTime, value : PropositionTime) = program FoldAddition
derived verb productAdvanced(job : PolynomialProductJob, k : PropositionTime, i : PropositionTime, total : PropositionTime)
derived verb productFinished(job : PolynomialProductJob, k : PropositionTime, total : PropositionTime)

rule factorFirst(e) = firstFactorExponent(e) @ zeroTime <= factorExponent(e) @ zeroTime
rule factorNext(e, next) = factorExponent(e) @ zeroTime & nextFactorExponent(e, next) @ zeroTime <= factorExponent(next) @ zeroTime
rule exposeProduct(left, right, result, t) = I <= productRequest(left, right, result) @ t
rule productZeroStart(left, right, result, t) = productRequest(left, right, result) @ t <= start(nZero, cZero) @ zeroTime
rule productZeroEnd(left, right, result, t) = productRequest(left, right, result) @ t <= end(nZero, cZero) @ zeroTime
rule requestPair(left, right, result, t, i, j, a, b) =
  productRequest(left, right, result) @ t & factorExponent(i) @ zeroTime & factorExponent(j) @ zeroTime
  & coefficient(left, i, a) @ zeroTime & coefficient(right, j, b) @ zeroTime
  <= pairWork(PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, }, i, j, a, b) @ zeroTime
rule requestRow(left, right, result, t, k, i) =
  productRequest(left, right, result) @ t & exponent(k) @ zeroTime & factorExponent(i) @ zeroTime
  <= rowCandidate(PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, }, k, i) @ zeroTime
rule beginProductFold(left, right, result, t, k, first) =
  productRequest(left, right, result) @ t & exponent(k) @ zeroTime & firstFactorExponent(first) @ zeroTime
  <= productFold(PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, }, k, first, zeroTime) @ zeroTime

// These I members are passive until their containing Program is activated.
rule operandStart(value) = I <= start(PeanoOperand { value = value, }, PeanoPosition { value = value, }) @ zeroTime
rule operandUnfold(value) = I <= unfoldOperand(PeanoOperand { value = value, }, value) @ zeroTime
rule requestDegree(job, i, j, a, b, t) =
  pairWork(job, i, j, a, b) @ t <= add(nZero, PeanoOperand { value = j, }, PairDegreeCall { job = job, i = i, j = j, }) @ i
rule requestPairProduct(job, i, j, a, b, t) =
  pairWork(job, i, j, a, b) @ t <= multiply(PeanoOperand { value = a, }, PeanoOperand { value = b, }, PairProductCall { job = job, i = i, j = j, }) @ zeroTime
rule acceptPair(job, i, j, a, b, t, degree, value) =
  pairWork(job, i, j, a, b) @ t
  & sum(PairDegreeCall { job = job, i = i, j = j, }) @ degree
  & product(PairProductCall { job = job, i = i, j = j, }) @ value
  <= pairValue(job, i, j, degree, value) @ zeroTime
rule presentContribution(job, i, j, k, value) =
  pairValue(job, i, j, k, value) @ zeroTime & exponent(k) @ zeroTime
  <= rowContribution(job, k, i, value) @ zeroTime

// Strict Peano comparison is itself a positive recursive relation.
rule lessZero(tail) = comparePeano(zeroTime, after(tail)) @ zeroTime <= lessPeano(zeroTime, after(tail)) @ zeroTime
rule compareTails(a, b) = comparePeano(after(a), after(b)) @ zeroTime <= comparePeano(a, b) @ zeroTime
rule liftLess(a, b) = comparePeano(after(a), after(b)) @ zeroTime & lessPeano(a, b) @ zeroTime <= lessPeano(after(a), after(b)) @ zeroTime
rule compareLower(job, k, i, t) = rowCandidate(job, k, i) @ t <= comparePeano(k, i) @ zeroTime
rule compareUpper(job, k, i, t, last, degree, value) =
  rowCandidate(job, k, i) @ t & pairValue(job, i, last, degree, value) @ zeroTime & lastFactorExponent(last) @ zeroTime
  <= comparePeano(degree, k) @ zeroTime
rule lowerZero(job, k, i, t) = rowCandidate(job, k, i) @ t & lessPeano(k, i) @ zeroTime <= rowContribution(job, k, i, zeroTime) @ zeroTime
rule upperZero(job, k, i, t, last, degree, value) =
  rowCandidate(job, k, i) @ t & pairValue(job, i, last, degree, value) @ zeroTime
  & lastFactorExponent(last) @ zeroTime & lessPeano(degree, k) @ zeroTime
  <= rowContribution(job, k, i, zeroTime) @ zeroTime

// Fold contributions in the declared row order. Known zero is a direct step.
rule skipZeroRow(job, k, i, total, t) =
  productFold(job, k, i, total) @ t & rowContribution(job, k, i, zeroTime) @ zeroTime
  <= productAdvanced(job, k, i, total) @ zeroTime
rule requestRowAddition(job, k, i, total, tail, t) =
  productFold(job, k, i, total) @ t & rowContribution(job, k, i, after(tail)) @ zeroTime
  <= foldAddition(job, k, i, total, after(tail)) @ zeroTime
rule sumRow(job, k, i, total, value, t) =
  foldAddition(job, k, i, total, value) @ t <= add(nZero, PeanoOperand { value = value, }, RowSumCall { job = job, k = k, i = i, }) @ total
rule acceptRowSum(job, k, i, total, value, t, nextTotal) =
  foldAddition(job, k, i, total, value) @ t & sum(RowSumCall { job = job, k = k, i = i, }) @ nextTotal
  <= productAdvanced(job, k, i, nextTotal) @ zeroTime
rule advanceRow(job, k, i, total, t, next) =
  productAdvanced(job, k, i, total) @ t & nextFactorExponent(i, next) @ zeroTime
  <= productFold(job, k, next, total) @ zeroTime
rule finishRows(job, k, i, total, t) =
  productAdvanced(job, k, i, total) @ t & lastFactorExponent(i) @ zeroTime
  <= productFinished(job, k, total) @ zeroTime
rule publishProduct(left, right, result, requestedAt, k, total) =
  productFinished(PolynomialProductJob { left = left, right = right, result = result, requestedAt = requestedAt, }, k, total) @ zeroTime
  <= coefficient(result, k, total) @ zeroTime

program PolynomialProduct(left : Polynomial, right : Polynomial, result : Polynomial, t : PropositionTime) {
  use exposeProduct { left = left, right = right, result = result, t = t }
  use productZeroStart { left = left, right = right, result = result, t = t }
  use productZeroEnd { left = left, right = right, result = result, t = t }
  use requestPair { left = left, right = right, result = result, t = t }
  use requestRow { left = left, right = right, result = result, t = t }
  use beginProductFold { left = left, right = right, result = result, t = t }
  use compareLower { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
  use compareUpper { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
  use lowerZero { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
  use upperZero { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
  use skipZeroRow { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
  use requestRowAddition { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
  use advanceRow { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
  use finishRows { job = PolynomialProductJob { left = left, right = right, result = result, requestedAt = t, } }
}
program PairWork(job : PolynomialProductJob, i : PropositionTime, j : PropositionTime, a : PropositionTime, b : PropositionTime, t : PropositionTime) {
  use operandStart { value = j }
  use operandUnfold { value = j }
  use operandStart { value = a }
  use operandUnfold { value = a }
  use operandStart { value = b }
  use operandUnfold { value = b }
  use requestDegree { job = job, i = i, j = j, a = a, b = b, t = t }
  use requestPairProduct { job = job, i = i, j = j, a = a, b = b, t = t }
  use acceptPair { job = job, i = i, j = j, a = a, b = b, t = t }
}
program FoldAddition(job : PolynomialProductJob, k : PropositionTime, i : PropositionTime, total : PropositionTime, value : PropositionTime, t : PropositionTime) {
  use operandStart { value = value }
  use operandUnfold { value = value }
  use sumRow { job = job, k = k, i = i, total = total, value = value, t = t }
  use acceptRowSum { job = job, k = k, i = i, total = total, value = value, t = t }
}
program ProductFromP = PolynomialProduct(p)
program ProductPQ = ProductFromP(q)
program ProductGround = ProductPQ(r, zeroTime)
program ProductDirect = PolynomialProduct(p, q, r, zeroTime)
