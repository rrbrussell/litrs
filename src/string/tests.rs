use crate::{Literal, StringLit, test_util::assert_parse_ok_eq};

// ===== Utility functions =======================================================================

macro_rules! check {
    ($lit:literal, $has_escapes:expr, $num_hashes:expr) => {
        let input = stringify!($lit);
        let expected = StringLit {
            raw: input,
            value: if $has_escapes { Some($lit.to_string()) } else { None },
            num_hashes: $num_hashes,
        };

        assert_parse_ok_eq(input, StringLit::parse(input), expected.clone(), "StringLit::parse");
        assert_parse_ok_eq(
            input, Literal::parse(input), Literal::String(expected), "Literal::parse");
        assert_eq!(StringLit::parse(input).unwrap().value(), $lit);
        assert_eq!(StringLit::parse(input).unwrap().into_value(), $lit);
    };
}


// ===== Actual tests ============================================================================

#[test]
fn simple() {
    check!("", false, None);
    check!("a", false, None);
    check!("peter", false, None);
    check!("Sei gegrüßt, Bärthelt!", false, None);
    check!("أنا لا أتحدث العربية", false, None);
    check!("お前はもう死んでいる", false, None);
    check!("Пушки - интересные музыкальные инструменты", false, None);
    check!("lit 👌 😂 af", false, None);
}

