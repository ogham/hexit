//! The evaluation step, which involves taking a series of `Exp` values and
//! “executing” them, resulting in a series of bytes.
//!
//! Execution is performed in a recursive manner: for each top-level `Exp` in
//! the sequence, it gets evaluated into a vector of bytes, all of which are
//! concatenated together at the end. If evaluating an expression involves
//! evaluating sub-expressions, these are converted to bytes too.
//!
//! As it is trivially easy to write short programs that produce massive
//! amounts of output (such as `x999(x999(x999(FF)))`, there is a customisable
//! limit for how long the complete output is allowed to get.

use std::fmt;

use log::*;

use crate::ast::*;
use crate::constants::{Table, Constant};


/// Evaluates all the expressions in the iterator into a vector of bytes,
/// returning an error if one occurs without processing the rest.
pub fn evaluate_exps<'src>(exps: impl IntoIterator<Item=Exp<'src>>, constants: &Table, limit: Option<usize>) -> Result<Vec<u8>, Error<'src>> {
    let evaluator = Evaluator { constants, limit };
    let mut bytes = Vec::new();

    for exp in exps {
        let val = evaluator.evaluate_exp(exp)?;
        bytes.extend(val.eval_to_bytes()?);
    }

    Ok(bytes)
}

/// The internal “evaluation environment”, which holds the values that get
/// looked up during evaluation.
struct Evaluator<'consts> {
    constants: &'consts Table,
    limit: Option<usize>,
}

/// A “value in flight”. Even though Hexit produces bytes as its output, it
/// still has to deal with values before the width of the output type is
/// known. This is similar to the `{integer}` or `{float}` types in Rust, as
/// opposed to `u32` or `f64`.
#[derive(PartialEq, Debug)]
enum Value<'src> {

    /// A value that is known to be an individual byte. This can be printed
    /// directly, passed to a bitwise or repeat function, or made wider.
    Byte(u8),

    /// A value that is known to be a series of bytes with a known length.
    /// This can be printed directly, and passed to a bitwise or repeat
    /// function, but cannot be made wider or narrower (as it does not
    /// strictly have an “endianness” — it’s a sequence, not a number).
    VariableBytes(Vec<u8>),

    /// A value that is known to have a width of a certain size. This cannot
    /// be printed directly (as the endianness is not known) nor passed to a
    /// repeat function, but it can be passed to a bitwise function, and made
    /// wider.
    MultiByte(MultiByteValue),

    /// A numeric value where the width is not yet known. This cannot be
    /// printed directly (as the size and endianness is not known) nor passed
    /// to bitwise or repeat functions, but can be given a width and
    /// endianness.
    RawNumber(&'src str),

    /// A floating-point value where the width is not yet known. This cannot be
    /// printed directly (as the size and endianness is not known) nor passed
    /// to bitwise or repeat functions, but can be given a width and
    /// endianness, as long as the width is one of the floating-point widths.
    RawFloat(&'src str),
}

/// A value that is known to have a width of a certain size. This is its own
/// type to simplify error handling.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum MultiByteValue {
    Sixteen(u16),
    ThirtyTwo(u32),
    SixtyFour(u64),
}


impl<'consts> Evaluator<'consts> {

