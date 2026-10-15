//! The source path distinguishes syntax, frontier, elaboration, profile,
//! certificate scope and evaluation outcomes, by full closure and by
//! bounded session. Run individually through `scripts/test_safe.py`.
use super::exploration::*;
use super::proposal::*;
use crate::ForwardSession;
use crate::typed::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const HEAD: &str = "hypotheticalSuccessionImpediment";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(name: &str) -> Module {
    crate::load_formal_module(
        root().join(format!("examples/experiments/legal_geometry/{name}.res")),
    )
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

fn riggs_decision(base: &Module) -> Decision {
    let attr = experiment(base, "historicalRiggs").goal.clone();
    let mut imp = attr.clone();
    imp.verb = HEAD.into();
    Decision {
        components: vec![attr, imp],
        criterion: Criterion::Component(1),
    }
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

#[derive(Default)]
struct CountingClock(usize);
impl crate::DerivationClock for CountingClock {
    fn next_derivation_time(&mut self) -> DerivationTime {
        self.0 += 1;
        DerivationTime {
            name: format!("d{}", self.0),
        }
    }
}

#[test]
fn proposal_source_path_distinguishes_syntax_elaboration_profile_scope_and_evaluation() {
    let base = fixture("riggs_cases");
    let historical = experiment(&base, "historicalRiggs");
    let decision = riggs_decision(&base);
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

    // S_SYNTAX keeps the text and stops before any file is written.
    let broken = run("broken", "rule broken( = <= ");
    assert!(matches!(broken.status, ProposalStatus::Syntax(_)));
    assert!(broken.stages.is_empty());
    assert_eq!(broken.original_text, "rule broken( = <= ");
    assert!(broken.rules.is_empty());
    assert!(!broken.file.exists());

    // Frontier on the AST: an import or a declaration in the text is
    // rejected before any file is resolved or written.
    let smuggled = run(
        "smuggledImport",
        &format!("import \"../../examples/experiments/legal_geometry/general.res\"\n{PURPOSEFUL}"),
    );
    assert_eq!(
        smuggled.status,
        ProposalStatus::StructuralReject(BTreeSet::from(["S_IMPORT".to_string()]))
    );
    assert_eq!(smuggled.stages, vec!["parsed"]);
    assert!(!smuggled.file.exists());
    let declares = run(
        "declares",
        &format!("derived verb smuggled(will : Fact)\n{PURPOSEFUL}"),
    );
    assert_eq!(
        declares.status,
        ProposalStatus::StructuralReject(BTreeSet::from(["S_DECLARATION".to_string()]))
    );
    assert_eq!(declares.stages, vec!["parsed"]);
    assert!(!declares.file.exists());
    let both = run(
        "both",
        &format!("import \"../x.res\"\nconst smuggled : Fact\n{PURPOSEFUL}"),
    );
    assert_eq!(
        both.status,
        ProposalStatus::StructuralReject(BTreeSet::from([
            "S_DECLARATION".to_string(),
            "S_IMPORT".to_string()
        ]))
    );
    assert!(!both.file.exists());

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

    // Structural rejection by the profile: too many atoms.
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

    // Admitted but outside the certificate's proven scope.
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

#[test]
fn proposal_stages_separate_admission_from_evaluation_and_session_matches_full_closure() {
    let base = fixture("riggs_cases");
    let historical = experiment(&base, "historicalRiggs");
    let decision = riggs_decision(&base);
    let imports = [root().join("examples/experiments/legal_geometry/riggs_problem_manifest.res")];
    let workdir = root().join("target/exploration/proposals-staged");
    let profile = profile();
    let ctx = ProposalContext {
        imports: &imports,
        workdir: &workdir,
        profile: &profile,
        max_clauses: 2,
        seeds: &historical.input,
        decision: &decision,
    };
    let episode = Episode::prepare(&ctx).unwrap();
    let proposal = SourceProposal {
        id: "purposeful".into(),
        text: PURPOSEFUL.into(),
    };

    // Admission alone does not evaluate: no closure, no verdict.
    let admitted = admit(&proposal, &ctx, &episode).unwrap().unwrap();
    assert_eq!(
        admitted.report.stages,
        vec!["parsed", "elaborated", "extracted", "admitted"]
    );
    assert_eq!(admitted.heads, BTreeSet::from([HEAD.to_string()]));
    assert!(matches!(admitted.report.status, ProposalStatus::Syntax(_)));
    let reference = episode.reference(&historical.input).unwrap();
    let admitted = check_scope(admitted, &reference).unwrap();

    // Full-closure evaluation of the admitted proposal equals the composition.
    let full = evaluate_admitted(admitted.clone(), &ctx, &reference);
    let composed = process(&proposal, &ctx).unwrap();
    assert_eq!(full, composed);

    // Stepwise evaluation: stopping early is pending, completing matches.
    let mut early = EvaluationSession::begin(admitted.clone(), &ctx, reference.clone()).unwrap();
    let first = early.step();
    assert_eq!(first.round, 1);
    assert!(!first.complete);
    let pending = early.finish();
    assert!(
        matches!(pending.status, ProposalStatus::EvaluationPending(_)),
        "{:?}",
        pending.status
    );
    assert_eq!(status_label(&pending.status), "evaluation_pending");
    assert!(!pending.stages.contains(&"evaluated"));

    let mut session = EvaluationSession::begin(admitted, &ctx, reference).unwrap();
    let mut rounds = 0;
    while !session.is_complete() {
        session.step();
        rounds += 1;
        assert!(rounds <= 64);
    }
    let stepped = session.finish();
    assert_eq!(stepped, full);
    assert_eq!(status_label(&stepped.status), "evaluated_compatible");
}

fn normalized(c: &Closure) -> (BTreeSet<String>, BTreeMap<String, usize>) {
    let events = c.known.iter().map(|e| format!("{e:?}")).collect();
    let mut proofs = BTreeMap::new();
    for d in &c.derivations {
        *proofs
            .entry(format!("{:?}\t{:?}", d.event, d.record))
            .or_insert(0) += 1;
    }
    (events, proofs)
}

fn rule_origins(m: &Module, c: &Closure) -> BTreeSet<(String, String)> {
    c.derivations
        .iter()
        .map(|d| {
            (
                d.record.rule.clone(),
                m.origin(&d.record.rule)
                    .map(|o| o.module.clone())
                    .unwrap_or_else(|| "<no origin>".into()),
            )
        })
        .collect()
}

fn stepped_closure(seeds: &[Event], m: &Module) -> Result<(Closure, usize), String> {
    let mut session = ForwardSession::for_module(seeds.to_vec(), m).map_err(|e| e.to_string())?;
    let mut clock = CountingClock::default();
    let mut rounds = 0;
    while !session.is_complete() {
        if rounds >= 64 {
            return Err("no fixed point within 64 rounds".into());
        }
        session.step(&mut clock);
        rounds += 1;
    }
    Ok((session.closure().clone(), rounds))
}

#[test]
fn forward_session_matches_close_program_on_riggs_candidates_and_geometry_fixtures() {
    // Equivalence of completed closures: full events, justification
    // multiset (rule, ordered antecedents, substitution) and rule origins.
    // Only derivedAt is normalised away. Any discrepancy is kept as a
    // counterexample in target/exploration and fails the test.
    let mut rows = String::from("theory\tcase\trounds\tevents\tproofs\tstatus\n");
    let mut discrepancies: Vec<String> = Vec::new();
    let mut comparisons = 0;
    let mut compare = |label: &str, case: &str, seeds: &[Event], m: &Module| {
        comparisons += 1;
        let full = crate::close_program(seeds.to_vec(), m, &mut CountingClock::default()).unwrap();
        let (fe, fp) = normalized(&full);
        match stepped_closure(seeds, m) {
            Err(e) => {
                discrepancies.push(format!("{label}\t{case}\tsession: {e}"));
                rows.push_str(&format!(
                    "{label}\t{case}\t-\t{}\t{}\tsession_error\n",
                    fe.len(),
                    fp.values().sum::<usize>()
                ));
            }
            Ok((stepped, rounds)) => {
                let (se, sp) = normalized(&stepped);
                let mut status = "equal";
                if fe != se {
                    status = "events_differ";
                    let only_full: Vec<_> = fe.difference(&se).take(2).cloned().collect();
                    let only_step: Vec<_> = se.difference(&fe).take(2).cloned().collect();
                    discrepancies.push(format!(
                        "{label}\t{case}\tevents only in close_program {only_full:?}; only in session {only_step:?}"
                    ));
                } else if fp != sp {
                    status = "proofs_differ";
                    let diff: Vec<_> = fp
                        .iter()
                        .filter(|(k, v)| sp.get(*k) != Some(v))
                        .chain(sp.iter().filter(|(k, v)| fp.get(*k) != Some(v)))
                        .take(2)
                        .map(|(k, v)| format!("{k} x{v}"))
                        .collect();
                    discrepancies
                        .push(format!("{label}\t{case}\tproof multiset differs: {diff:?}"));
                } else if rule_origins(m, &full) != rule_origins(m, &stepped) {
                    status = "origins_differ";
                    discrepancies.push(format!("{label}\t{case}\trule origins differ"));
                }
                rows.push_str(&format!(
                    "{label}\t{case}\t{rounds}\t{}\t{}\t{status}\n",
                    fe.len(),
                    fp.values().sum::<usize>()
                ));
            }
        }
    };

    // Riggs: every catalogue candidate on every declared dossier, and the
    // reference theory itself.
    let base = fixture("riggs_cases");
    let templates = fixture("riggs_candidate_rules");
    let head = templates
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == HEAD => Some(v.clone()),
            _ => None,
        })
        .unwrap();
    let mut rules: Vec<Rule> = templates
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Rule(r) if r.name.starts_with("candidateImpediment") => Some(r.clone()),
            _ => None,
        })
        .collect();
    rules.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(rules.len(), 64);
    let cases: Vec<&Experiment> = base
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Experiment(e) => Some(e),
            _ => None,
        })
        .collect();
    assert_eq!(cases.len(), 6);
    for r in &rules {
        let m = install(
            &base,
            std::slice::from_ref(&head),
            std::slice::from_ref(r),
            "RiggsCandidateRules",
        )
        .unwrap();
        for c in &cases {
            compare(&r.name, &c.name, &c.input, &m);
        }
    }
    for c in &cases {
        compare("RiggsSuccessionReference", &c.name, &c.input, &base);
    }
    // Geometry fixtures: every seeded experiment of the ontologies.
    for name in ["general", "patrimonial", "reference"] {
        let m = fixture(name);
        for d in &m.declarations {
            if let Declaration::Experiment(e) = d {
                if e.input_kind == ExperimentInputKind::Seeds {
                    compare(name, &e.name, &e.input, &m);
                }
            }
        }
    }
    drop(compare);

    let dir = root().join("target/exploration");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("forward-equivalence.tsv"), &rows).unwrap();
    let counterexamples = dir.join("forward-equivalence-counterexamples.tsv");
    if discrepancies.is_empty() {
        let _ = std::fs::remove_file(&counterexamples);
    } else {
        std::fs::write(
            &counterexamples,
            format!("theory\tcase\tdetail\n{}\n", discrepancies.join("\n")),
        )
        .unwrap();
    }
    assert!(comparisons >= 64 * 6 + 6);
    assert!(
        discrepancies.is_empty(),
        "{} of {comparisons} comparisons differ; first: {}",
        discrepancies.len(),
        discrepancies[0]
    );
}
