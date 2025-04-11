//! TODO and WIP
#![allow(unused)]

use nom::{
  IResult,
  Parser,
  character::complete::one_of,
};

/*
<nonzero_digit> ::= [1-9]
<digit> ::= [0-9]
<letter> ::= [a-z]
<identifier> ::= <letter>+ <digit>*
<escaped> ::= "{" | "}" | "[" | "]" | "(" | ")"
/* any character NOT in escaped */
<noescape_anychar> ::= "#"
<repeat> ::= "!" <nonzero_digit>? <digit>
<source> ::= <identifier> | "[" (<noescape_anychar>)+ "]"
<label> ::= <nonzero_digit>? <digit> "@"
<labeled_source> ::= <label>? <source>
<filter_expr> ::= (<pipe> | <pipe_spaced>) <identifier>
<equals> ::= "="
<equals_spaced> ::= " = "
<key_value> ::= <letter>+ (<equals> | <equals_spaced>) (<letter>+ | <digit>+)
<key_value_seq> ::= (<key_value> ", ")*
<key_value_expr> ::= ":" " "? <key_value_seq> <key_value>
<pipe> ::= "|"
<pipe_spaced> ::= " | "
<block> ::= "(" <labeled_source> <key_value_expr>? <filter_expr>* ")" <repeat>?
<group> ::= "{" <block>+ "}" <repeat>?
<expr> ::= (<group> | <block>)+
*/

fn nonzero_digit(input: &str) -> IResult<&str, char> {
    one_of("123456789").parse(input)
}

fn digit(input: &str) -> IResult<&str, char> {
    one_of("0123456789").parse(input)
}

fn letter(input: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyz").parse(input)
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
}
