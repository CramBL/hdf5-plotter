#[cfg(feature = "rplotly")]
pub mod plotly;
#[cfg(features = "rplotters")]
pub mod rplotters;

use crate::{
    config::{Config, PlotArgs},
    my_hdf5::util::print_group_info,
};

pub fn handle_plot_cmd(plot_args: &PlotArgs, cfg: &Config) -> anyhow::Result<()> {
    //

    // Open the HDF5 file
    log::debug!("opening: {:?}", plot_args.src_hdf5.as_path());
    let file = hdf5::File::open(plot_args.src_hdf5.as_path())?;

    print_group_info(&file)?;

    if let Some(ref dname) = plot_args.dataset_name {
        let dataset = file.dataset(dname)?;
        log::info!("{dataset:?}");
        // // Read the dataset as Vec<i32> or Vec<f64> depending on the content type
        let data_2dim = dataset.read_2d::<f32>().unwrap();
        let data_2dim_folded = data_2dim.fold_axis(ndarray::Axis(1), 0_f32, |_c, d| *d);

        let sum = data_2dim_folded.sum();
        let (mut min, mut max) = (f32::MAX, f32::MIN);
        for d in &data_2dim_folded {
            if *d < min {
                min = *d;
            } else if *d > max {
                max = *d;
            }
        }
        let avg = sum / (data_2dim_folded.len() as f32);
        println!("Sum={sum}, Avg={avg}, Min={min}, Max={max}");

        // Plot the data
        #[cfg(features = "rplotters")]
        rplotters::plot_data(dxxx.as_slice().unwrap(), dxxx.len(), 1, min, max)?;

        #[cfg(feature = "rplotly")]
        plotly::plotly(data_2dim_folded.as_slice().unwrap());
    }

    Ok(())
}
