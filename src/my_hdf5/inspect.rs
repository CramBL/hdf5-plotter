use std::fmt;

use hdf5::{Dataset, H5Type};
use ndarray::{ArrayD, Axis, NdProducer};
use termcolor::{Color, StandardStream};

use crate::my_hdf5::util::NativePrimitiveType;
use crate::util::{print_color, print_colored_quoted};

pub fn print_dataset_info(
    dataset: &hdf5::Dataset,
    n_samples: usize,
    out: &mut StandardStream,
) -> anyhow::Result<()> {
    print_colored_quoted(
        out,
        Color::Magenta,
        "  - Dataset: ",
        format!("{}", dataset.name()),
    )?;
    print_colored_quoted(
        out,
        Color::Yellow,
        "     - Layout: ",
        format!("{:?}", dataset.layout()),
    )?;
    print_colored_quoted(
        out,
        Color::Yellow,
        "     - Attribute names: ",
        format!("{:?}", dataset.attr_names().unwrap_or_default()),
    )?;
    print_colored_quoted(
        out,
        Color::Yellow,
        "     - Resizable: ",
        format!("{}", dataset.is_resizable()),
    )?;
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

    print_colored_quoted(
        out,
        Color::Yellow,
        "     - Data type: ",
        format!("{}", NativePrimitiveType::from_dtype(&dtype)),
    )?;
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

pub fn print_group_info(
    group: &hdf5::Group,
    n_samples: usize,
    out: &mut StandardStream,
) -> anyhow::Result<()> {
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

pub fn print_preview_n_samples<T: H5Type + fmt::Display + std::fmt::Debug>(
    data: &ArrayD<T>,
    n: usize,
    out: &mut StandardStream,
) -> anyhow::Result<()> {
    take_n_from_dims_print(data, n, out)?;
    take_n_from_axes_print(data, n, out)?;
    Ok(())
}

pub fn take_n_from_axes_print<T: H5Type + fmt::Display>(
    data: &ArrayD<T>,
    n: usize,
    out: &mut StandardStream,
) -> anyhow::Result<()> {
    // Get the shape of the dataset
    let shape = data.shape();
    log::debug!("{shape:?}");
    let dims = data.ndim();

    print_colored_quoted(
        out,
        Color::Blue,
        "===> Displaying preview of axes of ",
        format!("{dims}-D array"),
    )?;
    log::debug!("Raw array preview: {data}");

    // Iterate over each dimension and print the first 10 elements along that axis
    for dim_index in 0..dims {
        let axis_len = shape[dim_index];
        let axis = data.lanes(Axis(dim_index));

        let info = format!("Axis[{dim_index}] len: {axis_len}");

        let mut i = 0;
        for element in axis.into_iter().take(n) {
            if i == 0 {
                print_color(
                    out,
                    Color::Yellow,
                    format!("{info} dim: {}, ndim: {}", element.dim(), element.ndim()),
                )?;
            }
            print_color(out, Color::White, format!("\t\t[{i}]: {element}"))?;
            if let Some(first_element) = element.get(0) {
                log::trace!("\t\t[{i}][0]: {first_element}");
            }
            i += 1;
        }
    }

    print_color(out, Color::Blue, "<===")?;
    Ok(())
}

pub fn take_n_from_dims_print<T: H5Type + std::fmt::Debug>(
    data: &ArrayD<T>,
    n: usize,
    out: &mut StandardStream,
) -> anyhow::Result<()> {
    // Get the shape of the dataset
    let shape = data.shape();
    let dims = data.ndim();
    log::trace!("dims={dims}");
    print_colored_quoted(
        out,
        Color::Blue,
        "===> Displaying preview of dimension of ",
        format!("{dims}-D dataset"),
    )?;

    // Iterate over each dimension and print the first 10 elements
    for dim_index in 0..dims {
        print_colored_quoted(
            out,
            Color::Blue,
            format!("First {n} elements of dimension "),
            format!("{dim_index}"),
        )?;
        let mut elements = vec![];
        for i in 0..shape[dim_index].min(n) {
            // Start index at the origin for all dimensions
            let mut index = vec![0; dims];
            // Set the index of the current dimension to vary
            index[dim_index] = i;
            // Convert Vec<usize> to &[usize] for indexing
            let element = &data[index.as_slice()];
            elements.push(element);
        }
        println!("{elements:?}");
    }

    print_color(out, Color::Blue, "<===")?;
    Ok(())
}
