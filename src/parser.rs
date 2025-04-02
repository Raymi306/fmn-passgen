//! TODO and WIP
use nom::{
  IResult,
  Parser,
  branch::alt,
  bytes::complete::escaped,
  bytes::complete::is_not,
  bytes::complete::tag_no_case,
  character::complete::alpha1,
  character::complete::alphanumeric1,
  character::complete::anychar,
  character::complete::char,
  character::complete::digit1,
  character::complete::multispace0,
  character::complete::one_of,
  combinator::map_res,
  combinator::opt,
  combinator::value,
  multi::many1,
  sequence::delimited,
  sequence::pair,
  sequence::preceded,
  sequence::separated_pair,
};

const ESCAPE_CONTROL_CHAR: char = '\\';

fn escaped_chars(input: &str) -> IResult<&str, char> {
    one_of("(){}[]\\").parse(input)
}

#[derive(Debug, Clone)]
enum Source {
    Word,
    Digit,
    Symbol,
    Characters(String),
}

fn label_parser(input: &str) -> IResult<&str, u8> {
    let result = pair(map_res(digit1, |s: &str| s.parse::<u8>()), char('@')).parse(input)?;
    Ok((result.0, result.1.0))
}

// TODO need to handle escaping for all instances of delimited

fn source_parser(input: &str) -> IResult<&str, Source> {
    alt((
        value(Source::Word, alt((tag_no_case("word"), tag_no_case("w")))),
        value(Source::Digit, alt((tag_no_case("digit"), tag_no_case("d")))),
        value(Source::Symbol, alt((tag_no_case("symbol"), tag_no_case("s")))),
        delimited(
            char('['),
            escaped(many1(), ESCAPE_CONTROL_CHAR, escaped_chars),
            char(']'),
        ).map(|s: &str| Source::Characters(s.to_string())),
        //delimited(char('['), is_not("]"), char(']')).map(|s: &str| Source::Characters(s.to_string())),
    )).parse(input)
}

fn key_value_pairs_parser(input: &str) -> IResult<&str, (&str, &str)> {
    preceded(
        char(':'),
        separated_pair(
            alpha1,
            delimited(multispace0, char('='), multispace0),
            alphanumeric1,
        ),
    ).parse(input)
}

fn block_parser(input: &str) -> IResult<&str, &str> {
    let raw_result: IResult<&str, &str> = delimited(
        char('('),
        is_not(")"),
        char(')')
    ).parse(input);
    let label = opt(label_parser);
    unimplemented!();
}

fn repeat_parser(input: &str) -> IResult<&str, (char, &str)> {
    pair(char('!'), digit1).parse(input)
}

fn all_blocks_parser(input: &str) -> IResult<&str, Vec<&str>> {
    many1(block_parser).parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_label() {
        let result_err = label_parser("xyz@");
        assert!(result_err.is_err());
        let result_ok = label_parser("1@");
        assert!(result_ok.is_ok());
    }
    #[test]
    fn test_source() {
        let result_err = source_parser("foo");
        assert!(result_err.is_err());
        let result = source_parser("[1234]").unwrap();
        // TODO destructuring to clean this up
        if let Source::Characters(inner) = result.1 {
            assert_eq!("1234", &inner);
        } else {
            panic!();
        }
    }
    #[test]
    fn test_demo() {
        /*
        let result = delimited(
            char('['),
            escaped(many1(anychar), ESCAPE_CONTROL_CHAR, escaped_chars),
            char(']'),
        ).parse("[1234]").unwrap();
        */
        //let result = escaped(anychar, ESCAPE_CONTROL_CHAR, one_of("(){}[]")).parse("[1234]");
        let result = escaped(digit1, '\\', one_of("\"n\\")).parse("1234");
        println!("{:?}", result);
        assert!(false);
    }
    /*
    #[test]
    fn test_block() {
        let result_err = block("(abcd");
        assert!(result_err.is_err());
        let result_ok = block("(abcd)");
        assert!(result_ok.is_ok());
    }
    #[test]
    fn test_repeat() {
        let result_err = repeat("!(");
        assert!(result_err.is_err());
        let result_ok = repeat("!32");
        assert!(result_ok.is_ok());
    }
    #[test]
    fn test_all_blocks() {
        let result = all_blocks("(abcd)(foo)(bar)").unwrap();
        assert_eq!(result.1.len(), 3);
        assert!(result.0.is_empty());
    }
    */
}
