use std::io;

use termcolor::{ColorChoice, WriteColor};

pub trait WriteStyled {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()>;
}

pub enum StylePolicy {
    Auto,
    Off,
    On,
}

impl StylePolicy {
    pub fn color_choice(&self) -> ColorChoice {
        use StylePolicy::*;
        match self {
            Auto if atty::is(atty::Stream::Stdout) => ColorChoice::Auto,
            On => ColorChoice::Always,
            _ => ColorChoice::Never,
        }
    }
}
