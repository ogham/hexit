//! Command-line option parsing and the representation of the options.

use std::ffi::OsStr;
use std::fmt;
use std::num::ParseIntError;
use std::path::PathBuf;

use log::*;

use crate::console::UseColours;
use crate::input::Input;
use crate::style::{Style, LetterCase};
use crate::verify::Verification;


/// What Hexit should do after it’s been successfully invoked.
#[derive(PartialEq, Debug)]
pub enum RunningMode {

    /// Hexit should execute (hexecute) using the given options, interpreting
    /// and running a program and writing its output somewhere.
    Run(Options),

    /// Hexit should check whether the given input is syntactically correct.
    SyntaxCheck(Input),

    /// Hexit should list the available constants.
    ListConstants {

        /// If given, only list constants that contain this substring.
        filter: Option<String>,
    },
}

/// The options necessary to run Hexit.
#[derive(PartialEq, Debug)]
pub struct Options {

    /// Where the input program comes from.
    pub input: Input,

    /// Where the output gets written to.
    pub output: Output,

    /// How the output bytes should be formatted.
    pub format: Format,

    /// How the length of the output should be verified, if at all.
    pub verification: Verification,
}

/// Where the output gets written to.
#[derive(PartialEq, Debug)]
pub enum Output {

    /// Output should be written to standard output.
    Stdout,

    /// Output should be written to a new file at the given path.
    File(PathBuf),
}

/// How the output bytes should be formatted.
#[derive(PartialEq, Debug)]
pub enum Format {

    /// Perform no formatting, producing a stream of bytes in the range 0–255.
    Raw,

    /// Format the stream of bytes using the given options.
    Formatted(Style),
}

impl RunningMode {

    /// Parses and interprets a set of options from the user’s command-line
    /// arguments.
    ///
    /// This returns an `Ok` set of options if successful and running
    /// normally, a `Help` or `Version` variant if one of those options is
    /// specified, or an error variant if there’s an invalid option or
    /// inconsistency within the options after they were parsed.
    #[allow(unused_results)]
    pub fn getopts<C>(args: C) -> OptionsResult
    where C: IntoIterator,
          C::Item: AsRef<OsStr>,
    {
        let mut opts = getopts::Options::new();
        opts.optflag("?", "help",            "show list of command-line options");
        opts.optflag("v", "version",         "show version of hexit");
        opts.optopt ("",  "color",           "when to use terminal colors",                                "WHEN");
        opts.optopt ("",  "colour",          "when to use terminal colours",                               "WHEN");
        opts.optflag("",  "list-constants",  "print the list of available constants");

        opts.optflag("c", "check-syntax",    "instead of running, check that syntax is valid");
        opts.optopt ("e", "expression",      "evaluate this expression instead of reading from a file",    "EXPR");
        opts.optopt ("o", "output",          "output to this file instead of printing the results",        "PATH");

        opts.optflag("r", "raw",             "print raw bytes without formatting");
        opts.optopt ("P", "prefix",          "string to print before each pair of hex characters",         "STR");
        opts.optopt ("S", "suffix",          "string to print after each pair of hex characters",          "STR");
        opts.optopt ("s", "separator",       "string to print between successive pairs of hex characters", "STR");
        opts.optflag("l", "lowercase",       "print hex characters in lowercase");

        opts.optopt ("",  "verify-length",   "ensure that the output has this exact length",               "NUM");
        opts.optopt ("",  "verify-boundary", "ensure that the output has a length with a given multiple",  "NUM");

        let matches = match opts.parse(args) {
            Ok(m)  => m,
            Err(e) => return OptionsResult::InvalidOptionsFormat(e),
        };

        if matches.opt_present("version") {
            OptionsResult::Version(UseColours::deduce(&matches))
        }
        else if let Some(reason) = Self::check_help(&matches) {
            OptionsResult::Help(reason, UseColours::deduce(&matches))
        }
        else {
            match Self::deduce(&matches) {
                Ok(opts) => OptionsResult::Ok(opts),
                Err(e)   => OptionsResult::InvalidOptions(e),
            }
        }
    }

