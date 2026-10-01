//! Finite, test-only Known -> FOUR -> formula evaluation experiment.
//! Pointwise truth inequalities and forward support are separate checks.

use std::collections::BTreeSet;

use super::{BOTH, Evidence, FALSE, Fusion, TRUE, UNKNOWN};
use crate::algebra::{CommutativeResiduatedLattice, Lattice, Monoid, Preorder};
use crate::derive::match_antecedents;
use crate::typed::{
    Antecedent, DerivationTime, Event, HornClause, Polarity, PropositionKind, PropositionType,
    Rule, RuleBody, TimeExpr,
};
use crate::{DerivationClock, close};

mod model_entailment;

// Knowledge has signed events, never a Formula::Unit.
type Known = BTreeSet<Event>;

#[derive(Clone, Copy, Debug)]
enum Model {
    Meet,
    Fusion,
}

impl Model {
    fn unit(self) -> Evidence {
        match self {
            Self::Meet => Evidence::unit(),
            Self::Fusion => Fusion::unit().0,
        }
    }
    fn tensor(self, a: Evidence, b: Evidence) -> Evidence {
        match self {
            Self::Meet => a.tensor(&b),
            Self::Fusion => Fusion(a).tensor(&Fusion(b)).0,
        }
    }
    fn residual(self, b: Evidence, c: Evidence) -> Evidence {
        match self {
            Self::Meet => b.residual(&c),
            Self::Fusion => Fusion(b).residual(&Fusion(c)).0,
        }
    }
}

#[derive(Clone, Debug)]
enum Formula {
    Atom(Event),
    Unit,
    Bottom,
    Not(Box<Self>),
    Tensor(Box<Self>, Box<Self>),
    Join(Box<Self>, Box<Self>),
    Meet(Box<Self>, Box<Self>),
    Residual(Box<Self>, Box<Self>),
}

fn atom(name: &str) -> Event {
    Event {
        verb: name.into(),
        args: vec![],
        polarity: Polarity::Positive,
        proposition: PropositionType {
            kind: if ["A", "B", "P"].contains(&name) {
                PropositionKind::Seeded
            } else {
                PropositionKind::Derived
            },
            time: TimeExpr::At("tau".into()),
        },
        transition: None,
    }
}

fn opposite(event: &Event) -> Event {
    let mut result = event.clone();
    result.polarity = match event.polarity {
        Polarity::Positive => Polarity::EvidentialNot,
        Polarity::EvidentialNot => Polarity::Positive,
    };
    result
}

fn atomic_value(known: &Known, event: &Event) -> Evidence {
    Evidence {
        p: known.contains(event),
        n: known.contains(&opposite(event)),
    }
}

fn evaluate(model: Model, known: &Known, formula: &Formula) -> Evidence {
    match formula {
        Formula::Atom(event) => atomic_value(known, event),
        Formula::Unit => model.unit(),
        Formula::Bottom => FALSE,
        Formula::Not(inner) => evaluate(model, known, inner).evidential_not(),
        Formula::Tensor(a, b) => model.tensor(evaluate(model, known, a), evaluate(model, known, b)),
        Formula::Join(a, b) => evaluate(model, known, a).join(&evaluate(model, known, b)),
        Formula::Meet(a, b) => evaluate(model, known, a).meet(&evaluate(model, known, b)),
        Formula::Residual(b, c) => {
            model.residual(evaluate(model, known, b), evaluate(model, known, c))
        }
    }
}

fn truth_inequality(model: Model, known: &Known, a: &Formula, c: &Formula) -> bool {
    evaluate(model, known, a).le(&evaluate(model, known, c))
}

fn formula(antecedent: &Antecedent) -> Formula {
    match antecedent {
        Antecedent::Unit => Formula::Unit,
        Antecedent::Event(event) => Formula::Atom(event.clone()),
        Antecedent::And(a, b) => Formula::Tensor(Box::new(formula(a)), Box::new(formula(b))),
    }
}

fn known_from_mask(mask: usize, atoms: &[Event]) -> Known {
    let mut known = Known::new();
    for (i, event) in atoms.iter().enumerate() {
        if mask & (1 << (2 * i)) != 0 {
            known.insert(event.clone());
        }
        if mask & (1 << (2 * i + 1)) != 0 {
            known.insert(opposite(event));
        }
    }
    known
}

fn ground_rule(name: &str, antecedent: Antecedent, consequent: Event) -> Rule {
    Rule {
        name: name.into(),
        parameters: vec![],
        body: RuleBody::HornClause(HornClause {
            antecedent,
            consequent,
        }),
    }
}

#[derive(Default)]
struct Clock(usize);
impl DerivationClock for Clock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}

