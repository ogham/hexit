use std::fmt;

use log::*;

use crate::ast::*;
use crate::constants::{ConstantsTable, Constant, UnknownConstant};


/// Evaluates all the expressions in the iterator into a vector of bytes,
/// returning an error if one occurs without processing the rest.
pub fn evaluate_exps<'src>(exps: impl IntoIterator<Item=Exp<'src>>, constants: &ConstantsTable, limit: Option<usize>) -> Result<Vec<u8>, Error<'src>> {
    eval(exps, constants, limit, 0)
}

fn eval<'src>(exps: impl IntoIterator<Item=Exp<'src>>, constants: &ConstantsTable, _limit: Option<usize>, depth: usize) -> Result<Vec<u8>, Error<'src>> {
    if depth > 6 {
        return Err(Error::TooMuchRecursion);
    }

    let evaluator = Evaluator::new(constants);
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
    MultiByte(MultiByteValue<'src>),
}

#[derive(PartialEq, Debug)]
pub enum MultiByteValue<'src> {
    Sixteen(u16),
    ThirtyTwo(u32),
    SixtyFour(u64),
    RawNumber(&'src str),
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
            Self::MultiByte(MultiByteValue::RawNumber(s)) => {
                match s.parse() {
                    Ok(v) => {
                        Ok(vec![ v ])
                    }
                    Err(e) => {
                        warn!("Parse error: {}", e);
                        Err(Error::TopLevelBigDecimal(MultiByteValue::RawNumber(s)))
                    }
                }
            }
            Self::MultiByte(v) => {
                Err(Error::TopLevelBigDecimal(v))
            }
        }
    }

    fn to_two_variable_bytes(self, endianify: impl Fn(u16) -> [u8; 2]) -> Self {
        let bytes = match self {
            Self::Byte(b) => {
                endianify(u16::from(b))
            }
            Self::MultiByte(MultiByteValue::Sixteen(o2)) => {
                endianify(u16::from(o2))
            }
            Self::MultiByte(MultiByteValue::RawNumber(s)) => {
                endianify(u16::from_str_radix(s, 10).unwrap())
            }
            _ => {
                todo!("val: {:?}", self)
            }
        };

        Value::VariableBytes(bytes.to_vec())
    }

    fn to_four_variable_bytes(self, endianify: impl Fn(u32) -> [u8; 4]) -> Self {
        let bytes = match self {
            Self::Byte(b) => {
                endianify(u32::from(b))
            }
            Self::MultiByte(MultiByteValue::Sixteen(o2)) => {
                endianify(u32::from(o2))
            }
            Self::MultiByte(MultiByteValue::ThirtyTwo(o4)) => {
                endianify(o4)
            }
            Self::MultiByte(MultiByteValue::RawNumber(s)) => {
                endianify(u32::from_str_radix(s, 10).unwrap())
            }
            _ => {
                todo!("val: {:?}", self)
            }
        };

        Value::VariableBytes(bytes.to_vec())
    }

    fn to_eight_variable_bytes(self, endianify: impl Fn(u64) -> [u8; 8]) -> Self {
        let bytes = match self {
            Self::Byte(b) => {
                endianify(u64::from(b))
            }
            Self::MultiByte(MultiByteValue::Sixteen(o2)) => {
                endianify(u64::from(o2))
            }
            Self::MultiByte(MultiByteValue::RawNumber(s)) => {
                endianify(u64::from_str_radix(s, 10).unwrap())
            }
            _ => {
                todo!("val: {:?}", self)
            }
        };

        Value::VariableBytes(bytes.to_vec())
    }
}


struct Evaluator<'consts> {
    constants: &'consts ConstantsTable,
}

impl<'consts> Evaluator<'consts> {

    fn new(constants: &'consts ConstantsTable) -> Self {
        Self { constants }
    }

