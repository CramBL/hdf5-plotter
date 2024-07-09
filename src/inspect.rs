use crate::{
    config::{Config, InspectArgs},
    my_hdf5::util::print_group_info,
};

pub fn handle_inspect(args: &InspectArgs, cfg: &Config) -> anyhow::Result<()> {
    log::trace!("{args:?}");

    log::debug!("opening: {:?}", args.src_hdf5.as_path());
    let file = hdf5::File::open(args.src_hdf5.as_path())?;

    print_group_info(&file)?;

    Ok(())
}
