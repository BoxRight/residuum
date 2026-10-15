//! The source path distinguishes syntax, elaboration, profile, certificate
//! scope and evaluation outcomes. Run individually through `scripts/test_safe.py`.
use super::exploration::*;
use super::proposal::*;
use crate::typed::*;
use std::collections::BTreeSet;
use std::path::PathBuf;

const HEAD: &str = "hypotheticalSuccessionImpediment";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cases() -> Module {
    crate::load_formal_module(root().join("examples/experiments/legal_geometry/riggs_cases.res"))
        .unwrap()
}

fn experiment<'a>(m: &'a Module, name: &str) -> &'a Experiment {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Experiment(e) if e.name == name => Some(e),
            _ => None,
        })
        .unwrap()
}

fn profile() -> Profile {
    Profile::p1().with_problem(
        [HEAD],
        [
            "operativeDisposition",
            "deathRecorded",
            "survivorRecorded",
            "intentionalKilling",
            "deathCausedBy",
            "acquisitionPurposeRecorded",
            "testamentaryEntitlement",
            "acquisitionLinkedToWrong",
            "purposefulAcquisition",
            "patrimonialAsset",
        ],
        ["intentionalKilling", "acquisitionPurposeRecorded"],
    )
}

const PURPOSEFUL: &str = "rule purposefulImpediment(will, testator, beneficiary, object, authority, effective, act, death, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
";

const NOT_KILLING: &str = "rule innocentImpediment(will, testator, beneficiary, object, authority, effective, act, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & ~intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
";

