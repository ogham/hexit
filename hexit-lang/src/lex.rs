use std::fmt;
use std::str::CharIndices;

use crate::pos::Placed;
use crate::tokens::Token;


/// Tokenises a line of Hexit into a vector of tokens that contain references
/// to the original source string. Returns an error if there is a problem with
/// the input, using the line number to indicate which line had the problem.
pub fn lex_source<'src>(line_number: usize, input_source: &'src str) -> Result<Vec<Token<'src>>, Error<'src>> {
    let mut lexer = Lexer::new(line_number, input_source);
    while lexer.next_token()? {}
    lexer.last_token()?;
    Ok(lexer.tokens)
}


/// The lexing processor. A lexer analyses the input string, one character at
/// a time, mutating an internal state depending on what the character was.
struct Lexer<'src> {

    /// The line number of the input program. An error thrown during lexing
    /// contains this number to show to the user.
    line_number: usize,

    /// The character iterator that yields characters to analyse.
    iter: CharIndices<'src>,

    /// The input source string, which gets referenced as slices in the tokens.
    input_source: &'src str,

    /// The lexer's current state, which changes as characters are read.
    state: State,

    /// The vector of tokens that gets built up.
    tokens: Vec<Token<'src>>,
}

/// The state that a lexer can be in.
#[derive(Debug, Copy, Clone)]
enum State {

    /// We have just started lexing, or the previous character emitted a token
    /// that does not depend on the _next_ character, so we are ready for
    /// anything.
    Ready,

    /// We have just read one or more whitespace characters.
    ReadWhitespace,

    /// We have just read one or more alphanumeric characters, the first of
    /// which occurred at the anchor index of the source string.
    ReadAlphanum { anchor: usize },

    /// We have just read the opening `[` bracket of a form, or one of the
    /// characters within the form.
    ReadForm { anchor: usize },

    /// We have just read the opening `"` quote of a quoted string, or one of
    /// the characters within the string.
    ReadQuote { anchor: usize, backslash: bool },

    /// We are done parsing this line, so no further token should be produced.
    Done,
}


#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
impl<'src> Lexer<'src> {

    /// Creates a new lexer with the given parameters.
    fn new(line_number: usize, input_source: &'src str) -> Self {
        let state = State::Ready;
        let iter = input_source.char_indices();
        let tokens = Vec::new();
        Self { line_number, input_source, iter, state, tokens }
    }

    /// Analyse the next character from the iterator, possibly changing the
    /// internal state or pushing one or two tokens onto the internal vector.
    ///
    /// Returns `true` if there are more characters to read, `false` if there
    /// are no more, and an error if there is a problem with the user’s input.
    fn next_token(&mut self) -> Result<bool, Error<'src>> {
        let (column, c) = match self.iter.next() {
            Some(tuple) => tuple,
            None        => return Ok(false),
        };

        match (c, self.state) {
            (c, State::Ready) if c.is_ascii_alphanumeric() || c == '_' => {
                self.state = State::ReadAlphanum { anchor: column };
            }
            (c, State::ReadAlphanum { .. }) if c.is_ascii_alphanumeric() || c == '_' => {
                // continue reading alphanums
            }
            (c, State::ReadWhitespace) if c.is_ascii_alphanumeric() || c == '_' => {
                self.tokens.push(Token::Whitespace);
                self.state = State::ReadAlphanum { anchor: column };
            }

            (c, State::Ready) if c.is_whitespace() => {
                self.state = State::ReadWhitespace;
            }
            (c, State::ReadWhitespace) if c.is_whitespace() => {
                // continue reading whitespace
            }

            (c, State::ReadAlphanum { anchor }) if c.is_ascii_whitespace() => {
                let alphanum_string = self.span(anchor, column);
                self.tokens.push(Token::Alphanum(alphanum_string));
                self.state = State::ReadWhitespace;
            }

            ('[', State::ReadWhitespace) => {
                self.tokens.push(Token::Whitespace);
                self.state = State::ReadForm { anchor: column };
            }
            ('[', State::Ready) => {
                self.state = State::ReadForm { anchor: column };
            }
            ('[', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, column);
                self.tokens.push(Token::Alphanum(alphanum_string));
                self.state = State::ReadForm { anchor: column };
            }
            (']', State::ReadForm { anchor }) => {
                let form_string = self.span(anchor + 1, column);
                self.tokens.push(Token::Form(form_string));
                self.state = State::Ready;
            }
            (_, State::ReadForm { anchor: _ }) => {
                // continue reading the form
            }

