//! Guarded residual search, certified against the unchanged module-aware closure.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    Abducible, AbductionFailure, RequirementFormula, ResidualExplanation, ResidualProof,
    SearchLimits, SearchMetrics, query_program_residual_with_abducibles,
};
use crate::derive::{DerivationClock, close_program, substitute_event};
use crate::elab::Error;
use crate::programs::{compile_program, program_rule_origin, shared_parameter};
use crate::typed::*;

#[cfg(test)]
mod tests;

/// A grounded activation condition used by a search step. This is not an Event
/// asserted into Known and not a modification of the referenced Rule's body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualActivation {
    pub verb: String,
    pub behavior: BehaviorRef,
    pub rule: RuleRef,
    pub trigger: Event,
    pub substitution: Substitution,
    pub rule_origin: Option<DeclarationOrigin>,
    pub program_origin: Option<ProgramRuleOrigin>,
    /// Links the projected search step to its actual producer and association.
    pub search_rule: RuleRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifiedActiveExplanation {
    /// A proof in the guarded search view. Projected RuleRefs are resolved by
    /// activations below; this is not a proof of the original unguarded body.
    pub candidate: ResidualExplanation,
    pub activations: Vec<ResidualActivation>,
    /// Hypothetical only. These are the original evaluator's events/records,
    /// including their actual Rule/Program/Module identities and clock stamps.
    pub hypothetical: Closure,
}

/// Certified executable explanations, distinct from the Horn-only
/// [`super::ProgramResidualQuery`]. A successful return is not a completeness claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveProgramResidualQuery {
    pub goal: Event,
    /// None iff no explanation was certified. Some(Unit) is a successful
    /// explanation needing no hypotheses, not empty knowledge or a failed search.
    /// Join may retain Unit alongside pending alternatives: inspect each
    /// explanation's candidate.required rather than only testing this summary.
    pub required: Option<RequirementFormula>,
    /// Separate certified branches, including branches with identical requirements.
    pub explanations: Vec<CertifiedActiveExplanation>,
    pub metrics: SearchMetrics,
    /// Present iff explanations is empty. Search/validation errors use Err and
    /// must not be converted to None, Unit, or a partial successful result.
    pub failure: Option<AbductionFailure>,
}

/// Query executable module behavior, keeping Known immutable. Seeded hypotheses
/// are admitted in both polarities by default; only positive triggers activate.
/// Ground goals and the existing head-determined parameter restriction remain.
/// The caller supplies correctly typed, ground runtime arguments, as for the
/// Horn API; signature validation does not re-elaborate external Event values.
/// Defaults are unsupported. The caller must ensure every hypothetical closure
/// is finite, as for close_program: SearchLimits bounds search, not execution.
/// The supplied clock stamps hypothetical derivations, never residual leaves.
/// This does not replace [`super::query_program_residual`]: that API certifies
/// explicit Horn proofs without executable association guards or clock stamps.
pub fn query_active_program_residual<C: DerivationClock>(
    module: &Module,
    evidence: &[Event],
    goal: &Event,
    limits: SearchLimits,
    clock: &mut C,
) -> Result<ActiveProgramResidualQuery, Error> {
    let allowed = super::engine::default_abducibles(module);
    query_active_program_residual_with_abducibles(module, evidence, goal, &allowed, limits, clock)
}

/// Restrict the seeded hypotheses admitted by the active query, by verb/polarity.
/// An empty allowed list forbids new hypotheses; it does not prevent using Known
/// or deriving a trigger. All certification and termination preconditions of
/// [`query_active_program_residual`] still apply.
pub fn query_active_program_residual_with_abducibles<C: DerivationClock>(
    module: &Module,
    evidence: &[Event],
    goal: &Event,
    allowed: &[Abducible],
    limits: SearchLimits,
    clock: &mut C,
) -> Result<ActiveProgramResidualQuery, Error> {
    let (view, templates) = guarded_view(module)?;
    let search = query_program_residual_with_abducibles(&view, evidence, goal, allowed, limits)?;
    certify(module, evidence, goal, search, &templates, clock)
}

