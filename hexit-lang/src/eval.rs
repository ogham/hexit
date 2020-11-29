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

        if let Value::RawNumber(s) = val {
            match s.parse() {
                Ok(v) => {
                    bytes.push(v)
                }
                Err(e) => {
                    warn!("Parse error: {}", e);
                    return Err(Error::TopLevelBigDecimal(s))
                }
            }
        }
        else {
            bytes.extend(val.eval_to_bytes());
        }
    }

    Ok(bytes)
}


/// A “value in flight”.
#[derive(PartialEq, Debug)]
enum Value<'src> {
    Byte(u8),
    Sixteen(u16),
    ThirtyTwo(u32),
    SixtyFour(u64),
    VariableBytes(Vec<u8>),
    RawNumber(&'src str),
}

impl Value<'_> {
    fn eval_to_bytes(self) -> Vec<u8> {
        match self {
            Self::Byte(byte) => {
                vec![ byte ]
            }
            Self::VariableBytes(bytes) => {
                bytes
            }
            Self::Sixteen(_) | Self::ThirtyTwo(_) | Self::SixtyFour(_) | Self::RawNumber(_) => {
                panic!("top level number")
            }
        }
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
                Ok(Value::RawNumber(number))
            }

            Exp::Constant { name } => {
                match self.constants.lookup(&name) {
                    Ok(Constant::Eight(num)) => {
                        Ok(Value::Byte(num))
                    }
                    Ok(Constant::Sixteen(num)) => {
                        Ok(Value::Sixteen(num))
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
                Ok(Value::VariableBytes(unix_time.to_be_bytes().to_vec()))
            }
        }
    }

    fn run_function<'src>(&self, name: FunctionName, mut args: Vec<Exp<'src>>) -> Result<Value<'src>, Error<'src>> {
        use std::iter::once;

        match name {
            FunctionName::MultiByte(MultiByteType::Be32) => {
                if args.len() != 1 {
                    return Err(Error::InvalidArgs);
                }

                let arg = args.remove(0);
                let val = self.evaluate_exp(arg)?;

                match val {
                    Value::Byte(b)      => Ok(Value::VariableBytes(u32::from(b).to_be_bytes().to_vec())),
                    Value::Sixteen(o2)  => Ok(Value::VariableBytes(u32::from(o2).to_be_bytes().to_vec())),
                    Value::RawNumber(s) => Ok(Value::VariableBytes(u32::from_str_radix(s, 10).unwrap().to_be_bytes().to_vec())),
                    _                   => todo!("val: {:?}", val)
                }
            }

            FunctionName::MultiByte(MultiByteType::Le32) => {
                if args.len() != 1 {
                    return Err(Error::InvalidArgs);
                }

                let arg = args.remove(0);
                let val = self.evaluate_exp(arg)?;

                match val {
                    Value::Byte(b)      => Ok(Value::VariableBytes(u32::from(b).to_le_bytes().to_vec())),
                    Value::Sixteen(o2)  => Ok(Value::VariableBytes(u32::from(o2).to_le_bytes().to_vec())),
                    Value::RawNumber(s) => Ok(Value::VariableBytes(u32::from_str_radix(s, 10).unwrap().to_le_bytes().to_vec())),
                    _                   => todo!("val: {:?}", val)
                }
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


/// An error that can occur while evaluating a tree of expressions.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {

    /// A decimal number was too big for a byte at the top level, such as
    /// `[9999999]`.
    TopLevelBigDecimal(&'src str),

    /// A decimal number was too big for its target, such as `be16[99999999]`.
    TooBigDecimal(&'src str),

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
                   Err(Error::TopLevelBigDecimal("256")));
    }
}
