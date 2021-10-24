//! The lexical analysis step, or tokenising. This involves

use std::fmt;
use std::str::CharIndices;

use crate::pos::Placed;
use crate::tokens::Token;


/// Tokenises a line of Hexit into a vector of tokens that contain references
/// to the original source string. Returns an error if there is a problem with
/// the input, using the line number to indicate which line had the problem.
pub fn lex_source<'src>(line_number: usize, input_source: &'src str) -> Result<Vec<Token<'src>>, Error<'src>> {
    let mut lexer = Lexer::new(line_number, input_source);
    while lexer.next_token() {}
    lexer.last_token()?;
    Ok(lexer.tokens)
}


/// The lexing processor. A lexer analyses the input string, one character at
/// a time, mutating an internal state depending on what the character was.
struct Lexer<'src> {

    /// The line number of the input program. An error thrown during lexing
    /// contains this number to show to the user. This does not change.
    line_number: usize,

    /// The input source string, which gets referenced as slices in the tokens.
    /// This does not change.
    input_source: &'src str,

    /// The character iterator that yields characters to analyse.
    iter: CharIndices<'src>,

    /// The column number of this line, which gets incremented as characters
    /// get read from the iterator.
    column_number: usize,

    /// The lexer’s current state, which changes as characters are read.
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
    ReadAlphanum { anchor: Anchor },

    /// We have just read the opening `[` bracket of a form, or one of the
    /// characters within the form.
    ReadForm { anchor: Anchor },

    /// We have just read the opening `"` quote of a quoted string, or one of
    /// the characters within the string. We also need to store whether we
    /// have just read a backslash, in order to handle nested quotes.
    ReadQuote { anchor: Anchor, backslash: bool },

    /// We are done parsing this line, so no further token should be produced.
    Done,
}

/// A stored position in the source string.
#[derive(Debug, Copy, Clone)]
struct Anchor {

    /// The number of bytes into the source string. This is used to index the
    /// string in order to produce `Placed` slices.
    index: usize,

    /// The number of characters into the source string, taking into account
    /// possibly multi-byte characters. This gets shown to the user in case of
    /// an error.
    column_number: usize,
}


#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
impl<'src> Lexer<'src> {

    /// Creates a new lexer with the given parameters.
    fn new(line_number: usize, input_source: &'src str) -> Self {
        let iter = input_source.char_indices();
        let column_number = 0;
        let state = State::Ready;
        let tokens = Vec::new();

        Self { line_number, input_source, iter, column_number, state, tokens }
    }

