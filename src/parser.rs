//! TODO and WIP
#![allow(missing_docs)]

use std::mem::MaybeUninit;

use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, tag, tag_no_case};
use nom::character::complete::{char, digit1, none_of, one_of};
use nom::combinator::{all_consuming, map, map_res, opt, recognize, value};
use nom::error::{Error, ErrorKind};
use nom::multi::{many0, many1};
use nom::sequence::{pair, preceded, separated_pair, terminated};
use nom::{IResult, Parser};

use crate::types::Block;
use crate::types::Expression;
use crate::types::ExpressionItem;
use crate::types::Filter;
use crate::types::Group;
use crate::types::KeyValue;
use crate::types::LabeledSource;
use crate::types::Source;

/* TODO update me and drop an interactive link
<nonzero_digit> ::= [1-9]
<digit> ::= [0-9]
<letter> ::= [a-z]
<identifier> ::= <letter>+ <digit>*
<escaped> ::= "{" | "}" | "[" | "]" | "(" | ")" | "\"
/* any character NOT in escaped */
/* escapes are generally just not handled in this BNF */
<noescape_anychar> ::= "#"
<source> ::= <identifier> | "[" (<noescape_anychar>)+ "]"
<label> ::= <nonzero_digit>? <digit> "@"
<labeled_source> ::= <label>? <source>
<pipe> ::= "|"
<pipe_spaced> ::= " | "
<filter_expr> ::= (<pipe> | <pipe_spaced>) <identifier>
<equals> ::= "="
<equals_spaced> ::= " = "
<key_value> ::= <letter>+ (<equals> | <equals_spaced>) (<letter>+ | <digit>+)
<key_value_seq> ::= (<key_value> ", ")*
<key_value_expr> ::= ":" " "? <key_value_seq> <key_value>
<repeat> ::= "!" <nonzero_digit>? <digit>
<block> ::= "(" <labeled_source> <key_value_expr>? <filter_expr>* ")" <repeat>?
<group> ::= "{" <block>+ "}" <repeat>?
<expr> ::= (<group> | <block>)+
*/

// SAFETY do not access from reentrant code (interrupt handler, recursive call, etc)
// TODO consider playing with thread_local instead
static mut DYNAMIC_SOURCES: MaybeUninit<Vec<(String, Vec<ExpressionItem>)>> = MaybeUninit::uninit();

const ESCAPED: &str = "\\{}[]()";

#[expect(unsafe_code)]
pub unsafe fn init_dynamic_sources() {
    unsafe {
        #[expect(static_mut_refs)]
        DYNAMIC_SOURCES.write(Vec::new());
    }
}

fn nonzero_digit(input: &str) -> IResult<&str, char> {
    one_of("123456789").parse(input)
}

fn digit(input: &str) -> IResult<&str, char> {
    one_of("0123456789").parse(input)
}

fn letter(input: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyz").parse(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(many1(letter), many0(digit))).parse(input)
}

fn not_escaped(input: &str) -> IResult<&str, char> {
    none_of(ESCAPED).parse(input)
}

fn anychar_escaped_transform(input: &str) -> IResult<&str, String> {
    escaped_transform(
        not_escaped,
        '\\',
        alt((
            value("{", tag("{")),
            value("}", tag("}")),
            value("[", tag("[")),
            value("]", tag("]")),
            value("(", tag("(")),
            value(")", tag(")")),
            value("\\", tag("\\")),
        )),
    )
    .parse(input)
}

fn repeat(input: &str) -> IResult<&str, u8> {
    map_res(
        preceded(char('!'), recognize(pair(nonzero_digit, opt(digit)))),
        str::parse::<u8>,
    )
    .parse(input)
}

fn character_list(input: &str) -> IResult<&str, Option<String>> {
    terminated(
        preceded(char('['), opt(anychar_escaped_transform)),
        char(']'),
    )
    .parse(input)
}

