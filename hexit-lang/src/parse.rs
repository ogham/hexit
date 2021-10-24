//! The parsing stage, which involves taking a series of `Token` values and
//! building a series of `Exp` values.

use std::borrow::Cow;
use std::fmt;

use log::*;

use crate::ast::*;
use crate::pos::Placed;
use crate::tokens::Token;


/// Parses the given iterator of tokens into a vector of expressions, failing
/// at the first parse error.
pub fn parse_tokens<'src>(iter: impl IntoIterator<Item=Token<'src>>) -> Result<Vec<Exp<'src>>, Error<'src>> {
    let mut iter = iter.into_iter();
    let mut parser = Parser::new(&mut iter);
    parser.parse()?;
    Ok(parser.exps)
}

/// The internal parser.
struct Parser<'iter, 'src, I> {

    /// The iterator that the parser gets tokens to examine from.
    iter: &'iter mut I,

    /// The list of expressions that gets built up over time.
    exps: Vec<Exp<'src>>,

    /// The parser’s current state.
    state: State<'src>,

    /// If this parser is parsing tokens that occur after the open parenthesis
    /// of a function, rather than at the top-level, this holds the position
    /// of the parenthesis token.
    function_start: Option<Placed<&'src str>>,
}

/// The state of a parser.
#[derive(Debug, Copy, Clone)]
enum State<'src> {

    /// We have just started parsing, or the previous token emitted an
    /// expression that does not depend on the _next_ token, so we are ready
    /// to parse anything.
    Ready,

    /// We have just parsed an alphanumeric token, so the next stept to take
    /// depends on what next comes out the iterator.
    ReadAlphanum(Placed<&'src str>),
}


impl<'iter, 'src, I> Parser<'iter, 'src, I> {

    /// Creates a new parser that reads from the given iterator.
    fn new(iter: &'iter mut I) -> Self {
        let state = State::Ready;
        let exps = Vec::new();
        let function_start = None;
        Self { iter, exps, state, function_start }
    }
}

#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
impl<'iter, 'src, I: 'iter + Iterator<Item=Token<'src>>> Parser<'iter, 'src, I> {

    /// Perform parsing, exhausting the iterator and building up the internal
    /// vector of expressions.
    fn parse(&mut self) -> Result<(), Error<'src>> {
        while let Some(token) = self.iter.next() {
            trace!("Read token {:?}", token);

            match (token, self.state) {
                (Token::Alphanum(slice), State::Ready) => {
                    self.state = State::ReadAlphanum(slice);
                }

                (Token::Alphanum(_), State::ReadAlphanum(_)) => {
                    unreachable!();
                }

                (Token::Open(open), State::ReadAlphanum(slice)) => {
                    let mut sub_parser = Parser::new(self.iter);
                    sub_parser.function_start = Some(open);
                    sub_parser.parse()?;
                    let args = sub_parser.exps;

                    let name = match parse_function_name(slice)? {
                        Some(n) => n,
                        None    => return Err(Error::InvalidFunctionName(slice)),
                    };

                    self.exps.push(Exp::Function { name, args });
                    self.state = State::Ready;
                }

                (Token::Open(span), State::Ready) => {
                    return Err(Error::StrayCharacter(span));
                }

                (Token::Close(span), State::Ready) => {
                    if self.function_start.is_none() {
                        return Err(Error::StrayCharacter(span));
                    }

                    self.state = State::Ready;
                    self.function_start = None;  // skip check below
                    break;
                }
                (Token::Close(span), State::ReadAlphanum(slice)) => {
                    if self.function_start.is_none() {
                        return Err(Error::StrayCharacter(span));
                    }

                    let alphanums = parse_alphanums(slice)?;
                    self.add(alphanums, slice)?;
                    self.state = State::Ready;
                    self.function_start = None;  // skip check below
                    break;
                }

                (Token::Form(form_slice), State::ReadAlphanum(alpha_slice)) => {
                    let form = parse_form(form_slice)?;

                    let name = match parse_function_name(alpha_slice)? {
                        Some(n) => n,
                        None    => return Err(Error::InvalidFunctionName(alpha_slice)),
                    };

                    self.exps.push(Exp::Function { name, args: vec![ form ] });
                    self.state = State::Ready;
                }

                (Token::Form(slice), State::Ready) => {
                    let form = parse_form(slice)?;
                    self.exps.push(form);
                }

                (Token::Quoted(slice), State::Ready) => {
                    let chars = parse_backslashes(slice)?;
                    self.exps.push(Exp::StringLiteral { chars });
                }

                (Token::Quoted(quote_slice), State::ReadAlphanum(alpha_slice)) => {
                    let alphanums = parse_alphanums(alpha_slice)?;
                    self.add(alphanums, alpha_slice)?;
                    let chars = parse_backslashes(quote_slice)?;
                    self.exps.push(Exp::StringLiteral { chars });
                    self.state = State::Ready;
                }

                (Token::Whitespace, State::Ready) => {
                    // ignore it
                }

                (Token::Whitespace, State::ReadAlphanum(slice)) => {
                    let alphanums = parse_alphanums(slice)?;
                    self.add(alphanums, slice)?;
                    self.state = State::Ready;
                }

                (Token::Stray(_), _) => {
                    unreachable!("Stray token not filtered out by read");
                }
            }

            trace!("Parse state is {:?}", self.state);
        }

