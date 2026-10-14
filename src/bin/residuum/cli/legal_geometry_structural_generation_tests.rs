//! Structural generation certificate over a declared slice, not normative learning.
use super::*;
use std::io::{BufWriter, Write as IoWrite};

const MAX_ATOMS: usize = 4;
const ALPHABET: [&str; 12] = [
    "give",
    "do",
    "notDo",
    "Act",
    "~Act",
    "obligationFiled",
    "obligation",
    "right",
    "permissionFiled",
    "permission",
    "prohibition",
    "omissionProhibition",
];

fn verb<'a>(m: &'a Module, name: &str) -> &'a Verb {
    m.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Verb(v) if v.name == name => Some(v),
            _ => None,
        })
        .unwrap()
}

fn constants(m: &Module) -> BTreeMap<String, Type> {
    m.declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Const(c) => Some((c.name.clone(), c.ty.clone())),
            _ => None,
        })
        .collect()
}

fn variables() -> BTreeMap<String, Type> {
    [
        ("norm", Type::Entity("Fact".into())),
        ("act", Type::Entity("Fact".into())),
        ("subject", Type::Entity("Person".into())),
        ("beneficiary", Type::Entity("Person".into())),
        ("object", Type::Entity("Thing".into())),
        ("conduct", Type::Entity("Conduct".into())),
        ("authority", Type::Entity("Authority".into())),
        ("actAuthority", Type::Entity("Authority".into())),
        ("effective", Type::PropositionTime),
        ("actEffective", Type::PropositionTime),
        ("t", Type::PropositionTime),
    ]
    .into_iter()
    .map(|(n, t)| (n.into(), t))
    .collect()
}

fn template(m: &Module, label: &str) -> Event {
    let negative = label.starts_with('~');
    let name = label.trim_start_matches('~');
    let v = verb(m, name);
    let args = v
        .args
        .iter()
        .map(|a| {
            let name = match a.name.as_str() {
                "id" => {
                    if name == "Act" {
                        "act"
                    } else {
                        "norm"
                    }
                }
                "recipient" => "beneficiary",
                "indirectObject" => "object",
                "authority" if name == "Act" => "actAuthority",
                "effective" if name == "Act" => "actEffective",
                other => other,
            };
            Term::Var(name.into())
        })
        .collect();
    let time = if v.kind == VerbKind::Effect {
        TimeExpr::After(Box::new(TimeExpr::Var("t".into())))
    } else {
        TimeExpr::Var("t".into())
    };
    let kind = match v.kind {
        VerbKind::Seeded => PropositionKind::Seeded,
        VerbKind::Derived => PropositionKind::Derived,
        VerbKind::Effect => PropositionKind::Effect,
    };
    Event {
        polarity: if negative {
            Polarity::EvidentialNot
        } else {
            Polarity::Positive
        },
        verb: name.into(),
        args,
        proposition: PropositionType {
            kind,
            time: time.clone(),
        },
        transition: (v.kind == VerbKind::Effect).then(|| StateTransitionType {
            input: StateTime::At(TimeExpr::Var("t".into())),
            output: StateTime::At(time),
        }),
    }
}

fn body(events: &[Event]) -> Antecedent {
    let mut iter = events.iter();
    let Some(first) = iter.next() else {
        return Antecedent::Unit;
    };
    iter.fold(Antecedent::Event(first.clone()), |left, e| {
        Antecedent::And(Box::new(left), Box::new(Antecedent::Event(e.clone())))
    })
}

fn rule(m: &Module, id: usize, input: &[Event]) -> Rule {
    let consequent = template(m, "compliance");
    let mut used = event_vars(&consequent);
    for e in input {
        used.extend(event_vars(e));
    }
    Rule {
        name: format!("generatedStructural{id}"),
        parameters: variables()
            .into_iter()
            .filter(|(name, _)| used.contains(name))
            .map(|(name, ty)| Arg { name, ty })
            .collect(),
        body: RuleBody::HornClause(HornClause {
            antecedent: body(input),
            consequent,
        }),
    }
}

fn vars(t: &Term, result: &mut BTreeSet<String>) {
    match t {
        Term::Var(n) => {
            result.insert(n.clone());
        }
        Term::Constructor { argument, .. } => vars(argument, result),
        Term::RecordLiteral { fields, .. } => {
            for f in fields {
                vars(&f.value, result);
            }
        }
        _ => (),
    }
}

fn event_vars(e: &Event) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for t in &e.args {
        vars(t, &mut out);
    }
    vars(&e.proposition.time.to_term(), &mut out);
    out
}