fn source(input: &str) -> IResult<&str, Source> {
    if let Ok(chars) = character_list(input) {
        return Ok((
            chars.0,
            Source::CharacterList(chars.1.unwrap_or_else(String::new)),
        ));
    }
    #[expect(unsafe_code, static_mut_refs)]
    unsafe {
        let dynamic_sources = DYNAMIC_SOURCES.assume_init_ref();
        let all_sources = [
            value(Source::Word(None, None), tag_no_case("word")),
            value(Source::Letter, tag_no_case("letter")),
            value(Source::Symbol, tag_no_case("symbol")),
            value(Source::Digit, tag_no_case("digit")),
            value(Source::Koremutake(None, None), tag_no_case("koremutake")),
        ];
        for mut branch in all_sources {
            let result: IResult<&str, Source> = branch.parse(input);
            if result.is_ok() {
                return result;
            }
        }
        for (label, expressions) in dynamic_sources {
            if input.starts_with(label) {
                return Ok((
                    &input[label.len()..],
                    Source::Custom(label.clone(), expressions.clone()),
                ));
            }
        }
    };
    Err(nom::Err::Failure(Error::new(input, ErrorKind::Alt)))
}

fn label(input: &str) -> IResult<&str, u8> {
    map_res(
        terminated(recognize(pair(nonzero_digit, opt(digit))), char('@')),
        str::parse::<u8>,
    )
    .parse(input)
}

fn labeled_source(input: &str) -> IResult<&str, LabeledSource> {
    map(pair(opt(label), source), |(label, source)| LabeledSource {
        source,
        label,
    })
    .parse(input)
}

fn pipe(input: &str) -> IResult<&str, &str> {
    tag("|").parse(input)
}

fn pipe_spaced(input: &str) -> IResult<&str, &str> {
    tag(" | ").parse(input)
}

fn filter_expr(input: &str) -> IResult<&str, Filter> {
    preceded(
        alt((pipe, pipe_spaced)),
        alt((
            value(Filter::Reversed, tag_no_case("reversed")),
            value(Filter::Upper, tag_no_case("upper")),
            value(Filter::Lower, tag_no_case("lower")),
            value(Filter::CapitalizeFirst, tag_no_case("capitalizefirst")),
            value(Filter::CapitalizeLast, tag_no_case("capitalizelast")),
            value(
                Filter::CapitalizeNotFirst,
                tag_no_case("capitalizenotfirst"),
            ),
            value(Filter::CapitalizeNotLast, tag_no_case("capitalizenotlast")),
        )),
    )
    .parse(input)
}

fn equals(input: &str) -> IResult<&str, &str> {
    tag("=").parse(input)
}

fn equals_spaced(input: &str) -> IResult<&str, &str> {
    tag(" = ").parse(input)
}

fn equals_alt(input: &str) -> IResult<&str, &str> {
    alt((equals, equals_spaced)).parse(input)
}

fn key_value(input: &str) -> IResult<&str, KeyValue> {
    map(
        alt((
            separated_pair(
                tag_no_case("min"),
                equals_alt,
                map_res(digit1, str::parse::<u8>),
            ),
            separated_pair(
                tag_no_case("max"),
                equals_alt,
                map_res(digit1, str::parse::<u8>),
            ),
        )),
        |(key, value)| match key.to_ascii_lowercase().as_str() {
            "min" => KeyValue::Min(value),
            "max" => KeyValue::Max(value),
            _ => unreachable!(),
        },
    )
    .parse(input)
}

fn key_value_seq(input: &str) -> IResult<&str, Vec<KeyValue>> {
    many0(terminated(key_value, alt((tag(", "), tag(","))))).parse(input)
}

fn key_value_expr(input: &str) -> IResult<&str, Vec<KeyValue>> {
    map(
        preceded(alt((tag(": "), tag(":"))), pair(key_value_seq, key_value)),
        |(mut seq, kv)| {
            let mut result = Vec::new();
            result.append(&mut seq);
            result.push(kv);
            result
        },
    )
    .parse(input)
}

fn block_inner(input: &str) -> IResult<&str, (LabeledSource, Vec<Filter>)> {
    map_res(
        (labeled_source, opt(key_value_expr), many0(filter_expr)),
        |(mut labeled_source, key_values, filters)| {
            if let Some(kv) = key_values {
                if !matches!(
                    labeled_source.source,
                    Source::Word(_, _) | Source::Koremutake(_, _)
                ) {
                    return Err(());
                }
                let mut min = None;
                let mut max = None;
                for inner in kv {
                    match inner {
                        KeyValue::Min(v) => min = Some(v),
                        KeyValue::Max(v) => max = Some(v),
                    }
                }
                labeled_source.source = match labeled_source.source {
                    Source::Word(_, _) => Source::Word(min, max),
                    Source::Koremutake(_, _) => Source::Koremutake(min, max),
                    _ => unreachable!(),
                };
            }
            Ok((labeled_source, filters))
        },
    )
    .parse(input)
}