    /// Analyses the next character from the iterator, possibly changing the
    /// internal state or pushing one or two tokens onto the internal vector.
    ///
    /// Returns `true` if there are more characters to read, and `false` if
    /// there are no more. This function cannot fail, as the “unknown
    /// character” error is checked for and handled later, in order to have
    /// front comments.
    fn next_token(&mut self) -> bool {
        let (index, c) = match self.iter.next() {
            Some(tuple)  => tuple,
            None         => return false,
        };

        let column_number = self.column_number;

        match (c, self.state) {
            (c, State::Ready) if c.is_ascii_alphanumeric() || c == '_' => {
                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadAlphanum { anchor: new_anchor };
            }
            (c, State::ReadAlphanum { .. }) if c.is_ascii_alphanumeric() || c == '_' => {
                // continue reading alphanums
            }
            (c, State::ReadWhitespace) if c.is_ascii_alphanumeric() || c == '_' => {
                self.tokens.push(Token::Whitespace);

                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadAlphanum { anchor: new_anchor };
            }

            (c, State::Ready) if c.is_whitespace() => {
                self.state = State::ReadWhitespace;
            }
            (c, State::ReadWhitespace) if c.is_whitespace() => {
                // continue reading whitespace
            }

            (c, State::ReadAlphanum { anchor }) if c.is_ascii_whitespace() => {
                let alphanum_string = self.span(anchor, index);
                self.tokens.push(Token::Alphanum(alphanum_string));

                self.state = State::ReadWhitespace;
            }

            ('[', State::ReadWhitespace) => {
                self.tokens.push(Token::Whitespace);

                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadForm { anchor: new_anchor };
            }
            ('[', State::Ready) => {
                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadForm { anchor: new_anchor };
            }
            ('[', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, index);
                self.tokens.push(Token::Alphanum(alphanum_string));

                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadForm { anchor: new_anchor };
            }
            (']', State::ReadForm { mut anchor }) => {
                anchor.index += 1;
                let form_string = self.span(anchor, index);
                self.tokens.push(Token::Form(form_string));

                self.state = State::Ready;
            }
            (_, State::ReadForm { anchor: _ }) => {
                // continue reading the form
            }

            ('"', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, index);
                self.tokens.push(Token::Alphanum(alphanum_string));

                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadQuote { anchor: new_anchor, backslash: false };
            }
            ('"', State::ReadWhitespace) => {
                self.tokens.push(Token::Whitespace);

                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadQuote { anchor: new_anchor, backslash: false };
            }
            ('"', State::Ready) => {
                let new_anchor = Anchor { index, column_number };
                self.state = State::ReadQuote { anchor: new_anchor, backslash: false };
            }
            ('\\', State::ReadQuote { anchor, backslash: false }) => {
                self.state = State::ReadQuote { anchor, backslash: true };
            }
            ('"', State::ReadQuote { mut anchor, backslash: false }) => {
                anchor.index += 1;
                let quoted = self.span(anchor, index);
                self.tokens.push(Token::Quoted(quoted));

                self.state = State::Ready;
            }
            (_, State::ReadQuote { anchor, .. }) => {
                // continue reading the string, but switch off backslash flag
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
                let char_position = Anchor { index, column_number };
                let open_string = self.span(char_position, index + 1);
                self.tokens.push(Token::Open(open_string));
            }
            (')', State::Ready) => {
                let char_position = Anchor { index, column_number };
                let close_string = self.span(char_position, index + 1);
                self.tokens.push(Token::Close(close_string));
            }
            ('(', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, index);
                self.tokens.push(Token::Alphanum(alphanum_string));

                let char_position = Anchor { index, column_number };
                let open_string = self.span(char_position, index + 1);
                self.tokens.push(Token::Open(open_string));
                self.state = State::Ready;
            }
            (')', State::ReadAlphanum { anchor }) => {
                let alphanum_string = self.span(anchor, index);
                self.tokens.push(Token::Alphanum(alphanum_string));

                let char_position = Anchor { index, column_number };
                let close_string = self.span(char_position, index + 1);
                self.tokens.push(Token::Close(close_string));
                self.state = State::Ready;
            }

            (c, _) => {
                let char_position = Anchor { index, column_number };
                let char_string = self.span(char_position, index + c.len_utf8());
                self.tokens.push(Token::Stray(char_string));
                self.state = State::Ready;
            }
        }

        self.column_number += 1;
        true
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

    /// Returns a `Placed` string slice of the original input string between
    /// the given anchor and end position.
    fn span(&self, anchor: Anchor, to: usize) -> Placed<&'src str> {
        Placed {
            contents: &self.input_source[ anchor.index .. to ],
            line_number: self.line_number,
            column_number: anchor.column_number,
        }
    }

    /// Returns a `Placed` string slice of the original input string that
    /// starts at the given anchor and lasts until the end of the string.
    fn span_rest(&self, anchor: Anchor) -> Placed<&'src str> {
        Placed {
            contents: &self.input_source[ anchor.index .. ],
            line_number: self.line_number,
            column_number: anchor.column_number,
        }
    }
}


/// An error that can occur during lexing.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {

    /// The lexer was expecting a closing `"` quote when the input ended.
    UnclosedString(Placed<&'src str>),

    /// The lexer was expecting a closing `]` bracket when the input ended.
    UnclosedForm(Placed<&'src str>),
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedString(s)    => write!(f, "Unclosed string {:?}", s.contents),
            Self::UnclosedForm(o)      => write!(f, "Unclosed form {:?}", o.contents),
        }
    }
}

impl<'src> Error<'src> {

