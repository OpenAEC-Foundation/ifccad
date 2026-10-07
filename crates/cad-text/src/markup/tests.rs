use super::*;

fn kinds(source: &str) -> Vec<MarkupTokenKind> {
    lex(source, MarkupLimits::default())
        .unwrap()
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

#[test]
fn unknown_and_malformed_codes_have_located_failures() {
    for (source, offset, code) in [
        ("a\\z99;b", 1, TextMarkupErrorCode::UnsupportedCode),
        ("a\\H2", 1, TextMarkupErrorCode::MalformedCode),
        ("a\\Hwat;", 1, TextMarkupErrorCode::InvalidNumber),
        ("a\\H1e999;", 1, TextMarkupErrorCode::InvalidNumber),
        ("a}", 1, TextMarkupErrorCode::UnbalancedScope),
        ("{a", 0, TextMarkupErrorCode::UnbalancedScope),
        ("a\\", 1, TextMarkupErrorCode::MalformedCode),
        ("a\\U+12", 1, TextMarkupErrorCode::InvalidUnicode),
        ("a\\U+D800", 1, TextMarkupErrorCode::InvalidUnicode),
        ("a\\U+DC00", 1, TextMarkupErrorCode::InvalidUnicode),
        ("a%<foo>%", 1, TextMarkupErrorCode::DynamicField),
        ("a\\Xb", 1, TextMarkupErrorCode::UnsupportedCode),
    ] {
        let error = lex(source, Default::default()).unwrap_err();
        assert_eq!(error.offset, offset, "{source}");
        assert_eq!(error.code, code, "{source}");
    }
}

#[test]
fn structure_and_lexical_controls_are_not_conflated() {
    assert_eq!(
        kinds("\\P\\PA^JB\\NC^ID\nE\r\nF"),
        vec![
            MarkupTokenKind::ParagraphBreak,
            MarkupTokenKind::ParagraphBreak,
            MarkupTokenKind::Literal("A".into()),
            MarkupTokenKind::CaretControl(CaretControl::LineFeed),
            MarkupTokenKind::Literal("B".into()),
            MarkupTokenKind::ColumnBreak,
            MarkupTokenKind::Literal("C".into()),
            MarkupTokenKind::Tab,
            MarkupTokenKind::Literal("D".into()),
            MarkupTokenKind::RawLineFeed,
            MarkupTokenKind::Literal("E".into()),
            MarkupTokenKind::RawCarriageReturnLineFeed,
            MarkupTokenKind::Literal("F".into()),
        ]
    );
    assert_eq!(
        kinds("a\\Pb\\N"),
        vec![
            MarkupTokenKind::Literal("a".into()),
            MarkupTokenKind::ParagraphBreak,
            MarkupTokenKind::Literal("b".into()),
            MarkupTokenKind::ColumnBreak
        ]
    );
    assert_eq!(
        kinds("^M"),
        vec![MarkupTokenKind::CaretControl(CaretControl::CarriageReturn)]
    );
}

#[test]
fn scopes_resets_and_relative_parameters_keep_their_source_meaning() {
    let tokens = kinds("{\\L\\H0.2x;a\\H1.25x;b\\l}c\\H4;\\W0.8;\\pxt4,8;");
    assert_eq!(tokens[0], MarkupTokenKind::ScopeStart);
    assert_eq!(
        tokens[1],
        MarkupTokenKind::Decoration {
            decoration: MarkupDecoration::Underline,
            enabled: true,
            toggle: false
        }
    );
    assert_eq!(
        tokens[2],
        MarkupTokenKind::Scalar {
            property: MarkupScalarProperty::Height,
            value: 0.2,
            relative: true
        }
    );
    assert_eq!(
        tokens[4],
        MarkupTokenKind::Scalar {
            property: MarkupScalarProperty::Height,
            value: 1.25,
            relative: true
        }
    );
    assert_eq!(
        tokens[6],
        MarkupTokenKind::Decoration {
            decoration: MarkupDecoration::Underline,
            enabled: false,
            toggle: false
        }
    );
    assert_eq!(tokens[7], MarkupTokenKind::ScopeEnd);
    assert_eq!(
        tokens[9],
        MarkupTokenKind::Scalar {
            property: MarkupScalarProperty::Height,
            value: 4.0,
            relative: false
        }
    );
    assert_eq!(
        tokens[11],
        MarkupTokenKind::ParagraphProperties("xt4,8".into())
    );
}

#[test]
fn escaped_unicode_and_special_characters_remain_literal() {
    assert_eq!(
        kinds(r"\{\}\\\;\~\U+0041B\U+D83D\U+DE00%%d%%p%%c"),
        vec![
            MarkupTokenKind::Literal("{".into()),
            MarkupTokenKind::Literal("}".into()),
            MarkupTokenKind::Literal("\\".into()),
            MarkupTokenKind::Literal(";".into()),
            MarkupTokenKind::Literal("\u{a0}".into()),
            MarkupTokenKind::Literal("A".into()),
            MarkupTokenKind::Literal("B".into()),
            MarkupTokenKind::Literal("😀".into()),
            MarkupTokenKind::Literal("°".into()),
            MarkupTokenKind::Literal("±".into()),
            MarkupTokenKind::Literal("Ø".into()),
        ]
    );
    let tokens = lex("😀\\H2;", Default::default()).unwrap();
    assert_eq!(tokens[1].range, 4..8, "locations use UTF-8 byte offsets");
    assert!(lex("%%q", Default::default()).is_err());
    assert!(lex("x\0y", Default::default()).is_err());
}

#[test]
fn source_font_paragraph_and_stack_bodies_keep_uninterpreted_scope_data() {
    assert_eq!(
        kinds(r"\fArial|b1|i0;\S1/2;\pxt4,D8;"),
        vec![
            MarkupTokenKind::Font {
                body: "Arial|b1|i0".into(),
                shx: false
            },
            MarkupTokenKind::Stack("1/2".into()),
            MarkupTokenKind::ParagraphProperties("xt4,D8".into()),
        ]
    );
    assert!(lex(r"\f;", Default::default()).is_err());
    assert!(lex(r"\S/;", Default::default()).is_err());
    assert!(lex(r"\C257;", Default::default()).is_err());
    assert_eq!(
        kinds(r"\C0;\C256;"),
        vec![
            MarkupTokenKind::IndexedColor(0),
            MarkupTokenKind::IndexedColor(256)
        ]
    );
    assert!(lex(r"\c16777216;", Default::default()).is_err());
    assert!(lex(r"\A3;", Default::default()).is_err());
    // Decimal marker/units cannot be qualified by a lexer merely accepting a body.
    assert_eq!(
        kinds(r"\pb2,a3,se4;"),
        vec![MarkupTokenKind::ParagraphProperties("b2,a3,se4".into())]
    );
}

#[test]
fn parser_budgets_are_checked_on_exact_boundaries() {
    let limits = MarkupLimits {
        max_bytes: 2,
        max_depth: 1,
        max_tokens: 1,
    };
    assert!(lex("é", limits).is_ok());
    assert_eq!(
        lex("éa", limits).unwrap_err().code,
        TextMarkupErrorCode::ResourceLimit
    );
    let limits = MarkupLimits {
        max_bytes: 100,
        max_depth: 1,
        max_tokens: 2,
    };
    assert!(lex("{}", limits).is_ok());
    assert_eq!(
        lex("{{}}", limits).unwrap_err().code,
        TextMarkupErrorCode::ResourceLimit
    );
    assert_eq!(
        lex(
            "a\\P",
            MarkupLimits {
                max_tokens: 1,
                ..limits
            }
        )
        .unwrap_err()
        .code,
        TextMarkupErrorCode::ResourceLimit
    );
    let nested = format!("{}{}", "{".repeat(257), "}".repeat(257));
    assert_eq!(
        lex(&nested, Default::default()).unwrap_err().code,
        TextMarkupErrorCode::ResourceLimit
    );
}

#[test]
fn literal_escaping_blocks_codes_and_dynamic_field_recognition() {
    let literal = "\\P {x} %%u %<field>% ^I 😀\u{a0}";
    let escaped = escape_mtext_literal(literal).unwrap();
    let tokens = lex(&escaped, Default::default()).unwrap();
    let mut restored = String::new();
    for token in tokens {
        match token.kind {
            MarkupTokenKind::Literal(text) => restored.push_str(&text),
            other => panic!("literal became {other:?}"),
        }
    }
    assert_eq!(restored, literal);
    assert!(escape_mtext_literal("a\nb").is_err());
}

#[test]
fn fields_inside_code_bodies_do_not_evade_source_restrictions() {
    let error = lex(r"\S%<field>%/2;", Default::default()).unwrap_err();
    assert_eq!(error.code, TextMarkupErrorCode::DynamicField);
    assert_eq!(error.offset, 2);
    assert_eq!(
        lex(r"\f%<field>%;", Default::default()).unwrap_err().code,
        TextMarkupErrorCode::DynamicField
    );
}

#[test]
fn unicode_surrogate_and_escaped_control_boundaries_are_checked() {
    for source in [
        r"\U+D800\U+0041",
        r"\U+DFFF",
        r"\U+0000",
        r"\U+2028",
        "%%000",
        r"\H0;",
        r"\W-1;",
        r"\Q90;",
        r"\TNaN;",
    ] {
        assert!(lex(source, Default::default()).is_err(), "{source}");
    }
    assert_eq!(
        kinds(r"\U+D800\U+DC00"),
        vec![MarkupTokenKind::Literal("𐀀".into())]
    );
    assert_eq!(
        kinds(r"\U+DBFF\U+DFFF"),
        vec![MarkupTokenKind::Literal("\u{10ffff}".into())]
    );
}

#[test]
fn secondary_color_syntax_is_not_silently_reclassified_as_literal_text() {
    assert_eq!(
        lex(r"\C1;2;gradient", Default::default()).unwrap_err().code,
        TextMarkupErrorCode::UnsupportedCode
    );
    assert_eq!(
        kinds(r"\C1;2 doors"),
        vec![
            MarkupTokenKind::IndexedColor(1),
            MarkupTokenKind::Literal("2 doors".into())
        ]
    );
}
