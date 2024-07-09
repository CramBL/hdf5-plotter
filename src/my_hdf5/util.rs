pub fn print_dataset_info(dataset: &hdf5::Dataset) -> anyhow::Result<()> {
    println!("  - Dataset: {}", dataset.name());
    let dtype = dataset.dtype()?;
    let shape = dataset.shape();

    println!("    - Data type: {:?}", dtype);
    println!("    - Shape: {:?}", shape);

    // Print the first few elements to understand the content
    if dtype.is::<f32>() {
        let data = dataset.read_2d::<f32>().unwrap();
        println!("    - First elements: {:?}", &data);
    } else if dtype.is::<i32>() {
        let data = dataset.read_2d::<i32>().unwrap();
        println!("    - First elements: {:?}", &data);
    } else if dtype.is::<f64>() {
        let data = dataset.read_2d::<f64>().unwrap();
        println!("    - First elements: {:?}", &data);
    } else if dtype.is::<i64>() {
        let data = dataset.read_2d::<i64>().unwrap();
        println!("    - First elements: {:?}", &data);
    } else {
        println!("    - Unhandled data type: {:?}", dtype);
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