    /// Check whether the given set of matches require the help text to be
    /// printed; if so, returns the reason, and if not, returns nothing.
    fn check_help(matches: &getopts::Matches) -> Option<HelpReason> {
        if matches.opt_present("help") {
            Some(HelpReason::Flag)
        }
        else if ! matches.opt_present("expression") && ! matches.opt_present("list-constants") && matches.free.is_empty() {
            Some(HelpReason::NoArguments)
        }
        else {
            None
        }
    }

    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        if matches.opt_present("list-constants") {
            let filter = match matches.free.len() {
                0 => None,
                1 => Some(matches.free[0].clone()),
                _ => return Err(OptionsError::TooManyConstantSearches),
            };
            Ok(Self::ListConstants { filter })
        }
        else if matches.opt_present("check-syntax") {
            let input = Input::deduce(matches)?;
            Ok(Self::SyntaxCheck(input))
        }
        else {
            let input = Input::deduce(matches)?;
            let output = Output::deduce(matches);
            let format = Format::deduce(matches);
            let verification = Verification::deduce(matches)?;
            Ok(Self::Run(Options { input, output, format, verification }))
        }
    }
}


impl Input {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        if let Some(expr_string) = matches.opt_str("expression") {
            return Ok(Input::Expression(expr_string));
        }

        match matches.free.len() {
            1 => {
                let path = &matches.free[0];

                if path == "-" { Ok(Input::Stdin) }
                          else { Ok(Input::File(PathBuf::from(path))) }
            },

            0 => Err(OptionsError::NoInputFiles),
            _ => Err(OptionsError::TooManyInputFiles),
        }
    }
}


impl Output {
    fn deduce(matches: &getopts::Matches) -> Self {
       match matches.opt_str("output") {
           Some(path)  => Output::File(PathBuf::from(path)),
           None        => Output::Stdout,
       }
    }
}


impl Format {
    fn deduce(matches: &getopts::Matches) -> Self {
       match matches.opt_present("raw") {
           true  => Format::Raw,
           false => Format::Formatted(Style::deduce(matches)),
       }
    }
}


impl Style {
    fn deduce(matches: &getopts::Matches) -> Self {
        let prefix    = matches.opt_str("prefix");
        let suffix    = matches.opt_str("suffix");
        let separator = matches.opt_str("separator");

        let case = LetterCase::deduce(matches);
        Style { prefix, suffix, separator, case }
    }
}


impl LetterCase {
    fn deduce(matches: &getopts::Matches) -> Self {
        match matches.opt_present("lowercase") {
            true  => LetterCase::Lower,
            false => LetterCase::Upper,
        }
    }
}


impl Verification {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        let length   = matches.opt_str("verify-length");
        let boundary = matches.opt_str("verify-boundary");

        match (length, boundary) {
            (None,    None   )  => Ok(Verification::AnythingGoes),
            (Some(l), None   )  => Ok(Verification::ExactLength(l.parse()?)),
            (None,    Some(b))  => Ok(Verification::Multiple(b.parse()?)),
            (Some(_), Some(_))  => Err(OptionsError::TooMuchVerification),
        }
    }
}


impl UseColours {
    fn deduce(matches: &getopts::Matches) -> Self {
        match matches.opt_str("color").or_else(|| matches.opt_str("colour")).unwrap_or_default().as_str() {
            "automatic" | "auto" | ""  => Self::Automatic,
            "always"    | "yes"        => Self::Always,
            "never"     | "no"         => Self::Never,
            otherwise => {
                warn!("Unknown colour setting {:?}", otherwise);
                Self::Automatic
            },
        }
    }
}


/// The result of the `Options::getopts` function.
#[derive(PartialEq, Debug)]
pub enum OptionsResult {