            ('"', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, column);
                self.tokens.push(Token::Alphanum(alphanum_string));
                self.state = State::ReadQuote { anchor: column, backslash: false };
            }
            ('"', State::ReadWhitespace) => {
                self.tokens.push(Token::Whitespace);
                self.state = State::ReadQuote { anchor: column, backslash: false };
            }
            ('"', State::Ready) => {
                self.state = State::ReadQuote { anchor: column, backslash: false };
            }
            ('\\', State::ReadQuote { anchor, backslash: false }) => {
                self.state = State::ReadQuote { anchor, backslash: true };
            }
            ('"', State::ReadQuote { anchor, backslash: false }) => {
                let quoted = self.span(anchor + 1, column);
                self.tokens.push(Token::Quoted(quoted));
                self.state = State::Ready;
            }
            (_, State::ReadQuote { anchor, .. }) => {
                // continue reading the string
                self.state = State::ReadQuote { anchor, backslash: false };
            }

            ('#', _) => {
                // we are done parsing this line
                self.state = State::Done;
            }
            (_, State::Done) => {
                // we are DONE
            }

            ('(', State::Ready) => {
                let open_string = self.span(column, column + 1);
                self.tokens.push(Token::Open(open_string));
            }
            (')', State::Ready) => {
                let close_string = self.span(column, column + 1);
                self.tokens.push(Token::Close(close_string));
            }
            ('(', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, column);
                self.tokens.push(Token::Alphanum(alphanum_string));
                let open_string = self.span(column, column + 1);
                self.tokens.push(Token::Open(open_string));
                self.state = State::Ready;
            }
            (')', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, column);
                self.tokens.push(Token::Alphanum(alphanum_string));
                let close_string = self.span(column, column + 1);
                self.tokens.push(Token::Close(close_string));
                self.state = State::Ready;
            }

            (c, _) => {
                let char_string = self.span(column, column + 1);
                return Err(Error::UnknownCharacter(char_string));
            }
        }

        Ok(true)
    }

    /// Adds one final token to the vector, or throws an error, depending on
    /// the internal state after all characters have been analysed.
    fn last_token(&mut self) -> Result<(), Error<'src>> {
        match self.state {
            State::Ready | State::ReadWhitespace | State::Done => {
                // nothing more to add
                Ok(())
            }
            State::ReadAlphanum { anchor } => {
                let alphanum_string = self.span_rest(anchor);
                self.tokens.push(Token::Alphanum(alphanum_string));
                Ok(())
            }
            State::ReadQuote { anchor, .. } => {
                let unclosed_string = self.span_rest(anchor);
                Err(Error::UnclosedString(unclosed_string))
            }
            State::ReadForm { anchor, .. } => {
                let unclosed_form = self.span_rest(anchor);
                Err(Error::UnclosedForm(unclosed_form))
            }
        }
    }

    fn span(&self, anchor: usize, column: usize) -> Placed<&'src str> {
        let contents = &self.input_source[ anchor .. column ];
        Placed { contents, line_number: self.line_number }
    }

    fn span_rest(&self, anchor: usize) -> Placed<&'src str> {
        let contents = &self.input_source[ anchor .. ];
        Placed { contents, line_number: self.line_number }
    }
}


/// An error that can occur during lexing.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {

    /// There was an unknown character in the input.
    UnknownCharacter(Placed<&'src str>),

    /// The lexer was expecting a closing `"` quote when the input ended.
    UnclosedString(Placed<&'src str>),

    /// The lexer was expecting a closing `]` bracket when the input ended.
    UnclosedForm(Placed<&'src str>),
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCharacter(c)  => write!(f, "Unknown character {:?}", c.contents),
            Self::UnclosedString(s)    => write!(f, "Unclosed string {:?}", s.contents),
            Self::UnclosedForm(o)      => write!(f, "Unclosed form {:?}", o.contents),
        }
    }
}

