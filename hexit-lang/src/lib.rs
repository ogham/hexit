//! The hexit language crate, the “back end” of the Hexit program, which can
//! lex and parse Hexit code from strings and evaluate it into a vector of
//! bytes. Things like reading from files, displaying errors, and formatting
//! the bytes into text is done in the ‘hexit’ crate.
//!
//! Interpreting a Hexit program is done in two steps, the first of which has
//! two steps itself:
//!
//! 1. First, the program gets “read” from a string — it gets tokenised into a
//!    series of `Token` values, which get parsed into a series of `Exp` values.
//! 2. Next, once all of the input program has been read, it gets “run” — the
//!    expressions are evaluated, resulting in a series of bytes.

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
pub mod constants;
mod eval;
mod lex;
mod parse;
mod pos;
mod read;
mod tokens;


/// A Hexit program.
pub struct Program<'src> {
    exps: Vec<ast::Exp<'src>>,
}

impl<'src> Program<'src> {

    /// Reads a Hexit program from a series of strings of Hexit source,
    /// returning a valid program or at least one read error.
    pub fn read(input_source_lines: &'src [impl AsRef<str>]) -> Result<Self, Vec<read::Error<'src>>> {
        let mut all_exps = Vec::new();
        let mut all_errors = Vec::new();

        for (line_index, input_line) in input_source_lines.iter().enumerate() {
            let input_line = input_line.as_ref();
            debug!("Reading line → {:?}", input_line);

            let line_number = line_index + 1;
            match read::tokenise_and_parse(input_line, line_number) {
                Ok(exps)  => all_exps.extend(exps),
                Err(e)    => all_errors.push(e),
            };
        }

        if all_errors.is_empty() {
            Ok(Self { exps: all_exps })
        }
        else {
            Err(all_errors)
        }
    }

    /// Runs this Hexit program, returning the vector of bytes that it has
    /// produced, or an evaluation error.
    pub fn run(self, constants: &constants::Table, limit: Option<usize>) -> Result<Vec<u8>, eval::Error<'src>> {
        debug!("Running expressions → {:#?}", self.exps);

        let bytes = eval::evaluate_exps(self.exps, constants, limit)?;
        Ok(bytes)
    }
}
