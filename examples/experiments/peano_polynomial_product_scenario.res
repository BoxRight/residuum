// Explicit dense inputs and bounds only. Products, sums and fold totals are derived.
module PeanoPolynomialProductScenario
import "./peano_polynomial_product.res"

experiment principalProduct {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(after(after(zeroTime)))) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    nextFactorExponent(after(zeroTime), after(after(zeroTime))) @ zeroTime,
    nextFactorExponent(after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    exponent(after(after(after(zeroTime)))) @ zeroTime,
    exponent(after(after(after(after(zeroTime))))) @ zeroTime,
    exponent(after(after(after(after(after(zeroTime)))))) @ zeroTime,
    exponent(after(after(after(after(after(after(zeroTime))))))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
    inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
    inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
    inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
    inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
    inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
    inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
    multiplyPolynomial(p, q, r) @ zeroTime,
  }
  query residual coefficient(r, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime
}

experiment zeroProduct {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(zeroTime)) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
    inputCoefficient(zeroPolynomial, zeroTime, zeroTime) @ zeroTime,
    inputCoefficient(zeroPolynomial, after(zeroTime), zeroTime) @ zeroTime,
    multiplyPolynomial(p, zeroPolynomial, r) @ zeroTime,
  }
  query residual coefficient(r, zeroTime, zeroTime) @ zeroTime
}

experiment unitProduct {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(zeroTime)) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
    inputCoefficient(unitPolynomial, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(unitPolynomial, after(zeroTime), zeroTime) @ zeroTime,
    multiplyPolynomial(p, unitPolynomial, r) @ zeroTime,
  }
  query residual coefficient(r, after(zeroTime), after(after(zeroTime))) @ zeroTime
}

experiment gapsProduct {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(after(zeroTime))) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    nextFactorExponent(after(zeroTime), after(after(zeroTime))) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    exponent(after(after(after(zeroTime)))) @ zeroTime,
    exponent(after(after(after(after(zeroTime))))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), zeroTime) @ zeroTime,
    inputCoefficient(p, after(after(zeroTime)), after(zeroTime)) @ zeroTime,
    inputCoefficient(q, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
    inputCoefficient(q, after(after(zeroTime)), after(zeroTime)) @ zeroTime,
    multiplyPolynomial(p, q, r) @ zeroTime,
  }
  query residual coefficient(r, after(after(zeroTime)), after(after(zeroTime))) @ zeroTime
}

experiment independentProducts {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(zeroTime)) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), zeroTime) @ zeroTime,
    inputCoefficient(q, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(q, after(zeroTime), after(zeroTime)) @ zeroTime,
    inputCoefficient(gapsLeft, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(gapsLeft, after(zeroTime), after(zeroTime)) @ zeroTime,
    inputCoefficient(gapsRight, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(gapsRight, after(zeroTime), after(zeroTime)) @ zeroTime,
    multiplyPolynomial(p, q, r) @ zeroTime,
    multiplyPolynomial(gapsLeft, gapsRight, gapsResult) @ after zeroTime,
  }
  query residual coefficient(r, after(zeroTime), after(zeroTime)) @ zeroTime
}

experiment partialProduct {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(zeroTime)) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
    inputCoefficient(q, zeroTime, after(after(zeroTime))) @ zeroTime,
    inputCoefficient(q, after(zeroTime), after(zeroTime)) @ zeroTime,
    multiplyPolynomialFromP(q, r) @ zeroTime,
  }
  query residual coefficient(r, after(zeroTime), after(after(after(after(after(zeroTime)))))) @ zeroTime
}

experiment absentProduct {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(zeroTime)) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
    inputCoefficient(q, zeroTime, after(after(zeroTime))) @ zeroTime,
    inputCoefficient(q, after(zeroTime), after(zeroTime)) @ zeroTime,
  }
  query residual coefficient(r, after(zeroTime), after(after(after(after(after(zeroTime)))))) @ zeroTime
}

experiment computedProductThenSum {
  seeds {
    firstFactorExponent(zeroTime) @ zeroTime,
    lastFactorExponent(after(zeroTime)) @ zeroTime,
    nextFactorExponent(zeroTime, after(zeroTime)) @ zeroTime,
    exponent(zeroTime) @ zeroTime,
    exponent(after(zeroTime)) @ zeroTime,
    exponent(after(after(zeroTime))) @ zeroTime,
    inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
    inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
    inputCoefficient(q, zeroTime, after(after(zeroTime))) @ zeroTime,
    inputCoefficient(q, after(zeroTime), after(zeroTime)) @ zeroTime,
    inputCoefficient(zeroPolynomial, zeroTime, zeroTime) @ zeroTime,
    inputCoefficient(zeroPolynomial, after(zeroTime), zeroTime) @ zeroTime,
    inputCoefficient(zeroPolynomial, after(after(zeroTime)), zeroTime) @ zeroTime,
    multiplyPolynomial(p, q, r) @ zeroTime,
    addPolynomial(r, zeroPolynomial, secondResult) @ zeroTime,
  }
  query residual coefficient(secondResult, after(zeroTime), after(after(after(after(after(zeroTime)))))) @ zeroTime
}

