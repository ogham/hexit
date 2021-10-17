//! This build script gets run during every build. Its purpose is to put
//! together the files used for the `--help` and `--version`, which need to
//! come in both coloured and non-coloured variants. The main usage text is
//! contained in `src/usage.txt`; to make it easier to edit, backslashes (\)
//! are used instead of the beginning of ANSI escape codes.
//!
//! The version string is quite complex: we want to show the version,
//! current Git hash, and compilation date when building *debug*
//! versions, but just the version for *release* versions.
//!
//! This script generates the string from the environment variables
//! that Cargo adds (http://doc.crates.io/environment-variables.html)
//! and runs `git` to get the SHA1 hash. It then writes the strings
//! into files, which we can include during compilation.

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::SystemTime;


/// The build script entry point.
fn main() -> io::Result<()> {
    let usage   = include_str!("src/usage.txt");
    let tagline = "hexit \\1;35m●\\0m hex bytes emitter";
    let url     = "https://hexit.binarystar.systems/";

    let ver =
        if is_debug_build() {
            format!("{}\nv{} \\1;31m(pre-release debug build!)\\0m\n\\1;4;34m{}\\0m", tagline, cargo_version(), url)
        }
        else if is_development_version() {
            format!("{}\nv{} [{}] built on {} \\1;31m(pre-release!)\\0m\n\\1;4;34m{}\\0m", tagline, cargo_version(), git_hash(), build_date(), url)
        }
        else {
            format!("{}\nv{}\n\\1;4;34m{}\\0m", tagline, cargo_version(), url)
        };

    // We need to create these files in the Cargo output directory.
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Pretty version text
    let mut f = File::create(&out.join("version.pretty.txt"))?;
    writeln!(f, "{}", convert_codes(&ver))?;

    // Bland version text
    let mut f = File::create(&out.join("version.bland.txt"))?;
    writeln!(f, "{}", strip_codes(&ver))?;

    // Pretty usage text
    let mut f = File::create(&out.join("usage.pretty.txt"))?;
    writeln!(f, "{}", convert_codes(tagline))?;
    writeln!(f)?;
    write!(f, "{}", convert_codes(usage))?;

    // Bland usage text
    let mut f = File::create(&out.join("usage.bland.txt"))?;
    writeln!(f, "{}", strip_codes(tagline))?;
    writeln!(f)?;
    write!(f, "{}", strip_codes(usage))?;

    Ok(())
}

/// Converts the escape codes in ‘usage.txt’ to ANSI escape codes.
fn convert_codes(input: &str) -> String {
    input.replace("\\", "\x1B[")
}

/// Removes escape codes from ‘usage.txt’.
fn strip_codes(input: &str) -> String {
    input.replace("\\0m", "")
         .replace("\\1m", "")
         .replace("\\4m", "")
         .replace("\\31m", "")
         .replace("\\32m", "")
         .replace("\\33m", "")
         .replace("\\1;31m", "")
         .replace("\\1;32m", "")
         .replace("\\1;33m", "")
         .replace("\\1;34m", "")
         .replace("\\1;35m", "")
         .replace("\\1;36m", "")
         .replace("\\1;4;34", "")
}

/// Retrieve the project’s current Git hash, as a string.
fn git_hash() -> String {
    use std::process::Command;

    String::from_utf8_lossy(
        &Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output().unwrap()
            .stdout).trim().to_string()
}

/// Whether we should show pre-release info in the version string.
///
/// Both weekly releases and actual releases are --release releases,
/// but actual releases will have a proper version number.
fn is_development_version() -> bool {
    cargo_version().ends_with("-pre") || env::var("PROFILE").unwrap() == "debug"
}

/// Whether we are building in debug mode.
fn is_debug_build() -> bool {
    env::var("PROFILE").unwrap() == "debug"
}

/// Retrieves the [package] version in Cargo.toml as a string.
fn cargo_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap()
}

/// Formats the current date as a year-month-day string.
fn build_date() -> String {
    let now = SystemTime::now();
    let mut timestamp = humantime::format_rfc3339(now).to_string();
    timestamp.truncate(10);  // YYYY-MM-DD is 10 bytes
    timestamp
}
