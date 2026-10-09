// Inputs are coefficients and requests; query goals are observations, not seeds.
module PeanoPolynomialScenario
import "./peano_polynomial_addition.res"

experiment principal {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
        inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(p, q, r) @ zeroTime,
    }
    query residual coefficient(r, after(after(zeroTime)), after(after(after(after(after(after(zeroTime))))))) @ zeroTime
}

experiment zeroRight {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(zeroPolynomial, zeroTime, zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(p, zeroPolynomial, preserved) @ zeroTime,
    }
    query residual coefficient(preserved, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime
}

experiment zeroLeft {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(zeroPolynomial, zeroTime, zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(zeroPolynomial, p, preserved) @ zeroTime,
    }
    query residual coefficient(preserved, zeroTime, after(zeroTime)) @ zeroTime
}

experiment bothZero {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(zeroPolynomial, zeroTime, zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(zeroPolynomial, zeroPolynomial, preserved) @ zeroTime,
    }
    query residual coefficient(preserved, after(zeroTime), zeroTime) @ zeroTime
}

experiment intermediateZeros {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(gapsLeft, zeroTime, after(after(zeroTime))) @ zeroTime,
        inputCoefficient(gapsLeft, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(gapsLeft, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(gapsLeft, after(after(after(zeroTime))), after(zeroTime)) @ zeroTime,
        inputCoefficient(gapsRight, zeroTime, zeroTime) @ zeroTime,
        inputCoefficient(gapsRight, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(gapsRight, after(after(zeroTime)), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(gapsRight, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(gapsLeft, gapsRight, gapsResult) @ zeroTime,
    }
    query residual coefficient(gapsResult, after(zeroTime), zeroTime) @ zeroTime
}

experiment concurrent {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
        inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        inputCoefficient(gapsLeft, zeroTime, after(after(zeroTime))) @ zeroTime,
        inputCoefficient(gapsLeft, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(gapsLeft, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(gapsLeft, after(after(after(zeroTime))), after(zeroTime)) @ zeroTime,
        inputCoefficient(gapsRight, zeroTime, zeroTime) @ zeroTime,
        inputCoefficient(gapsRight, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(gapsRight, after(after(zeroTime)), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(gapsRight, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(p, q, r) @ zeroTime,
        addPolynomial(gapsLeft, gapsRight, gapsResult) @ zeroTime,
    }
    query residual coefficient(r, zeroTime, after(after(after(after(after(after(zeroTime))))))) @ zeroTime
}

experiment partial {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
        inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomialFromP(q, r) @ zeroTime,
    }
    query residual coefficient(r, after(after(zeroTime)), after(after(after(after(after(after(zeroTime))))))) @ zeroTime
}

experiment absent {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
        inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
    }
    query residual coefficient(r, after(after(zeroTime)), after(after(after(after(after(after(zeroTime))))))) @ zeroTime
}

experiment missingCoefficient {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(p, q, r) @ zeroTime,
    }
    query residual coefficient(r, after(zeroTime), after(after(zeroTime))) @ zeroTime
}

experiment distinctRequestTimes {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
        inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(p, q, r) @ zeroTime,
        addPolynomial(q, p, secondResult) @ after zeroTime,
    }
    query residual coefficient(secondResult, after(after(zeroTime)), after(after(after(after(after(after(zeroTime))))))) @ zeroTime
}

experiment chained {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(zeroTime)) @ zeroTime,
        inputCoefficient(p, after(zeroTime), after(after(zeroTime))) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), after(after(after(after(zeroTime))))) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(zeroTime)))))) @ zeroTime,
        inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, zeroTime, zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(zeroPolynomial, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(p, q, r) @ zeroTime,
        addPolynomial(r, zeroPolynomial, secondResult) @ zeroTime,
    }
    query residual coefficient(secondResult, zeroTime, after(after(after(after(after(after(zeroTime))))))) @ zeroTime
}

// No nNine/nEleven spines are supplied; rules construct them from the coefficients.
experiment largerCoefficients {
    seeds {
        exponent(zeroTime) @ zeroTime,
        exponent(after(zeroTime)) @ zeroTime,
        exponent(after(after(zeroTime))) @ zeroTime,
        exponent(after(after(after(zeroTime)))) @ zeroTime,
        inputCoefficient(p, zeroTime, after(after(after(after(after(after(after(after(after(zeroTime)))))))))) @ zeroTime,
        inputCoefficient(p, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(p, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(p, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        inputCoefficient(q, zeroTime, after(after(after(after(after(after(after(after(after(after(after(zeroTime)))))))))))) @ zeroTime,
        inputCoefficient(q, after(zeroTime), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(zeroTime)), zeroTime) @ zeroTime,
        inputCoefficient(q, after(after(after(zeroTime))), zeroTime) @ zeroTime,
        addPolynomial(p, q, r) @ zeroTime,
    }
    query residual coefficient(r, zeroTime, after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(after(zeroTime))))))))))))))))))))) @ zeroTime
}
