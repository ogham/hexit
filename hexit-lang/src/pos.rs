//! Things to do with positions in source code (line and column numbers).


/// A token at a particular line and column position. This is used when
/// printing out errors, to help the user pinpoint the position at which an
/// error occurred.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Placed<T> {

    /// The payload, the token that was read from a source file of Hexit.
    pub contents: T,

    /// The line number that the token was read at, starting at 1.
    pub line_number: usize,

    /// The column number that the token was read at in its line, starting at 0.
    pub column_number: usize,
}

impl<'a> Placed<&'a str> {

    /// Returns a new `Placed` containing a substring of the original. This is
    /// used when printing out an error that occurs within a particular token.
    pub fn substring(self, from: usize, to: usize) -> Self {
        self.contents[ from .. to ].at(self.line_number, self.column_number + from)
    }
}


/// A helper trait to more easily put things within `Placed` values.
pub trait At: Sized {

    /// Puts a value within a `Placed` with the given numbers.
    fn at(self, line_number: usize, column_number: usize) -> Placed<Self>;
}

impl<T> At for T {
    fn at(self, line_number: usize, column_number: usize) -> Placed<Self> {
        Placed { contents: self, line_number, column_number }
    }
}
