use hdf5_test::{
    config::{Command, Config},
    inspect::handle_inspect,
    plot::handle_plot_cmd,
};

fn main() -> anyhow::Result<()> {
    let cfg = Config::init()?;
    log::trace!("{cfg:?}");

    match cfg.command {
        Some(ref c) => match c {
            Command::Plot(ref args) => handle_plot_cmd(args, &cfg)?,
            Command::TestSwMr => hdf5_test::swmr::multiple_reader()?,
            Command::Inspect(ref args) => handle_inspect(args, &cfg)?,
        },
        None => log::trace!("No subcommand"),
    }

    Ok(())
}