        if let Some(open) = self.function_start {
            return Err(Error::UnclosedFunction(open));
        }

        if let State::ReadAlphanum(slice) = self.state {
            let alphanums = parse_alphanums(slice)?;
            self.add(alphanums, slice)?;
        }

        Ok(())
    }

    fn add(&mut self, alphanums: Alphanums<'src>, original_slice: Placed<&'src str>) -> Result<(), Error<'src>> {
        match alphanums {
            Alphanums::Bytes(bytes) => {
                self.exps.extend(bytes.iter().copied().map(Exp::Char));
                Ok(())
            }
            Alphanums::ConstantName(name) => {
                self.exps.push(Exp::Constant { name });
                Ok(())
            }
            Alphanums::FunctionName(_) => {
                Err(Error::StrayFunctionName(original_slice))
            }
        }
    }
}


/// A string of alphanumeric characters has several different interpretations,
/// which unfortunately _also_ vary depending on the surrounding tokens.
#[derive(PartialEq, Debug)]
enum Alphanums<'src> {

    /// The characters form a function name, such as `x11`.
    FunctionName(FunctionName),

    /// The characters form a constant name, such as `BGP_OPEN`.
    ConstantName(&'src str),

    /// The characters form some hex bytes, such as `09F7`.
    Bytes(Vec<u8>),
}

/// Parses a string of alphanumeric characters into some `Alphanums`,
/// returning an error if the string does not match any of the known patterns.
#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
fn parse_alphanums(span: Placed<&'_ str>) -> Result<Alphanums<'_>, Error<'_>> {
    let input = span.contents;

    if is_constant_name(input) {
        Ok(Alphanums::ConstantName(input))
    }
    else if let Some(name) = parse_function_name(span)? {
        Ok(Alphanums::FunctionName(name))
    }
    else {
        let mut bytes = Vec::new();
        let mut chars = input.char_indices();

        #[allow(clippy::cast_possible_truncation)]
        while let Some((index, first_char)) = chars.next() {
            let first_value = match first_char.to_digit(16) {
                Some(f) => f as u8,
                None => {
                    let placed = span.substring(index, index + 1);
                    return Err(Error::StrayCharacter(placed));
                }
            };

            let (index2, second_char) = match chars.next() {
                Some(t) => t,
                None => {
                    let placed = span.substring(index, index + 1);
                    return Err(Error::SingleHex(placed));
                }
            };

            let second_value = match second_char.to_digit(16) {
                Some(f) => f as u8,
                None => {
                    let placed = span.substring(index2, index2 + 1);
                    return Err(Error::StrayCharacter(placed));
                }
            };

            bytes.push(first_value * 16 + second_value);
        }

        Ok(Alphanums::Bytes(bytes))
    }
}