    /// Evaluates this expression by converting it into a “value in flight”,
    /// which possibly involves evaluating the expression’s sub-expressions.
    fn evaluate_exp<'src>(&self, exp: Exp<'src>) -> Result<Value<'src>, Error<'src>> {
        trace!("Evaluating expression → {:#?}", exp);

        match exp {
            Exp::Char(byte) => {
                Ok(Value::Byte(byte))
            }

            Exp::Dec(number) => {
                Ok(Value::RawNumber(number))
            }

            Exp::Constant { name } => {
                match self.constants.lookup(name) {
                    Some(Constant::Eight(num)) => {
                        Ok(Value::Byte(num))
                    }
                    Some(Constant::Sixteen(num)) => {
                        Ok(Value::MultiByte(MultiByteValue::Sixteen(num)))
                    }
                    None => {
                        return Err(Error::UnknownConstant(name));
                    }
                }
            }

            Exp::Function { name, args } => {
                self.run_function(name, args)
            }

            Exp::StringLiteral { chars } => {
                let bytes = chars.bytes().collect::<Vec<_>>();
                Ok(Value::VariableBytes(bytes))
            }

            Exp::IPv4 { bytes } => {
                Ok(Value::VariableBytes(bytes.to_vec()))
            }

            Exp::IPv6 { bytes } => {
                Ok(Value::VariableBytes(bytes.to_vec()))
            }

            Exp::Timestamp(unix_time) => {
                Ok(Value::MultiByte(MultiByteValue::ThirtyTwo(unix_time)))
            }

            Exp::Float(number) => {
                Ok(Value::RawFloat(number))
            }

            Exp::Bits(bit_vec) => {
                if bit_vec.len() <= 8 {
                    let mut num = 0_u8;

                    for (index, bit) in bit_vec.into_iter().rev().enumerate() {
                        if bit {
                            num += 2_u8.pow(index as u32);
                        }
                    }

                    Ok(Value::Byte(num))
                }
                else if bit_vec.len() <= 16 {
                    let mut num = 0_u16;

                    for (index, bit) in bit_vec.into_iter().rev().enumerate() {
                        if bit {
                            num += 2_u16.pow(index as u32);
                        }
                    }

                    Ok(Value::MultiByte(MultiByteValue::Sixteen(num)))
                }
                else if bit_vec.len() <= 32 {
                    let mut num = 0_u32;

                    for (index, bit) in bit_vec.into_iter().rev().enumerate() {
                        if bit {
                            num += 2_u32.pow(index as u32);
                        }
                    }

                    Ok(Value::MultiByte(MultiByteValue::ThirtyTwo(num)))
                }
                else if bit_vec.len() <= 64 {
                    let mut num = 0_u64;

                    for (index, bit) in bit_vec.into_iter().rev().enumerate() {
                        if bit {
                            num += 2_u64.pow(index as u32);
                        }
                    }

                    Ok(Value::MultiByte(MultiByteValue::SixtyFour(num)))
                }
                else {
                    Err(Error::TopLevelBigDecimal(LargeNumber::FoundBits(bit_vec.len())))
                }
            }
        }
    }

    /// Runs the function with the given name, using the list of expressions
    /// as its arguments. The arguments have not yet been evaluated
    /// themselves, so that the number of arguments can first be checked.
    fn run_function<'src>(&self, name: FunctionName, args: Vec<Exp<'src>>) -> Result<Value<'src>, Error<'src>> {
        trace!("Running function → {:?}", name);
        trace!("Function arguments → {:#?}", args);

        match name {
            FunctionName::MultiByte(MultiByteType::Be16) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                val.to_two_variable_bytes(u16::to_be_bytes)
            }

            FunctionName::MultiByte(MultiByteType::Le16) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                val.to_two_variable_bytes(u16::to_le_bytes)
            }

            FunctionName::MultiByte(MultiByteType::Be32) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                val.to_four_variable_bytes(u32::to_be_bytes, f32::to_be_bytes)
            }

            FunctionName::MultiByte(MultiByteType::Le32) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                val.to_four_variable_bytes(u32::to_le_bytes, f32::to_le_bytes)
            }

            FunctionName::MultiByte(MultiByteType::Be64) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                val.to_eight_variable_bytes(u64::to_be_bytes, f64::to_be_bytes)
            }

            FunctionName::MultiByte(MultiByteType::Le64) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                val.to_eight_variable_bytes(u64::to_le_bytes, f64::to_le_bytes)
            }

            FunctionName::Repeat(amount) => {
                let mut bytes = Vec::new();

                for exp in args {
                    let sub_bytes = self.evaluate_exp(exp)?.eval_to_bytes()?;
                    bytes.extend(&sub_bytes);
                }

                // Check whether this would hit the limit, because it’s
                // possible for repeat functions to generate lots of
                // output very quickly
                if let Some(limit) = self.limit {
                    if limit <= bytes.len() * usize::from(amount) {
                        return Err(Error::TooMuchOutput);
                    }
                }

                let mut result_bytes = Vec::new();
                for _ in 0 .. amount {
                    result_bytes.extend(&bytes);
                }

                Ok(Value::VariableBytes(result_bytes))
            }

            FunctionName::Bitwise(bitwise_operator) => {
                let mut iter = args.into_iter().map(|exp| self.evaluate_exp(exp));
                let mut result = match iter.next() {
                    Some(val)  => val?,
                    None       => return Err(Error::InvalidArgs("No arguments for and function".into())),
                };

                for next_val in iter {
                    let next_val = next_val?;
                    result = result.apply_bitwise(next_val, bitwise_operator)?;
                }

                Ok(result)
            }

            FunctionName::BitwiseNot => {
                let mut bytes = Vec::<u8>::new();

                for exp in args {
                    let sub_bytes = self.evaluate_exp(exp)?.eval_to_bytes()?;
                    bytes.extend(&sub_bytes);
                }

                for b in &mut bytes {
                    *b = !*b;
                }

                Ok(Value::VariableBytes(bytes))
            }
        }
    }
}