#[test]
fn special_whitespace() {
    let strings = ["\n", "\t", "foo\tbar", "🦊\n", "\r\n"];

    for &s in &strings {
        let input = format!(r#""{}""#, s);
        let input_raw = format!(r#"r"{}""#, s);
        for (input, num_hashes) in vec![(input, None), (input_raw, Some(0))] {
            let expected = StringLit {
                raw: &*input,
                value: None,
                num_hashes,
            };
            assert_parse_ok_eq(
                &input, StringLit::parse(&*input), expected.clone(), "StringLit::parse");
            assert_parse_ok_eq(
                &input, Literal::parse(&*input), Literal::String(expected), "Literal::parse");
            assert_eq!(StringLit::parse(&*input).unwrap().value(), s);
            assert_eq!(StringLit::parse(&*input).unwrap().into_value(), s);
        }
    }
}

#[test]
fn simple_escapes() {
    check!("a\nb", true, None);
    check!("\nb", true, None);
    check!("a\n", true, None);
    check!("\n", true, None);

    check!("\x60犬 \t 猫\r馬\n うさぎ \0ネズミ", true, None);
    check!("నా \\పిల్లి లావుగా ఉంది", true, None);
    check!("నా \\పిల్లి లావుగా 🐈\"ఉంది", true, None);
    check!("\\నా\\ పిల్లి లావుగా\" ఉంది\"", true, None);
    check!("\"నా \\🐈 పిల్లి లావుగా \" ఉంది\\", true, None);

    check!("\x00", true, None);
    check!(" \x01", true, None);
    check!("\x0c 🦊", true, None);
    check!(" 🦊\x0D ", true, None);
    check!("\\x13", true, None);
    check!("\"x30", true, None);
}

#[test]
fn unicode_escapes() {
    check!("\u{0}", true, None);
    check!(" \u{00}", true, None);
    check!("\u{b} ", true, None);
    check!(" \u{B} ", true, None);
    check!("\u{7e}", true, None);
    check!("నక్క\u{E4}", true, None);
    check!("\u{e4} నక్క", true, None);
    check!(" \u{fc}నక్క ", true, None);
    check!("\u{Fc}", true, None);
    check!("\u{fC}🦊\nлиса", true, None);
    check!("лиса\u{FC}", true, None);
    check!("лиса\u{b10}నక్క🦊", true, None);
    check!("\"నక్క\u{B10}", true, None);
    check!("лиса\\\u{0b10}", true, None);
    check!("ли🦊са\\\"\u{0b10}", true, None);
    check!("నక్క\\\\u{0b10}", true, None);
    check!("\u{2764}Füchsin", true, None);
    check!("Füchse \u{1f602}", true, None);
    check!("cd\u{1F602}ab", true, None);

    check!("\u{0}🦊", true, None);
    check!("лиса\u{0__}", true, None);
    check!("\\🦊\u{3_b}", true, None);
    check!("🦊\u{1_F_6_0_2}Füchsin", true, None);
    check!("నక్క\\\u{1_F6_02_____}నక్క", true, None);
}

#[test]
fn raw_string() {
    check!(r"", false, Some(0));
    check!(r"a", false, Some(0));
    check!(r"peter", false, Some(0));
    check!(r"Sei gegrüßt, Bärthelt!", false, Some(0));
    check!(r"أنا لا أتحدث العربية", false, Some(0));
    check!(r"お前はもう死んでいる", false, Some(0));
    check!(r"Пушки - интересные музыкальные инструменты", false, Some(0));
    check!(r"lit 👌 😂 af", false, Some(0));

    check!(r#""#, false, Some(1));
    check!(r#"a"#, false, Some(1));
    check!(r##"peter"##, false, Some(2));
    check!(r###"Sei gegrüßt, Bärthelt!"###, false, Some(3));
    check!(r########"lit 👌 😂 af"########, false, Some(8));

    check!(r#"foo " bar"#, false, Some(1));
    check!(r##"foo " bar"##, false, Some(2));
    check!(r#"foo """" '"'" bar"#, false, Some(1));
    check!(r#""foo""#, false, Some(1));
    check!(r###""foo'"###, false, Some(3));
    check!(r#""x'#_#s'"#, false, Some(1));
    check!(r"#", false, Some(0));
    check!(r"foo#", false, Some(0));
    check!(r"##bar", false, Some(0));
    check!(r###""##foo"##bar'"###, false, Some(3));

    check!(r"さび\n\t\r\0\\x60\u{123}フェリス", false, Some(0));
    check!(r#"さび\n\t\r\0\\x60\u{123}フェリス"#, false, Some(1));
}

#[test]
fn parse_err() {
    assert_err!(StringLit, r#"""#, UnterminatedString, None);
    assert_err!(StringLit, r#""犬"#, UnterminatedString, None);
    assert_err!(StringLit, r#""Jürgen"#, UnterminatedString, None);
    assert_err!(StringLit, r#""foo bar baz"#, UnterminatedString, None);

    assert_err!(StringLit, r#""fox"peter"#, UnexpectedChar, 5..10);
    assert_err!(StringLit, r#""fox"peter""#, UnexpectedChar, 5..11);
    assert_err!(StringLit, r#""fox"🦊"#, UnexpectedChar, 5..9);

    assert_err!(StringLit, "\"\r\"", IsolatedCr, 1);
    assert_err!(StringLit, "\"fo\rx\"", IsolatedCr, 3);
    assert_err!(StringLit, "r\"\r\"", IsolatedCr, 2);
    assert_err!(StringLit, "r\"fo\rx\"", IsolatedCr, 4);
}

#[test]
fn invald_ascii_escapes() {
    assert_err!(StringLit, r#""\x80""#, NonAsciiXEscape, 1..5);
    assert_err!(StringLit, r#""🦊\x81""#, NonAsciiXEscape, 5..9);
    assert_err!(StringLit, r#"" \x8a""#, NonAsciiXEscape, 2..6);
    assert_err!(StringLit, r#""\x8Ff""#, NonAsciiXEscape, 1..5);
    assert_err!(StringLit, r#""\xa0 ""#, NonAsciiXEscape, 1..5);
    assert_err!(StringLit, r#""నక్క\xB0""#, NonAsciiXEscape, 13..17);
    assert_err!(StringLit, r#""\xc3నక్క""#, NonAsciiXEscape, 1..5);
    assert_err!(StringLit, r#""\xDf🦊""#, NonAsciiXEscape, 1..5);
    assert_err!(StringLit, r#""నక్క\xffనక్క""#, NonAsciiXEscape, 13..17);
    assert_err!(StringLit, r#""\xfF ""#, NonAsciiXEscape, 1..5);
    assert_err!(StringLit, r#"" \xFf""#, NonAsciiXEscape, 2..6);
    assert_err!(StringLit, r#""నక్క  \xFF""#, NonAsciiXEscape, 15..19);
}

#[test]
fn invald_escapes() {
    assert_err!(StringLit, r#""\a""#, UnknownEscape, 1..3);
    assert_err!(StringLit, r#""foo\y""#, UnknownEscape, 4..6);
    assert_err!(StringLit, r#""\"#, UnterminatedString, None);
    assert_err!(StringLit, r#""\x""#, UnterminatedEscape, 1..3);
    assert_err!(StringLit, r#""🦊\x1""#, UnterminatedEscape, 5..8);
    assert_err!(StringLit, r#"" \xaj""#, InvalidXEscape, 2..6);
    assert_err!(StringLit, r#""నక్క\xjb""#, InvalidXEscape, 13..17);
}

#[test]
fn invalid_unicode_escapes() {
    assert_err!(StringLit, r#""\u""#, UnicodeEscapeWithoutBrace, 1..3);
    assert_err!(StringLit, r#""🦊\u ""#, UnicodeEscapeWithoutBrace, 5..7);
    assert_err!(StringLit, r#""\u3""#, UnicodeEscapeWithoutBrace, 1..3);

    assert_err!(StringLit, r#""\u{""#, UnterminatedUnicodeEscape, 1..4);
    assert_err!(StringLit, r#""\u{12""#, UnterminatedUnicodeEscape, 1..6);
    assert_err!(StringLit, r#""🦊\u{a0b""#, UnterminatedUnicodeEscape, 5..11);
    assert_err!(StringLit, r#""\u{a0_b  ""#, UnterminatedUnicodeEscape, 1..10);

    assert_err!(StringLit, r#""\u{_}నక్క""#, InvalidStartOfUnicodeEscape, 4);
    assert_err!(StringLit, r#""\u{_5f}""#, InvalidStartOfUnicodeEscape, 4);

    assert_err!(StringLit, r#""fox\u{x}""#, NonHexDigitInUnicodeEscape, 7);
    assert_err!(StringLit, r#""\u{0x}🦊""#, NonHexDigitInUnicodeEscape, 5);
    assert_err!(StringLit, r#""నక్క\u{3bx}""#, NonHexDigitInUnicodeEscape, 18);
    assert_err!(StringLit, r#""\u{3b_x}лиса""#, NonHexDigitInUnicodeEscape, 7);
    assert_err!(StringLit, r#""\u{4x_}""#, NonHexDigitInUnicodeEscape, 5);

    assert_err!(StringLit, r#""\u{1234567}""#, TooManyDigitInUnicodeEscape, 10);
    assert_err!(StringLit, r#""నక్క\u{1234567}🦊""#, TooManyDigitInUnicodeEscape, 22);
    assert_err!(StringLit, r#""నక్క\u{1_23_4_56_7}""#, TooManyDigitInUnicodeEscape, 26);
    assert_err!(StringLit, r#""\u{abcdef123}лиса""#, TooManyDigitInUnicodeEscape, 10);

    assert_err!(StringLit, r#""\u{110000}fox""#, InvalidUnicodeEscapeChar, 1..10);
}