/// Determines whether this set of alphanums is a valid constant name: it must
/// start with at least one uppercase letter, and then contain uppercase
/// letters or digits or underscores.
#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate(mutators = not(lit_int, binop_num)))]
fn is_constant_name(input: &str) -> bool {
    // by this point, non-ASCII characters should already be handled
    input.len() >= 3 &&
        input.contains('_') &&
        input[0..1].chars().all(|c| c.is_ascii_uppercase()) &&
        input[1..].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Parses a string of characters into a function name, returning an error if
/// the string does not match any of the known function names.
#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
fn parse_function_name(span: Placed<&'_ str>) -> Result<Option<FunctionName>, Error<'_>> {
    let input = span.contents;
    if input.is_empty() {
        unreachable!("Empty function name")
    }
    else if input.len() >= 2 && input.as_bytes()[0] == b'x' && input[1..].bytes().all(|b| b.is_ascii_digit()) {
        match input[1..].parse() {
            Ok(0) => {
                warn!("Repeat amount of zero");
                Err(Error::InvalidRepeatAmount(span))
            }
            Err(e) => {
                warn!("Error parsing repeat amount: {}", e);
                Err(Error::InvalidRepeatAmount(span))
            }
            Ok(repeat_amount) => {
                Ok(Some(FunctionName::Repeat(repeat_amount)))
            }
        }
    }
    else {
        match input {
            "be16" => Ok(Some(FunctionName::MultiByte(MultiByteType::Be16))),
            "be32" => Ok(Some(FunctionName::MultiByte(MultiByteType::Be32))),
            "be64" => Ok(Some(FunctionName::MultiByte(MultiByteType::Be64))),
            "le16" => Ok(Some(FunctionName::MultiByte(MultiByteType::Le16))),
            "le32" => Ok(Some(FunctionName::MultiByte(MultiByteType::Le32))),
            "le64" => Ok(Some(FunctionName::MultiByte(MultiByteType::Le64))),
            "and"  => Ok(Some(FunctionName::Bitwise(BitwiseFold::And))),
            "or"   => Ok(Some(FunctionName::Bitwise(BitwiseFold::Or))),
            "xor"  => Ok(Some(FunctionName::Bitwise(BitwiseFold::Xor))),
            "not"  => Ok(Some(FunctionName::BitwiseNot)),
            _      => Ok(None),
        }
    }
}

/// Parses the contents of a form into an expression, returning an error if
/// the string does not match any of the known form patterns. The string
/// should not include the surrounding `[` and `]` characters.
#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
fn parse_form(span: Placed<&'_ str>) -> Result<Exp<'_>, Error<'_>> {
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    let input = span.contents;

    if input.is_empty() {
        Err(Error::InvalidForm(span))
    }
    else if input.chars().all(|c| c.is_ascii_digit()) {
        Ok(Exp::Dec(input))
    }
    else if let Ok(ip) = Ipv4Addr::from_str(input) {
        Ok(Exp::IPv4 { bytes: ip.octets() })
    }
    else if let Ok(ip) = Ipv6Addr::from_str(input) {
        Ok(Exp::IPv6 { bytes: ip.octets() })
    }
    else if let Some(bit_vec) = parse_bit_form(input) {
        Ok(Exp::Bits(bit_vec))
    }
    else if let Some(float) = parse_float_form(input) {
        Ok(Exp::Float(float))
    }
    else if let Ok(time) = humantime::parse_rfc3339_weak(input) {
        let unix_time = time.duration_since(std::time::SystemTime::UNIX_EPOCH).expect("epoch fail");
        Ok(Exp::Timestamp(unix_time.as_secs() as u32))  // TODO: 64-bit timestamps
    }
    else {
        Err(Error::InvalidForm(span))
    }
}

/// Examines the contents of a form to see if it looks like a series of bits;
/// if it does, parses it into a vector of bits, and if not, returns `None`.
#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate(mutators = not(lit_int, binop_num)))]
fn parse_bit_form(input: &str) -> Option<Vec<bool>> {
    if input.starts_with('b') && input[1..].bytes().all(|c| c == b'1' || c == b'0' || c == b'_') {
        let mut bit_vec = Vec::with_capacity(input.len() - 1);  // skip mutation testing again

        for byte in input[1..].bytes() {
            match byte {
                b'0'  => bit_vec.push(false),
                b'1'  => bit_vec.push(true),
                b'_'  => {/* skip */},
                _     => unreachable!(),
            }
        }

        if bit_vec.is_empty() {
            None
        }
        else {
            Some(bit_vec)
        }
    }
    else {
        None
    }
}

/// Examines the contents of a form to see if it looks like a floating point
/// form, returning the float part of the input string if it does. This cannot
/// return the parsed value yet, as we don’t know whether it should be an
/// `f32` or an `f64`.
#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate(mutators = not(lit_int, binop_num)))]
fn parse_float_form(input: &str) -> Option<&str> {
    if let Some(float_chars) = input.strip_prefix('f') {
        // TODO: is this the best way to do this?
        let _: f64 = float_chars.parse().ok()?;
        Some(float_chars)
    }
    else {
        None
    }
}

/// Parse the contents of a quoted string into its canonical form by handling
/// escaped backslashes and quotes. This returns a copy of the original string
/// slice if it does not need to be modified; otherwise, it allocates and
/// returns a new string.
#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate(mutators = not(lit_int, binop_num)))]
fn parse_backslashes<'src>(span: Placed<&'src str>) -> Result<Cow<'src, str>, Error<'src>> {
    let input = span.contents;

    if ! input.contains('\\') {
        return Ok(input.into());
    }

    // The resulting string must be, at a minimum, half the length of the
    // original (as "\n" will turn into one byte).
    let mut result = String::with_capacity(input.len() / 2);  // this doesn’t need mutation testing

    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }

        match chars.next() {
            Some('n')  => result.push('\n'),
            Some('r')  => result.push('\r'),
            Some('t')  => result.push('\t'),
            Some(nc)   => result.push(nc),
            None       => unreachable!("String ends with backslash"),
        }
    }

    Ok(result.into())
}


