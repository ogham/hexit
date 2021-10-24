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

    /// Returns a new `Placed` containing a substring of the original based
    /// around the ‘from’ and ‘to’ indices, adjusting the column number.
    ///
    /// This is used to create a substring of an `Alphanums` value when an
    /// error occurs parsing it, so that the part that errored can be shown to
    /// the user in isolation. It is safe to index as all the characters in
    /// the source string will have been shown to be ASCII already.
    pub fn substring_ascii(self, from: usize, to: usize) -> Self {
        self.contents[ from .. to ].at(self.line_number, self.column_number + from)
    }

    /// Returns a new `Placed` containing a substring of the original based
    /// around the ‘from’ and ‘to’ indices, adjusting the column number based
    /// on the ‘from_column’ count.
    ///
    /// This is used to create a substring of a quoted value when an error
    /// occurs parsing a backslash-escaped character, so the part that errored
    /// can be shown to the user. As characters in the string may be
    /// multi-byte, the ‘from’ index may be larger than the number of columns.
    pub fn substring_mb(self, from: usize, from_column: usize, to: usize) -> Self {
        self.contents[ from .. to ].at(self.line_number, self.column_number + from_column)
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
