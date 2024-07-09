#[cfg(feature = "rplotly")]
pub mod rplotly;
#[cfg(features = "rplotters")]
pub mod rplotters;

use std::{fmt::{self, Display}, fs, ops::{Add, Div, Mul, Sub}};

use anyhow::bail;
use hdf5::{Dataset, H5Type};
use num_traits::{ real::Real, Bounded, FromPrimitive, ToPrimitive, Zero};
use ndarray::{Array1, Array2, ArrayBase, Axis, Data, Ix1};
use serde::Serialize;

use crate::{
    config::{Config, PlotArgs},
    my_hdf5::util::NativePrimitiveType,
};

pub fn handle_plot_cmd(plot_args: &PlotArgs, cfg: &Config) -> anyhow::Result<()> {
    // Open the HDF5 file
    log::debug!("opening: {:?}", plot_args.src_hdf5.as_path());
    let file = hdf5::File::open(plot_args.src_hdf5.as_path())?;

        let dataset = file.dataset(&plot_args.dataset_name)?;
        let dtype = dataset.dtype()?;

        match NativePrimitiveType::from_dtype(&dtype)  {
            NativePrimitiveType::Integer32b => {
                read_and_process_dataset_nonfloats::<u32>(&dataset, plot_args.axis, plot_args.subsample)?;
            },
            NativePrimitiveType::Integer64b => {
                read_and_process_dataset_nonfloats::<u64>(&dataset, plot_args.axis, plot_args.subsample)?;
            },
            NativePrimitiveType::UnsignedInteger32b => {
                read_and_process_dataset_nonfloats::<i32>(&dataset, plot_args.axis, plot_args.subsample)?;
            },
            NativePrimitiveType::UnsignedInteger64b => {
                read_and_process_dataset_nonfloats::<i64>(&dataset, plot_args.axis, plot_args.subsample)?;
            },
            NativePrimitiveType::Pointer(_) => {
                read_and_process_dataset_nonfloats::<usize>(&dataset, plot_args.axis, plot_args.subsample)?;
            },
            NativePrimitiveType::Float32b => {
                read_and_process_dataset_floats::<f32>(&dataset, plot_args.axis, plot_args.subsample)?;
            },
            NativePrimitiveType::Float64b => {
                read_and_process_dataset_floats::<f32>(&dataset, plot_args.axis, plot_args.subsample)?;
            },
        }

    Ok(())
}

fn read_and_process_dataset_floats<T>(dataset: &Dataset, axis: usize, nth_sample: usize) -> anyhow::Result<()>
where T: H5Type + Serialize + num_traits::identities::Zero + FromPrimitive + Clone + Copy + std::ops::Div + Bounded + PartialOrd + Display + Real + for<'a> std::iter::Sum<&'a T>, for<'a> &'a T: Add<T, Output = T>, <T as Div>::Output: fmt::Display
{
    let ndims = dataset.ndim();

    match ndims {
        1 => {
            // Read the dataset into a 1D ndarray
            let data_1dim: Array1<T> = dataset.read_1d()?;
            process_and_plot_floats(data_1dim.view(), nth_sample)?;
        },
        2 => {
            // Read the dataset into a 2D ndarray
            let data_2dim: Array2<T> = dataset.read_2d()?;
            let data_2dim_folded = data_2dim.fold_axis(Axis(axis), T::zero(), |acc, &x| acc + x);
            process_and_plot_floats(data_2dim_folded.view(), nth_sample)?;
        },
        _ => {
            bail!("Unsupported dataset dimensionality: {ndims}");
        }
    }
    Ok(())
}

fn read_and_process_dataset_nonfloats<T>(dataset: &Dataset, axis: usize, nth_sample: usize) -> anyhow::Result<()>
where T: H5Type + Serialize + num_traits::identities::Zero + FromPrimitive + ToPrimitive + Clone + Copy + std::ops::Div + Bounded + PartialOrd + Display + Ord + for<'a> std::iter::Sum<&'a T> + std::ops::Sub<<T as std::ops::Div>::Output>, for<'a> &'a T: Add<T, Output = T>, <T as Div>::Output: fmt::Display + ToPrimitive

