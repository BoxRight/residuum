module EpochT2

import "f4_t1.res"

// F4, theory T2 = T1 + after experience 2 (allergic person present; Right not
// in N). No rule of T1 derives right, so the valuation already holds: only the
// certificate of g0a is extended. The valuation stays as a constraint.
rule certG0aE2(t) = evaluatedAt(e2) @ t <= unrevisedThrough(g0a, e2) @ t

// Shared panel: identical seeds in every theory T0..T3.
experiment s1Dossier {
    seeds {
        accompanies(ana, rex, host) @ sA,
        propertyA(rex) @ sA,
        evaluatedAt(e0) @ sA,
        evaluatedAt(e1) @ sA,
        evaluatedAt(e3) @ sA,
    }
    query residual prohibition(g0, ana, enterWith, rex, host, administrative, e0) @ sA
}

experiment plainDog {
    seeds {
        accompanies(bea, fido, host) @ sB,
        ~propertyA(fido) @ sB,
        evaluatedAt(e0) @ sB,
        evaluatedAt(e1) @ sB,
        evaluatedAt(e2) @ sB,
        evaluatedAt(e3) @ sB,
    }
    query residual prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sB
}

experiment unknownA {
    seeds {
        accompanies(bea, fido, host) @ sC,
        evaluatedAt(e0) @ sC,
        evaluatedAt(e1) @ sC,
    }
    query residual prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sC
}

experiment s2Dossier {
    seeds {
        accompanies(cal, kim, host) @ sD,
        ~propertyA(kim) @ sD,
        allergicPresent(dan, host) @ sD,
        evaluatedAt(e2) @ sD,
        evaluatedAt(e3) @ sD,
    }
    query residual right(g3, cal, enterWith, kim, host, legislative, e3) @ sD
}

experiment s3Dossier {
    seeds {
        accompanies(eva, max, host) @ sE,
        ~propertyA(max) @ sE,
        serviceDog(max, eva) @ sE,
        evaluatedAt(e2) @ sE,
        evaluatedAt(e3) @ sE,
    }
    query residual right(g3, eva, enterWith, max, host, legislative, e3) @ sE
}

experiment s4Question {
    seeds {
        accompanies(eva, pip, host) @ sF,
        ~propertyA(pip) @ sF,
        serviceDog(pip, eva) @ sF,
        allergicPresent(dan, host) @ sF,
        evaluatedAt(e3) @ sF,
    }
    query residual right(g3, eva, enterWith, pip, host, legislative, e3) @ sF
}

experiment twoDogs {
    seeds {
        accompanies(ana, rex, host) @ sA,
        propertyA(rex) @ sA,
        accompanies(bea, fido, host) @ sA,
        ~propertyA(fido) @ sA,
        evaluatedAt(e1) @ sA,
    }
    query residual prohibition(g0a, bea, enterWith, fido, host, administrative, e1) @ sA
}

