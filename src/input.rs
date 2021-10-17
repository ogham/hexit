//! The input sources that a Hexit program can be read from.

use std::fs::File;
use std::fmt;
use std::io::{self, Read};
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
    /// as a `String`, or an I/O error if something goes wrong reading it.
    pub fn read(&self) -> io::Result<String> {
        match self {
            Self::Expression(input_string) => {
                info!("Reading from string");
                Ok(input_string.clone())
            }

            Self::Stdin => {
                info!("Reading from standard input");
                let stdin = io::stdin();
                let mut handle = stdin.lock();

                let mut contents = String::new();
                handle.read_to_string(&mut contents)?;
                debug!("Successfully read stdin");
                Ok(contents)
            }

            Self::File(path) => {
                info!("Reading from file {:?}", path);
                let mut file = File::open(path)?;

                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                debug!("Successfully read file contents");
                Ok(contents)
            }
        }
    }
}