fn depth(t: &Term) -> usize {
    match t {
        Term::Constructor { argument, .. } => 1 + depth(argument),
        _ => 0,
    }
}

fn assignable(actual: &Type, expected: &Type) -> bool {
    actual == expected
        || match expected {
            Type::Optional(inner) => assignable(actual, inner),
            _ => false,
        }
}

fn term_check(
    t: &Term,
    expected: &Type,
    scope: &BTreeMap<String, Type>,
    cs: &BTreeMap<String, Type>,
    reasons: &mut BTreeSet<String>,
) {
    if depth(t) > 2 {
        reasons.insert("B_AFTER_DEPTH".into());
    }
    match t {
        Term::None => {
            if !matches!(expected, Type::Optional(_)) {
                reasons.insert("S_TYPE".into());
            }
        }
        Term::Var(n) | Term::Const(n) => {
            let actual = if matches!(t, Term::Var(_)) {
                scope.get(n)
            } else {
                cs.get(n)
            };
            match actual {
                None => {
                    reasons.insert("S_UNKNOWN_TERM".into());
                }
                Some(ty) if !assignable(ty, expected) => {
                    reasons.insert("S_TYPE".into());
                }
                _ => (),
            }
        }
        Term::Constructor { argument, .. } => {
            if !assignable(&Type::PropositionTime, expected) {
                reasons.insert("S_TYPE".into());
            }
            term_check(argument, &Type::PropositionTime, scope, cs, reasons);
        }
        Term::RecordLiteral { .. } => {
            reasons.insert("B_RECORD_DEPTH".into());
        }
    }
}

fn identities(e: &Event) -> BTreeSet<String> {
    fn visit(t: &Term, out: &mut BTreeSet<String>) {
        match t {
            Term::Var(n) => {
                out.insert(format!("var:{n}"));
            }
            Term::Const(n) => {
                out.insert(format!("const:{n}"));
            }
            Term::Constructor { argument, .. } => visit(argument, out),
            Term::RecordLiteral { fields, .. } => {
                for f in fields {
                    visit(&f.value, out);
                }
            }
            Term::None => (),
        }
    }
    let mut out = BTreeSet::new();
    for t in &e.args {
        visit(t, &mut out);
    }
    visit(&e.proposition.time.to_term(), &mut out);
    out
}

// Validator for this ordinary, record-free, unsubtyped profile only.
// It neither implements theta-subsumption nor assesses legal adequacy.
fn classify(m: &Module, r: &Rule, connected: bool) -> BTreeSet<String> {
    let RuleBody::HornClause(clause) = &r.body else {
        panic!("declared Horn profile");
    };
    let mut input = Vec::new();
    atoms(&clause.antecedent, &mut input);
    let mut reasons = BTreeSet::new();
    if input.len() > MAX_ATOMS {
        reasons.insert("B_ATOMS".into());
    }
    if r.parameters.len() > 16 {
        reasons.insert("B_VARIABLES".into());
    }
    let scope: BTreeMap<_, _> = r
        .parameters
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    if scope.len() != r.parameters.len() {
        reasons.insert("S_DUPLICATE_BINDER".into());
    }
    let cs = constants(m);
    let mut available = BTreeSet::new();
    for e in &input {
        available.extend(event_vars(e));
    }
    if !event_vars(&clause.consequent).is_subset(&available) {
        reasons.insert("S_UNBOUND_HEAD".into());
    }
    if clause.consequent.verb != "compliance" || clause.consequent.polarity != Polarity::Positive {
        reasons.insert("P_HEAD_INTERFACE".into());
    }
    for (head, e) in input
        .iter()
        .map(|e| (false, *e))
        .chain(std::iter::once((true, &clause.consequent)))
    {
        let Some(v) = m.declarations.iter().find_map(|d| match d {
            Declaration::Verb(v) if v.name == e.verb => Some(v),
            _ => None,
        }) else {
            reasons.insert("S_UNKNOWN_VERB".into());
            continue;
        };
        if !head && !ALPHABET.contains(&e.verb.as_str()) {
            reasons.insert("P_BODY_VOCABULARY".into());
        }
        if e.polarity == Polarity::EvidentialNot && e.verb != "Act" {
            reasons.insert("P_POLARITY".into());
        }
        if e.args.len() != v.args.len() {
            reasons.insert("S_ARITY".into());
        }
        for (t, a) in e.args.iter().zip(&v.args) {
            term_check(t, &a.ty, &scope, &cs, &mut reasons);
        }
        term_check(
            &e.proposition.time.to_term(),
            &Type::PropositionTime,
            &scope,
            &cs,
            &mut reasons,
        );
    }
    if connected {
        let mut reached = identities(&clause.consequent);
        let mut remaining: Vec<_> = input.iter().map(|e| identities(e)).collect();
        loop {
            let before = remaining.len();
            remaining.retain(|xs| {
                if xs.is_disjoint(&reached) {
                    true
                } else {
                    reached.extend(xs.clone());
                    false
                }
            });
            if before == remaining.len() {
                break;
            }
        }
        if !remaining.is_empty() {
            reasons.insert("S_DISCONNECTED".into());
        }
    }
    reasons
}