    /// The options were parsed successfully.
    Ok(RunningMode),

    /// There was an error (from `getopts`) parsing the arguments.
    InvalidOptionsFormat(getopts::Fail),

    /// There was an error with the combination of options the user selected.
    InvalidOptions(OptionsError),

    /// Can’t run any checks because there’s help to display!
    Help(HelpReason, UseColours),

    /// One of the arguments was `--version`, to display the version number.
    Version(UseColours),
}

/// The reason that help is being displayed. If it’s for the `--help` flag,
/// then we shouldn’t return an error exit status.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum HelpReason {

    /// Help was requested with the `--help` flag.
    Flag,

    /// There were no files to run, so display help instead.
    NoArguments,
}

/// Something wrong with the combination of options the user has picked.
#[derive(PartialEq, Debug)]
pub enum OptionsError {

    /// The user didn’t provide any input files, not even `-`.
    NoInputFiles,

    /// The user provided too many input files on the command-line.
    TooManyInputFiles,

    /// The user provided both verification options.
    TooMuchVerification,

    /// The user provided too many constant substrings to search for.
    TooManyConstantSearches,

    /// The user provided a verification option with an unparseable number.
    InvalidVerificationNumber(ParseIntError),
}

impl From<ParseIntError> for OptionsError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidVerificationNumber(error)
    }
}