impl<'src> Error<'src> {
    pub fn source_pos(&self) -> &Placed<&'src str> {
        match self {
            Self::UnknownCharacter(c) => c,
            Self::UnclosedString(s)   => s,
            Self::UnclosedForm(form)  => form,
        }
    }
}



#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use crate::pos::At;
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(lex_source(0, ""),
                   Ok(vec![ ]));
    }

    #[test]
    fn stray() {
        assert_eq!(lex_source(0, "&"),
                   Err(Error::UnknownCharacter("&".at(0))));
    }

    #[test]
    fn some_bytes() {
        assert_eq!(lex_source(1, "1A2B"),
                   Ok(vec![ Token::Alphanum("1A2B".at(1)) ]));
    }

    #[test]
    fn a_quoted() {
        assert_eq!(lex_source(2, "\"PANL\""),
                   Ok(vec![ Token::Quoted("PANL".at(2)) ]));
    }

    #[test]
    fn a_form() {
        assert_eq!(lex_source(3, "[FORM]"),
                   Ok(vec![ Token::Form("FORM".at(3)) ]));
    }

    #[test]
    fn eventually_a_quoted() {
        assert_eq!(lex_source(4, "    \"PANL\""),
                   Ok(vec![ Token::Whitespace,
                            Token::Quoted("PANL".at(4)) ]));
    }

    #[test]
    fn eventually_a_form() {
        assert_eq!(lex_source(5, "    [FORM]"),
                   Ok(vec![ Token::Whitespace,
                            Token::Form("FORM".at(5)) ]));
    }

    #[test]
    fn unclosed_quote() {
        assert_eq!(lex_source(6, "\"FORM"),
                   Err(Error::UnclosedString("\"FORM".at(6))));
    }

    #[test]
    fn unclosed_form() {
        assert_eq!(lex_source(7, "[FORM"),
                   Err(Error::UnclosedForm("[FORM".at(7))));
    }

    #[test]
    fn in_parentheses() {
        assert_eq!(lex_source(8, "(AB34)"),
                   Ok(vec![ Token::Open("(".at(8)),
                            Token::Alphanum("AB34".at(8)),
                            Token::Close(")".at(8)) ]));
    }

    #[test]
    fn function_call() {
        assert_eq!(lex_source(9, "x86(AB34)"),
                   Ok(vec![ Token::Alphanum("x86".at(9)),
                            Token::Open("(".at(9)),
                            Token::Alphanum("AB34".at(9)),
                            Token::Close(")".at(9)), ]));
    }

    #[test]
    fn surrounded_by_quotes() {
        assert_eq!(lex_source(10, "\"\"\"\"A\"\"\"\""),
                   Ok(vec![ Token::Quoted("".at(10)),
                            Token::Quoted("".at(10)),
                            Token::Alphanum("A".at(10)),
                            Token::Quoted("".at(10)),
                            Token::Quoted("".at(10)), ]));
    }

    #[test]
    fn quotes_backslashes() {
        assert_eq!(lex_source(11, "\"\\\"\""),
                   Ok(vec![ Token::Quoted("\\\"".at(11)) ]));
    }

    #[test]
    fn mixture() {
        assert_eq!(lex_source(12, "1A2B[FORM]\"PANL\"[FORM]1A2B\"PANL\"1A2B"),
                   Ok(vec![ Token::Alphanum("1A2B".at(12)),
                            Token::Form("FORM".at(12)),
                            Token::Quoted("PANL".at(12)),
                            Token::Form("FORM".at(12)),
                            Token::Alphanum("1A2B".at(12)),
                            Token::Quoted("PANL".at(12)),
                            Token::Alphanum("1A2B".at(12)), ]));
    }

    #[test]
    fn a_lowly_underscore() {
        assert_eq!(lex_source(13, "___ _"),
                   Ok(vec![ Token::Alphanum("___".at(13)),
                            Token::Whitespace,
                            Token::Alphanum("_".at(13)), ]));
    }

    #[test]
    fn whitespace_then_quoted_nothing() {
        assert_eq!(lex_source(14, "    \"\""),
                   Ok(vec![ Token::Whitespace,
                            Token::Quoted("".at(14)) ]));
    }
}
