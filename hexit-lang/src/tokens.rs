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
}