fn render_rule(r: &Rule) -> String {
    let RuleBody::HornClause(c) = &r.body else {
        unreachable!()
    };
    let mut input = Vec::new();
    atoms(&c.antecedent, &mut input);
    let premise = if input.is_empty() {
        "I".into()
    } else {
        input
            .iter()
            .map(|e| source_event(e))
            .collect::<Vec<_>>()
            .join(" & ")
    };
    let mut used = event_vars(&c.consequent);
    for e in input {
        used.extend(event_vars(e));
    }
    format!(
        "rule {}({}) = {} <= {}",
        r.name,
        r.parameters
            .iter()
            .filter(|p| used.contains(&p.name))
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        premise,
        source_event(&c.consequent)
    )
}

fn source_event(e: &Event) -> String {
    let rendered = render_event(e);
    let (prefix, _) = rendered.rsplit_once(" @ ").unwrap();
    fn time(t: &TimeExpr) -> String {
        match t {
            TimeExpr::At(n) | TimeExpr::Var(n) => n.clone(),
            TimeExpr::After(inner) => match inner.as_ref() {
                TimeExpr::At(n) | TimeExpr::Var(n) => format!("after {n}"),
                _ => format!("after({})", time(inner)),
            },
        }
    }
    format!("{prefix} @ {}", time(&e.proposition.time))
}

fn ground(m: &Module, e: &Event) -> Event {
    let values: BTreeMap<String, String> = [
        ("norm", "gNorm"),
        ("act", "gAct"),
        ("subject", "gSubject"),
        ("beneficiary", "gBeneficiary"),
        ("object", "gObject"),
        ("conduct", "gConduct"),
        ("authority", "administrative"),
        ("actAuthority", "private"),
        ("effective", "gDate"),
        ("actEffective", "gDate"),
        ("t", "gTau"),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b.into()))
    .collect();
    fn term(t: &Term, v: &BTreeMap<String, String>) -> Term {
        match t {
            Term::Var(n) => Term::Const(v[n].clone()),
            Term::Constructor {
                constructor,
                argument,
            } => Term::Constructor {
                constructor: *constructor,
                argument: Box::new(term(argument, v)),
            },
            _ => t.clone(),
        }
    }
    let mut result = e.clone();
    result.args = e.args.iter().map(|t| term(t, &values)).collect();
    result.proposition.time =
        TimeExpr::from_term(&term(&e.proposition.time.to_term(), &values)).unwrap();
    if verb(m, &e.verb).kind == VerbKind::Effect {
        result.transition = Some(StateTransitionType {
            input: StateTime::At(TimeExpr::At("gTau".into())),
            output: StateTime::At(result.proposition.time.clone()),
        });
    }
    result
}