/// An error that can occur during parsing.
#[derive(PartialEq, Debug)]
pub enum Error<'src> {

    /// A block of alphanumeric characters intended to be read as hex bytes
    /// ended with one character unpaired with another, such as `3` or `D7B`.
    SingleHex(Placed<&'src str>),

    /// A block of alphanumeric characters intended to be read as hex bytes
    StrayCharacter(Placed<&'src str>),

    /// A function name was not followed by an opening `(` parenthesis.
    StrayFunctionName(Placed<&'src str>),

    /// A block of alphanumeric characters was placed before an opening `(`
    /// token, signifying the name of a function, but the characters do not
    /// form a valid function name.
    InvalidFunctionName(Placed<&'src str>),

    /// A function name indicated that it was a repeat function, but the
    /// number of times to repeat failed to be parsed: it was either zero,
    /// such as `x0`, or too large, such as `x99999999999`.
    InvalidRepeatAmount(Placed<&'src str>),

    /// A form contained contents that did not match one of the known format,
    /// such as `[plum pudding]`.
    InvalidForm(Placed<&'src str>),

    /// The parser saw an opening `(` token and started reading
    /// sub-expressions for the function’s arguments, but before reading a
    /// closing `)` token, the stream of tokens ran out.
    UnclosedFunction(Placed<&'src str>),
}

impl<'src> fmt::Display for Error<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleHex(c)                => write!(f, "Unpaired hex character {:?}", c.contents),
            Self::StrayCharacter(c)           => write!(f, "Stray character {:?}", c.contents),
            Self::StrayFunctionName(name)     => write!(f, "Function name {:?} not followed by arguments", name.contents),
            Self::InvalidFunctionName(name)   => write!(f, "Invalid function name {:?}", name.contents),
            Self::InvalidRepeatAmount(ra)     => write!(f, "Invalid repeat amount {:?}", ra.contents),
            Self::InvalidForm(form)           => write!(f, "Could not interpret form {:?}", form.contents),
            Self::UnclosedFunction(_)         => write!(f, "Unclosed function"),
        }
    }
}

impl<'src> Error<'src> {

    /// Returns the `Placed` token at the heart of the error, to tell the user
    /// at which point in the source file the error occurred.
    pub fn source_pos(&self) -> &Placed<&'src str> {
        match self {
            Self::SingleHex(c)                => c,
            Self::StrayCharacter(c)           => c,
            Self::StrayFunctionName(name)     => name,
            Self::InvalidFunctionName(name)   => name,
            Self::InvalidRepeatAmount(ra)     => ra,
            Self::InvalidForm(form)           => form,
            Self::UnclosedFunction(open)      => open,
        }
    }
}


#[cfg(test)]
mod test_parse_alphanums {
    use pretty_assertions::assert_eq;
    use crate::pos::At;
    use super::*;

    #[test]
    #[should_panic]
    fn utf8_byte() {
        let _ = parse_alphanums("é".at(1, 0));
    }

    #[test]
    fn one_byte() {
        assert_eq!(parse_alphanums("EF".at(1, 0)),
                   Ok(Alphanums::Bytes(vec![ 0xEF ])));
    }

    #[test]
    fn two_bytes() {
        assert_eq!(parse_alphanums("EF12".at(1, 0)),
                   Ok(Alphanums::Bytes(vec![ 0xEF, 0x12 ])));
    }

    #[test]
    fn half_a_byte() {
        assert_eq!(parse_alphanums("E".at(1, 0)),
                   Err(Error::SingleHex("E".at(1, 0))));
    }

    #[test]
    fn not_a_byte() {
        assert_eq!(parse_alphanums("Ex".at(1, 0)),
                   Err(Error::StrayCharacter("x".at(1, 1))));
    }

    #[test]
    fn first_g() {
        assert_eq!(parse_alphanums("FG".at(1, 0)),
                   Err(Error::StrayCharacter("G".at(1, 1))));
    }

    #[test]
    fn second_g() {
        assert_eq!(parse_alphanums("GF".at(1, 0)),
                   Err(Error::StrayCharacter("G".at(1, 0))));
    }

    #[test]
    fn constant_name() {
        assert_eq!(parse_alphanums("DNS_AAAA".at(1, 0)),
                   Ok(Alphanums::ConstantName("DNS_AAAA")));
    }

    #[test]
    fn shortest_possible_constant() {
        assert_eq!(parse_alphanums("A_B".at(1, 0)),
                   Ok(Alphanums::ConstantName("A_B")));
    }

    #[test]
    fn constant_ending_with_numbers() {
        assert_eq!(parse_alphanums("DNS_EUI48".at(1, 0)),
                   Ok(Alphanums::ConstantName("DNS_EUI48")));
    }

    #[test]
    fn constant_too_short() {
        assert_eq!(parse_alphanums("_A".at(1, 0)),
                   Err(Error::StrayCharacter("_".at(1, 0))));
    }

    #[test]
    fn constant_still_too_short() {
        assert_eq!(parse_alphanums("A_".at(1, 0)),
                   Err(Error::StrayCharacter("_".at(1, 1))));
    }
}


#[cfg(test)]
mod test_parse_form {
    use pretty_assertions::assert_eq;
    use crate::pos::At;
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(parse_form("".at(1, 0)),
                   Err(Error::InvalidForm("".at(1, 0))));
    }

    #[test]
    fn numbers() {
        assert_eq!(parse_form("1234567".at(1, 0)),
                   Ok(Exp::Dec("1234567")));
    }

    #[test]
    fn ipv4() {
        assert_eq!(parse_form("127.0.0.1".at(1, 0)),
                   Ok(Exp::IPv4 { bytes: [127, 0, 0,  1] }));
    }

    #[test]
    fn ipv6() {
        assert_eq!(parse_form("::1".at(1, 0)),
                   Ok(Exp::IPv6 { bytes: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1] }));
    }

    #[test]
    fn bits() {
        assert_eq!(parse_form("b0110110".at(1, 0)),
                   Ok(Exp::Bits(vec![false, true, true, false, true, true, false])));
    }

    #[test]
    fn bits_underscore() {
        assert_eq!(parse_form("b011_0110".at(1, 0)),
                   Ok(Exp::Bits(vec![false, true, true, false, true, true, false])));
    }

    #[test]
    fn no_bits() {
        assert_eq!(parse_form("b".at(1, 0)),
                   Err(Error::InvalidForm("b".at(1, 0))));
    }

    #[test]
    fn bad_under_bits() {
        assert_eq!(parse_form("b_".at(1, 0)),
                   Err(Error::InvalidForm("b_".at(1, 0))));
    }

    #[test]
    fn no_such_thing_as_two() {
        assert_eq!(parse_form("b0110112".at(1, 0)),
                   Err(Error::InvalidForm("b0110112".at(1, 0))));
    }

    #[test]
    fn something_else() {
        assert_eq!(parse_form("something_else".at(1, 0)),
                   Err(Error::InvalidForm("something_else".at(1, 0))));
    }

    #[test]
    fn float_well() {
        assert_eq!(parse_form("f1.5".at(1, 0)),
                   Ok(Exp::Float("1.5")));
    }

    #[test]
    fn float_badly() {
        assert_eq!(parse_form("foo".at(1, 0)),
                   Err(Error::InvalidForm("foo".at(1, 0))));
    }
}


