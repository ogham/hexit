//! Converting the series of bytes that result from evaluating a Hexit program
//! to characters to print as output.

use std::io::{self, Write};


/// How to format the output bytes as ASCII characters.
///
/// When not in Raw mode, where output bytes are emitted directly, a `Style`
/// is used to turn one byte into two or more ASCII characters, which can be
/// `0` to `9`, `a` to `f`, or `A` to `F`. These characters can have strings
/// before, after, or between them.
///
/// The default behaviour is to have uppercase characters with no extra
/// strings anywhere.
#[derive(PartialEq, Debug, Default)]
pub struct Style {

    /// The string to print _before_ each pair of characters.
    pub prefix: Option<String>,

    /// The string to print _after_ each pair of characters.
    pub suffix: Option<String>,

    /// The string to print _between_ successive pairs of characters.
    pub separator: Option<String>,

    /// Whether you like your letters minuscule.
    pub case: LetterCase,
}

/// The case of the alphabetic formatted hex characters.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum LetterCase {
    Upper,
    Lower,
}

impl Default for LetterCase {
    fn default() -> Self {
        LetterCase::Upper
    }
}

impl Style {

    /// Given a source iterator of bytes, and a sink to write to, formats each
    /// byte read with the style prefix, suffix, separator, and case before
    /// writing it to the sink.
    pub fn format(&self, source: impl Iterator<Item=u8>, mut sink: impl Write) -> io::Result<usize> {
        let mut first = true;
        let mut count = 0;

        for byte in source {
            if first {
                first = false;
            }
            else if let Some(ref sep) = self.separator {
                write!(sink, "{}", sep)?;
            }

            if let Some(ref prefix) = self.prefix {
                write!(sink, "{}", prefix)?;
            }

            match self.case {
                LetterCase::Lower => write!(sink, "{:02x}", byte)?,
                LetterCase::Upper => write!(sink, "{:02X}", byte)?,
            }

            if let Some(ref suffix) = self.suffix {
                write!(sink, "{}", suffix)?;
            }

            count += 1;
        }

        writeln!(sink)?;

        Ok(count)
    }
}


#[cfg(test)]
#[allow(unused_results)]
mod test {
    use super::*;

    #[test]
    fn plain() {
        let style = Style::default();

        let bytes = [ 0x67_u8, 0x30, 0x19, 0x41, 0xAB ];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"67301941AB\n", &*output);
    }

    #[test]
    fn lowercase() {
        let mut style = Style::default();
        style.case = LetterCase::Lower;

        let bytes = [ 0x67_u8, 0x30, 0x19, 0x41, 0xAB ];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"67301941ab\n", &*output);
    }

    #[test]
    fn separator() {
        let mut style = Style::default();
        style.separator = Some(String::from(" "));

        let bytes = [ 0x67_u8, 0x30, 0x19, 0x41, 0xAB ];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"67 30 19 41 AB\n", &*output);
    }

    #[test]
    fn prefix() {
        let mut style = Style::default();
        style.prefix = Some(String::from(":"));

        let bytes = [ 0x67_u8, 0x30, 0x19, 0x41, 0xAB ];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b":67:30:19:41:AB\n", &*output);
    }

    #[test]
    fn suffix() {
        let mut style = Style::default();
        style.suffix = Some(String::from(";"));

        let bytes = [ 0x67_u8, 0x30, 0x19, 0x41, 0xAB ];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"67;30;19;41;AB;\n", &*output);
    }

    #[test]
    fn the_whole_kitten_caboodle() {
        let style = Style {
            prefix:    Some(String::from("0x")),
            suffix:    Some(String::from("!")),
            separator: Some(String::from(" ")),
            case:      LetterCase::Upper,
        };

        let bytes = [ 0x67_u8, 0x30, 0x19, 0x41, 0xAB ];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"0x67! 0x30! 0x19! 0x41! 0xAB!\n", &*output);
    }

    #[test]
    fn unit() {
        let style = Style::default();

        let byte = [ 0xF0_u8 ];

        let mut output = Vec::new();
        style.format(byte.iter().copied(), &mut output).unwrap();
        assert_eq!(b"F0\n", &*output);
    }

    #[test]
    fn unit_styled() {
        let style = Style {
            prefix:    Some(String::from("[")),
            suffix:    Some(String::from("]")),
            separator: Some(String::from("UNUSED")),
            case:      LetterCase::Upper,
        };

        let byte = [ 0xF0_u8 ];

        let mut output = Vec::new();
        style.format(byte.iter().copied(), &mut output).unwrap();
        assert_eq!(b"[F0]\n", &*output);
    }

    #[test]
    fn void() {
        let style = Style::default();

        let bytes = [];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"\n", &*output);
    }

    #[test]
    fn void_style() {
        let style = Style {
            prefix:    Some(String::from("UNUSED")),
            suffix:    Some(String::from("ALSO UNUSED")),
            separator: Some(String::from("THIS TOO IS UNUSED")),
            case:      LetterCase::Upper,
        };

        let bytes = [];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"\n", &*output);
    }

    #[test]
    fn zeroes() {
        let style = Style::default();
        let bytes = [ 0x00_u8, 0x00, 0x01 ];

        let mut output = Vec::new();
        style.format(bytes.iter().copied(), &mut output).unwrap();
        assert_eq!(b"000001\n", &*output);
    }
}
