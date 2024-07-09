use hdf5::Datatype;
use ndarray::{Axis, IntoNdProducer};
use std::mem::size_of;

pub fn print_dataset_info(dataset: &hdf5::Dataset) -> anyhow::Result<()> {
    log::info!("  - Dataset: {}", dataset.name());
    log::info!("     - Layout: {:?}", dataset.layout());
    log::info!(
        "     - Attribute names: {:?}",
        dataset.attr_names().unwrap_or_default()
    );
    log::debug!("     - Resizable: {}", dataset.is_resizable());
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
    log::trace!("    - Data type: {dtype:?}, {}B", dtype.size());

    log::info!(
        "    - Data type: {}",
        NativePrimitiveType::from_dtype(&dtype)
    );
    log::info!("    - Shape: {shape:?}");

    match NativePrimitiveType::from_dtype(&dtype) {
        NativePrimitiveType::Integer32b => {
            let data = dataset.read_2d::<i32>().unwrap();
            log::info!("    - First 10 elements: {:?}", &data);
        }
        NativePrimitiveType::Integer64b => {
            let data = dataset.read_2d::<i64>().unwrap();
            log::info!("    - First 10 elements: {:?}", &data);
        }
        NativePrimitiveType::UnsignedInteger32b => {
            todo!()
        }
        NativePrimitiveType::UnsignedInteger64b => {
            let data = dataset.read_2d::<u64>().unwrap();
            log::info!("    - First 10 elements: {:?}", &data);
        }
        NativePrimitiveType::Float32b => {
            let data = dataset.read_2d::<f32>().unwrap();

            for idx in 0..shape.len() {
                let axis_len = data.lanes(Axis(idx)).into_iter().count();
                let axis = data.lanes(Axis(idx));
                log::info!("Axis[{idx}] len = {axis_len}");
                let mut i = 0;
                for element in axis {
                    if i == 0 {
                        log::trace!("dim: {}, ndim: {}", element.dim(), element.ndim());
                    }
                    log::info!("\t\t[{i}]: {element}");
                    log::debug!("\t\t[{i}]: {}", element.get(0).unwrap());
                    i += 1;
                    if i == 10 {
                        break;
                    }
                }
            }
        }
        NativePrimitiveType::Float64b => {
            let data = dataset.read_2d::<f64>().unwrap();
            let d = data.lanes(Axis(0));
            let axis0 = d.into_producer();
            let mut idx = 0;
            for e in axis0.into_producer() {
                log::info!("    - First 10 elements: {e}");
                idx += 1;
                if idx == 10 {
                    break;
                }
            }
        }
        NativePrimitiveType::Pointer(_) => todo!(),
    }

    Ok(())
}

pub fn print_group_info(group: &hdf5::Group) -> anyhow::Result<()> {
    let gname = group.name();
    log::info!("Group: '{gname}'");

    if let Ok(attr) = group.attr_names() {
        log::info!("Attributes: {attr:?}");
    } else {
        log::warn!("No attributes found");
    }

    for member in group.member_names()? {
        log::info!("Dataset '{member}'");
        if let Ok(dataset) = group.dataset(&member) {
            log::info!("{dataset:?}");
            if let Ok(dataset) = dataset.as_dataset() {
                print_dataset_info(&dataset)?;
            } else if let Ok(subgroup) = dataset.as_group() {
                print_group_info(&subgroup)?;
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
