#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Placed<T> {
    pub contents: T,
    pub line_number: usize,
    pub column_number: usize,
}

impl<'a> Placed<&'a str> {
    pub fn substring(self, from: usize, to: usize) -> Self {
        self.contents[ from .. to ].at(self.line_number, self.column_number + from)
    }

    pub fn substring_to_end(self, from: usize) -> Self {
        self.contents[ from .. ].at(self.line_number, self.column_number + from)
    }
}


pub trait At: Sized {
    fn at(self, line_number: usize, column_number: usize) -> Placed<Self>;
}

impl<T> At for T {
    fn at(self, line_number: usize, column_number: usize) -> Placed<Self> {
        Placed { contents: self, line_number, column_number }
    }
}
