//! Compatibility proof for the proposed shared-binder compound body.
//! Existing Rule bodies remain separate; no compound AST is implemented here.
use super::*;
use residuum::typed::{Antecedent, Arg, Rule, RuleBody, Term, Type};
use std::path::Path;

fn rule<'a>(module: &'a residuum::typed::Module, name: &str) -> &'a Rule {
    module
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Rule(rule) if rule.name == name => Some(rule),
            _ => None,
        })
        .unwrap()
}

fn relation(rule: &Rule) -> (&Event, &Event) {
    let RuleBody::HornClause(clause) = &rule.body else {
        panic!("expected Horn relation")
    };
    let Antecedent::Event(antecedent) = &clause.antecedent else {
        panic!("expected single antecedent")
    };
    (antecedent, &clause.consequent)
}

#[test]
fn partial_beta_is_compatible_with_each_relation_of_a_shared_parameter_body() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let module = residuum::load_formal_module(
        root.join("examples/legal/modules/default_comparison/partial_normal_sale_scenario.res"),
    )
    .unwrap();
    let expected_parameters = vec![
        Arg {
            name: "object".into(),
            ty: Type::Entity("Thing".into()),
        },
        Arg {
            name: "buyer".into(),
            ty: Type::Entity("Person".into()),
        },
    ];
    let mut report = String::from(
        "Partial beta compatibility, not an implemented compound Rule.\nShared original parameters: t, seller, object, buyer.\nApplied prefix: t=tau, seller=juan.\nResidual binders: object:Thing, buyer:Person (values pending, not globally free variables).\n",
    );
    for (base, partial, completed, direct, before, after) in [
        (
            "normalEntry",
            "entryForJuan",
            "completedEntry",
            "directEntry",
            "saleBasis(juan, object, buyer) @ tau",
            "normalChosen(juan, object, buyer) @ after(tau)",
        ),
        (
            "normalDuty",
            "dutyForJuan",
            "completedDuty",
            "directDuty",
            "normalChosen(juan, object, buyer) @ after(tau)",
            "obligation(duty, juan, transfer, object, buyer, private, effectiveDate) @ after(tau)",
        ),
    ] {
        let original = rule(&module, base);
        let specialized = rule(&module, partial);
        assert_eq!(specialized.parameters, expected_parameters);
        assert_eq!(original.parameters[2..], specialized.parameters);
        let (antecedent, consequent) = relation(specialized);
        assert_eq!(render_event(antecedent), before);
        assert_eq!(render_event(consequent), after);
        for event in [antecedent, consequent] {
            // Both remaining variables occur in both sides of both relations.
            // Applied seller is always a constant; no dangling old binder remains.
            let variables: Vec<_> = event
                .args
                .iter()
                .filter_map(|term| match term {
                    Term::Var(name) => Some(name.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(variables, vec!["object", "buyer"]);
            assert!(event.args.contains(&Term::Const("juan".into())));
            assert!(!event.args.contains(&Term::Var("seller".into())));
        }
        let completed = rule(&module, completed);
        let direct = rule(&module, direct);
        assert!(completed.parameters.is_empty() && direct.parameters.is_empty());
        assert_eq!(
            completed.body, direct.body,
            "successive partial applications equal the direct full application"
        );
        for event in [relation(completed).0, relation(completed).1] {
            assert!(event.args.iter().all(|term| !matches!(term, Term::Var(_))));
        }
        // Elaboration does not mutate the original lambda definition.
        assert_eq!(original.parameters.len(), 4);
        assert!(
            relation(original)
                .0
                .args
                .contains(&Term::Var("seller".into()))
        );
        writeln!(report, "{partial}(object,buyer): {before} <= {after}").unwrap();
        writeln!(
            report,
            "  staged full application = direct application; original definition intact."
        )
        .unwrap();
    }
    // B has exactly the same typed representation on both sides of the chain.
    assert_eq!(
        relation(rule(&module, "entryForJuan")).1,
        relation(rule(&module, "dutyForJuan")).0
    );
    writeln!(report, "Boundary: the current elaborator cannot traverse a compound/nested RuleBody, because no such variant exists.\nFor children with no own binders, mapping this same substitution over all children conserves the current beta semantics.\nChildren with their own binders require separate scope-aware substitution; not tested or implemented here.\n").unwrap();
    let path = root.join("target/partial-normal-sale-compatibility.txt");
    std::fs::write(&path, &report).unwrap();
    println!("{report}\nReport: {}", path.display());
}
