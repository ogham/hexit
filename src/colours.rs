use ansi_term::Style;
use ansi_term::Color::*;


/// The colours used in Hexit’s terminal UI, for reporting errors in programs.
#[derive(Default, Copy, Clone)]
pub struct Colours {
    pub error: Style,
    pub warning: Style,
}

impl Colours {
    pub fn pretty() -> Self {
        Colours {
            error:   Red.bold(),
            warning: Yellow.bold(),
        }
    }

    pub fn plain() -> Self {
        Self::default()
    }
}