pub(super) fn certify<C: DerivationClock>(
    module: &Module,
    evidence: &[Event],
    goal: &Event,
    search: super::ProgramResidualQuery,
    templates: &BTreeMap<String, ResidualActivation>,
    clock: &mut C,
) -> Result<ActiveProgramResidualQuery, Error> {
    let mut metrics = search.metrics;
    let mut explanations = Vec::new();
    for candidate in search.explanations {
        let mut input = evidence.to_vec();
        leaves(&candidate.required, &mut input);
        let hypothetical = close_program(input, module, clock)?;
        if !hypothetical.known.contains(goal) {
            metrics.rejected_not_forward_supported += 1;
            continue;
        }
        let mut activations = Vec::new();
        activation_steps(&candidate.proof, templates, &mut activations);
        explanations.push(CertifiedActiveExplanation {
            candidate,
            activations,
            hypothetical,
        });
    }
    let failure =
        explanations
            .is_empty()
            .then_some(if metrics.rejected_not_forward_supported > 0 {
                AbductionFailure::NotForwardSupported
            } else {
                AbductionFailure::NotAbducible
            });
    let mut formulas = Vec::new();
    for explanation in &explanations {
        let f = RequirementFormula::from(&explanation.candidate.required);
        if !formulas.contains(&f) {
            formulas.push(f);
        }
    }
    let required = formulas
        .into_iter()
        .reduce(|a, b| RequirementFormula::Join(Box::new(a), Box::new(b)));
    Ok(ActiveProgramResidualQuery {
        goal: goal.clone(),
        required,
        explanations,
        metrics,
        failure,
    })
}

fn leaves(a: &Antecedent, out: &mut Vec<Event>) {
    match a {
        Antecedent::Unit => (),
        Antecedent::Event(e) => out.push(e.clone()),
        Antecedent::And(a, b) => {
            leaves(a, out);
            leaves(b, out);
        }
    }
}

fn activation_steps(
    proof: &ResidualProof,
    templates: &BTreeMap<String, ResidualActivation>,
    out: &mut Vec<ResidualActivation>,
) {
    match proof {
        ResidualProof::Rule {
            rule,
            substitution,
            antecedent,
            ..
        } => {
            if let Some(template) = templates.get(&rule.name) {
                let mut condition = template.clone();
                let bindings = substitution
                    .bindings
                    .iter()
                    .map(|b| (b.parameter.clone(), b.value.clone()))
                    .collect();
                condition.trigger = substitute_event(&condition.trigger, &bindings);
                condition.substitution = substitution.clone();
                out.push(condition);
            }
            activation_steps(antecedent, templates, out);
        }
        ResidualProof::Tensor(a, b) => {
            activation_steps(a, templates, out);
            activation_steps(b, templates, out);
        }
        _ => (),
    }
}