impl fmt::Display for OptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputFiles                    => write!(f, "No input files"),
            Self::TooManyInputFiles               => write!(f, "Too many input files"),
            Self::TooMuchVerification             => write!(f, "Too much verification"),
            Self::TooManyConstantSearches         => write!(f, "Too many constant searches"),
            Self::InvalidVerificationNumber(pie)  => write!(f, "Invalid verification: {}", pie),
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    // help/version tests

    #[test]
    fn no_args() {
        let nothing: &[&OsStr] = &[];
        assert_eq!(RunningMode::getopts(nothing),
                   OptionsResult::Help(HelpReason::NoArguments, UseColours::Automatic));
    }

    #[test]
    fn help() {
        assert_eq!(RunningMode::getopts(&[ "--help" ]),
                   OptionsResult::Help(HelpReason::Flag, UseColours::Automatic));
    }

    #[test]
    fn help_no_colour() {
        assert_eq!(RunningMode::getopts(&[ "--help", "--colour=never" ]),
                   OptionsResult::Help(HelpReason::Flag, UseColours::Never));
    }

    #[test]
    fn version() {
        assert_eq!(RunningMode::getopts(&[ "--version" ]),
                   OptionsResult::Version(UseColours::Automatic));
    }

    #[test]
    fn version_yes_color() {
        assert_eq!(RunningMode::getopts(&[ "--version", "--color", "always" ]),
                   OptionsResult::Version(UseColours::Always));
    }

    // list constants tests

    #[test]
    fn list_constants() {
        assert_eq!(RunningMode::getopts(&[ "--list-constants" ]),
                   OptionsResult::Ok(RunningMode::ListConstants { filter: None }));
    }

    // check syntax tests

    #[test]
    fn check_syntax_input_file() {
        assert_eq!(RunningMode::getopts(&[ "--check-syntax", "star.hexit" ]),
                   OptionsResult::Ok(RunningMode::SyntaxCheck(Input::File(PathBuf::from("star.hexit")))));
    }

    #[test]
    fn check_syntax_expression() {
        assert_eq!(RunningMode::getopts(&[ "--check-syntax", "-e", "101" ]),
                   OptionsResult::Ok(RunningMode::SyntaxCheck(Input::Expression(String::from("101")))));
    }

    #[test]
    fn check_syntax_stdin() {
        assert_eq!(RunningMode::getopts(&[ "--check-syntax", "-" ]),
                   OptionsResult::Ok(RunningMode::SyntaxCheck(Input::Stdin)));
    }

    // running tests

    #[test]
    fn run_input_file() {
        assert_eq!(RunningMode::getopts(&[ "starchild_numerology.hexit" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::File(PathBuf::from("starchild_numerology.hexit")),
                       ..default_args()
                   })));
    }

    #[test]
    fn run_expression() {
        assert_eq!(RunningMode::getopts(&[ "-e", "be32" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::Expression(String::from("be32")),
                       ..default_args()
                   })));
    }

    #[test]
    fn run_stdin() {
        assert_eq!(RunningMode::getopts(&[ "-" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::Stdin,
                       ..default_args()
                   })));
    }

    #[test]
    fn run_with_output() {
        assert_eq!(RunningMode::getopts(&[ "star.hexit", "-o", "wibble" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::File(PathBuf::from("star.hexit")),
                       output: Output::File(PathBuf::from("wibble")),
                       ..default_args()
                   })));
    }

    #[test]
    fn run_with_formatting_1() {
        assert_eq!(RunningMode::getopts(&[ "-e", "star.hexit", "--prefix=0x", "--separator= " ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::Expression(String::from("star.hexit")),
                       format: Format::Formatted(Style {
                           prefix: Some("0x".into()),
                           separator: Some(" ".into()),
                           ..Style::default()
                       }),
                       ..default_args()
                   })));
    }

    #[test]
    fn run_with_formatting_2() {
        assert_eq!(RunningMode::getopts(&[ "star.hexit", "--suffix=;", "--lowercase" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::File(PathBuf::from("star.hexit")),
                       format: Format::Formatted(Style {
                           suffix: Some(";".into()),
                           case: LetterCase::Lower,
                           ..Style::default()
                       }),
                       ..default_args()
                   })));
    }

    #[test]
    fn run_with_formatting_3() {
        assert_eq!(RunningMode::getopts(&[ "star.hexit", "--raw" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::File(PathBuf::from("star.hexit")),
                       format: Format::Raw,
                       ..default_args()
                   })));
    }

    #[test]
    fn run_with_verification_length() {
        assert_eq!(RunningMode::getopts(&[ "starchild_numerology.hexit", "--verify-length", "32" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::File(PathBuf::from("starchild_numerology.hexit")),
                       verification: Verification::ExactLength(32),
                       ..default_args()
                   })));
    }

    #[test]
    fn run_with_verification_multiple() {
        assert_eq!(RunningMode::getopts(&[ "starchild_numerology.hexit", "--verify-boundary", "8" ]),
                   OptionsResult::Ok(RunningMode::Run(Options {
                       input: Input::File(PathBuf::from("starchild_numerology.hexit")),
                       verification: Verification::Multiple(8),
                       ..default_args()
                   })));
    }

    // errors tests

    #[test]
    fn invalid_option() {
        assert_eq!(RunningMode::getopts(&[ "--crumbadu" ]),
                   OptionsResult::InvalidOptionsFormat(getopts::Fail::UnrecognizedOption("crumbadu".into())));
    }

    #[test]
    fn double_verification() {
        assert_eq!(RunningMode::getopts(&[ "--verify-length=1", "--verify-boundary=2", "star.hexit" ]),
                   OptionsResult::InvalidOptions(OptionsError::TooMuchVerification));
    }

    #[test]
    fn double_constance() {
        assert_eq!(RunningMode::getopts(&[ "--list-constants", "A", "B" ]),
                   OptionsResult::InvalidOptions(OptionsError::TooManyConstantSearches));
    }

    #[test]
    fn double_input() {
        assert_eq!(RunningMode::getopts(&[ "a", "b", ]),
                   OptionsResult::InvalidOptions(OptionsError::TooManyInputFiles));
    }

    fn default_args() -> Options {
        Options {
            input: Input::Stdin,
            output: Output::Stdout,
            format: Format::Formatted(Style::default()),
            verification: Verification::AnythingGoes,
        }
    }
}