#[cfg(test)]
mod test_parse_function_name {
    use pretty_assertions::assert_eq;
    use crate::pos::At;
    use super::*;

    #[test]
    #[should_panic]
    fn empty() {
        let _ = parse_function_name("".at(1, 0));
    }

    #[test]
    fn once() {
        assert_eq!(parse_function_name("x1".at(1, 0)),
                   Ok(Some(FunctionName::Repeat(1))));
    }

    #[test]
    fn eleven_times() {
        assert_eq!(parse_function_name("x11".at(1, 0)),
                   Ok(Some(FunctionName::Repeat(11))));
    }

    #[test]
    fn nonce() {
        assert_eq!(parse_function_name("x0".at(1, 0)),
                   Err(Error::InvalidRepeatAmount("x0".at(1, 0))));
    }

    #[test]
    fn too_many_times() {
        assert_eq!(parse_function_name("x99999999999".at(1, 0)),
                   Err(Error::InvalidRepeatAmount("x99999999999".at(1, 0))));
    }

    #[test]
    fn be16() {
        assert_eq!(parse_function_name("be16".at(1, 0)),
                   Ok(Some(FunctionName::MultiByte(MultiByteType::Be16))));
    }

    #[test]
    fn be32() {
        assert_eq!(parse_function_name("be32".at(1, 0)),
                   Ok(Some(FunctionName::MultiByte(MultiByteType::Be32))));
    }