// Independent valuation-based oracle for this ground signed Horn fragment.
// It never invokes matching, instantiation or the production closure.
fn support_closure(model: Model, input: &Known, rules: &[Rule]) -> Known {
    let mut known = input.clone();
    for _ in 0..=8 {
        let mut next = known.clone();
        for rule in rules {
            assert!(rule.parameters.is_empty());
            let RuleBody::HornClause(clause) = &rule.body else {
                panic!("ground Horn only")
            };
            if evaluate(model, &known, &formula(&clause.antecedent)).p {
                next.insert(clause.consequent.clone());
            }
        }
        if next == known {
            return known;
        }
        known = next;
    }
    panic!("finite experiment exceeded its eight signed atoms");
}

#[test]
fn four_knowledge_states_value_atoms_without_conflating_unit() {
    let p = atom("P");
    for (mask, expected) in [(0, UNKNOWN), (1, TRUE), (2, FALSE), (3, BOTH)] {
        let known = known_from_mask(mask, std::slice::from_ref(&p));
        let snapshot = known.clone();
        for model in [Model::Meet, Model::Fusion] {
            let positive = Formula::Atom(p.clone());
            let negative = Formula::Not(Box::new(positive.clone()));
            assert_eq!(evaluate(model, &known, &positive), expected);
            assert_eq!(
                evaluate(model, &known, &negative),
                expected.evidential_not()
            );
            assert_eq!(
                evaluate(model, &known, &Formula::Atom(opposite(&p))),
                expected.evidential_not()
            );
            assert_eq!(evaluate(model, &known, &Formula::Unit), model.unit());
            assert!(evaluate(model, &known, &Formula::Unit).p);
            assert_eq!(evaluate(model, &known, &Formula::Bottom), FALSE);
        }
        assert_eq!(known, snapshot, "valuation does not assert evidence");
    }
}

#[test]
fn formula_evaluation_preserves_pointwise_residuation_in_both_models() {
    let atoms = [atom("A"), atom("B"), atom("C")];
    let [a, b, c] = atoms.clone().map(Formula::Atom);
    let product = Formula::Tensor(Box::new(a.clone()), Box::new(b.clone()));
    let residual = Formula::Residual(Box::new(b.clone()), Box::new(c.clone()));
    let join = Formula::Join(Box::new(a.clone()), Box::new(b.clone()));
    let meet = Formula::Meet(Box::new(a.clone()), Box::new(b.clone()));
    for mask in 0..64 {
        let known = known_from_mask(mask, &atoms);
        let av = atomic_value(&known, &atoms[0]);
        let bv = atomic_value(&known, &atoms[1]);
        for model in [Model::Meet, Model::Fusion] {
            assert_eq!(
                evaluate(model, &known, &join),
                Evidence {
                    p: av.p || bv.p,
                    n: av.n && bv.n
                }
            );
            assert_eq!(
                evaluate(model, &known, &meet),
                Evidence {
                    p: av.p && bv.p,
                    n: av.n || bv.n
                }
            );
            assert_eq!(evaluate(model, &known, &product).p, av.p && bv.p);
            assert_eq!(
                truth_inequality(model, &known, &product, &c),
                truth_inequality(model, &known, &a, &residual)
            );
        }
    }
}

#[test]
fn adding_evidence_is_pointwise_knowledge_monotone() {
    let atoms = [atom("A"), atom("B"), atom("C")];
    for small in 0..64 {
        for large in 0..64 {
            let f = known_from_mask(small, &atoms);
            let g = known_from_mask(large, &atoms);
            assert_eq!(
                f.is_subset(&g),
                atoms
                    .iter()
                    .all(|a| atomic_value(&f, a).le_knowledge(atomic_value(&g, a)))
            );
            if f.is_subset(&g) {
                for atom in &atoms {
                    assert!(
                        atomic_value(&f, &opposite(atom))
                            .le_knowledge(atomic_value(&g, &opposite(atom)))
                    );
                }
            }
        }
    }
}

#[test]
fn atomic_knowledge_monotonicity_does_not_extend_to_all_fusion_formulas() {
    let a = atom("A");
    let b = atom("B");
    let f = Known::from([opposite(&a), b.clone()]);
    let mut g = f.clone();
    g.insert(a.clone());
    let product = Formula::Tensor(Box::new(Formula::Atom(a)), Box::new(Formula::Atom(b)));
    assert!(f.is_subset(&g));
    assert_eq!(evaluate(Model::Fusion, &f, &product), FALSE);
    assert_eq!(evaluate(Model::Fusion, &g, &product), TRUE);
    assert!(
        !evaluate(Model::Fusion, &f, &product).le_knowledge(evaluate(Model::Fusion, &g, &product))
    );
    let negated_product = Formula::Not(Box::new(product));
    assert!(evaluate(Model::Fusion, &f, &negated_product).p);
    assert!(!evaluate(Model::Fusion, &g, &negated_product).p);
}