fn directory() -> PathBuf {
    let d = root().join("target/legal-structural-generation");
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn complete_role_shared_slice_has_structural_certificates() {
    let m = module("general");
    assert!(
        m.declarations
            .iter()
            .all(|d| !matches!(d,Declaration::Entity(e) if !e.supertypes.is_empty()))
    );
    let templates: Vec<_> = ALPHABET.iter().map(|n| template(&m, n)).collect();
    let witnesses: Vec<_> = templates.iter().map(|e| ground(&m, e)).collect();
    let expected = ground(&m, &template(&m, "compliance"));
    let dir = directory();
    let mut table = BufWriter::new(std::fs::File::create(dir.join("enumeration.tsv")).unwrap());
    writeln!(
        table,
        "id\tlength\tword\tadmitted\tconditions\tground_matches\trule"
    )
    .unwrap();
    let mut counts = String::from("length\tenumerated\tadmitted\trejected\n");
    let mut id = 0;
    let mut accepted_total = 0;
    for n in 0..=MAX_ATOMS {
        let total = 12usize.pow(n as u32);
        let mut accepted = 0;
        for rank in 0..total {
            let mut value = rank;
            let mut word = vec![0; n];
            for index in (0..n).rev() {
                word[index] = value % 12;
                value /= 12;
            }
            let input: Vec<_> = word.iter().map(|i| templates[*i].clone()).collect();
            let candidate = rule(&m, id, &input);
            let reasons = classify(&m, &candidate, true);
            let has_act = word.iter().any(|i| matches!(i, 3 | 4));
            let has_norm = word.iter().any(|i| *i >= 5);
            assert_eq!(reasons.is_empty(), has_act && has_norm);
            if !reasons.is_empty() {
                assert_eq!(reasons, BTreeSet::from(["S_UNBOUND_HEAD".into()]));
            }
            let matches = residuum::instantiate_from_known(&candidate, &witnesses);
            assert_eq!(matches.len(), usize::from(reasons.is_empty()));
            if let Some(proof) = matches.first() {
                assert_eq!(proof.event, expected);
                assert_eq!(proof.record.rule, candidate.name);
                assert_eq!(proof.record.antecedents.len(), n);
                for (event, index) in proof.record.antecedents.iter().zip(&word) {
                    assert_eq!(event, &witnesses[*index]);
                }
                accepted += 1;
            }
            writeln!(
                table,
                "{id}\t{n}\t{}\t{}\t{}\t{}\t{}",
                word.iter()
                    .map(|i| ALPHABET[*i])
                    .collect::<Vec<_>>()
                    .join(" & "),
                reasons.is_empty(),
                reasons.into_iter().collect::<Vec<_>>().join(","),
                matches.len(),
                render_rule(&candidate)
            )
            .unwrap();
            id += 1;
        }
        let formula =
            12i64.pow(n as u32) - 10i64.pow(n as u32) - 5i64.pow(n as u32) + 3i64.pow(n as u32);
        assert_eq!(accepted as i64, formula);
        accepted_total += accepted;
        writeln!(counts, "{n}\t{total}\t{accepted}\t{}", total - accepted).unwrap();
    }
    table.flush().unwrap();
    assert_eq!(id, 22621);
    assert_eq!(accepted_total, 10850);
    std::fs::write(dir.join("counts.tsv"), counts).unwrap();
    // Certified lower bound, not an exhaustive count of the parent grammar.
    let cs = constants(&m);
    let count = |ty: &Type| cs.values().filter(|t| *t == ty).count() as u128;
    let persons = count(&Type::Entity("Person".into()));
    let facts = count(&Type::Entity("Fact".into()));
    let things = count(&Type::Entity("Thing".into()));
    let conduct = count(&Type::Entity("Conduct".into()));
    let authorities = count(&Type::Entity("Authority".into()));
    let times = count(&Type::PropositionTime) * 3;
    let ground_right = facts * persons.pow(2) * things * conduct * authorities * times.pow(2);
    let ground_head = facts.pow(2) * persons.pow(2) * things * authorities * times.pow(2);
    let bound = ground_right.pow(4) * ground_head;
    assert_eq!(ground_right, 364500);
    assert_eq!(ground_head, 820125);
    assert_eq!(bound, 14476720225405007812500000000u128);
    std::fs::write(dir.join("parent-lower-bound.tsv"),format!(
        "ground_right_atoms\tground_compliance_heads\tfour_ordered_body_lower_bound\n{ground_right}\t{ground_head}\t{bound}\n")).unwrap();
    // The parent's atom budget alone does not bound I nodes. Preserve this
    // counterexample rather than silently declaring the whole parent finite.
    let mut unit = rule(&m, 60000, &[]);
    unit.parameters.clear();
    if let RuleBody::HornClause(c) = &mut unit.body {
        c.consequent = expected.clone();
    }
    let mut unit_report = String::from("unit_leaves\tatoms\tadmitted\tmatches\tast_size\n");
    let mut previous_size = 0;
    for leaves in 1..=32 {
        assert!(classify(&m, &unit, true).is_empty());
        let matches = residuum::instantiate_from_known(&unit, &[]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].event, expected);
        assert!(matches[0].record.antecedents.is_empty());
        let size = format!("{:?}", unit.body).len();
        assert!(size > previous_size);
        previous_size = size;
        writeln!(unit_report, "{leaves}\t0\ttrue\t1\t{size}").unwrap();
        if let RuleBody::HornClause(c) = &mut unit.body {
            c.antecedent =
                Antecedent::And(Box::new(c.antecedent.clone()), Box::new(Antecedent::Unit));
        }
    }
    std::fs::write(dir.join("unit-budget-counterexample.tsv"), unit_report).unwrap();
    println!(
        "complete declared slice: {id}, accepted {accepted_total}, rejected {}; parent lower bound {bound}",
        id - accepted_total
    );
}

