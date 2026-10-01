//! Algebraic contracts, independent of evidence and effect execution.
//!
//! Implementing these methods does not prove their laws. Implementations must
//! satisfy the contract in `docs/monotonic-core-laws.md`. Formula representations
//! may be preordered; their quotient by mutual entailment is the ordered carrier.
//! No current TypedAST or runtime type implements this contract yet.
//! The requirement adapter interprets query formulas in an explicitly supplied
//! model; it does not choose a propositional model or decide general entailment.

mod requirement;
pub use requirement::interpret_requirement;

/// Comparison in an exact supplied preorder, not a preference policy.
/// Below and Above are strict (one inequality holds, the reverse does not).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderComparison {
    Equivalent,
    Below,
    Above,
    Incomparable,
}

/// Reflexive, transitive semantic order, distinct from representation equality.
pub trait Preorder {
    fn le(&self, other: &Self) -> bool;

    fn equivalent(&self, other: &Self) -> bool {
        self.le(other) && other.le(self)
    }

    /// Uses this same semantic order in both directions. Only exact `le`
    /// implementations justify Incomparable; failure of proof search does not.
    fn compare(&self, other: &Self) -> OrderComparison {
        match (self.le(other), other.le(self)) {
            (true, true) => OrderComparison::Equivalent,
            (true, false) => OrderComparison::Below,
            (false, true) => OrderComparison::Above,
            (false, false) => OrderComparison::Incomparable,
        }
    }
}

/// Least upper bounds and greatest lower bounds in the same semantic order.
/// Lattice identities hold up to `Preorder::equivalent`.
pub trait Lattice: Preorder + Sized {
    fn join(&self, other: &Self) -> Self;
    fn meet(&self, other: &Self) -> Self;
}

/// Associative product with a bilateral unit; no commutativity is implied here.
pub trait Monoid: Sized {
    fn tensor(&self, other: &Self) -> Self;
    fn unit() -> Self;
}

/// A commutative, monotone monoid product on a lattice, with right residual.
///
/// `b.residual(c)` denotes B multimap C, requiring for all A, B, C:
/// `a.tensor(b).le(c) == a.le(&b.residual(c))`.
/// Commutativity also gives `a.tensor(b).le(c) == b.le(&a.residual(c))`.
/// No integrality, tensor idempotence, completeness, or fixed-point termination
/// follows merely from implementing this interface.
pub trait CommutativeResiduatedLattice: Lattice + Monoid {
    fn residual(&self, consequent: &Self) -> Self;
}

#[cfg(test)]
pub(crate) mod tests;
