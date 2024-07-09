#[cfg(feature = "rplotly")]
pub mod rplotly;
#[cfg(features = "rplotters")]
pub mod rplotters;

use std::{fmt::{self, Display}, fs, ops::{Add, Div}};

use hdf5::{Dataset, H5Type};
use num_traits::{ real::Real, Bounded, FromPrimitive};
use plotly::common::{Anchor, Font, Title};
use ndarray::{Array2, Axis};
use plotly::{layout::Annotation, Layout};
use ::plotly::{common::Mode, Plot, Scatter};
use serde::Serialize;

use crate::{
    config::{Config, PlotArgs},
    my_hdf5::util::{print_group_info, NativePrimitiveType},
};

pub fn handle_plot_cmd(plot_args: &PlotArgs, cfg: &Config) -> anyhow::Result<()> {
    // Open the HDF5 file
    log::debug!("opening: {:?}", plot_args.src_hdf5.as_path());
    let file = hdf5::File::open(plot_args.src_hdf5.as_path())?;

        let dataset = file.dataset(&plot_args.dataset_name)?;
        let dtype = dataset.dtype()?;

        match NativePrimitiveType::from_dtype(&dtype)  {
            NativePrimitiveType::Integer32b => {
                read_and_process_dataset_nonfloats::<u32>(&dataset, plot_args.axis)?;
            },
            NativePrimitiveType::Integer64b => {
                read_and_process_dataset_nonfloats::<u64>(&dataset, plot_args.axis)?;
            },
            NativePrimitiveType::UnsignedInteger32b => {
                read_and_process_dataset_nonfloats::<i32>(&dataset, plot_args.axis)?;
            },
            NativePrimitiveType::UnsignedInteger64b => {
                read_and_process_dataset_nonfloats::<i64>(&dataset, plot_args.axis)?;
            },
            NativePrimitiveType::Float32b => {
                read_and_process_dataset_floats::<f32>(&dataset, plot_args.axis)?;
            },
            NativePrimitiveType::Float64b => {
                read_and_process_dataset_floats::<f32>(&dataset, plot_args.axis)?;
            },
            NativePrimitiveType::Pointer(_) => {
                read_and_process_dataset_nonfloats::<usize>(&dataset, plot_args.axis)?;
            },
        }

    Ok(())
}

fn read_and_process_dataset_floats<T>(dataset: &Dataset, axis: usize) -> anyhow::Result<()>
where T: H5Type + Serialize + num_traits::identities::Zero + FromPrimitive + Clone + Copy + std::ops::Div + Bounded + PartialOrd + Display + Real, for<'a> &'a T: Add<T, Output = T>, <T as Div>::Output: fmt::Display
{
    // Read the dataset into a 2D ndarray
    let data_2dim: Array2<T> = dataset.read_2d()?;

    // Fold the data along the specified axis
    let data_2dim_folded = data_2dim.fold_axis(Axis(axis), T::zero(), |acc, &x| acc + x);

    // Calculate statistics
    let sum = data_2dim_folded.sum();
    let min = data_2dim_folded.fold(<T as Bounded>::max_value(), |a, &b| a.min(b));
    let max = data_2dim_folded.fold(<T as Bounded>::min_value(), |a, &b| a.max(b));
    let avg  = sum / T::from_usize(data_2dim_folded.len()).unwrap();
    println!("Sum={sum}, Avg={avg}, Min={min}, Max={max}");

    // Plot the data
    plot(data_2dim_folded.into_iter().collect())?;

    Ok(())
}

fn read_and_process_dataset_nonfloats<T>(dataset: &Dataset, axis: usize) -> anyhow::Result<()>
where T: H5Type + Serialize + num_traits::identities::Zero + FromPrimitive + Clone + Copy + std::ops::Div + Bounded + PartialOrd + Display + Ord, for<'a> &'a T: Add<T, Output = T>, <T as Div>::Output: fmt::Display
{
    // Read the dataset into a 2D ndarray
    let data_2dim: Array2<T> = dataset.read_2d()?;

    // Fold the data along the specified axis
    let data_2dim_folded = data_2dim.fold_axis(Axis(axis), T::zero(), |acc, &x| acc + x);

    // Calculate statistics
    let sum = data_2dim_folded.sum();
    let min = data_2dim_folded.fold(<T as Bounded>::max_value(), |a, &b| a.min(b));
    let max = data_2dim_folded.fold(<T as Bounded>::min_value(), |a, &b| a.max(b));
    let avg  = sum / T::from_usize(data_2dim_folded.len()).unwrap();
    println!("Sum={sum}, Avg={avg}, Min={min}, Max={max}");

    // Plot the data
    plot(data_2dim_folded.into_iter().collect())?;

    Ok(())
}


pub fn plot<T>(data: Vec<T>) -> anyhow::Result<()>
where T: Serialize + Clone + 'static
{
    // Plot the data
    #[cfg(features = "rplotters")]
    rplotters::plot_data(dxxx.as_slice().unwrap(), dxxx.len(), 1, min, max)?;

    #[cfg(feature = "rplotly")]
    rplotly::plotly(data);

    Ok(())
}