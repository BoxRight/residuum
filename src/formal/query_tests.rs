use super::*;

#[test]
fn query_clauses_preserve_parser_parity_and_legacy_defaults() {
    for query in [
        "residual target(item) @ after tau",
        "fourFusion target(item) @ tau",
        "residual target(item) @ after tau using rules { r, s, } with { rule r.hidden = item, rule s.time = after(tau), }",
        "residual target(item) @ after tau using program Partial with { program Partial.hidden = item }",
        "active residual target(item) @ after tau",
        "active residual target(item) @ after tau using program P with { rule r.record = Box { value = after(tau), } }",
        "residual target(item) @ after tau using rules {} with {}",
    ] {
        let source = format!("module QuerySyntax experiment e {{ seeds {{}} query {query} }}");
        let surface = parse_formal(&source).unwrap();
        assert_eq!(surface, parse_formal_manual(&source).unwrap(), "{query}");
        let FormalDeclaration::Experiment(e) = &surface.declarations[0] else {
            panic!("experiment")
        };
        assert_eq!(e.query_options.active, query.starts_with("active "));
        if !query.contains("using") && !query.starts_with("active ") {
            assert_eq!(e.query_options, SurfaceQueryOptions::default());
        }
    }
}

#[test]
fn query_clause_lists_reject_missing_separators_and_ambiguous_forms() {
    for clauses in [
        "using rules { r s }",
        "using rules { r } with { rule r.x = item rule r.t = tau }",
        "using rules { r } with { r.x = item }",
        "using rules { r } with { rule r.x item }",
        "using rules { r } with { rule r = item }",
        "using rules { r } using program P",
        "using rules { r } with {} with {}",
        "with {} using rules { r }",
    ] {
        let source = format!(
            "module QuerySyntax experiment e {{ seeds {{}} query residual target(item) @ tau {clauses} }}"
        );
        assert!(parse_formal(&source).is_err(), "{clauses}");
        assert!(parse_formal_manual(&source).is_err(), "{clauses}");
    }
}