fn block(input: &str) -> IResult<&str, Block> {
    map(
        pair(
            terminated(preceded(char('('), block_inner), char(')')),
            opt(repeat),
        ),
        |((labeled_source, filters), repeat)| Block {
            source: labeled_source,
            filters,
            repeat: repeat.unwrap_or(1),
        },
    )
    .parse(input)
}

fn group(input: &str) -> IResult<&str, ExpressionItem> {
    map(
        pair(
            terminated(
                preceded(char('{'), pair(many1(block), many0(filter_expr))),
                char('}'),
            ),
            opt(repeat),
        ),
        |((blocks, filters), repeat)| {
            ExpressionItem::Group(Group {
                blocks,
                filters,
                repeat: repeat.unwrap_or(1),
            })
        },
    )
    .parse(input)
}

fn custom_source_label(input: &str) -> IResult<&str, &str> {
    terminated(identifier, char('#')).parse(input)
}

fn definition(input: &str) -> IResult<&str, (&str, Vec<ExpressionItem>)> {
    pair(
        custom_source_label,
        many1(alt((group, map(block, ExpressionItem::Block)))),
    )
    .parse(input)
    /*
    |(label, expression_items)| {
        ()
    }
    */
}

fn definition_seq(input: &str) -> IResult<&str, Vec<(&str, Vec<ExpressionItem>)>> {
    many0(terminated(definition, alt((tag(", "), tag(","))))).parse(input)
}

fn definition_expr(input: &str) -> IResult<&str, ()> {
    map(
        terminated(pair(definition_seq, definition), tag("; ")),
        |(def_seq, def)| {
            #[expect(unsafe_code)]
            let buf = unsafe {
                #[expect(static_mut_refs)]
                DYNAMIC_SOURCES.assume_init_mut()
            };
            for (label, expression_items) in def_seq {
                buf.push((label.to_owned(), expression_items));
            }
            buf.push((def.0.to_owned(), def.1));
        },
    )
    .parse(input)
}

