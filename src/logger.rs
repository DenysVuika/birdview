//! Logging utils

use env_logger::{
    fmt::{Color, Formatter, Style, StyledValue},
    Builder, Env, Target,
};
use log::{Level, Record};
use std::io::Write;

fn colored_level(style: &mut Style, level: Level) -> StyledValue<&'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO"),
        Level::Warn => style.set_color(Color::Yellow).value("WARN"),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}

fn custom_formatter(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let mut style = buf.style();
    let level = colored_level(&mut style, record.level());

    let mut style = buf.style();
    if record.level() == Level::Error {
        style.set_color(Color::Red); //.set_bold(true);
    }

    writeln!(buf, "[{}] - {}", level, style.value(record.args()))
}

/// Sets up the default logger
pub fn init_logger() {
    Builder::from_env(Env::default().filter_or("LOG", "info"))
        .target(Target::Stdout)
        .format(custom_formatter)
        .init();
}
