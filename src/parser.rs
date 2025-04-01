//! TODO and WIP
use nom::{
  IResult,
  Parser,
  branch::alt,
  bytes::complete::is_not,
  bytes::complete::tag_no_case,
  character::complete::alpha1,
  character::complete::alphanumeric1,
  character::complete::char,
  character::complete::digit1,
  character::complete::multispace0,
  combinator::map,
  combinator::opt,
  multi::many1,
  sequence::delimited,
  sequence::pair,
  sequence::preceded,
  sequence::separated_pair,
};

fn label_parser(input: &str) -> IResult<&str, (&str, u8)> {
    //map(pair(digit1, char('@')), |digits| u8::from(digits)).parse(input)
    unimplemented!()
}

fn source_parser(input: &str) -> IResult<&str, &str> {
    alt((
        tag_no_case("word"), tag_no_case("w"),
        tag_no_case("digit"), tag_no_case("d"),
        tag_no_case("symbol"), tag_no_case("s"),
        delimited(char('['), is_not("]"), char(']')),
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

    /*
    #[test]
    fn test_label() {
        let result_err = label("xyz@");
        assert!(result_err.is_err());
        let result_ok = label("1@");
        assert!(result_ok.is_ok());
    }
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
