use hdf5::{Dataset, Datatype, H5Type};

use ndarray::{s, ArrayD, Axis, IntoNdProducer, NdProducer, Slice};
use ndarray::{IxDyn, SliceInfo, SliceInfoElem};
use termcolor::{Color, StandardStream};
use std::fmt;
use std::mem::size_of;
use std::ops::Index;

use crate::my_hdf5::inspect::print_preview_n_samples;
use crate::util::{print_color, print_colored_quoted};

pub fn print_dataset_info(dataset: &hdf5::Dataset, n_samples: usize, out: &mut StandardStream) -> anyhow::Result<()> {

    print_colored_quoted(out, Color::Magenta, "  - Dataset: ", format!("{}", dataset.name()))?;
    print_colored_quoted(out, Color::Yellow, "     - Layout: ", format!("{:?}", dataset.layout()))?;
    print_colored_quoted(out, Color::Yellow, "     - Attribute names: ", format!("{:?}", dataset.attr_names().unwrap_or_default()))?;
    print_colored_quoted(out, Color::Yellow, "     - Resizable: ", format!("{}", dataset.is_resizable()))?;
    if let Ok(dataset_access) = dataset.access_plist() {
        log::debug!("Dataset access: ");
        if let Ok(proplistclass) = dataset_access.class() {
            log::debug!("\t Property List Class: {proplistclass}");
        }
        log::debug!("\t Properties: {:?}", dataset_access.properties());
        log::debug!("\t {dataset_access:?}");
    }
    let dtype = dataset.dtype()?;
    let shape = dataset.shape();
    
    print_colored_quoted(out, Color::Yellow, "     - Data type: ", format!("{}",NativePrimitiveType::from_dtype(&dtype)))?;
    print_colored_quoted(out, Color::Yellow, "     - Shape: ", format!("{shape:?}"))?;
    log::trace!(" {dtype:?}, {}B", dtype.size());

    match NativePrimitiveType::from_dtype(&dtype) {
        NativePrimitiveType::Integer32b => {
            let data = dataset.read_dyn()?;
            print_preview_n_samples::<i32>(&data, n_samples, out)?;
        }
        NativePrimitiveType::Integer64b => {
            let data = dataset.read_dyn()?;
            print_preview_n_samples::<i64>(&data, n_samples, out)?;
        }
        NativePrimitiveType::UnsignedInteger32b => {
            let data = dataset.read_dyn()?;
            print_preview_n_samples::<u32>(&data, n_samples, out)?;
        }
        NativePrimitiveType::UnsignedInteger64b => {
            let data = dataset.read_dyn()?;
            print_preview_n_samples::<u64>(&data, n_samples, out)?;
        }
        NativePrimitiveType::Float32b => {
            let data = dataset.read_dyn()?;
            print_preview_n_samples::<f32>(&data, n_samples, out)?;
        }
        NativePrimitiveType::Float64b => {
            let data = dataset.read_dyn()?;
            print_preview_n_samples::<f64>(&data, n_samples, out)?;
        }
        NativePrimitiveType::Pointer(_) => {
            let data = dataset.read_dyn()?;
            print_preview_n_samples::<usize>(&data, n_samples, out)?;
        }
    }

    Ok(())
}

pub fn print_group_info(group: &hdf5::Group, n_samples: usize, out: &mut StandardStream) -> anyhow::Result<()> {
    let gname = group.name();
    print_colored_quoted(out, Color::Cyan, "Group:", format!("{gname}"))?;

    if let Ok(attr) = group.attr_names() {
        print_colored_quoted(out, Color::Blue, "Attributes:", format!("{attr:?}"))?;
    } else {
        log::warn!("No attributes found");
    }

    for member in group.member_names()? {

        print_colored_quoted(out, Color::Magenta, "Dataset:", format!("{member}"))?;
        if let Ok(dataset) = group.dataset(&member) {
            if let Ok(dataset) = dataset.as_dataset() {
                print_dataset_info(&dataset, n_samples, out)?;
            } else if let Ok(subgroup) = dataset.as_group() {
                print_group_info(&subgroup, n_samples, out)?;
            } else {
                log::error!("Unhandled object type for: {member}");
            }
        } else {
            log::error!("Failed opening dataset: '{member}' in '{gname}'");
        }
    }

    Ok(())
}

#[derive(Debug)]
enum NativePrimitiveType {
    Integer32b,
    Integer64b,
    UnsignedInteger32b,
    UnsignedInteger64b,
    Float32b,
    Float64b,
    Pointer(usize),
}

impl NativePrimitiveType {
    pub fn from_dtype(dtype: &Datatype) -> Self {
        if dtype.is::<i32>() {
            return Self::Integer32b;
        } else if dtype.is::<i64>() {
            return Self::Integer64b;
        } else if dtype.is::<usize>() {
            return Self::Pointer(size_of::<usize>());
        } else if dtype.is::<u32>() {
            return Self::UnsignedInteger32b;
        } else if dtype.is::<u64>() {
            return Self::UnsignedInteger64b;
        } else if dtype.is::<f32>() {
            return Self::Float32b;
        } else if dtype.is::<f64>() {
            return Self::Float64b;
        } else {
            todo!("Unsupported datatype");
        }
    }
}

impl std::fmt::Display for NativePrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NativePrimitiveType::Pointer(size) => write!(f, "{}-bit pointer", size * 8),
            NativePrimitiveType::Integer32b => write!(f, "32-bit integer"),
            NativePrimitiveType::Integer64b => write!(f, "64-bit integer"),
            NativePrimitiveType::UnsignedInteger32b => write!(f, "32-bit unsigned integer"),
            NativePrimitiveType::UnsignedInteger64b => write!(f, "64-bit unsigned integer"),
            NativePrimitiveType::Float32b => write!(f, "32-bit float"),
            NativePrimitiveType::Float64b => write!(f, "64-bit float"),
        }
    }
}