{
    let ndims = dataset.ndim();

    match ndims {
        1 => {
            // Read the dataset into a 1D ndarray
            let data_1dim: Array1<T> = dataset.read_1d()?;
            process_and_plot_nonfloats(data_1dim.view(), nth_sample)?;
        },
        2 => {
            // Read the dataset into a 2D ndarray
            let data_2dim: Array2<T> = dataset.read_2d()?;
            let data_2dim_folded = data_2dim.fold_axis(Axis(axis), T::zero(), |acc, &x| acc + x);
            process_and_plot_nonfloats(data_2dim_folded.view(), nth_sample)?;
        },
        _ => {
            bail!("Unsupported dataset dimensionality: {ndims}");
        }
    }

    Ok(())
}


fn process_and_plot_floats<T, D>(data: ArrayBase<D, Ix1>,  nth_sample: usize) -> anyhow::Result<()>
where
    T: Zero + FromPrimitive + Copy + Bounded + PartialOrd + Display + serde::Serialize + std::ops::Div + Real + 'static + for<'a> std::iter::Sum<&'a T>,
    D: Data<Elem = T>,
{
    let sampled_data: Vec<T> = data.iter().step_by(nth_sample).cloned().collect();
    // Calculate statistics
    let sum = sampled_data.iter().sum::<T>();
    let min = sampled_data.iter().fold(<T as Bounded>::max_value(), |a, &b| a.min(b));
    let max = sampled_data.iter().fold(<T as Bounded>::min_value(), |a, &b| a.max(b));
    let avg = sum / T::from_usize(sampled_data.len()).unwrap();
    let len = sampled_data.len();

    // Calculate spread and deviation
    let variance = sampled_data.iter().fold(T::zero(), |acc, &x| acc + (x - avg) * (x - avg)) / T::from_usize(len).unwrap();
    let std_dev = variance.sqrt();

    log::info!("Length={len}, Sum={sum}, Avg={avg}, Min={min}, Max={max}, σ={std_dev}, σ²={variance}");

    plot(sampled_data)?;
    Ok(())
}

fn process_and_plot_nonfloats<T, D>(data: ArrayBase<D, Ix1>,  nth_sample: usize) -> anyhow::Result<()>
where
    T: Zero + FromPrimitive + ToPrimitive + Copy + Bounded + PartialOrd + Display + serde::Serialize + std::ops::Div + Ord + 'static + for<'a> std::iter::Sum<&'a T> + std::ops::Sub<<T as std::ops::Div>::Output>,
    D: Data<Elem = T>, <T as Div>::Output: std::fmt::Display,
    <T as Div>::Output: ToPrimitive
{
    let sampled_data: Vec<T> = data.iter().step_by(nth_sample).cloned().collect();
    // Calculate statistics
    let sum = sampled_data.iter().sum::<T>();
    let min = sampled_data.iter().fold(<T as Bounded>::max_value(), |a, &b| a.min(b));
    let max = sampled_data.iter().fold(<T as Bounded>::min_value(), |a, &b| a.max(b));
    let avg = sum / T::from_usize(sampled_data.len()).unwrap();
    let len = sampled_data.len();

    // Calculate variance and standard deviation
    let variance = sampled_data.iter().fold(T::zero(), |acc, &x| {
        let diff = x.to_f64().unwrap() - avg.to_f64().unwrap();
        acc + T::from_f64(diff * diff).unwrap()
    }) / T::from_usize(len).unwrap();
    let std_dev = variance.to_f64().unwrap().sqrt();

    log::info!("Length={len}, Sum={sum}, Avg={avg}, Min={min}, Max={max}, σ={std_dev}, σ²={variance}");

    plot(sampled_data)?;
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