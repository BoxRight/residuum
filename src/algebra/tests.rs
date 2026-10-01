use std::fmt::Debug;

use super::{CommutativeResiduatedLattice, Lattice, Monoid, Preorder};

pub(crate) mod evidence_pairs;
mod requirements;

// Exhaustive property checking on an explicitly finite, complete carrier.
// These checks establish the models' laws, not the semantics of TypedAST.
fn assert_laws<L: CommutativeResiduatedLattice + Debug>(carrier: &[L]) {
    let equivalent = |left: L, right: L| {
        assert!(
            left.equivalent(&right),
            "{left:?} is not equivalent to {right:?}"
        );
    };
    let unit = L::unit();
    assert!(carrier.iter().any(|value| value.equivalent(&unit)));
    for a in carrier {
        assert!(a.le(a), "reflexivity: {a:?}");
        equivalent(unit.tensor(a), a.tensor(&unit));
        assert!(unit.tensor(a).equivalent(a), "left unit: {a:?}");
        assert!(a.tensor(&unit).equivalent(a), "right unit: {a:?}");
        assert!(unit.residual(a).equivalent(a), "residual unit: {a:?}");
        assert!(a.join(a).equivalent(a), "join idempotence: {a:?}");
        assert!(a.meet(a).equivalent(a), "meet idempotence: {a:?}");
        for b in carrier {
            for result in [a.join(b), a.meet(b), a.tensor(b), a.residual(b)] {
                assert!(
                    carrier.iter().any(|value| value.equivalent(&result)),
                    "operation left the carrier: {result:?}"
                );
            }
            equivalent(a.join(b), b.join(a));
            equivalent(a.meet(b), b.meet(a));
            equivalent(a.tensor(b), b.tensor(a));
            assert!(a.join(&a.meet(b)).equivalent(a), "join absorption");
            assert!(a.meet(&a.join(b)).equivalent(a), "meet absorption");
            assert!(a.le(&b.residual(&a.tensor(b))), "adjunction unit");
            assert!(b.residual(a).tensor(b).le(a), "adjunction counit");
            for c in carrier {
                if a.le(b) && b.le(c) {
                    assert!(a.le(c), "transitivity: {a:?}, {b:?}, {c:?}");
                }
                // Universal bounds, rather than one particular join/meet formula.
                assert_eq!(a.join(b).le(c), a.le(c) && b.le(c), "least upper bound");
                assert_eq!(c.le(&a.meet(b)), c.le(a) && c.le(b), "greatest lower bound");
                equivalent(a.join(&b.join(c)), a.join(b).join(c));
                equivalent(a.meet(&b.meet(c)), a.meet(b).meet(c));
                equivalent(a.tensor(&b.tensor(c)), a.tensor(b).tensor(c));
                // Both directions, including inequalities that are false.
                assert_eq!(
                    a.tensor(b).le(c),
                    a.le(&b.residual(c)),
                    "right residuation: {a:?}, {b:?}, {c:?}"
                );
                assert_eq!(
                    a.tensor(b).le(c),
                    b.le(&a.residual(c)),
                    "left factor via commutativity: {a:?}, {b:?}, {c:?}"
                );
                equivalent(a.tensor(&b.join(c)), a.tensor(b).join(&a.tensor(c)));
                equivalent(a.join(b).tensor(c), a.tensor(c).join(&b.tensor(c)));
                equivalent(a.tensor(b).residual(c), a.residual(&b.residual(c)));
                equivalent(a.residual(&b.meet(c)), a.residual(b).meet(&a.residual(c)));
                equivalent(a.join(b).residual(c), a.residual(c).meet(&b.residual(c)));
                for d in carrier {
                    if a.le(c) && b.le(d) {
                        assert!(a.tensor(b).le(&c.tensor(d)), "tensor monotonicity");
                    }
                    if a.le(b) && c.le(d) {
                        assert!(b.residual(c).le(&a.residual(d)), "residual variance");
                    }
                }
            }
        }
    }
}

// Powerset of the commutative group Z/2Z. Bits encode membership of 0 and 1.
// Product is pointwise group addition, not intersection or union. Residuation
// is computed by its universal condition, independently of the test adjunction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GroupSubset(u8);