impl<'src> Value<'src> {

    /// Converts this “value in flight” into a series of bytes, or return an
    /// error if the conversion is not possible. This is used when printing
    /// bytes at the top level, or converting values to sequences for a
    /// repetition function.
    fn eval_to_bytes(self) -> Result<Vec<u8>, Error<'src>> {
        match self {
            Self::Byte(byte) => {
                Ok(vec![ byte ])
            }
            Self::VariableBytes(bytes) => {
                Ok(bytes)
            }
            Self::MultiByte(v) => {
                Err(Error::TopLevelBigDecimal(LargeNumber::Known(v)))
            }
            Self::RawNumber(s) => {
                match s.parse() {
                    Ok(v) => {
                        Ok(vec![ v ])
                    }
                    Err(e) => {
                        warn!("Parse error: {}", e);
                        Err(Error::TopLevelBigDecimal(LargeNumber::FoundRawNumber(s)))
                    }
                }
            }
            Self::RawFloat(s) => {
                Err(Error::TopLevelBigDecimal(LargeNumber::FoundRawFloat(s)))
            }
        }
    }

    /// Converts this “value in flight” into a 2-byte value, using the given
    /// function to perform the conversion with a certain endianness, or
    /// return an error if the conversion is not possible. This is used when
    /// passing a value to the `be16` or `le16` functions. Values cannot be
    /// made more narrow.
    fn to_two_variable_bytes(self, endianify: impl Fn(u16) -> [u8; 2]) -> Result<Self, Error<'src>> {
        let bytes = match self {
            Self::Byte(b) => {
                endianify(u16::from(b))
            }
            Self::VariableBytes(bytes) => {
                let message = format!("Tried to turn variable bytes ({:?}) into 2 bytes", bytes);
                return Err(Error::InvalidArgs(message));
            }
            Self::MultiByte(MultiByteValue::Sixteen(o2)) => {
                endianify(o2)
            }
            Self::MultiByte(MultiByteValue::ThirtyTwo(o4)) => {
                let message = format!("Tried to turn four bytes ({:?}) into 2 bytes", o4);
                return Err(Error::InvalidArgs(message));
            }
            Self::MultiByte(MultiByteValue::SixtyFour(o8)) => {
                let message = format!("Tried to turn eight bytes ({:?}) into 2 bytes", o8);
                return Err(Error::InvalidArgs(message));
            }
            Self::RawNumber(s) => {
                match s.parse() {
                    Ok(num) => endianify(num),
                    Err(e) => {
                        warn!("Parse error: {}", e);
                        return Err(Error::TooBigDecimal(LargeNumber::FoundRawNumber(s)));
                    }
                }
            }
            Self::RawFloat(s) => {
                return Err(Error::TooBigDecimal(LargeNumber::FoundRawFloat(s)));
            }
        };

        Ok(Value::VariableBytes(bytes.to_vec()))
    }

    /// Converts this “value in flight” into a 4-byte value, using the given
    /// function to perform the conversion with a certain endianness, or
    /// return an error if the conversion is not possible. This is used when
    /// passing a value to the `be32` or `le32` functions. Values cannot be
    /// made more narrow.
    fn to_four_variable_bytes(self, endianify: impl Fn(u32) -> [u8; 4], flendianify: impl Fn(f32) -> [u8; 4]) -> Result<Self, Error<'src>> {
        let bytes = match self {
            Self::Byte(b) => {
                endianify(u32::from(b))
            }
            Self::VariableBytes(bytes) => {
                let message = format!("Tried to turn variable bytes ({:?}) into 4 bytes", bytes);
                return Err(Error::InvalidArgs(message));
            }
            Self::MultiByte(MultiByteValue::Sixteen(o2)) => {
                endianify(u32::from(o2))
            }
            Self::MultiByte(MultiByteValue::ThirtyTwo(o4)) => {
                endianify(o4)
            }
            Self::MultiByte(MultiByteValue::SixtyFour(o8)) => {
                let message = format!("Tried to turn eight bytes ({:?}) into 4 bytes", o8);
                return Err(Error::InvalidArgs(message));
            }
            Self::RawNumber(s) => {
                match s.parse() {
                    Ok(num) => endianify(num),
                    Err(e) => {
                        warn!("Parse error: {}", e);
                        return Err(Error::TooBigDecimal(LargeNumber::FoundRawNumber(s)));
                    }
                }
            }
            Self::RawFloat(s) => {
                match s.parse() {
                    Ok(num) => flendianify(num),
                    Err(e) => {
                        warn!("Parse error: {}", e);
                        return Err(Error::TooBigDecimal(LargeNumber::FoundRawFloat(s)));
                    }
                }
            }
        };

        Ok(Value::VariableBytes(bytes.to_vec()))
    }

    /// Converts this “value in flight” into an 8-byte value, using the given
    /// function to perform the conversion with a certain endianness, or
    /// return an error if the conversion is not possible. This is used when
    /// passing a value to the `be64` or `le64` functions.
    fn to_eight_variable_bytes(self, endianify: impl Fn(u64) -> [u8; 8], flendianify: impl Fn(f64) -> [u8; 8]) -> Result<Self, Error<'src>> {
        let bytes = match self {
            Self::Byte(b) => {
                endianify(u64::from(b))
            }
            Self::VariableBytes(bytes) => {
                let message = format!("Tried to turn variable bytes ({:?}) into 8 bytes", bytes);
                return Err(Error::InvalidArgs(message));
            }
            Self::MultiByte(MultiByteValue::Sixteen(o2)) => {
                endianify(u64::from(o2))
            }
            Self::MultiByte(MultiByteValue::ThirtyTwo(o4)) => {
                endianify(u64::from(o4))
            }
            Self::MultiByte(MultiByteValue::SixtyFour(o8)) => {
                endianify(o8)
            }
            Self::RawNumber(s) => {
                match s.parse() {
                    Ok(num) => endianify(num),
                    Err(e) => {
                        warn!("Parse error: {}", e);
                        return Err(Error::TooBigDecimal(LargeNumber::FoundRawNumber(s)));
                    }
                }
            }
            Self::RawFloat(s) => {
                match s.parse() {
                    Ok(num) => flendianify(num),
                    Err(e) => {
                        warn!("Parse error: {}", e);
                        return Err(Error::TooBigDecimal(LargeNumber::FoundRawFloat(s)));
                    }
                }
            }
        };

        Ok(Value::VariableBytes(bytes.to_vec()))
    }

    /// Applies the given bitwise function to this “value in flight”, with the
    /// given value as the other operand (which has already been evaluated).
    /// Returns an error if the two types are incompatible (such as a 32-bit
    /// and 16-bit number, or byte sequences of different lengths) or are raw
    /// (a number or float where the size is not yet known).
    fn apply_bitwise(self, next_val: Self, bitwise_op: BitwiseFold) -> Result<Self, Error<'src>> {
        match (self, next_val) {
            (Self::Byte(left),
             Self::Byte(right)) => {
                let result = bitwise_op.apply_u8(left, right);
                Ok(Self::Byte(result))
            }

            (Self::MultiByte(MultiByteValue::Sixteen(left)),
             Self::MultiByte(MultiByteValue::Sixteen(right))) => {
                let result = bitwise_op.apply_u16(left, right);
                Ok(Self::MultiByte(MultiByteValue::Sixteen(result)))
            }

            (Self::MultiByte(MultiByteValue::ThirtyTwo(left)),
             Self::MultiByte(MultiByteValue::ThirtyTwo(right))) => {
                let result = bitwise_op.apply_u32(left, right);
                Ok(Self::MultiByte(MultiByteValue::ThirtyTwo(result)))
            }

            (Self::MultiByte(MultiByteValue::SixtyFour(left)),
             Self::MultiByte(MultiByteValue::SixtyFour(right))) => {
                let result = bitwise_op.apply_u64(left, right);
                Ok(Self::MultiByte(MultiByteValue::SixtyFour(result)))
            }

            (Self::RawNumber(left),
             Self::RawNumber(right)) => {
                let message = format!("ANDing together two raw numbers ({} and {})", left, right);
                return Err(Error::InvalidArgs(message));
            }

            (Self::VariableBytes(lefts),
             Self::VariableBytes(rights)) => {
                if lefts.len() != rights.len() {
                    let message = format!("ANDing together bytestrings of different lengths ({} and {})", lefts.len(), rights.len());
                    return Err(Error::InvalidArgs(message));
                }

                let bytes = lefts.into_iter()
                                 .zip(rights.into_iter())
                                 .map(|(l,r)| bitwise_op.apply_u8(l, r))
                                 .collect();

                Ok(Self::VariableBytes(bytes))
            }

            (a, b) => {
                let message = format!("ANDing together two weird things ({:?} and {:?})", a, b);
                return Err(Error::InvalidArgs(message));
            }
        }
    }
}