#[test]
fn structural_examples_distinguish_profile_checks_from_elaboration() {
    let m = module("general");
    let mut cases: Vec<(&str, Rule, Vec<&str>, bool)> = Vec::new();
    let r = rule(
        &m,
        50000,
        &[template(&m, "obligation"), template(&m, "Act")],
    );
    cases.push(("ordinary_obligation_act", r.clone(), vec![], true));
    cases.push((
        "permission_negative_act",
        rule(
            &m,
            50001,
            &[template(&m, "permission"), template(&m, "~Act")],
        ),
        vec![],
        true,
    ));
    let mut swapped = r.clone();
    if let RuleBody::HornClause(c) = &mut swapped.body {
        if let Antecedent::And(_, b) = &mut c.antecedent {
            if let Antecedent::Event(e) = b.as_mut() {
                e.args[1] = Term::Var("beneficiary".into());
                e.args[4] = Term::Var("subject".into());
            }
        }
    }
    cases.push(("swapped_person_roles", swapped, vec![], true));
    cases.push((
        "missing_act_binding",
        rule(&m, 50002, &[template(&m, "obligation")]),
        vec!["S_UNBOUND_HEAD"],
        true,
    ));
    let mut wrong_type = r.clone();
    if let RuleBody::HornClause(c) = &mut wrong_type.body {
        c.consequent.args[2] = Term::Const("gObject".into());
    }
    cases.push(("thing_in_person_slot", wrong_type, vec!["S_TYPE"], false));
    let mut arity = r.clone();
    if let RuleBody::HornClause(c) = &mut arity.body {
        c.consequent.args.pop();
    }
    cases.push(("wrong_arity", arity, vec!["S_ARITY"], false));
    let mut time = r.clone();
    if let RuleBody::HornClause(c) = &mut time.body {
        c.consequent.proposition.time = TimeExpr::At("gSubject".into());
    }
    cases.push(("person_in_time_slot", time, vec!["S_TYPE"], false));
    let mut unknown = r.clone();
    if let RuleBody::HornClause(c) = &mut unknown.body {
        c.consequent.args[0] = Term::Const("missingConstant".into());
    }
    cases.push(("unknown_constant", unknown, vec!["S_UNKNOWN_TERM"], false));
    let mut negative = r.clone();
    if let RuleBody::HornClause(c) = &mut negative.body {
        c.consequent.polarity = Polarity::EvidentialNot;
    }
    cases.push((
        "negative_head_outside_profile",
        negative,
        vec!["P_HEAD_INTERFACE", "P_POLARITY"],
        true,
    ));
    let too_many = rule(
        &m,
        50003,
        &[
            template(&m, "obligation"),
            template(&m, "Act"),
            template(&m, "right"),
            template(&m, "permission"),
            template(&m, "give"),
        ],
    );
    cases.push(("five_antecedents", too_many, vec!["B_ATOMS"], true));
    let disconnected = rule(
        &m,
        50004,
        &[template(&m, "obligation"), template(&m, "Act"), {
            let mut e = ground(&m, &template(&m, "give"));
            e.args[0] = Term::Const("gLegislator".into());
            e
        }],
    );
    cases.push((
        "disconnected_ground_effect",
        disconnected,
        vec!["S_DISCONNECTED"],
        true,
    ));
    let mut bad_scope = r.clone();
    if let RuleBody::HornClause(c) = &mut bad_scope.body {
        c.consequent.args[0] = Term::Var("missingBinder".into());
    }
    cases.push((
        "undeclared_head_parameter",
        bad_scope,
        vec!["S_UNKNOWN_TERM", "S_UNBOUND_HEAD"],
        false,
    ));
    let mut depth_case = r.clone();
    if let RuleBody::HornClause(c) = &mut depth_case.body {
        c.consequent.args[6] = Term::Constructor {
            constructor: Constructor::After,
            argument: Box::new(Term::Constructor {
                constructor: Constructor::After,
                argument: Box::new(Term::Constructor {
                    constructor: Constructor::After,
                    argument: Box::new(Term::Var("effective".into())),
                }),
            }),
        };
    }
    cases.push(("after_depth_three", depth_case, vec!["B_AFTER_DEPTH"], true));
    let mut sixteen_plus = rule(
        &m,
        50005,
        &[
            template(&m, "obligation"),
            template(&m, "permission"),
            template(&m, "right"),
            template(&m, "Act"),
        ],
    );
    if let RuleBody::HornClause(c) = &mut sixteen_plus.body {
        fn rename(a: &mut Antecedent, index: &mut usize, added: &mut Vec<Arg>) {
            match a {
                Antecedent::And(x, y) => {
                    rename(x, index, added);
                    rename(y, index, added);
                }
                Antecedent::Event(e) => {
                    if *index == 1 || *index == 2 {
                        for (slot, base, ty) in [
                            (0, "norm", Type::Entity("Fact".into())),
                            (5, "authority", Type::Entity("Authority".into())),
                            (6, "effective", Type::PropositionTime),
                        ] {
                            let name = format!("{base}{index}");
                            e.args[slot] = Term::Var(name.clone());
                            added.push(Arg { name, ty });
                        }
                    }
                    *index += 1;
                }
                Antecedent::Unit => (),
            }
        }
        rename(&mut c.antecedent, &mut 0, &mut sixteen_plus.parameters);
    }
    assert_eq!(sixteen_plus.parameters.len(), 17);
    cases.push((
        "seventeen_typed_binders",
        sixteen_plus,
        vec!["B_VARIABLES"],
        true,
    ));
    let mut negative_body = r.clone();
    if let RuleBody::HornClause(c) = &mut negative_body.body {
        if let Antecedent::And(a, _) = &mut c.antecedent {
            if let Antecedent::Event(e) = a.as_mut() {
                e.polarity = Polarity::EvidentialNot;
            }
        }
    }
    cases.push((
        "negative_norm_outside_profile",
        negative_body,
        vec!["P_POLARITY"],
        true,
    ));
    let mut counterexample = r.clone();
    if let RuleBody::HornClause(c) = &mut counterexample.body {
        c.consequent.proposition.time = TimeExpr::After(Box::new(TimeExpr::After(Box::new(
            TimeExpr::Var("t".into()),
        ))));
    }
    cases.push((
        "ast_time_not_source_writable",
        counterexample,
        vec![],
        false,
    ));
    let dir = directory();
    let mut report = String::from("case\tstructural_admission\tconditions\telaboration\trule\n");
    for (name, r, expected, elaborates) in cases {
        let reasons = classify(&m, &r, true);
        assert_eq!(
            reasons,
            expected.into_iter().map(String::from).collect(),
            "{name}"
        );
        let source = format!(
            "module StructuralExample\nimport \"../../examples/experiments/legal_geometry/general.res\"\n{}\n",
            render_rule(&r)
        );
        let path = dir.join(format!("{name}.res"));
        std::fs::write(&path, &source).unwrap();
        let loaded = residuum::load_formal_module(&path);
        assert_eq!(loaded.is_ok(), elaborates, "{name}: {loaded:?}");
        writeln!(
            report,
            "{name}\t{}\t{}\t{}\t{}",
            reasons.is_empty(),
            reasons.iter().cloned().collect::<Vec<_>>().join(","),
            loaded
                .as_ref()
                .map(|_| "accepted".to_string())
                .unwrap_or_else(|e| e.to_string())
                .replace(['\n', '\t'], " "),
            render_rule(&r)
        )
        .unwrap();
        if reasons.is_empty() && loaded.is_ok() && name == "ordinary_obligation_act" {
            let loaded = loaded.unwrap();
            let seeds = experiment(&m, "performance").input.clone();
            let original = close(&m, seeds.clone());
            let hypothetical = close(&loaded, seeds);
            assert_eq!(full_events(&original), full_events(&hypothetical));
            for d in &original.derivations {
                assert!(
                    hypothetical
                        .derivations
                        .iter()
                        .any(|p| format!("{:?}", p.event) == format!("{:?}", d.event)
                            && p.record == d.record)
                );
            }
            for d in &m.declarations {
                if !matches!(d, Declaration::Experiment(_)) {
                    assert_eq!(loaded.origin(d.name()), m.origin(d.name()));
                }
            }
            assert!(
                hypothetical
                    .derivations
                    .iter()
                    .any(|d| d.record.rule == r.name)
            );
            assert_eq!(loaded.origin(&r.name).unwrap().module, "StructuralExample");
        }
    }
    std::fs::write(dir.join("examples.tsv"), report).unwrap();
    println!(
        "16 examples: profile vs real elaboration distinguished; full reference proofs retained"
    );
}
