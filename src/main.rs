#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]
#![warn(nonstandard_style)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::match_bool)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::wildcard_imports)]

#![deny(unsafe_code)]


use std::fs::File;
use std::io::{self, Write};

use log::*;

use hexit_lang::Program;
use hexit_lang::constants::{Table, Constant};

mod colours;
mod console;
mod logger;
mod input;
mod options;
mod style;
mod verify;
use crate::options::{RunningMode, Options, Output, Format, OptionsResult, HelpReason};


fn main() {
    use std::env;
    use std::process::exit;

    logger::configure(env::var_os("HEXIT_DEBUG"));

    match RunningMode::getopts(env::args_os().skip(1)) {
        OptionsResult::Ok(run_mode) => {
            exit(run(run_mode));
        }

        OptionsResult::Help(help_reason, use_colours) => {
            if use_colours.should_use_colours() {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/usage.pretty.txt")));
            }
            else {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/usage.bland.txt")));
            }

            if help_reason == HelpReason::NoArguments {
                exit(exits::OPTIONS_ERROR);
            }
            else {
                exit(exits::SUCCESS);
            }
        }

        OptionsResult::Version(use_colours) => {
            if use_colours.should_use_colours() {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/version.pretty.txt")));
            }
            else {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/version.bland.txt")));
            }

            exit(exits::SUCCESS);
        }

        OptionsResult::InvalidOptionsFormat(oe) => {
            eprintln!("Invalid options: {:?}", oe);
            exit(exits::OPTIONS_ERROR);
        }

        OptionsResult::InvalidOptions(oe) => {
            eprintln!("Invalid options: {:?}", oe);
            exit(exits::OPTIONS_ERROR);
        }
    }
}


/// The main program entry point.
pub fn run(mode: RunningMode) -> i32 {
    info!("Running with mode → {:#?}", mode);

    match mode {
        RunningMode::Run(opts) => {
            let Options { input, output, format, verification, limit } = opts;
            let source_lines = match input.read() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}: {}", input, e);
                    return exits::IO_ERROR;
                }
            };

            let program = match Program::read(&source_lines) {
                Ok(p) => p,
                Err(es) => {
                    for e in es {
                        eprintln!("{}:{}:{}: syntax error: {}", input, e.source_pos().line_number, e.source_pos().column_number, e);
                    }
                    return exits::PROGRAM_ERROR;
                }
            };

            let constants = Table::builtin_set();
            let bytes = match program.run(&constants, limit) {
                Ok(bs) => bs,
                Err(e) => {
                    eprintln!("{}: runtime error: {}", input, e);

                    if let Some(note) = e.note() {
                        eprintln!("{}: note: {}", input, note);
                    }

                    return exits::PROGRAM_ERROR;
                }
            };

            let bytes_written_attempt = match output {
                Output::Stdout => {
                    let stdout = io::stdout();
                    let mut stdout = stdout.lock();

                    match format {
                        Format::Raw               => stdout.write(&bytes),
                        Format::Formatted(style)  => style.format(bytes.into_iter(), &mut stdout),
                    }
                },
                Output::File(path) => {
                    let mut file = match File::create(&path) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("{}: error creating file: {}", path.display(), e);
                            return exits::IO_ERROR;
                        }
                    };

                    match format {
                        Format::Raw               => file.write(&bytes),
                        Format::Formatted(style)  => style.format(bytes.into_iter(), &mut file),
                    }
                },
            };

            if let Err(e) = bytes_written_attempt {
                eprintln!("{}: error writing output: {}", input, e);
                return exits::IO_ERROR;
            }

            if let Err(e) = verification.verify(bytes_written_attempt.unwrap()) {
                eprintln!("{}: validation failed: {}", input, e);
                return exits::LENGTH_VERIFICATION_ERROR;
            }
        }

        RunningMode::SyntaxCheck(input) => {
            let source = match input.read() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}: {}", input, e);
                    return exits::IO_ERROR;
                }
            };

            match Program::read(&source) {
                Ok(_) => {
                    println!("{}: Syntax OK", input);
                },
                Err(es) => {
                    for e in es {
                        println!("{}:{}: syntax error: {}", input, e.source_pos().line_number, e);
                    }
                    return exits::PROGRAM_ERROR;
                }
            };
        }

        RunningMode::ListConstants { filter } => {
            let constants = Table::builtin_set();
            let stdout = io::stdout();
            let mut out_handle = stdout.lock();
            let mut found_any = false;

            for (name, value) in constants.all() {
                if let Some(filter) = &filter {
                    if ! name.contains(filter) {
                        continue;
                    }
                }

                match value {
                    Constant::Eight(v) => {
                        writeln!(out_handle, "{} => {} (8-bit)", name, v)
                    }
                    Constant::Sixteen(v) => {
                        writeln!(out_handle, "{} => {} (16-bit)", name, v)
                    }
                }.unwrap();

                found_any = true;
            }

            if ! found_any {
                eprintln!("hexit: No constants found containing {:?}", filter.unwrap());
                return exits::NO_CONSTANTS_FOUND;
            }
        }
    }

    exits::SUCCESS
}


mod exits {

    /// Exit code for when everything turns out OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when input could not be read, or output could not be written.
    pub const IO_ERROR: i32 = 1;

    /// Exit code for when the user’s Hexit program contained an error.
    pub const PROGRAM_ERROR: i32 = 2;

    /// Exit code for when the command-line options are invalid.
    pub const OPTIONS_ERROR: i32 = 3;

    /// Exit code for when length verification fails.
    pub const LENGTH_VERIFICATION_ERROR: i32 = 4;

    /// Exit code for when the user search for constants and none were found.
    pub const NO_CONSTANTS_FOUND: i32 = 4;
}
