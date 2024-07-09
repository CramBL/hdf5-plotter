use std::fmt;
use std::io::Write;
use termcolor::{ColorSpec, StandardStream, WriteColor};

pub fn print_color<T>(
    out: &mut StandardStream,
    color: termcolor::Color,
    txt: T,
) -> anyhow::Result<()>
where
    T: fmt::Display,
{
    out.set_color(ColorSpec::new().set_fg(Some(color)))?;
    writeln!(out, "{txt}")?;
    out.reset()?;
    Ok(())
}

pub fn print_colored_quoted<T, U>(
    out: &mut StandardStream,
    color: termcolor::Color,
    colored_txt: T,
    txt: U,
) -> anyhow::Result<()>
where
    T: fmt::Display,
    U: fmt::Display,
{
    out.set_color(ColorSpec::new().set_fg(Some(color)))?;
    write!(out, "{colored_txt}'")?;
    out.reset()?;
    write!(out, "{txt}")?;
    out.set_color(ColorSpec::new().set_fg(Some(color)))?;
    writeln!(out, "'")?;
    out.reset()?;
    Ok(())
}
