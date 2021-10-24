//! The reading stage, which involves taking a string slice, running it
//! through the lexing and parsing stages, and removing front and back
//! comments before returning it as an `Exp`.

use std::fmt;

use log::*;

use crate::{ast, lex, parse, pos, tokens};


/// Reads a Hexit program into a vector of expressions, by splitting the input
/// into a sequence of lines, lexing and parsing each line. An error is
/// returned as soon as something fails to be lexed or parsed.
pub fn tokenise_and_parse<'src>(input_line: &'src str, line_number: usize) -> Result<Vec<ast::Exp<'src>>, Error<'src>> {
    let mut line_tokens = lex::lex_source(line_number, input_line).map_err(Error::Lex)?;
    trace!("Tokens: {:#?}", line_tokens);

    strip_front_comment(&mut line_tokens);

    if let Some(first_invalid_char) = line_tokens.iter().find_map(|t| t.as_stray()) {
        return Err(Error::UnknownChar(first_invalid_char));
    }

    line_tokens.push(tokens::Token::Whitespace);

    let exps = parse::parse_tokens(&mut line_tokens.into_iter()).map_err(Error::Parse)?;
    Ok(exps)
}


/// Removes the front comment part of a vector of tokens by finding the last
/// colon token in the line, and if one exists, draining the vector of every
/// token up to and including that point.
fn strip_front_comment<'src>(line_tokens: &mut Vec<tokens::Token<'src>>) {
    if let Some(last_colon_index) = line_tokens.iter().rposition(|t| t.is_colon()) {
        line_tokens.drain(..=last_colon_index);
    }
}


/// A problem that occurred with the user’s input during parsing or lexing.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {
    UnknownChar(pos::Placed<&'src str>),
    Lex(lex::Error<'src>),
    Parse(parse::Error<'src>),
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownChar(c)  => write!(f, "Unknown character {:?}", c.contents),
            Self::Lex(le)         => le.fmt(f),
            Self::Parse(pe)       => pe.fmt(f),
        }
    }
}

impl<'src> Error<'src> {
    pub fn source_pos(&self) -> &pos::Placed<&'src str> {
        match self {
            Self::UnknownChar(c)  => c,
            Self::Lex(le)         => le.source_pos(),
            Self::Parse(pe)       => pe.source_pos(),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::pos::At;

    // empty and spaces tests

    #[test]
    fn empty() {
        assert_eq!(tokenise_and_parse("", 1),
                   Ok(vec![]));
    }

    #[test]
    fn space() {
        assert_eq!(tokenise_and_parse(" ", 1),
                   Ok(vec![]));
    }

    #[test]
    fn spaces() {
        assert_eq!(tokenise_and_parse("  ", 1),
                   Ok(vec![]));
    }

    // parse and lex error tests

    #[test]
    fn lonely() {
        assert_eq!(tokenise_and_parse("0", 1),
                   Err(Error::Parse(parse::Error::SingleHex("0".at(1, 0)))));
    }

    #[test]
    fn meme() {
        assert_eq!(tokenise_and_parse("E", 2),
                   Err(Error::Parse(parse::Error::SingleHex("E".at(2, 0)))));
    }

    #[test]
    fn otherwise() {
        assert_eq!(tokenise_and_parse("q", 3),
                   Err(Error::Parse(parse::Error::StrayCharacter("q".at(3, 0)))));
    }

    #[test]
    fn closure() {
        assert_eq!(tokenise_and_parse(")", 4),
                   Err(Error::Parse(parse::Error::StrayCharacter(")".at(4, 0)))));
    }

    #[test]
    fn exordium() {
        assert_eq!(tokenise_and_parse("[", 5),
                   Err(Error::Lex(lex::Error::UnclosedForm("[".at(5, 0)))));
    }

    #[test]
    fn weird_nested_form() {
        assert_eq!(tokenise_and_parse("[[:alpha:]]", 6),
                   Err(Error::UnknownChar("]".at(6, 10))));
    }

    // front comment stripping tests

    #[test]
    fn front_comment() {
        assert_eq!(tokenise_and_parse("Magic number: 03", 7),
                   Ok(vec![ ast::Exp::Char(3) ]));
    }

    #[test]
    fn front_comment_containing_chars() {
        assert_eq!(tokenise_and_parse("Magic••••number: 03", 8),
                   Ok(vec![ ast::Exp::Char(3) ]));
    }

    #[test]
    fn front_comment_containing_form() {
        assert_eq!(tokenise_and_parse("[Magic] number: 03", 9),
                   Ok(vec![ ast::Exp::Char(3) ]));
    }

    #[test]
    fn front_comment_containing_form_containing_colon() {
        assert_eq!(tokenise_and_parse("[[:alpha:]] number: 03", 10),
                   Ok(vec![ ast::Exp::Char(3) ]));
    }

    #[test]
    fn front_comment_containing_string() {
        assert_eq!(tokenise_and_parse("\"Magic\" number: 03", 11),
                   Ok(vec![ ast::Exp::Char(3) ]));
    }

    #[test]
    fn front_comment_containing_string_containing_colon() {
        assert_eq!(tokenise_and_parse("\"Magic:::number\": 03", 12),
                   Ok(vec![ ast::Exp::Char(3) ]));
    }
}
