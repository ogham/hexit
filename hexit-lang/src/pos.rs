#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Placed<T> {
    pub contents: T,
    pub line_number: usize,
}


pub trait At: Sized {
    fn at(self, line_number: usize) -> Placed<Self>;
}

impl<T> At for T {
    fn at(self, line_number: usize) -> Placed<Self> {
        Placed { contents: self, line_number }
    }
}