    /// Returns the `Placed` position that an error occurred in the source
    /// file. This gets shown to the user.
    pub fn source_pos(&self) -> &Placed<&'src str> {
        match self {
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
                   Ok(vec![ Token::Stray("&".at(0, 0)) ]));
    }

    #[test]
    fn utf8() {
        assert_eq!(lex_source(0, "é"),
                   Ok(vec![ Token::Stray("é".at(0, 0)) ]));
    }

    #[test]
    fn utf8_column() {
        assert_eq!(lex_source(0, "Aé"),
                   Ok(vec![ Token::Stray("é".at(0, 1)) ]));
    }

    #[test]
    fn some_bytes() {
        assert_eq!(lex_source(1, "1A2B"),
                   Ok(vec![ Token::Alphanum("1A2B".at(1, 0)) ]));
    }

    #[test]
    fn a_quoted() {
        assert_eq!(lex_source(2, "\"PANL\""),
                   Ok(vec![ Token::Quoted("PANL".at(2, 0)) ]));
    }

    #[test]
    fn a_form() {
        assert_eq!(lex_source(3, "[FORM]"),
                   Ok(vec![ Token::Form("FORM".at(3, 0)) ]));
    }

    #[test]
    fn eventually_a_quoted() {
        assert_eq!(lex_source(4, "    \"PANL\""),
                   Ok(vec![ Token::Whitespace,
                            Token::Quoted("PANL".at(4, 4)) ]));
    }

    #[test]
    fn eventually_a_form() {
        assert_eq!(lex_source(5, "    [FORM]"),
                   Ok(vec![ Token::Whitespace,
                            Token::Form("FORM".at(5, 4)) ]));
    }

    #[test]
    fn unclosed_quote() {
        assert_eq!(lex_source(6, "\"FORM"),
                   Err(Error::UnclosedString("\"FORM".at(6, 0))));
    }

    #[test]
    fn unclosed_form() {
        assert_eq!(lex_source(7, "[FORM"),
                   Err(Error::UnclosedForm("[FORM".at(7, 0))));
    }

    #[test]
    fn in_parentheses() {
        assert_eq!(lex_source(8, "(AB34)"),
                   Ok(vec![ Token::Open("(".at(8, 0)),
                            Token::Alphanum("AB34".at(8, 1)),
                            Token::Close(")".at(8, 5)) ]));
    }

    #[test]
    fn function_call() {
        assert_eq!(lex_source(9, "x86(AB34)"),
                   Ok(vec![ Token::Alphanum("x86".at(9, 0)),
                            Token::Open("(".at(9, 3)),
                            Token::Alphanum("AB34".at(9, 4)),
                            Token::Close(")".at(9, 8)), ]));
    }

    #[test]
    fn surrounded_by_quotes() {
        assert_eq!(lex_source(10, "\"\"\"\"A\"\"\"\""),
                   Ok(vec![ Token::Quoted("".at(10, 0)),
                            Token::Quoted("".at(10, 2)),
                            Token::Alphanum("A".at(10, 4)),
                            Token::Quoted("".at(10, 5)),
                            Token::Quoted("".at(10, 7)), ]));
    }

    #[test]
    fn quotes_backslashes() {
        assert_eq!(lex_source(11, "\"\\\"\""),
                   Ok(vec![ Token::Quoted("\\\"".at(11, 0)) ]));
    }

    #[test]
    fn mixture() {
        assert_eq!(lex_source(12, "1A2B[FORM]\"PANL\"[FORM]1A2B\"PANL\"1A2B"),
                   Ok(vec![ Token::Alphanum("1A2B".at(12, 0)),
                            Token::Form("FORM".at(12, 4)),
                            Token::Quoted("PANL".at(12, 10)),
                            Token::Form("FORM".at(12, 16)),
                            Token::Alphanum("1A2B".at(12, 22)),
                            Token::Quoted("PANL".at(12, 26)),
                            Token::Alphanum("1A2B".at(12, 32)), ]));
    }

    #[test]
    fn a_lowly_underscore() {
        assert_eq!(lex_source(13, "___ _"),
                   Ok(vec![ Token::Alphanum("___".at(13, 0)),
                            Token::Whitespace,
                            Token::Alphanum("_".at(13, 4)), ]));
    }

    #[test]
    fn whitespace_then_quoted_nothing() {
        assert_eq!(lex_source(14, "    \"\""),
                   Ok(vec![ Token::Whitespace,
                            Token::Quoted("".at(14, 4)) ]));
    }
}
