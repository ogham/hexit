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


/// A “value in flight”.
#[derive(PartialEq, Debug)]
enum Value<'src> {
    Byte(u8),
    VariableBytes(Vec<u8>),
    MultiByte(MultiByteValue),
    RawNumber(&'src str),
    RawFloat(&'src str),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum MultiByteValue {
    Sixteen(u16),
    ThirtyTwo(u32),
    SixtyFour(u64),
}


struct Evaluator<'consts> {
    constants: &'consts Table,
    limit: Option<usize>,
}

impl<'consts> Evaluator<'consts> {

    fn evaluate_exp<'src>(&self, exp: Exp<'src>) -> Result<Value<'src>, Error<'src>> {
        trace!("Evaluating expression -> {:#?}", exp);

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

    fn run_function<'src>(&self, name: FunctionName, args: Vec<Exp<'src>>) -> Result<Value<'src>, Error<'src>> {
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

    /// A decimal number was too big for its target, such as `be16[99999999]`.
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

#[derive(PartialEq, Debug)]
pub enum LargeNumber<'src> {
    Known(MultiByteValue),
    FoundRawNumber(&'src str),
    FoundRawFloat(&'src str),
    FoundBits(usize),
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopLevelBigDecimal(dec)  => write!(f, "{} does not fit in one byte", dec),
            Self::TooBigDecimal(dec)       => write!(f, "{} is too big for target", dec),
            Self::UnknownConstant(uc)      => write!(f, "Unknown constant ‘{}’", uc),
            Self::InvalidArgs(oh)          => write!(f, "Invalid arguments: {}", oh),
            Self::TooMuchOutput            => write!(f, "Too much output!"),
            Self::TooMuchRecursion         => write!(f, "Nested too deeply!"),
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
