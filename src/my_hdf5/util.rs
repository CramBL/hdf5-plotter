use std::mem::size_of;

use hdf5::Datatype;
use ndarray::{Axis, IntoNdProducer};

pub fn print_dataset_info(dataset: &hdf5::Dataset) -> anyhow::Result<()> {
    log::info!("  - Dataset: {}", dataset.name());
    let dtype = dataset.dtype()?;
    let shape = dataset.shape();

    log::info!("    - Data type: {dtype:?}");
    log::info!("    - Shape: {shape:?}");

    log::info!("DTYPE:{:?}", NativePrimitiveType::from_dtype(&dtype));

    // Print the first few elements to understand the content
    if dtype.is::<f32>() {
        let data = dataset.read_2d::<f32>().unwrap();
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
    } else if dtype.is::<i32>() {
        let data = dataset.read_2d::<i32>().unwrap();
        log::info!("    - First 10 elements: {:?}", &data);
    } else if dtype.is::<f64>() {
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
    } else if dtype.is::<i64>() {
        let data = dataset.read_2d::<i64>().unwrap();
        log::info!("    - First 10 elements: {:?}", &data);
    } else {
        log::info!("    - Unhandled data type: {:?}", dtype);
    }

    Ok(())
}

pub fn print_group_info(group: &hdf5::Group) -> anyhow::Result<()> {
    println!("Group: {}", group.name());

    for member in group.member_names()? {
        println!("{member}");
        let obj = group.dataset(&member).unwrap();
        println!("{:?}", obj);
        if let Ok(dataset) = obj.as_dataset() {
            print_dataset_info(&dataset)?;
        } else if let Ok(subgroup) = obj.as_group() {
            print_group_info(&subgroup)?;
        } else {
            println!("  - Unhandled object type for: {member}");
        }
    }

    Ok(())
}

#[derive(Debug)]
enum NativePrimitiveType {
    Integer(usize),
    UnsignedInteger(usize),
    Float(usize),
    Pointer(usize),
}

impl NativePrimitiveType {
    pub fn from_dtype(dtype: &Datatype) -> Self {
        log::debug!("dtype={dtype:?}");
        if dtype.is::<i32>() {
            return Self::Integer(size_of::<i32>());
        } else if dtype.is::<i64>() {
            return Self::Integer(size_of::<i64>());
        } else if dtype.is::<usize>() {
            return Self::Pointer(size_of::<usize>());
        } else if dtype.is::<u32>() {
            return Self::UnsignedInteger(size_of::<u32>());
        } else if dtype.is::<u64>() {
            return Self::UnsignedInteger(size_of::<u64>());
        } else if dtype.is::<f32>() {
            return Self::Float(size_of::<f32>());
        } else if dtype.is::<f64>() {
            return Self::Float(size_of::<f64>());
        } else {
            todo!("Unsupported datatype");
        }
    }
}