#[test]
fn proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation() {
    let base = cases();
    let historical = experiment(&base, "historicalRiggs");
    let attr = historical.goal.clone();
    let mut imp = attr.clone();
    imp.verb = HEAD.into();
    let decision = Decision {
        components: vec![attr, imp],
        criterion: Criterion::Component(1),
    };
    let imports = [root().join("examples/experiments/legal_geometry/riggs_problem_manifest.res")];
    let workdir = root().join("target/exploration/proposals");
    let profile = profile();
    let ctx = ProposalContext {
        imports: &imports,
        workdir: &workdir,
        profile: &profile,
        max_clauses: 2,
        seeds: &historical.input,
        decision: &decision,
    };
    let run = |id: &str, text: &str| {
        process(
            &SourceProposal {
                id: id.into(),
                text: text.into(),
            },
            &ctx,
        )
        .unwrap()
    };

    // Evaluated and compatible with the historical valuation.
    let ok = run("purposeful", PURPOSEFUL);
    assert_eq!(ok.module_name, "Candidatepurposeful");
    assert_eq!(ok.rules.len(), 1);
    assert_eq!(
        ok.stages,
        vec![
            "parsed",
            "elaborated",
            "extracted",
            "admitted",
            "in_certificate_scope",
            "evaluated"
        ]
    );
    match &ok.status {
        ProposalStatus::Evaluated {
            components,
            compatible,
            certificate,
        } => {
            assert_eq!(components, &[true, true]);
            assert!(*compatible);
            assert!(certificate.holds());
            assert_eq!(certificate.added_heads, BTreeSet::from([HEAD.to_string()]));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(status_label(&ok.status), "evaluated_compatible");
    assert!(ok.file.is_file());
    assert_eq!(ok.original_text, PURPOSEFUL);
    assert!(ok.source.contains("import \"../"));
    let reloaded = crate::load_formal_module(&ok.file).unwrap();
    assert_eq!(
        reloaded.origin("purposefulImpediment").unwrap().module,
        "Candidatepurposeful"
    );
    assert_eq!(
        reloaded
            .origin("literalTestamentaryEntitlement")
            .unwrap()
            .module,
        "RiggsSuccessionReference"
    );

    // Evaluated, certified, incompatible: the dossier records the killing.
    let no = run("innocent", NOT_KILLING);
    match &no.status {
        ProposalStatus::Evaluated {
            components,
            compatible,
            certificate,
        } => {
            assert_eq!(components, &[true, false]);
            assert!(!*compatible);
            assert!(certificate.holds());
            assert!(certificate.added_heads.is_empty());
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(status_label(&no.status), "evaluated_incompatible");

    // S_SYNTAX keeps the text and stops before any file is elaborated.
    let broken = run("broken", "rule broken( = <= ");
    assert!(matches!(broken.status, ProposalStatus::Syntax(_)));
    assert!(broken.stages.is_empty());
    assert_eq!(broken.original_text, "rule broken( = <= ");
    assert!(broken.rules.is_empty());

    // S_ELABORATION: unknown verb, then wrong arity.
    let unknown = run(
        "unknownVerb",
        "rule r(will, testator, beneficiary, object, authority, effective, t) =
    forged(will, testator) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
",
    );
    assert!(
        matches!(unknown.status, ProposalStatus::Elaboration(_)),
        "{:?}",
        unknown.status
    );
    assert_eq!(unknown.stages, vec!["parsed"]);
    let arity = run(
        "arity",
        "rule r(will, testator, beneficiary, object, authority, effective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
",
    );
    assert!(
        matches!(arity.status, ProposalStatus::Elaboration(_)),
        "{:?}",
        arity.status
    );

    // Structural rejection: too many atoms, and a declaration in the proposal.
    let five = run(
        "five",
        "rule r(will, testator, beneficiary, object, authority, effective, act, death, actAuthority, actEffective, t) =
    testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
    & intentionalKilling(act, beneficiary, testator, actAuthority, actEffective) @ t
    & acquisitionLinkedToWrong(will, act, death, beneficiary, object, authority, effective) @ t
    & acquisitionPurposeRecorded(act, will, beneficiary) @ t
    & purposefulAcquisition(will, act, death, beneficiary, object, authority, effective) @ t
    <= hypotheticalSuccessionImpediment(will, testator, beneficiary, object, authority, effective) @ t
",
    );
    match &five.status {
        ProposalStatus::StructuralReject(reasons) => {
            assert!(reasons.contains("B_ATOMS"), "{reasons:?}")
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(five.stages, vec!["parsed", "elaborated", "extracted"]);
    let declares = run(
        "declares",
        &format!(
            "derived verb smuggled(will : Fact)\nrule smuggledRule(will, t) = operativeDisposition(will, testator, beneficiary, object, authority, effective) @ t <= smuggled(will) @ t\n{PURPOSEFUL}"
        ),
    );
    match &declares.status {
        ProposalStatus::StructuralReject(reasons) => {
            assert!(reasons.contains("S_DECLARATION"), "{reasons:?}");
        }
        ProposalStatus::Elaboration(_) => {}
        other => panic!("{other:?}"),
    }
    let decl_only = run(
        "declOnly",
        &format!("derived verb smuggled(will : Fact)\n{PURPOSEFUL}"),
    );
    match &decl_only.status {
        ProposalStatus::StructuralReject(reasons) => {
            assert!(reasons.contains("S_DECLARATION"), "{reasons:?}");
        }
        other => panic!("{other:?}"),
    }

    // Admitted but outside the certificate's proven scope: the head is a
    // conclusion the reference already produces.
    let scope_profile = profile
        .clone()
        .with_problem(["testamentaryEntitlement"], [], []);
    let scoped = ProposalContext {
        profile: &scope_profile,
        ..ctx.clone()
    };
    let existing = process(
        &SourceProposal {
            id: "existingHead".into(),
            text: "rule shortcut(will, testator, beneficiary, object, authority, effective, t) =
    operativeDisposition(will, testator, beneficiary, object, authority, effective) @ t
    <= testamentaryEntitlement(will, testator, beneficiary, object, authority, effective) @ t
"
            .into(),
        },
        &scoped,
    )
    .unwrap();
    assert_eq!(
        existing.status,
        ProposalStatus::OutOfCertificateScope(BTreeSet::from([
            "testamentaryEntitlement".to_string()
        ]))
    );
    assert_eq!(
        existing.stages,
        vec!["parsed", "elaborated", "extracted", "admitted"]
    );

    // Harness misconfiguration is an error, never a status.
    assert!(
        process(
            &SourceProposal {
                id: "bad id".into(),
                text: PURPOSEFUL.into()
            },
            &ctx
        )
        .is_err()
    );
    let bad_decision = Decision {
        criterion: Criterion::Component(7),
        ..decision.clone()
    };
    let misconfigured = ProposalContext {
        decision: &bad_decision,
        ..ctx.clone()
    };
    assert!(
        process(
            &SourceProposal {
                id: "purposeful".into(),
                text: PURPOSEFUL.into()
            },
            &misconfigured
        )
        .is_err()
    );
    let no_imports = ProposalContext {
        imports: &[],
        ..ctx.clone()
    };
    assert!(
        process(
            &SourceProposal {
                id: "purposeful".into(),
                text: PURPOSEFUL.into()
            },
            &no_imports
        )
        .is_err()
    );
}
