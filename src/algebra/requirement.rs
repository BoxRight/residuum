use super::{Lattice, Monoid};
use crate::residual::RequirementFormula;
use crate::typed::Event;

/// Interpret a query requirement in an explicitly chosen algebraic model.
///
/// The caller supplies a valuation of logical atoms (including time/polarity),
/// respecting Event::logical_key rather than operational metadata.
/// For comparisons relative to a program, it must also establish that the
/// valuation satisfies that program's rule inequalities. No model, weakening,
/// ordering of atom names, or assumption of tensor idempotence is implicit.
/// The result is a model value, not an assertion in Known or a proof of <=_L.
/// Errors from unassigned atoms propagate rather than inventing a value.
pub fn interpret_requirement<L: Lattice + Monoid, E>(
    formula: &RequirementFormula,
    atom: &impl Fn(&Event) -> Result<L, E>,
) -> Result<L, E> {
    match formula {
        RequirementFormula::Unit => Ok(L::unit()),
        RequirementFormula::Event(event) => atom(event),
        RequirementFormula::Tensor(left, right) => {
            Ok(interpret_requirement(left, atom)?.tensor(&interpret_requirement(right, atom)?))
        }
        RequirementFormula::Join(left, right) => {
            Ok(interpret_requirement(left, atom)?.join(&interpret_requirement(right, atom)?))
        }
    }
}
