use termcolor::{Color, StandardStream};
use crate::{
    config::{Config, InspectArgs}, my_hdf5::inspect::print_group_info, util::print_colored_quoted
};

pub fn handle_inspect(args: &InspectArgs, cfg: &Config) -> anyhow::Result<()> {
    log::trace!("{args:?}");

    log::debug!("opening: {:?}", args.src_hdf5.as_path());
    let file = hdf5::File::open(args.src_hdf5.as_path())?;
    
    let mut stdout = StandardStream::stdout(cfg.color_when());
    print_colored_quoted(&mut stdout, Color::Green, "Inspecting ", format!("{file:?}"))?;

    print_group_info(&file, args.preview_samples, &mut stdout)?;

    Ok(())
}