#[test]
fn positive_support_matches_ground_signed_antecedents_including_unit() {
    let atoms = [atom("A"), atom("B"), atom("C")];
    let mut antecedents = vec![Antecedent::Unit];
    for a in &atoms {
        antecedents.push(Antecedent::Event(a.clone()));
        antecedents.push(Antecedent::Event(opposite(a)));
    }
    let leaves = antecedents.clone();
    for a in &leaves {
        for b in &leaves {
            antecedents.push(Antecedent::And(Box::new(a.clone()), Box::new(b.clone())));
        }
    }
    for mask in 0..64 {
        let known = known_from_mask(mask, &atoms);
        let events: Vec<_> = known.iter().cloned().collect();
        for antecedent in &antecedents {
            let matched = !match_antecedents(antecedent, &events, &[]).is_empty();
            for model in [Model::Meet, Model::Fusion] {
                assert_eq!(evaluate(model, &known, &formula(antecedent)).p, matched);
            }
        }
    }
}

#[test]
fn support_closure_equals_existing_close_and_is_knowledge_monotone() {
    let atoms = [atom("A"), atom("B"), atom("C"), atom("D")];
    // Eight polarity choices for A & B -> C, then C -> D.
    for signs in 0..8 {
        let [a, b, c] = std::array::from_fn::<_, 3, _>(|i| {
            if signs & (1 << i) == 0 {
                atoms[i].clone()
            } else {
                opposite(&atoms[i])
            }
        });
        let rules = [
            ground_rule(
                "combine",
                Antecedent::And(
                    Box::new(Antecedent::Event(a)),
                    Box::new(Antecedent::Event(b)),
                ),
                c.clone(),
            ),
            ground_rule("continue", Antecedent::Event(c), atoms[3].clone()),
        ];
        let mut closures = Vec::new();
        for mask in 0..256 {
            let known = known_from_mask(mask, &atoms);
            let before = known.clone();
            let actual: Known = close(
                known.iter().cloned().collect(),
                &rules,
                &mut Clock::default(),
            )
            .known
            .into_iter()
            .collect();
            for model in [Model::Meet, Model::Fusion] {
                assert_eq!(support_closure(model, &known, &rules), actual);
                assert_eq!(
                    support_closure(model, &actual, &rules),
                    actual,
                    "idempotence"
                );
            }
            assert!(known.is_subset(&actual), "extensivity");
            assert_eq!(known, before);
            closures.push(actual);
        }
        for small in 0..256 {
            for large in 0..256 {
                if small & large == small {
                    assert!(
                        closures[small].is_subset(&closures[large]),
                        "closure is knowledge monotone"
                    );
                }
            }
        }
    }
    let axiom = [ground_rule("axiom", Antecedent::Unit, atoms[2].clone())];
    let empty = Known::new();
    let actual: Known = close(vec![], &axiom, &mut Clock::default())
        .known
        .into_iter()
        .collect();
    for model in [Model::Meet, Model::Fusion] {
        assert_eq!(support_closure(model, &empty, &axiom), actual);
    }
    assert_eq!(actual, Known::from([atoms[2].clone()]));
}

#[test]
fn forward_closed_knowledge_need_not_satisfy_pointwise_truth_rule() {
    let [a, b, c] = [atom("A"), atom("B"), atom("C")];
    let antecedent = Antecedent::And(
        Box::new(Antecedent::Event(a.clone())),
        Box::new(Antecedent::Event(b.clone())),
    );
    let rule = ground_rule("combine", antecedent.clone(), c.clone());
    let known = Known::from([a, b]);
    let closed: Known = close(
        known.iter().cloned().collect(),
        std::slice::from_ref(&rule),
        &mut Clock::default(),
    )
    .known
    .into_iter()
    .collect();
    let mut with_opposition = closed.clone();
    with_opposition.insert(opposite(&c));
    let reclosed: Known = close(
        with_opposition.iter().cloned().collect(),
        &[rule],
        &mut Clock::default(),
    )
    .known
    .into_iter()
    .collect();
    assert_eq!(reclosed, with_opposition);
    for model in [Model::Meet, Model::Fusion] {
        let body = formula(&antecedent);
        let head = Formula::Atom(c.clone());
        assert!(truth_inequality(model, &closed, &body, &head));
        assert!(!truth_inequality(model, &reclosed, &body, &head));
        assert!(evaluate(model, &reclosed, &body).p && evaluate(model, &reclosed, &head).p);
    }
    // In fusion, the positive support of an implication itself tests the full
    // truth inequality, including the negative coordinate. This differs from
    // preservation of a positively supported head by the Horn engine.
    let implication = Formula::Residual(Box::new(formula(&antecedent)), Box::new(Formula::Atom(c)));
    assert!(evaluate(Model::Fusion, &closed, &implication).p);
    assert!(!evaluate(Model::Fusion, &reclosed, &implication).p);
    assert!(evaluate(Model::Meet, &reclosed, &implication).p);
}