const GROUP_SUBSETS: [GroupSubset; 4] = [
    GroupSubset(0),
    GroupSubset(1),
    GroupSubset(2),
    GroupSubset(3),
];

impl GroupSubset {
    fn contains(self, element: u8) -> bool {
        self.0 & (1 << element) != 0
    }
}

impl Preorder for GroupSubset {
    fn le(&self, other: &Self) -> bool {
        self.0 & other.0 == self.0
    }
}

impl Lattice for GroupSubset {
    fn join(&self, other: &Self) -> Self {
        Self(self.0 | other.0)
    }
    fn meet(&self, other: &Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl Monoid for GroupSubset {
    fn unit() -> Self {
        Self(1)
    } // {0}, not the empty set or the whole group.

    fn tensor(&self, other: &Self) -> Self {
        let mut bits = 0;
        for x in 0..2 {
            for y in 0..2 {
                if self.contains(x) && other.contains(y) {
                    bits |= 1 << ((x + y) % 2);
                }
            }
        }
        Self(bits)
    }
}

impl CommutativeResiduatedLattice for GroupSubset {
    fn residual(&self, consequent: &Self) -> Self {
        let mut bits = 0;
        for x in 0..2 {
            if (0..2).all(|b| !self.contains(b) || consequent.contains((x + b) % 2)) {
                bits |= 1 << x;
            }
        }
        Self(bits)
    }
}

// Three-element Lukasiewicz chain: 0 < 1 < 2, with integral unit 2.
// A second model prevents treating the first model's representation as the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Chain(u8);

const CHAIN: [Chain; 3] = [Chain(0), Chain(1), Chain(2)];

impl Preorder for Chain {
    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0
    }
}

impl Lattice for Chain {
    fn join(&self, other: &Self) -> Self {
        Self(self.0.max(other.0))
    }
    fn meet(&self, other: &Self) -> Self {
        Self(self.0.min(other.0))
    }
}

impl Monoid for Chain {
    fn unit() -> Self {
        Self(2)
    }
    fn tensor(&self, other: &Self) -> Self {
        Self((self.0 + other.0).saturating_sub(2))
    }
}

impl CommutativeResiduatedLattice for Chain {
    fn residual(&self, consequent: &Self) -> Self {
        Self((2 - self.0 + consequent.0).min(2))
    }
}

#[test]
fn group_powerset_satisfies_all_algebraic_laws_exhaustively() {
    assert_laws(&GROUP_SUBSETS);
}

#[test]
fn lukasiewicz_chain_satisfies_all_algebraic_laws_exhaustively() {
    assert_laws(&CHAIN);
}

#[test]
fn tensor_meet_join_and_unit_remain_distinct_in_reference_models() {
    let odd = GroupSubset(2); // {1}
    assert_eq!(odd.tensor(&odd), GroupSubset::unit());
    assert_eq!(odd.meet(&odd), odd);
    assert_eq!(odd.join(&odd), odd);
    assert!(!odd.tensor(&odd).equivalent(&odd));
    assert!(!GroupSubset::unit().equivalent(&GroupSubset(0))); // Not bottom.
    assert!(!GroupSubset::unit().equivalent(&GroupSubset(3))); // Not top.
    assert!(!GroupSubset(3).le(&GroupSubset::unit()));
    assert_eq!(Chain(1).tensor(&Chain(1)), Chain(0));
    assert_eq!(Chain(1).meet(&Chain(1)), Chain(1));
    assert!(CHAIN.iter().all(|value| value.le(&Chain::unit())));
}

#[test]
fn monotone_map_need_not_have_a_right_adjoint() {
    let f = |_: &GroupSubset| GroupSubset::unit();
    for x in &GROUP_SUBSETS {
        for y in &GROUP_SUBSETS {
            if x.le(y) {
                assert!(f(x).le(&f(y)));
            }
        }
    }
    // For target bottom, no candidate g(bottom) satisfies the adjunction.
    // At x = bottom, f(x) <= bottom is false but x <= g(bottom) is true.
    let bottom = GroupSubset(0);
    assert!(GROUP_SUBSETS.iter().all(|candidate| {
        GROUP_SUBSETS
            .iter()
            .any(|x| f(x).le(&bottom) != x.le(candidate))
    }));
}