    fn evaluate_exp<'src>(&self, exp: Exp<'src>) -> Result<Value<'src>, Error<'src>> {
        match exp {
            Exp::Char(byte) => {
                Ok(Value::Byte(byte))
            }

            Exp::Dec(number) => {
                Ok(Value::MultiByte(MultiByteValue::RawNumber(number)))
            }

            Exp::Constant { name } => {
                match self.constants.lookup(&name) {
                    Ok(Constant::Eight(num)) => {
                        Ok(Value::Byte(num))
                    }
                    Ok(Constant::Sixteen(num)) => {
                        Ok(Value::MultiByte(MultiByteValue::Sixteen(num)))
                    }
                    Err(e) => {
                        return Err(Error::UnknownConstant(e));
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
        }
    }

    fn run_function<'src>(&self, name: FunctionName, args: Vec<Exp<'src>>) -> Result<Value<'src>, Error<'src>> {
        use std::iter::once;

        match name {
            FunctionName::MultiByte(MultiByteType::Be16) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                Ok(val.to_two_variable_bytes(u16::to_be_bytes))
            }

            FunctionName::MultiByte(MultiByteType::Le16) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                Ok(val.to_two_variable_bytes(u16::to_le_bytes))
            }

            FunctionName::MultiByte(MultiByteType::Be32) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                Ok(val.to_four_variable_bytes(u32::to_be_bytes))
            }

            FunctionName::MultiByte(MultiByteType::Le32) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                Ok(val.to_four_variable_bytes(u32::to_le_bytes))
            }

            FunctionName::MultiByte(MultiByteType::Be64) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                Ok(val.to_eight_variable_bytes(u64::to_be_bytes))
            }

            FunctionName::MultiByte(MultiByteType::Le64) => {
                let arg = only_arg(args)?;
                let val = self.evaluate_exp(arg)?;
                Ok(val.to_eight_variable_bytes(u64::to_le_bytes))
            }

            FunctionName::Repeat(amount) => {
                let mut result_bytes = Vec::new();

                let bytes = eval(args, &self.constants, None, 4)?;

                for _ in 0 .. amount {
                    result_bytes.extend(&bytes);
                }

                Ok(Value::VariableBytes(result_bytes))
            }
            _ => {
                todo!("Func: {:?}", name)
            }
        }
    }
}

/// Returns the only argument in the vector if just one is present, or returns
/// an “invalid arguments” error.
fn only_arg<'src>(mut args: Vec<Exp<'src>>) -> Result<Exp<'src>, Error<'src>> {
    if args.len() != 1 {
        Err(Error::InvalidArgs)
    }
    else {
        Ok(args.remove(0))
    }
}


/// An error that can occur while evaluating a tree of expressions.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {

    /// A decimal number was too big for a byte at the top level, such as
    /// `[9999999]`.
    TopLevelBigDecimal(MultiByteValue<'src>),

    /// A decimal number was too big for its target, such as `be16[99999999]`.
    TooBigDecimal(MultiByteValue<'src>),

    /// A constant value was referenced that does not exist.
    UnknownConstant(UnknownConstant),

    /// A function had the wrong type or number of arguments passed to it.
    InvalidArgs,

    /// The amount of output hit the limit.
    TooMuchOutput,

    /// The recursion depth hit the limit.
    TooMuchRecursion,
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopLevelBigDecimal(dec)  => write!(f, "Decimal {:?} does not fit in one byte", dec),
            Self::TooBigDecimal(dec)       => write!(f, "Decimal {:?} too big for target", dec),
            Self::UnknownConstant(uc)      => uc.fmt(f),
            Self::InvalidArgs              => write!(f, "Invalid argument count"),
            Self::TooMuchOutput            => write!(f, "Too much output!"),
            Self::TooMuchRecursion         => write!(f, "Nested too deeply!"),
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
        assert_eq!(evaluate_exps(exps, &ConstantsTable::empty(), None),
                   Ok(vec![]));
    }

    #[test]
    fn one_top_level_byte() {
        let exps = vec![ Exp::Char(0x73) ];
        assert_eq!(evaluate_exps(exps, &ConstantsTable::empty(), None),
                   Ok(vec![ 0x73 ]));
    }

    #[test]
    fn top_level_decimal_73() {
        let exps = vec![ Exp::Dec("73") ];
        assert_eq!(evaluate_exps(exps, &ConstantsTable::empty(), None),
                   Ok(vec![ 73 ]));
    }

    #[test]
    fn top_level_decimal_255() {
        let exps = vec![ Exp::Dec("255") ];
        assert_eq!(evaluate_exps(exps, &ConstantsTable::empty(), None),
                   Ok(vec![ 255 ]));
    }

    #[test]
    fn top_level_decimal_256() {
        let exps = vec![ Exp::Dec("256") ];
        assert_eq!(evaluate_exps(exps, &ConstantsTable::empty(), None),
                   Err(Error::TopLevelBigDecimal(MultiByteValue::RawNumber("256"))));
    }
}
