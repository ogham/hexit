use std::borrow::Cow;


/// An expression in a Hexit syntax tree.
#[derive(PartialEq, Debug)]
pub enum Exp<'src> {

    /// A hex character that’s been parsed into a byte.
    Char(u8),

    /// A decimal number.
    /// This has _not yet_ been parsed, because we do not know its storage size.
    Dec(&'src str),

    /// A constant, referred to by its name.
    Constant {

        /// The name of the constant.
        name: &'src str,
    },

    /// A function call.
    Function {

        /// The name of the function to call.
        name: FunctionName,

        /// The arguments to pass to the function.
        args: Vec<Exp<'src>>,
    },

    /// A string literal.
    StringLiteral {

        /// The bytes that make up the string.
        /// This is a reference to the original string’s bytes, unless the
        /// string features backslashes or escape characters, which need to be
        /// processed before the bytes can be read.
        chars: Cow<'src, str>,
    },

    /// An IPv4 address.
    IPv4 {
        bytes: [u8; 4],
    },

    /// An IPv6 address.
    IPv6 {
        bytes: [u8; 16],
    },

    /// An ISO 8601 timestamp.
    Timestamp(u32),

    /// A series of individual bits.
    Bits(Vec<bool>),
}

/// The name of a function to call.
///
/// Unlike regular programming languages, there are only so many of these, so
/// this can be an enum rather than a string.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum FunctionName {
    MultiByte(MultiByteType),
    Bitwise(BitwiseFold),
    BitwiseNot,
    Repeat(RepeatAmount),
}

/// One of the multi-byte-type function names.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum MultiByteType {
    Be16,
    Be32,
    Be64,
    Le16,
    Le32,
    Le64,
}

/// One of the bitwise function names.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum BitwiseFold {
    And,
    Or,
    Xor,
}

/// The amount that some bytes can be repeated in the repetition `FunctionName`.
/// This is kept small because the resulting bytes get stored in memory first.
pub type RepeatAmount = u16;