impl BitwiseFold {
    fn apply_u8(self, left: u8, right: u8) -> u8 {
        match self {
            Self::And => left & right,
            Self::Or  => left | right,
            Self::Xor => left ^ right,
        }
    }

    fn apply_u16(self, left: u16, right: u16) -> u16 {
        match self {
            Self::And => left & right,
            Self::Or  => left | right,
            Self::Xor => left ^ right,
        }
    }

    fn apply_u32(self, left: u32, right: u32) -> u32 {
        match self {
            Self::And => left & right,
            Self::Or  => left | right,
            Self::Xor => left ^ right,
        }
    }

    fn apply_u64(self, left: u64, right: u64) -> u64 {
        match self {
            Self::And => left & right,
            Self::Or  => left | right,
            Self::Xor => left ^ right,
        }
    }
}


/// Returns the only argument in the vector if just one is present, or returns
/// an “invalid arguments” error.
fn only_arg<'src>(mut args: Vec<Exp<'src>>) -> Result<Exp<'src>, Error<'src>> {
    if args.len() == 1 {
        Ok(args.remove(0))
    }
    else {
        let message = format!("Pass only 1 arg, not {}", args.len());
        Err(Error::InvalidArgs(message))
    }
}


/// An error that can occur while evaluating a tree of expressions.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {

    /// A decimal number was too big for a byte at the top level, such as
    /// `[9999999]`.
    TopLevelBigDecimal(LargeNumber<'src>),

    /// A decimal number was too big for its target.
    TooBigDecimal(LargeNumber<'src>),

    /// A constant value was referenced that does not exist.
    UnknownConstant(&'src str),

    /// A function had the wrong type or number of arguments passed to it.
    InvalidArgs(String),

    /// The amount of output hit the limit.
    TooMuchOutput,

    /// The recursion depth hit the limit.
    TooMuchRecursion,
}

/// A number that was too big for its target. This is used in error handling.
#[derive(PartialEq, Debug)]
pub enum LargeNumber<'src> {

    /// A value with a known size was too big, such as `be16(be32[1234])`.
    /// Hexit does not narrow values, so this is an error.
    Known(MultiByteValue),

    /// A raw decimal number was too big, such as `be16[99999999]`.
    FoundRawNumber(&'src str),

    /// A raw floating-point number was put in a `be16` or `le16`.
    /// Floating-point numbers can only be put in 32-bit or 64-bit widths.
    FoundRawFloat(&'src str),

    /// A number of bits were too many, such as `be16[b0101_0101_0101_0101_1].
    FoundBits(usize),
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopLevelBigDecimal(dec)  => write!(f, "{} at top level", dec),
            Self::TooBigDecimal(dec)       => write!(f, "{} is too big for target", dec),
            Self::UnknownConstant(uc)      => write!(f, "Unknown constant ‘{}’", uc),
            Self::InvalidArgs(oh)          => write!(f, "Invalid arguments: {}", oh),
            Self::TooMuchOutput            => write!(f, "Too much output!"),
            Self::TooMuchRecursion         => write!(f, "Nested too deeply!"),
        }
    }
}

impl<'src> Error<'src> {
    pub fn note(&self) -> Option<&'static str> {
        match self {
            Self::TopLevelBigDecimal(LargeNumber::FoundRawNumber(_)) => {
                Some("Top-level multi-byte values must be given an endianness using a function such as ‘be16’ or ‘le32’")
            }
            Self::TopLevelBigDecimal(LargeNumber::FoundRawFloat(_)) => {
                Some("Top-level floating point values must be given an endianness and width using a function such as ‘be32’ or ‘le64’")
            }
            _ => {
                None
            }
        }
    }
}

