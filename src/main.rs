use hdf5_test::{config::Config, plot::handle_plot_cmd};

fn main() -> anyhow::Result<()> {
    let cfg = Config::init()?;
    log::trace!("{cfg:?}");

    match cfg.command {
        Some(ref c) => match c {
            hdf5_test::config::Command::Plot(ref args) => handle_plot_cmd(args, &cfg)?,
            hdf5_test::config::Command::TestSwMr => hdf5_test::swmr::multiple_reader()?,
        },
        None => log::trace!("No subcommand"),
    }

    Ok(())
}
