//! Reference models only: no choice of propositional semantics for Residuum.
//! FOUR with truth meet or Arieli-Avron fusion as the residuated product.

use super::assert_laws;
use crate::algebra::{CommutativeResiduatedLattice, Lattice, Monoid, Preorder};

mod satisfaction_bridge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Evidence {
    p: bool,
    n: bool,
}

const UNKNOWN: Evidence = Evidence { p: false, n: false };
const TRUE: Evidence = Evidence { p: true, n: false };
const FALSE: Evidence = Evidence { p: false, n: true };
const BOTH: Evidence = Evidence { p: true, n: true };
const VALUES: [Evidence; 4] = [FALSE, UNKNOWN, BOTH, TRUE];

impl Evidence {
    fn le_knowledge(self, other: Self) -> bool {
        (!self.p || other.p) && (!self.n || other.n)
    }
    fn knowledge_join(self, other: Self) -> Self {
        Self {
            p: self.p || other.p,
            n: self.n || other.n,
        }
    }
    fn knowledge_meet(self, other: Self) -> Self {
        Self {
            p: self.p && other.p,
            n: self.n && other.n,
        }
    }
    fn evidential_not(self) -> Self {
        Self {
            p: self.n,
            n: self.p,
        }
    }
}

// Preorder is the truth order. Knowledge order remains a separate operation.
impl Preorder for Evidence {
    fn le(&self, other: &Self) -> bool {
        (!self.p || other.p) && (!other.n || self.n)
    }
}
impl Lattice for Evidence {
    fn join(&self, other: &Self) -> Self {
        Self {
            p: self.p || other.p,
            n: self.n && other.n,
        }
    }
    fn meet(&self, other: &Self) -> Self {
        Self {
            p: self.p && other.p,
            n: self.n || other.n,
        }
    }
}
impl Monoid for Evidence {
    fn unit() -> Self {
        TRUE
    }
    fn tensor(&self, other: &Self) -> Self {
        self.meet(other)
    }
}
impl CommutativeResiduatedLattice for Evidence {
    // B multimap C, under truth order, with truth meet as product.
    fn residual(&self, consequent: &Self) -> Self {
        Self {
            p: !self.p || consequent.p,
            n: !self.n && consequent.n,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fusion(Evidence);
pub(crate) const FUSION_VALUES: [Fusion; 4] =
    [Fusion(FALSE), Fusion(UNKNOWN), Fusion(BOTH), Fusion(TRUE)];

impl Fusion {
    pub(crate) fn label(self) -> &'static str {
        match self.0 {
            UNKNOWN => "U",
            TRUE => "T",
            FALSE => "F",
            BOTH => "B",
        }
    }
}

impl Preorder for Fusion {
    fn le(&self, other: &Self) -> bool {
        self.0.le(&other.0)
    }
}
impl Lattice for Fusion {
    fn join(&self, other: &Self) -> Self {
        Self(self.0.join(&other.0))
    }
    fn meet(&self, other: &Self) -> Self {
        Self(self.0.meet(&other.0))
    }
}
impl Monoid for Fusion {
    fn unit() -> Self {
        Self(BOTH)
    }
    fn tensor(&self, other: &Self) -> Self {
        let a = self.0;
        let b = other.0;
        Self(Evidence {
            p: a.p && b.p,
            n: (!b.p || a.n) && (!a.p || b.n),
        })
    }
}
impl CommutativeResiduatedLattice for Fusion {
    fn residual(&self, consequent: &Self) -> Self {
        let b = self.0;
        let c = consequent.0;
        Self(Evidence {
            p: (!b.p || c.p) && (!c.n || b.n),
            n: b.p && c.n,
        })
    }
}

#[test]
fn bilattice_and_evidential_not_laws_hold_exhaustively() {
    assert_ne!(UNKNOWN, FALSE);
    assert_eq!(
        UNKNOWN.evidential_not(),
        UNKNOWN,
        "absence is not negative evidence"
    );
    assert_eq!(BOTH.evidential_not(), BOTH);
    for a in VALUES {
        assert_eq!(a.evidential_not().evidential_not(), a);
        assert!(UNKNOWN.le_knowledge(a) && a.le_knowledge(BOTH));
        assert!(FALSE.le(&a) && a.le(&TRUE));
        assert!(a.le_knowledge(a));
        assert_eq!(a.knowledge_join(a), a);
        assert_eq!(a.knowledge_meet(a), a);
        for b in VALUES {
            assert_eq!(
                a.le_knowledge(b),
                a.evidential_not().le_knowledge(b.evidential_not())
            );
            assert_eq!(a.le(&b), b.evidential_not().le(&a.evidential_not()));
            assert_eq!(
                a.join(&b).evidential_not(),
                a.evidential_not().meet(&b.evidential_not())
            );
            assert_eq!(
                a.meet(&b).evidential_not(),
                a.evidential_not().join(&b.evidential_not())
            );
            assert_eq!(
                a.knowledge_join(b).evidential_not(),
                a.evidential_not().knowledge_join(b.evidential_not())
            );
            assert_eq!(
                a.knowledge_meet(b).evidential_not(),
                a.evidential_not().knowledge_meet(b.evidential_not())
            );
            assert_eq!(a.knowledge_join(b), b.knowledge_join(a));
            assert_eq!(a.knowledge_meet(b), b.knowledge_meet(a));
            assert_eq!(a.knowledge_join(a.knowledge_meet(b)), a);
            assert_eq!(a.knowledge_meet(a.knowledge_join(b)), a);
            for c in VALUES {
                if a.le_knowledge(b) && b.le_knowledge(c) {
                    assert!(a.le_knowledge(c));
                }
                assert_eq!(
                    a.knowledge_join(b).le_knowledge(c),
                    a.le_knowledge(c) && b.le_knowledge(c)
                );
                assert_eq!(
                    c.le_knowledge(a.knowledge_meet(b)),
                    c.le_knowledge(a) && c.le_knowledge(b)
                );
                assert_eq!(
                    a.knowledge_join(b.knowledge_join(c)),
                    a.knowledge_join(b).knowledge_join(c)
                );
                assert_eq!(
                    a.knowledge_meet(b.knowledge_meet(c)),
                    a.knowledge_meet(b).knowledge_meet(c)
                );
                if a.le_knowledge(b) {
                    assert!(a.join(&c).le_knowledge(b.join(&c)));
                    assert!(a.meet(&c).le_knowledge(b.meet(&c)));
                }
                if a.le(&b) {
                    assert!(a.knowledge_join(c).le(&b.knowledge_join(c)));
                    assert!(a.knowledge_meet(c).le(&b.knowledge_meet(c)));
                }
            }
        }
    }
}

#[test]
fn truth_meet_product_satisfies_all_residuated_contract_laws() {
    assert_laws(&VALUES);
}

#[test]
fn distinct_fusion_product_satisfies_all_residuated_contract_laws() {
    assert_laws(&FUSION_VALUES);
}

#[test]
fn negation_residual_compatibility_depends_on_product() {
    assert_eq!(BOTH.residual(&FALSE), UNKNOWN);
    assert_eq!(
        BOTH.evidential_not(),
        BOTH,
        "truth Boolean complement is not EvidentialNot"
    );
    for a in FUSION_VALUES {
        // In fusion, ~A = A multimap ~I. The dualizing element is BOTH, not FALSE.
        assert_eq!(
            a.residual(&Fusion(Fusion::unit().0.evidential_not())).0,
            a.0.evidential_not()
        );
        for b in FUSION_VALUES {
            assert_eq!(
                a.residual(&b).0,
                a.tensor(&Fusion(b.0.evidential_not())).0.evidential_not()
            );
        }
    }
}

#[test]
fn fusion_distinguishes_tensor_meet_and_units_but_is_not_knowledge_monotone() {
    assert_eq!(Fusion::unit(), Fusion(BOTH));
    assert_ne!(Fusion::unit().0, UNKNOWN);
    assert_eq!(Fusion(UNKNOWN).tensor(&Fusion(UNKNOWN)), Fusion(FALSE));
    assert_eq!(Fusion(UNKNOWN).meet(&Fusion(UNKNOWN)), Fusion(UNKNOWN));
    assert!(!Fusion(TRUE).le(&Fusion::unit()), "not integral");
    assert!(FALSE.le_knowledge(BOTH));
    assert_eq!(Fusion(FALSE).tensor(&Fusion(TRUE)).0, FALSE);
    assert_eq!(Fusion(BOTH).tensor(&Fusion(TRUE)).0, TRUE);
    assert!(
        !FALSE.le_knowledge(TRUE),
        "fusion can lose negative support as knowledge increases"
    );
    // Positive support of fusion alone is still monotone, unlike the whole pair.
    for a in VALUES {
        for b in VALUES {
            for c in VALUES {
                for d in VALUES {
                    if a.le_knowledge(c) && b.le_knowledge(d) {
                        assert!(
                            !Fusion(a).tensor(&Fusion(b)).0.p || Fusion(c).tensor(&Fusion(d)).0.p
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn knowledge_accumulation_product_has_no_truth_residual() {
    assert_eq!(
        UNKNOWN.knowledge_join(TRUE),
        TRUE,
        "knowledge join has UNKNOWN as unit"
    );
    // At a=truth bottom, a join_k TRUE <=_t FALSE is false. But a <=_t r
    // holds for every r, so no residual can satisfy the adjunction.
    assert!(!FALSE.knowledge_join(TRUE).le(&FALSE));
    assert!(VALUES.iter().all(|r| {
        VALUES
            .iter()
            .any(|a| a.knowledge_join(TRUE).le(&FALSE) != a.le(r))
    }));
}

#[test]
fn unknown_unit_and_knowledge_monotonicity_exclude_truth_residuation() {
    // Enumerate all 4^4 possible columns f(a)=a tensor TRUE, not 4^16 products.
    // Any residuated product with UNKNOWN as unit must have such a column.
    let mut knowledge_monotone_with_unit = 0;
    for code in 0..256 {
        let image = [0, 1, 2, 3].map(|i| VALUES[(code >> (2 * i)) & 3]);
        if image[1] != TRUE {
            continue;
        } // UNKNOWN tensor TRUE = TRUE
        if !(0..4).all(|i| {
            (0..4).all(|j| !VALUES[i].le_knowledge(VALUES[j]) || image[i].le_knowledge(image[j]))
        }) {
            continue;
        }
        knowledge_monotone_with_unit += 1;
        let has_right_adjoint = VALUES.iter().all(|target| {
            VALUES
                .iter()
                .any(|residual| (0..4).all(|i| image[i].le(target) == VALUES[i].le(residual)))
        });
        assert!(!has_right_adjoint);
    }
    assert!(
        knowledge_monotone_with_unit > 0,
        "the incompatibility comes from residuation"
    );
}

#[test]
fn empty_knowledge_projects_to_unknown_per_atom_not_logical_unit() {
    // Test-only atomic projection: signed evidence, not the constant I.
    let value = |positive: &[&str], negative: &[&str], atom: &str| Evidence {
        p: positive.contains(&atom),
        n: negative.contains(&atom),
    };
    for atom in ["P", "Q"] {
        let absent = value(&[], &[], atom);
        assert_eq!(absent, UNKNOWN);
        assert_eq!(absent.evidential_not(), UNKNOWN);
        assert_ne!(absent, Fusion::unit().0);
        assert_ne!(absent, Evidence::unit());
        assert_ne!(absent, FALSE, "knowledge bottom is not truth bottom");
    }
    assert_eq!(value(&["P"], &[], "P"), TRUE);
    assert_eq!(value(&["P"], &["P"], "P"), BOTH);
    assert_eq!(value(&["P"], &["P"], "Q"), UNKNOWN);
    assert_eq!(Fusion::unit().0, BOTH);
    assert_eq!(
        Evidence::unit(),
        TRUE,
        "unit depends on the reference model"
    );
    assert_ne!(Fusion::unit().0, FALSE);
    assert_ne!(Evidence::unit(), FALSE);
}

#[test]
fn positive_support_candidate_distinguishes_unit_unknown_and_truth_bottom() {
    // Candidate satisfaction only; this does not choose M2's semantics.
    assert!(!UNKNOWN.p);
    assert!(!UNKNOWN.evidential_not().p);
    assert!(!FALSE.p);
    assert!(Evidence::unit().p);
    assert!(Fusion::unit().0.p);
    assert!(BOTH.p && BOTH.evidential_not().p);
    for x in VALUES {
        assert_eq!(Fusion::unit().tensor(&Fusion(x)).0.p, x.p);
        assert_eq!(Evidence::unit().tensor(&x).p, x.p);
    }
}

#[test]
fn evidential_contradictions_do_not_force_truth_bottom() {
    assert_eq!(BOTH.tensor(&BOTH.evidential_not()), BOTH);
    assert!(!BOTH.le(&FALSE));
    assert_eq!(Fusion(BOTH).tensor(&Fusion(BOTH.evidential_not())).0, BOTH);
    assert!(!Fusion(BOTH).le(&Fusion(FALSE)));
    assert_ne!(Evidence::unit(), UNKNOWN);
    assert!(
        Evidence::unit().p,
        "logical unit has positive support; it is not knowledge bottom"
    );
}

#[test]
fn truth_rule_inequalities_are_not_knowledge_upward_closed() {
    assert!(TRUE.le(&TRUE), "A <=_t G initially holds");
    let more_goal_evidence = TRUE.knowledge_join(FALSE);
    assert_eq!(more_goal_evidence, BOTH);
    assert!(TRUE.le_knowledge(more_goal_evidence));
    assert!(
        !TRUE.le(&more_goal_evidence),
        "adding negative evidence can falsify the truth inequality"
    );
    assert!(
        more_goal_evidence.p,
        "Horn's positive head remains supported"
    );
}
