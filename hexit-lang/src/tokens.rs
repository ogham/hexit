//! The token type. Values of this type get created by the lexer, and are
//! consumed by the parser.

use crate::pos::Placed;


/// A token that has been read from a source string.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Token<'src> {

    /// A whitespace token, which produces no bytes, but gets used to
    /// terminate runs of input.
    Whitespace,

    /// An alphanumeric token, such as `be32` or `01B3259`.
    Alphanum(Placed<&'src str>),

    /// A form token, such as `[::1]` or `[127.0.0.1]`.
    Form(Placed<&'src str>),

    /// An open parenthesis, `(`.
    Open(Placed<&'src str>),

    /// A close parenthesis, `)`.
    Close(Placed<&'src str>),

    /// A quoted string, such as `"vorbis"`.
    Quoted(Placed<&'src str>),

    /// Any other character, such as `é`. This is an error if encountered
    /// outside of a front comment.
    Stray(Placed<&'src str>),
}

impl<'src> Token<'src> {

    /// Returns whether this token is the colon, the front comment separator.
    /// This gets used when removing the front comment part of an input line.
    pub fn is_colon(&self) -> bool {
        if let Token::Stray(placed) = self {
            if placed.contents == ":" {
                return true;
            }
        }

        false
    }

    /// Returns the contents of a stray token, if this token is one. This gets
    /// used to determine whether to error out with a misplaced character.
    pub fn as_stray(&self) -> Option<Placed<&'src str>> {
        if let Token::Stray(placed) = self {
            Some(*placed)
        }
        else {
            None
        }
    }
}
