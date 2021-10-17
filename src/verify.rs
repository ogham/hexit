//! Verifying that the output’s length matches a property before printing it.


/// Hexit can be run with some **verification** that can be run after all the
/// output has been generated, making sure that its length matches some
/// property.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Verification {

    /// Hexit should verify that the output’s length _exactly_ matches the
    /// given number.
    ExactLength(OutputLength),

    /// Hexit should verify that the output’s length is a multiple of the
    /// given number.
    Multiple(OutputLength),

    /// Hexit should not verify anything and just print the output.
    AnythingGoes,
}

/// The number of bytes that get produced as a result of running Hexit.
pub type OutputLength = usize;

impl Verification {

    /// Verifies the computed output length to make sure it conforms to the
    /// user’s wishes, returning a string describing what the length _should_
    /// be if validation fails.
    pub fn verify(self, ol: OutputLength) -> Result<(), String> {
        if let Verification::ExactLength(exact) = self {
            if ol != exact {
                return Err(format!("{}", exact));
            }
        }

        if let Verification::Multiple(multiple) = self {
            if ol % multiple != 0 {
                return Err(format!("multiple of {}", multiple));
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn anything_1() {
        assert_eq!(Ok(()), Verification::AnythingGoes.verify(1));
    }

    #[test]
    fn anything_0() {
        assert_eq!(Ok(()), Verification::AnythingGoes.verify(0));
    }

    #[test]
    fn exact_hit() {
        assert_eq!(Ok(()), Verification::ExactLength(13).verify(13));
    }

    #[test]
    fn exact_miss() {
        assert_eq!(Err("13".into()), Verification::ExactLength(13).verify(3));
    }

    #[test]
    fn multiple_exact() {
        assert_eq!(Ok(()), Verification::Multiple(13).verify(13));
    }

    #[test]
    fn multiple_half() {
        assert_eq!(Ok(()), Verification::Multiple(13).verify(26));
    }

    #[test]
    fn multiple_miss() {
        assert_eq!(Err("multiple of 13".into()), Verification::Multiple(13).verify(3));
    }
}
