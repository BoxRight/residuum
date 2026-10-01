//! Structural residuation and pending requirements of the monotonic Horn fragment.
//! No residual or missing premise is asserted as evidence.

use crate::elab::Error;
use crate::typed::{Antecedent, Residual, ResidualClause, Rule, RuleBody};

mod query;
pub use query::query_residual;
mod program;
pub use program::{
    Abducible, AbductionFailure, ProgramResidualQuery, RequirementFormula, ResidualExplanation,
    ResidualProof, SearchLimits, SearchMetrics, query_program_residual,
    query_program_residual_with_abducibles,
};

#[cfg(test)]
mod tests;

/// Preserve the lambda while transforming (A & B <= C) into (A <= B multimap C).
/// The outermost conjunction is the explicit split; no reordering or flattening.
pub fn residuate(rule: &Rule) -> Result<Rule, Error> {
    let RuleBody::HornClause(clause) = &rule.body else {
        return Err(Error::new("residuate requires a Horn clause"));
    };
    let Antecedent::And(left, right) = &clause.antecedent else {
        return Err(Error::new("residuate requires a conjunctive antecedent"));
    };
    Ok(Rule {
        name: rule.name.clone(),
        parameters: rule.parameters.clone(),
        body: RuleBody::ResidualClause(ResidualClause {
            antecedent: (**left).clone(),
            consequent: Residual {
                required: (**right).clone(),
                consequent: clause.consequent.clone(),
            },
        }),
    })
}

/// Return the forward Horn view. An existing Horn lambda is unchanged.
pub fn unresiduate(rule: &Rule) -> Rule {
    match &rule.body {
        RuleBody::HornClause(_) => rule.clone(),
        RuleBody::ResidualClause(clause) => Rule {
            name: rule.name.clone(),
            parameters: rule.parameters.clone(),
            body: RuleBody::HornClause(clause.unresiduate()),
        },
    }
}
