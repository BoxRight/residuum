//! Finite operational refinement, not an implementation of algebra::Preorder
//! or a certificate authorizing arbitrary consumers outside the tested domain.

use super::*;
use crate::surface::FormalDeclaration;
use crate::typed::{DerivedEvent, ExperimentInputKind, PropositionKind, VerbKind};
use crate::{elaborate_formal, instantiate, parse_formal};

const SPEC: &str = include_str!("../../examples/experiments/owns_refinement_spec.res");
const STRONG: &str = include_str!("../../examples/experiments/owns_refinement_strong_spec.res");
const EXACT: &str = include_str!("../../examples/experiments/owns_refinement_exact.res");
const EXTRA: &str = include_str!("../../examples/experiments/owns_refinement_extra.res");
const MISSING: &str = include_str!("../../examples/experiments/owns_refinement_missing.res");
const DOMAIN: &str = include_str!("../../examples/experiments/owns_refinement_domain.res");

mod algebra_comparison;

#[derive(Debug)]
struct Obligation {
    guarantee: String,
    premise: Event,
    expected: Event,
    actual: Closure,
    satisfied: bool,
    witness: Option<DerivedEvent>,
}

#[derive(Debug)]
struct Refinement {
    implementation: Module,
    obligations: Vec<Obligation>,
}

impl Refinement {
    fn accepted(&self) -> bool {
        !self.obligations.is_empty() && self.obligations.iter().all(|o| o.satisfied)
    }

    fn counterexample(&self) -> Option<&Obligation> {
        self.obligations.iter().find(|o| !o.satisfied)
    }
}

fn rules(module: &Module) -> Vec<Rule> {
    module
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(rule) => Some(rule.clone()),
            _ => None,
        })
        .collect()
}

fn ownership_domain() -> Vec<Event> {
    let mut signatures = parse_formal(SPEC).unwrap();
    signatures
        .declarations
        .retain(|d| !matches!(d, FormalDeclaration::Rule(_)));
    let probes = elaborate_composed_fixture(signatures, parse_formal(DOMAIN).unwrap()).unwrap();
    let input = probes
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.input_kind == ExperimentInputKind::Seeds => {
                Some(e.input.clone())
            }
            _ => None,
        })
        .unwrap();
    let closure = close(input, &rules(&probes), &mut Clock::default());
    let owns: Vec<_> = closure
        .known
        .into_iter()
        .filter(|e| e.verb == "owns")
        .collect();
    assert_eq!(owns.len(), 8);
    assert!(
        owns.iter()
            .all(|e| e.proposition.kind == PropositionKind::Derived)
    );
    owns
}