impl<'src> fmt::Display for LargeNumber<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(mbv)           => mbv.fmt(f),
            Self::FoundRawNumber(num)  => write!(f, "Decimal number ‘{}’", num),
            Self::FoundRawFloat(num)   => write!(f, "Floating-point number ‘{}’", num),
            Self::FoundBits(length)    => write!(f, "Bit set of length {}", length),
        }
    }
}

impl fmt::Display for MultiByteValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sixteen(num)    => write!(f, "2-byte number ‘{}’", num),
            Self::ThirtyTwo(num)  => write!(f, "4-byte number ‘{}’", num),
            Self::SixtyFour(num)  => write!(f, "8-byte number ‘{}’", num),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn nothing() {
        let exps = vec![];
        assert_eq!(evaluate_exps(exps, &Table::empty(), None),
                   Ok(vec![]));
    }

    #[test]
    fn one_top_level_byte() {
        let exps = vec![ Exp::Char(0x73) ];
        assert_eq!(evaluate_exps(exps, &Table::empty(), None),
                   Ok(vec![ 0x73 ]));
    }

    #[test]
    fn top_level_decimal_73() {
        let exps = vec![ Exp::Dec("73") ];
        assert_eq!(evaluate_exps(exps, &Table::empty(), None),
                   Ok(vec![ 73 ]));
    }

    #[test]
    fn top_level_decimal_255() {
        let exps = vec![ Exp::Dec("255") ];
        assert_eq!(evaluate_exps(exps, &Table::empty(), None),
                   Ok(vec![ 255 ]));
    }

    #[test]
    fn top_level_decimal_256() {
        let exps = vec![ Exp::Dec("256") ];
        assert_eq!(evaluate_exps(exps, &Table::empty(), None),
                   Err(Error::TopLevelBigDecimal(LargeNumber::FoundRawNumber("256"))));
    }

    #[test]
    fn test_limit() {
        let exps = vec![ Exp::Function {
            name: FunctionName::Repeat(30000),
            args: vec![ Exp::Char(0x73), Exp::Char(0x73), Exp::Char(0x73) ]
        } ];

        assert_eq!(evaluate_exps(exps, &Table::empty(), Some(1000)),
                   Err(Error::TooMuchOutput));
    }
}