    #[test]
    fn le64() {
        assert_eq!(parse_function_name("le64".at(1, 0)),
                   Ok(Some(FunctionName::MultiByte(MultiByteType::Le64))));
    }

    #[test]
    fn missing_repeat_amount() {
        assert_eq!(parse_function_name("x".at(1, 0)),
                   Ok(None));
    }

    #[test]
    fn two_xs() {
        assert_eq!(parse_function_name("xx11".at(1, 0)),
                   Ok(None));
    }

    #[test]
    fn nonsense() {
        assert_eq!(parse_function_name("fhqwhgads".at(1, 0)),
                   Ok(None));
    }

    #[test]
    fn nonsense_numbers() {
        assert_eq!(parse_function_name("0123456789".at(1, 0)),
                   Ok(None));
    }
}


#[cfg(test)]
mod test_parse_quotes {
    use pretty_assertions::assert_eq;
    use crate::pos::At;
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(parse_backslashes("".at(1, 0)),
                   Ok(Cow::Borrowed("")));
    }

    #[test]
    fn longer() {
        assert_eq!(parse_backslashes("longer".at(1, 0)),
                   Ok(Cow::Borrowed("longer")));
    }

    #[test]
    fn backslash_slash() {
        assert_eq!(parse_backslashes("back\\\\slash".at(1, 0)),
                   Ok(Cow::from("back\\slash".to_string())));
    }

    #[test]
    fn backslash_quote() {
        assert_eq!(parse_backslashes("back\\\"slash".at(1, 0)),
                   Ok(Cow::from("back\"slash".to_string())));
    }

    #[test]
    #[should_panic]
    fn backslash_end() {
        let _ = parse_backslashes("back\\".at(1, 0));
    }
}