pub(super) fn guarded_view(
    module: &Module,
) -> Result<(Module, BTreeMap<String, ResidualActivation>), Error> {
    if module
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Default(_)))
    {
        return Err(Error::new(
            "active residual search requires a monotonic module without defaults",
        ));
    }
    let mut definitions = BTreeSet::new();
    let mut projected = Vec::new();
    let mut templates = BTreeMap::new();
    let names: BTreeSet<_> = module
        .declarations
        .iter()
        .map(|d| d.name().to_owned())
        .collect();
    for (association, d) in module.declarations.iter().enumerate() {
        let Declaration::Verb(v) = d else { continue };
        let Some(reference) = &v.program else {
            continue;
        };
        let (parameters, rules) = match reference.kind {
            BehaviorKind::Rule => {
                let r = module
                    .declarations
                    .iter()
                    .find_map(|d| match d {
                        Declaration::Rule(r) if r.name == reference.name => Some(r),
                        _ => None,
                    })
                    .ok_or_else(|| Error::new("unresolved associated rule"))?;
                definitions.insert(r.name.clone());
                (r.parameters.clone(), vec![r.clone()])
            }
            BehaviorKind::Program => {
                let p = module
                    .declarations
                    .iter()
                    .find_map(|d| match d {
                        Declaration::Program(p) if p.identity == reference.name => Some(p),
                        _ => None,
                    })
                    .ok_or_else(|| Error::new("unresolved associated program"))?;
                definitions.extend(p.rule_uses.iter().map(|u| u.rule.name.clone()));
                (
                    p.parameters
                        .iter()
                        .map(|p_arg| Arg {
                            name: shared_parameter(p, &p_arg.name),
                            ty: p_arg.ty.clone(),
                        })
                        .collect(),
                    compile_program(p, module)?,
                )
            }
        };
        let expected: Vec<_> = v
            .args
            .iter()
            .map(|a| a.ty.clone())
            .chain([Type::PropositionTime])
            .collect();
        if parameters.iter().map(|p| p.ty.clone()).collect::<Vec<_>>() != expected {
            return Err(Error::new(format!(
                "incompatible associated behavior {}",
                reference.name
            )));
        }
        // As in forward preparation, Effect-associated definitions are not
        // ordinary rules, and Effect events do not activate these definitions.
        if !matches!(v.kind, VerbKind::Seeded | VerbKind::Derived) {
            continue;
        }
        let trigger = Event {
            verb: v.name.clone(),
            polarity: Polarity::Positive,
            args: parameters[..v.args.len()]
                .iter()
                .map(|p| Term::Var(p.name.clone()))
                .collect(),
            proposition: PropositionType {
                kind: if v.kind == VerbKind::Seeded {
                    PropositionKind::Seeded
                } else {
                    PropositionKind::Derived
                },
                time: TimeExpr::Var(parameters.last().unwrap().name.clone()),
            },
            transition: None,
        };
        for (index, mut r) in rules.into_iter().enumerate() {
            let producer = r.name.clone();
            // Not a source identifier; association identity prevents accidental
            // merging when two verbs reference the same Rule or Program.
            r.name = format!("@residual:{association}:{index}:{producer}");
            if names.contains(&r.name) {
                return Err(Error::new("projected residual rule name collision"));
            }
            let mut clause = match &r.body {
                RuleBody::HornClause(c) => c.clone(),
                RuleBody::ResidualClause(c) => c.unresiduate(),
            };
            let mut atoms = Vec::new();
            leaves(&clause.antecedent, &mut atoms);
            if !atoms.contains(&trigger) {
                clause.antecedent = match clause.antecedent {
                    Antecedent::Unit => Antecedent::Event(trigger.clone()),
                    other => Antecedent::And(
                        Box::new(Antecedent::Event(trigger.clone())),
                        Box::new(other),
                    ),
                };
            }
            r.body = RuleBody::HornClause(clause);
            let program_origin = program_rule_origin(module, &producer);
            let rule_origin = program_origin
                .as_ref()
                .map(|p| p.rule_origin.clone())
                .or_else(|| module.origin(&producer).cloned());
            templates.insert(
                r.name.clone(),
                ResidualActivation {
                    verb: v.name.clone(),
                    behavior: reference.clone(),
                    rule: RuleRef { name: producer },
                    trigger: trigger.clone(),
                    substitution: Substitution {
                        bindings: Vec::new(),
                    },
                    rule_origin,
                    program_origin,
                    search_rule: RuleRef {
                        name: r.name.clone(),
                    },
                },
            );
            projected.push(r);
        }
    }
    let mut view = module.clone();
    view.declarations
        .retain(|d| !matches!(d, Declaration::Rule(r) if definitions.contains(&r.name)));
    for r in projected {
        if let Some(origin) = templates[&r.name].rule_origin.clone() {
            view.origins.insert(r.name.clone(), origin);
        }
        view.declarations.push(Declaration::Rule(r));
    }
    Ok((view, templates))
}
