//! The hexit language crate, the “back end” of the Hexit program, which can
//! lex and parse Hexit code from strings and evaluate it into a vector of
//! bytes. Things like reading from files, displaying errors, and formatting
//! the bytes into text is done in the ‘hexit’ crate.

#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(nonstandard_style)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::single_match_else)]
#![allow(clippy::wildcard_imports)]

#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::cast_possible_wrap)]
#![deny(clippy::cast_sign_loss)]
#![deny(unsafe_code)]

use log::*;

mod ast;
mod constants;
mod eval;
mod lex;
mod parse;
mod pos;
mod read;
mod tokens;

pub use self::constants::ConstantsTable;


/// A Hexit program.
pub struct Program<'src> {
    exps: Vec<ast::Exp<'src>>,
}

impl<'src> Program<'src> {

    /// Reads a Hexit program from a string of Hexit source, returning a valid
    /// program or a read error.
    pub fn read(input_source: &'src str) -> Result<Self, read::Error<'src>> {
        debug!("Reading string {:?}", input_source);
        let exps = read::tokenise_and_parse(input_source)?;
        trace!("Expressions {:#?}", exps);
        Ok(Self { exps })
    }

    /// Runs this Hexit program, returning the vector of bytes that it has
    /// produced, or an evaluation error.
    pub fn run(self, constants: &ConstantsTable, limit: Option<usize>) -> Result<Vec<u8>, eval::Error<'src>> {
        let bytes = eval::evaluate_exps(self.exps, constants, limit)?;
        Ok(bytes)
    }
}