#[cfg(test)]
mod test_edge_cases {
    use pretty_assertions::assert_eq;
    use crate::pos::At;
    use super::*;

    #[test]
    fn just_a_form() {
        let tokens = vec![ Token::Form("32".at(1, 5)) ];

        assert_eq!(parse_tokens(tokens),
                   Ok(vec![ Exp::Dec("32") ]));
    }

    #[test]
    fn a_content_constant() {
        let tokens = vec![ Token::Alphanum("GPS_QUERY".at(1, 5)) ];

        assert_eq!(parse_tokens(tokens),
                   Ok(vec![ Exp::Constant { name: "GPS_QUERY" } ]));
    }

    #[test]
    fn form_function() {
        let tokens = vec![ Token::Alphanum("le32".at(1, 0)),
                           Token::Form("32".at(1, 5)) ];

        assert_eq!(parse_tokens(tokens),
                   Ok(vec![ Exp::Function {
                       name: FunctionName::MultiByte(MultiByteType::Le32),
                       args: vec![ Exp::Dec("32") ],
                   } ]));
    }

    #[test]
    fn a_function() {
        let tokens = vec![ Token::Alphanum("x11".at(1, 0)),
                           Token::Open("(".at(1, 0)),
                           Token::Alphanum("AB".at(1, 0)),
                           Token::Close(")".at(1, 0)) ];

        assert_eq!(parse_tokens(tokens),
                   Ok(vec![ Exp::Function {
                       name: FunctionName::Repeat(11),
                       args: vec![ Exp::Char(0xAB) ],
                   } ]));
    }

    #[test]
    fn empty_function() {
        let tokens = vec![ Token::Alphanum("x11".at(1, 0)),
                           Token::Open("(".at(1, 0)),
                           Token::Close(")".at(1, 0)) ];

        assert_eq!(parse_tokens(tokens),
                   Ok(vec![ Exp::Function {
                       name: FunctionName::Repeat(11),
                       args: vec![],
                   } ]));
    }

    #[test]
    fn suddenly_close() {
        assert_eq!(parse_tokens(vec![ Token::Close(")".at(1, 0)) ]),
                   Err(Error::StrayCharacter(")".at(1, 0))));
    }

    #[test]
    fn suddenly_open() {
        assert_eq!(parse_tokens(vec![ Token::Open("(".at(1, 0)) ]),
                   Err(Error::StrayCharacter("(".at(1, 0))));
    }

    #[test]
    fn stray_function_name() {
        assert_eq!(parse_tokens(vec![ Token::Alphanum("le32".at(1, 0)) ]),
                   Err(Error::StrayFunctionName("le32".at(1, 0))));
    }

    #[test]
    fn unclosed_function() {
        let tokens = vec![ Token::Alphanum("x11".at(1, 0)),
                           Token::Open("(".at(1, 0)),
                           Token::Alphanum("AB".at(1, 0)) ];

        assert_eq!(parse_tokens(tokens),
                   Err(Error::UnclosedFunction("(".at(1, 0))));
    }

    #[test]
    fn byte_string() {
        let tokens = vec![ Token::Alphanum("11".at(1, 0)),
                           Token::Quoted("bytes".at(1, 2)) ];

        assert_eq!(parse_tokens(tokens),
                   Ok(vec![ Exp::Char(0x11), Exp::StringLiteral { chars: "bytes".into() } ]));
    }
}