// Each specification clause is a ground-checkable owns => consequence schema.
// Its head variables must be determined by its premise; no vacuous success.
fn check_guarantee(rule: &Rule) -> Result<(), Error> {
    let RuleBody::HornClause(clause) = &rule.body else {
        return Err(Error::new("finite refinement requires Horn guarantees"));
    };
    let Antecedent::Event(premise) = &clause.antecedent else {
        return Err(Error::new("finite refinement requires one owns premise"));
    };
    let TimeExpr::At(time) = &premise.proposition.time else {
        return Err(Error::new(
            "finite refinement requires a plain premise time",
        ));
    };
    if premise.verb != "owns" || premise.polarity != Polarity::Positive {
        return Err(Error::new(
            "finite refinement requires positive owns premises",
        ));
    }
    fn root(time: &TimeExpr) -> &str {
        match time {
            TimeExpr::At(name) => name,
            TimeExpr::After(inner) => root(inner),
        }
    }
    let mut bound: BTreeSet<_> = premise
        .args
        .iter()
        .filter_map(|t| match t {
            Term::Var(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    bound.insert(time.as_str());
    if rule
        .parameters
        .iter()
        .any(|p| !bound.contains(p.name.as_str()))
        || clause
            .consequent
            .args
            .iter()
            .any(|t| matches!(t, Term::Var(name) if !bound.contains(name.as_str())))
        || !bound.contains(root(&clause.consequent.proposition.time))
    {
        return Err(Error::new("finite guarantee has unresolved variables"));
    }
    Ok(())
}

fn refine(specification: &str, body: &str, domain: &[Event]) -> Result<Refinement, Error> {
    refine_parsed(specification, parse_formal(body)?, domain)
}

fn refine_parsed(
    specification: &str,
    body: crate::surface::FormalModule,
    domain: &[Event],
) -> Result<Refinement, Error> {
    if domain.is_empty() {
        return Err(Error::new("empty refinement domain"));
    }
    let specification = parse_formal(specification)?;
    let expected = elaborate_formal(specification.clone())?;
    let guarantees = rules(&expected);
    if guarantees.is_empty() {
        return Err(Error::new("specification has no guarantees"));
    }
    for guarantee in &guarantees {
        check_guarantee(guarantee)?;
    }
    let mut signatures = specification;
    signatures
        .declarations
        .retain(|d| !matches!(d, FormalDeclaration::Rule(_)));
    for declaration in &body.declarations {
        match declaration {
            FormalDeclaration::Rule(_) => {}
            FormalDeclaration::Verb(verb)
                if verb.kind == crate::surface::SurfaceVerbKind::Derived =>
            {
                if signatures.declarations.iter().any(
                    |d| matches!(d, FormalDeclaration::Verb(public) if public.name == verb.name),
                ) {
                    return Err(Error::new("implementation cannot shadow public signatures"));
                }
            }
            _ => {
                return Err(Error::new(
                    "finite implementation permits rules and private Derived verbs only",
                ));
            }
        }
    }
    let implementation = elaborate_composed_fixture(signatures, body)?;
    if implementation
        .declarations
        .iter()
        .any(|d| matches!(d, Declaration::Verb(v) if v.kind == VerbKind::Effect))
    {
        return Err(Error::new("finite refinement excludes effects"));
    }
    let implementation_rules = rules(&implementation);
    require_acyclic(&implementation_rules)?;
    let mut obligations = Vec::new();
    for premise in domain {
        // Only the hypothetical premise and real implementation are evaluated.
        // The spec is used to construct expected goals, never inference rules.
        let actual = close(
            vec![premise.clone()],
            &implementation_rules,
            &mut Clock::default(),
        );
        for guarantee in &guarantees {
            let goal = instantiate(guarantee, premise)
                .ok_or_else(|| {
                    Error::new("guarantee does not instantiate over the declared domain")
                })?
                .event;
            let witness = actual.derivations.iter().find(|d| d.event == goal).cloned();
            let satisfied = actual.known.contains(&goal);
            obligations.push(Obligation {
                guarantee: guarantee.name.clone(),
                premise: premise.clone(),
                expected: goal,
                actual: actual.clone(),
                satisfied,
                witness,
            });
        }
    }
    Ok(Refinement {
        implementation,
        obligations,
    })
}

#[test]
fn finite_refinement_accepts_exact_and_extra_but_rejects_missing_guarantee() {
    for source in [SPEC, STRONG, EXACT, EXTRA, MISSING, DOMAIN] {
        assert_eq!(
            parse_formal(source).unwrap(),
            crate::formal::parse_formal_manual(source).unwrap()
        );
    }
    let domain = ownership_domain();
    let exact = refine(SPEC, EXACT, &domain).unwrap();
    let extra = refine(SPEC, EXTRA, &domain).unwrap();
    let missing = refine(SPEC, MISSING, &domain).unwrap();
    assert!(exact.accepted());
    assert!(extra.accepted());
    assert!(!missing.accepted());
    assert_eq!(exact.obligations.len(), 8);
    assert_eq!(extra.obligations.len(), 8);
    let guarantee = rules(&elaborate_formal(parse_formal(SPEC).unwrap()).unwrap()).remove(0);
    assert!(
        rules(&extra.implementation)
            .iter()
            .all(|r| r.body != guarantee.body),
        "acceptance no longer requires the guaranteed clause to exist syntactically"
    );
    for obligation in &extra.obligations {
        assert!(obligation.actual.known.iter().any(|e| e.verb == "insured"));
        let witness = obligation.witness.as_ref().unwrap();
        assert_eq!(witness.record.rule, "completeProtection");
        assert_eq!(witness.record.antecedents[0].verb, "entitlement");
        assert!(
            obligation
                .actual
                .derivations
                .iter()
                .all(|d| d.record.rule != "protectionGuarantee")
        );
    }
    let counter = missing.counterexample().unwrap();
    assert_eq!(counter.guarantee, "protectionGuarantee");
    assert_eq!(counter.actual.known.len(), 2);
    assert_eq!(counter.actual.known[0], counter.premise);
    assert_eq!(counter.actual.known[1].verb, "insured");
    assert!(!counter.actual.known.contains(&counter.expected));
    assert_eq!(counter.expected.verb, "protected");
    println!(
        "exact: accepts; multistep + insured: accepts; insured only: rejects\n  counterexample: {} does not derive {}",
        render_event(&counter.premise),
        render_event(&counter.expected)
    );
}

#[test]
fn finite_spec_order_points_from_stronger_guarantees_to_weaker() {
    let domain = ownership_domain();
    // Specs are also ordinary programs. Supply only their rules as bodies,
    // because the common public signatures must not be redeclared.
    fn body(source: &str) -> crate::surface::FormalModule {
        let mut parsed = parse_formal(source).unwrap();
        parsed
            .declarations
            .retain(|d| matches!(d, FormalDeclaration::Rule(_)));
        parsed
    }
    assert!(refine_parsed(SPEC, body(SPEC), &domain).unwrap().accepted());
    assert!(
        refine_parsed(STRONG, body(STRONG), &domain)
            .unwrap()
            .accepted()
    );
    assert!(
        refine_parsed(SPEC, body(STRONG), &domain)
            .unwrap()
            .accepted()
    );
    let reverse = refine_parsed(STRONG, body(SPEC), &domain).unwrap();
    assert!(!reverse.accepted());
    assert_eq!(reverse.counterexample().unwrap().expected.verb, "insured");
    assert!(refine(STRONG, EXTRA, &domain).unwrap().accepted());
    println!(
        "operational refinement orientation: strong <= weak; weak <= strong is false on the eight probes"
    );
}

#[test]
fn finite_refinement_records_scope_and_preserves_ground_substitutions() {
    let domain = ownership_domain();
    let result = refine(SPEC, EXTRA, &domain).unwrap();
    for obligation in &result.obligations {
        assert_eq!(obligation.expected.args, obligation.premise.args);
        assert_eq!(
            obligation.expected.proposition.time,
            TimeExpr::After(Box::new(obligation.premise.proposition.time.clone()))
        );
        let bindings = &obligation
            .witness
            .as_ref()
            .unwrap()
            .record
            .substitution
            .bindings;
        assert_eq!(bindings.len(), 3);
        assert!(bindings.iter().any(|b| b.parameter == "owner"
            && b.value
                == crate::typed::SubstitutionValue::Term(obligation.premise.args[0].clone())));
        assert!(bindings.iter().any(|b| b.parameter == "object"
            && b.value
                == crate::typed::SubstitutionValue::Term(obligation.premise.args[1].clone())));
        assert!(bindings.iter().any(|b| b.parameter == "t"
            && b.value
                == crate::typed::SubstitutionValue::Time(
                    obligation.premise.proposition.time.clone()
                )));
    }
    assert!(refine(SPEC, EXTRA, &[]).is_err());
    assert!(
        refine(SPEC, "module EmptyBody", &domain)
            .unwrap()
            .counterexample()
            .is_some()
    );
    let identity = SPEC.replace(
        "protected(owner, object) @ after t",
        "owns(owner, object) @ t",
    );
    let already_supported = refine(&identity, "module EmptyBody", &domain).unwrap();
    assert!(already_supported.accepted());
    assert!(
        already_supported
            .obligations
            .iter()
            .all(|o| o.witness.is_none() && o.actual.known == [o.premise.clone()]),
        "the premise can supply the guarantee without inventing a derivation"
    );
    let changed = EXACT.replace("@ after t", "@ t");
    assert!(!refine(SPEC, &changed, &domain).unwrap().accepted());
    let unbounded = format!(
        "{EXACT}\nrule repeat(owner, object, t) = protected(owner, object) @ t <= protected(owner, object) @ after t\n"
    );
    assert!(
        refine(SPEC, &unbounded, &domain)
            .unwrap_err()
            .to_string()
            .contains("acyclic")
    );
}
