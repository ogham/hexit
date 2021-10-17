//! The reading stage, which involves taking a string slice, running it
//! through the lexing and parsing stages, and removing front and back
//! comments before returning it as an `Exp`.

use std::fmt;

use log::*;

use crate::{ast, lex, parse, pos, tokens};


/// Reads a Hexit program into a vector of expressions, by splitting the input
/// into a sequence of lines, lexing and parsing each line. An error is
/// returned as soon as something fails to be lexed or parsed.
pub fn tokenise_and_parse<'src>(input_source: &'src str) -> Result<Vec<ast::Exp<'src>>, Error<'src>> {
    let mut all_tokens = Vec::new();

    for (line_index, mut input_line) in input_source.lines().enumerate() {
        if let Some(colon_pos) = input_line.find(':') {
            if input_line[ .. colon_pos ].bytes().all(|b| b.is_ascii_whitespace() || b.is_ascii_alphanumeric()) {
                input_line = &input_line[ colon_pos + 1 .. ];
            }
        }

        let line_number = line_index + 1;
        let line_tokens = lex::lex_source(line_number, input_line).map_err(Error::Lex)?;
        trace!("Tokens: {:#?}", line_tokens);
        all_tokens.extend(line_tokens);
        all_tokens.push(tokens::Token::Whitespace);
    }

    let exps = parse::parse_tokens(&mut all_tokens.into_iter()).map_err(Error::Parse)?;
    Ok(exps)
}


/// A problem that occurred with the user’s input during parsing or lexing.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {
    Lex(lex::Error<'src>),
    Parse(parse::Error<'src>),
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(le)    => le.fmt(f),
            Self::Parse(pe)  => pe.fmt(f),
        }
    }
}

impl<'src> Error<'src> {
    pub fn source_pos(&self) -> &pos::Placed<&'src str> {
        match self {
            Self::Lex(le)    => le.source_pos(),
            Self::Parse(pe)  => pe.source_pos(),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::pos::At;

    #[test]
    fn empty() {
        assert_eq!(tokenise_and_parse(""),
                   Ok(vec![]));
    }

    #[test]
    fn space() {
        assert_eq!(tokenise_and_parse(" "),
                   Ok(vec![]));
    }

    #[test]
    fn spaces() {
        assert_eq!(tokenise_and_parse("  "),
                   Ok(vec![]));
    }


    #[test]
    fn lonely() {
        assert_eq!(tokenise_and_parse("0"),
                   Err(Error::Parse(parse::Error::SingleHex("0".at(1, 0)))));
    }

    #[test]
    fn meme() {
        assert_eq!(tokenise_and_parse("E"),
                   Err(Error::Parse(parse::Error::SingleHex("E".at(1, 0)))));
    }

    #[test]
    fn otherwise() {
        assert_eq!(tokenise_and_parse("q"),
                   Err(Error::Parse(parse::Error::StrayCharacter("q".at(1, 0)))));
    }

    #[test]
    fn atrophy() {
        assert_eq!(tokenise_and_parse("\\"),
                   Err(Error::Lex(lex::Error::UnknownCharacter("\\".at(1, 0)))));
    }

    #[test]
    fn closure() {
        assert_eq!(tokenise_and_parse(")"),
                   Err(Error::Parse(parse::Error::StrayCharacter(")".at(1, 0)))));
    }

    #[test]
    fn exordium() {
        assert_eq!(tokenise_and_parse("["),
                   Err(Error::Lex(lex::Error::UnclosedForm("[".at(1, 0)))));
    }
}
