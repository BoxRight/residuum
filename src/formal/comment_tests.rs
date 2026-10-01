use super::*;

const PLAIN: &str = include_str!("../../examples/transfer/formal.res");
const COMMENTED: &str = include_str!("../../examples/transfer/comments_formal.res");

#[test]
fn comments_preserve_surface_and_typed_transfer_ast() {
    let expected = parse_formal(PLAIN).unwrap();
    assert_eq!(parse_formal(COMMENTED).unwrap(), expected);
    assert_eq!(parse_formal_manual(COMMENTED).unwrap(), expected);
    assert_eq!(
        crate::parse_formal_to_typed(COMMENTED).unwrap(),
        crate::parse_formal_to_typed(PLAIN).unwrap()
    );
}

#[test]
fn comments_handle_eof_line_endings_and_adjacent_tokens() {
    let plain = "module Empty entity Thing";
    let expected = parse_formal(plain).unwrap();
    for source in [
        "module Empty entity Thing// no newline",
        "module Empty entity Thing# no newline",
        "/* leading */module/**/Empty/**/entity/**/Thing/**/",
        "# leading\rmodule Empty\r// line\rentity Thing",
        "// leading\r\nmodule Empty\r\n# line\r\nentity Thing",
        "module Empty /* // # é ≤ ⊸ */ entity Thing",
        "module Empty entity Thing // /* deliberately not a block opener",
        "module Empty entity Thing # /* deliberately not a block opener",
    ] {
        assert_eq!(lexer().parse(source).unwrap(), lex(source).unwrap());
        assert_eq!(parse_formal(source).unwrap(), expected, "{source}");
        assert_eq!(parse_formal_manual(source).unwrap(), expected, "{source}");
    }
    for source in ["", "// only", "# only", "/**/", " \n/* é */ // only"] {
        assert!(lexer().parse(source).unwrap().is_empty());
        assert!(lex(source).unwrap().is_empty());
        assert!(parse_formal(source).is_err(), "a module is still required");
        assert!(parse_formal_manual(source).is_err());
    }
}

#[test]
fn malformed_comments_and_tokens_are_rejected() {
    for source in [
        "/*",
        "module Empty /* never closed",
        "module Empty /* almost closed *",
        "module Empty /* valid */ /* second never closed",
        "module Empty /* é\n≤ */ /",
        "module Empty / entity Thing",
        "module Empty */",
        "mod/**/ule Empty",
        "module Empty entity Thing /* outer /* inner */ remaining */",
    ] {
        assert!(parse_formal(source).is_err(), "{source}");
        assert!(parse_formal_manual(source).is_err(), "{source}");
    }
}
