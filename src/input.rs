//! The input sources that a Hexit program can be read from.

use std::fs::File;
use std::fmt;
use std::io::{self, Read, BufRead, BufReader};
use std::path::PathBuf;

use log::*;


/// Where the input program comes from.
#[derive(PartialEq, Debug)]
pub enum Input {

    /// The program has been read from a command-line argument.
    Expression(String),

    /// The program should be read from standard input.
    Stdin,

    /// The program should be read from the file at the given path.
    File(PathBuf),
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expression(_)  => write!(f, "<expression>"),
            Self::Stdin          => write!(f, "<stdin>"),
            Self::File(path)     => write!(f, "{}", path.display()),
        }
    }
}

impl Input {

    /// Reads the complete Hexit program from the input source, returning it
    /// as a series of strings, or an I/O error if something goes wrong.
    pub fn read(&self) -> io::Result<Vec<String>> {
        match self {
            Self::Expression(input_string) => {
                info!("Reading from string");

                let lines = input_string.lines().map(|line| line.to_owned()).collect();
                Ok(lines)
            }

            Self::Stdin => {
                info!("Reading from standard input");
                let stdin = io::stdin();
                let handle = stdin.lock();

                let lines = read_all_lines(handle)?;
                debug!("Successfully read stdin");
                Ok(lines)
            }

            Self::File(path) => {
                info!("Reading from file → {:?}", path);
                let handle = File::open(path)?;

                let lines = read_all_lines(handle)?;
                debug!("Successfully read file contents");
                Ok(lines)
            }
        }
    }
}


/// Reads all the lines from the given `Read`-capable handle, returning them
/// as a vector and stopping as soon as an I/O error occurs.
fn read_all_lines(handle: impl Read) -> io::Result<Vec<String>> {
    let reader = BufReader::new(handle);
    let lines = reader.lines().collect::<io::Result<_>>()?;
    Ok(lines)
}