pub fn parse(input: &str) -> IResult<&str, Expression> {
    map(
        all_consuming(preceded(
            many0(definition_expr),
            many1(alt((group, map(block, ExpressionItem::Block)))),
        )),
        |items| Expression { items },
    )
    .parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nonzero_digit_ok() {
        let result = nonzero_digit("1").unwrap();
        assert_eq!(result.1, '1');
    }
    #[test]
    fn test_nonzero_digit_err() {
        nonzero_digit("0").unwrap_err();
    }
    #[test]
    fn test_digit_ok() {
        let result = digit("0").unwrap();
        assert_eq!(result.1, '0');
    }
    #[test]
    fn test_digit_err() {
        digit("x").unwrap_err();
    }
    #[test]
    fn test_identifier_ok() {
        let result = identifier("foo1").unwrap();
        assert_eq!(result.1, "foo1");
    }
    #[test]
    fn test_identifier_err() {
        identifier("1").unwrap_err();
    }
    #[test]
    fn test_not_escaped_ok() {
        let result = not_escaped("x").unwrap();
        assert_eq!(result.1, 'x');
    }
    #[test]
    fn test_not_escaped_err() {
        not_escaped("\\").unwrap_err();
    }
    #[test]
    fn test_anychar_escaped_transform() {
        let expectations = [
            ("x", "x".to_owned()),
            ("xx", "xx".to_owned()),
            ("x\\\\", "x\\".to_owned()),
            ("x\\[", "x[".to_owned()),
            ("x\\[\\]\\{\\}\\(\\)\\\\x", "x[]{}()\\x".to_owned()),
        ];

        for (input, expected) in expectations.into_iter() {
            let result = anychar_escaped_transform(input).unwrap();
            assert_eq!(result.1, expected);
        }
    }
    #[test]
    fn test_repeat_ok() {
        let expectations = [("!99", 99), ("!1", 1)];
        for (input, expected) in expectations.into_iter() {
            let result = repeat(input).unwrap();
            assert_eq!(result.1, expected);
        }
    }
    #[test]
    fn test_repeat_err() {
        let parameters = ["99", "!0", "!x", "100"];
        for parameter in parameters.into_iter() {
            repeat(parameter).unwrap_err();
        }
    }
    #[test]
    fn test_key_value_ok() {
        let expectations = [
            ("min = 1", KeyValue::Min(1)),
            ("max = 1", KeyValue::Max(1)),
            ("min=1", KeyValue::Min(1)),
            ("max=1", KeyValue::Max(1)),
        ];
        for (input, expected) in expectations.into_iter() {
            let result = key_value(input).unwrap();
            assert_eq!(result.1, expected);
        }
    }
    #[test]
    fn test_key_value_err() {
        let parameters = ["min = -1", "max= 1", "max =1", "max  =  1", "max = 256"];
        for parameter in parameters.into_iter() {
            key_value(parameter).unwrap_err();
        }
    }
    #[test]
    fn test_key_value_seq_ok() {
        let expectations = [
            ("min = 1,", vec![KeyValue::Min(1)]),
            ("min = 1,min = 1,", vec![KeyValue::Min(1), KeyValue::Min(1)]),
            (
                "min = 1, min = 1,",
                vec![KeyValue::Min(1), KeyValue::Min(1)],
            ),
        ];
        for (input, expected) in expectations.into_iter() {
            let result = key_value_seq(input).unwrap();
            assert_eq!(result.1, expected);
        }
    }
    #[test]
    fn test_definition_ok() {
        let expectations = ["foo#(word)"];
        for input in expectations.into_iter() {
            let result = definition(input).unwrap();
        }
    }
    #[test]
    fn test_parse_smoke() {
        let parameters = [
            "(word)",
            "(symbol)",
            "(letter)",
            "(digit)",
            "([abcd])",
            "([\\]])",
            "(word)(word)",
            "(1@word)",
            "(1@word: min=3)",
            "(1@word:min = 3)",
            "(1@word:min=3,max=11)",
            "(word | reversed|lower)",
            "(word)!4",
            "{(word:min=3, max=11 | lower)(1@symbol)(word | upper)(1@symbol)}!2(digit)!4(symbol)",
        ];
        for parameter in parameters.into_iter() {
            println!("{}", parameter);
            parse(parameter).unwrap();
        }
    }
    /*
    #[test]
    fn test_parse_ok() {
        let input =
            "{(word:min=3, max=11 | lower)(1@symbol)(word | upper)(1@symbol)}!2(digit)!4(symbol)";
        let result = parse(input).unwrap();
        assert_eq!(result.1.len(), 3);

        let group = &result.1[0];
        match group {
            ExpressionItem::Group(v) => {
                assert_eq!(v.repeat, 2);
                assert_eq!(v.blocks.len(), 4);
                let expected_1 = Block {
                    source: LabeledSource {
                        source: Source::Word(Some(3), Some(11)),
                        label: None,
                    },
                    filters: vec![Filter::Lower],
                    repeat: 1,
                };
                let expected_2 = Block {
                    source: LabeledSource {
                        source: Source::Symbol,
                        label: Some(1),
                    },
                    filters: vec![],
                    repeat: 1,
                };
                let expected_3 = Block {
                    source: LabeledSource {
                        source: Source::Word(None, None),
                        label: None,
                    },
                    filters: vec![Filter::Upper],
                    repeat: 1,
                };
                let expected_4 = Block {
                    source: LabeledSource {
                        source: Source::Symbol,
                        label: Some(1),
                    },
                    filters: vec![],
                    repeat: 1,
                };
                assert_eq!(expected_1, v.blocks[0]);
                assert_eq!(expected_2, v.blocks[1]);
                assert_eq!(expected_3, v.blocks[2]);
                assert_eq!(expected_4, v.blocks[3]);
            }
            _ => assert!(false),
        }
        let block_1 = &result.1[1];
        match block_1 {
            ExpressionItem::Block(v) => {
                assert_eq!(v.repeat, 4);
                assert!(v.filters.is_empty());
                assert!(matches!(v.source.source, Source::Digit));
            }
            _ => assert!(false),
        }
        let block_2 = &result.1[2];
        match block_2 {
            ExpressionItem::Block(v) => {
                assert_eq!(v.repeat, 1);
                assert!(v.filters.is_empty());
                assert!(matches!(v.source.source, Source::Symbol));
            }
            _ => assert!(false),
        }
    }
    */
}
