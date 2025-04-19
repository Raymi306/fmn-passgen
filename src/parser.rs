//! TODO and WIP
#![allow(unused)]

// provides:
// static WORDLIST: &[&str] = &[...]
include!(concat!(env!("OUT_DIR"), "/wordlist.rs"));

use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, is_a, tag, tag_no_case};
use nom::character::complete::{char, digit1, none_of, one_of};
use nom::combinator::{all_consuming, map, map_res, opt, recognize, success, value};
use nom::multi::{many0, many1};
use nom::sequence::{pair, preceded, separated_pair, terminated};
use nom::{IResult, Parser};
use rand::TryRngCore;
use rand::prelude::*;
use rand_core::UnwrapErr;

#[derive(Clone, Debug, PartialEq)]
enum SourceKind {
    Word(Option<u8>, Option<u8>),
    Letter,
    Symbol,
    Digit,
    CharacterList(String),
}

impl SourceKind {
    fn apply<R: TryRngCore>(&self, rng: UnwrapErr<R>) -> String {
        match &self {
            Self::Word(min, max) => {
                let filtered_indices: Vec<usize> = WORDLIST
                    .iter()
                    .enumerate()
                    .filter(|(_, word)| (min..=max).contains(&word.chars().count()))
                    .map(|(i, _)| i)
                    .collect();
                if filtered_indices.is_empty() {
                    return String::new();
                }
                let index = filtered_indices.choose(&mut rng).expect(
                    concat!(
                        "invariant 1: `filtered_indices` must not be empty and should have been guarded above.\n",
                        "invariant 2: size_hint on a slice iterator with no intermediary ",
                        "iterator adapters should always be accurate.",
                    )
                );
                WORDLIST[*index].to_owned()
            },
            /*
            Self::Letter => {

            },
            Self::Symbol => {

            },
            Self::Digit => {

            },
            Self::CharacterList(String) => {

            },
        }
        fn letter(&mut self) -> char {
            ('a'..='z').chain('A'..='Z').choose(&mut self.rng).unwrap()
        }
        fn symbol(&mut self) -> char {
            "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
            .chars()
            .choose(&mut self.rng).unwrap()
        }
        fn digit(&mut self) -> char {
            ('0'..='9').choose(&mut self.rng).unwrap()
        }
        fn character(&mut self, choices: String) -> char {
            choices.chars().choose(&mut self.rng).unwrap()
        }
        */
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct LabeledSource {
    source: SourceKind,
    label: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
enum Filter {
    Reversed,
    Upper,
    Lower,
    CapitalizeFirst,
    CapitalizeLast,
    CapitalizeNotFirst,
    CapitalizeNotLast,
}

impl Filter {
    fn apply(&self, input: &str) -> String {
        match &self {
            Self::Reversed => input.chars().rev().collect(),
            Self::Upper => input.to_uppercase(),
            Self::Lower => input.to_lowercase(),
            Self::CapitalizeFirst => {
                let first = input.chars().take(1).map(|c| c.to_ascii_uppercase());
                first.chain(input.chars().skip(1)).collect()
            },
            Self::CapitalizeLast => {
                // UTF character length weirdness reminder
                let num_chars = input.chars().count();
                let len_minus_1 = num_chars.saturating_sub(1);
                input.chars()
                    .take(len_minus_1)
                    .chain(
                        input.chars()
                            .skip(len_minus_1)
                            .take(1)
                            .map(|c| c.to_ascii_uppercase()),
                    )
                    .collect()
            },
            Self::CapitalizeNotFirst => {
                input.chars()
                    .take(1)
                    .chain(input.chars().skip(1).map(|c| c.to_ascii_uppercase()))
                    .collect()
            },
            Self::CapitalizeNotLast => {
                // UTF character length weirdness reminder
                let num_chars = input.chars().count();
                let len_minus_1 = num_chars.saturating_sub(1);
                input.chars()
                    .take(len_minus_1)
                    .map(|c| c.to_ascii_uppercase())
                    .chain(
                        input.chars()
                            .skip(len_minus_1)
                            .take(1)
                    )
                    .collect()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum KeyValue {
    Min(u8),
    Max(u8),
}

#[derive(Debug, PartialEq)]
struct Block {
    source: LabeledSource,
    filters: Vec<Filter>,
    // NOTE could make these u8 and just default to 1
    repeat: Option<u8>,
}

#[derive(Debug, PartialEq)]
struct Group {
    blocks: Vec<Block>,
    repeat: Option<u8>,
}

#[derive(Debug)]
enum ExpressionItem {
    Block(Block),
    Group(Group),
}

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
const ESCAPED: &str = "\\{}[]()";

fn nonzero_digit(input: &str) -> IResult<&str, char> {
    one_of("123456789").parse(input)
}

fn digit(input: &str) -> IResult<&str, char> {
    one_of("0123456789").parse(input)
}

fn digits(input: &str) -> IResult<&str, &str> {
    is_a("0123456789").parse(input)
}

fn letter(input: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyz").parse(input)
}

fn letters(input: &str) -> IResult<&str, &str> {
    is_a("abcdefghijklmnopqrstuvwxyz").parse(input)
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

fn character_list(input: &str) -> IResult<&str, String> {
    terminated(preceded(char('['), anychar_escaped_transform), char(']')).parse(input)
}

fn source(input: &str) -> IResult<&str, SourceKind> {
    if let Ok(chars) = character_list(input) {
        return Ok((chars.0, SourceKind::CharacterList(chars.1)));
    }
    alt((
        value(SourceKind::Word(None, None), tag_no_case("word")),
        value(SourceKind::Letter, tag_no_case("letter")),
        value(SourceKind::Symbol, tag_no_case("symbol")),
        value(SourceKind::Digit, tag_no_case("digit")),
    ))
    .parse(input)
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
                let mut min = None;
                let mut max = None;
                if !matches!(labeled_source.source, SourceKind::Word(_, _)) {
                    // only Word has key value support currently
                    return Err(());
                }
                for inner in kv {
                    match inner {
                        KeyValue::Min(v) => min = Some(v),
                        KeyValue::Max(v) => max = Some(v),
                    }
                }
                labeled_source.source = SourceKind::Word(min, max);
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
            repeat,
        },
    )
    .parse(input)
}

fn group(input: &str) -> IResult<&str, ExpressionItem> {
    map(
        pair(
            terminated(preceded(char('{'), many1(block)), char('}')),
            opt(repeat),
        ),
        |(blocks, repeat)| ExpressionItem::Group(Group { blocks, repeat }),
    )
    .parse(input)
}

fn parse(input: &str) -> IResult<&str, Vec<ExpressionItem>> {
    all_consuming(many1(alt((group, map(block, ExpressionItem::Block))))).parse(input)
}

// TODO remove me
#[warn(unused)]
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
    #[test]
    fn test_parse_ok() {
        let input =
            "{(word:min=3, max=11 | lower)(1@symbol)(word | upper)(1@symbol)}!2(digit)!4(symbol)";
        let result = parse(input).unwrap();
        assert_eq!(result.1.len(), 3);

        let group = &result.1[0];
        match group {
            ExpressionItem::Group(v) => {
                assert_eq!(v.repeat, Some(2));
                assert_eq!(v.blocks.len(), 4);
                let expected_1 = Block {
                    source: LabeledSource {
                        source: SourceKind::Word(Some(3), Some(11)),
                        label: None,
                    },
                    filters: vec![Filter::Lower],
                    repeat: None,
                };
                let expected_2 = Block {
                    source: LabeledSource {
                        source: SourceKind::Symbol,
                        label: Some(1),
                    },
                    filters: vec![],
                    repeat: None,
                };
                let expected_3 = Block {
                    source: LabeledSource {
                        source: SourceKind::Word(None, None),
                        label: None,
                    },
                    filters: vec![Filter::Upper],
                    repeat: None,
                };
                let expected_4 = Block {
                    source: LabeledSource {
                        source: SourceKind::Symbol,
                        label: Some(1),
                    },
                    filters: vec![],
                    repeat: None,
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
                assert_eq!(v.repeat, Some(4));
                assert!(v.filters.is_empty());
                assert!(matches!(v.source.source, SourceKind::Digit));
            }
            _ => assert!(false),
        }
        let block_2 = &result.1[2];
        match block_2 {
            ExpressionItem::Block(v) => {
                assert_eq!(v.repeat, None);
                assert!(v.filters.is_empty());
                assert!(matches!(v.source.source, SourceKind::Symbol));
            }
            _ => assert!(false),
        }
    }
}
