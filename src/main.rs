use hdf5::types::dyn_value::DynArray;
use hdf5::Result;
use hdf5::{File, H5Type};
use ndarray;

use std::error::Error;
use std::fs;
fn print_dataset_info(dataset: &hdf5::Dataset) -> Result<(), Box<dyn Error>> {
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

fn print_group_info(group: &hdf5::Group) -> Result<(), Box<dyn Error>> {
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

fn main() -> Result<(), Box<dyn Error>> {
    // Path to your HDF5 file
    let file_path = "20240703_140137_bifrost.h5";

    // Open the HDF5 file
    let file = File::open(file_path)?;
    println!("opened");

    // Assume the dataset is named "dataset"
    let dataset = file.dataset("current")?;
    println!("{dataset:?}");

    print_group_info(&file)?;

    // // Read the dataset as Vec<i32> or Vec<f64> depending on the content type
    let data = dataset.read_2d::<f32>().unwrap();

    let dxxx = data.fold_axis(ndarray::Axis(1), 0_f32, |c, d| *d);

    println!("---------");
    println!("{dxxx}");
    println!("{}", dxxx.sum());
    // println!("{data:?}");

    // let sum = data.sum();
    let (mut min, mut max) = (f32::MAX, f32::MIN);
    for d in &dxxx {
        if *d < min {
            min = *d;
        } else if *d > max {
            max = *d;
        }
    }

    // let avg = sum / (data.len() as f32);

    // println!("Sum={sum}, Avg={avg}, Min={min}, Max={max}");

    // Flatten the 2D data for plotting

    // Plot the data
    #[cfg(features = "rplotters")]
    rplotters::plot_data(dxxx.as_slice().unwrap(), dxxx.len(), 1, min, max)?;

    let data = dxxx.as_slice().unwrap();

    #[cfg(feature = "rplotly")]
    plotly(data);

    Ok(())
}

#[cfg(feature = "rplotly")]
fn plotly(data: &[f32]) {
    use plotly::common::Title;
    use plotly::layout::{Axis, Layout};
    use plotly::Plot;
    use plotly::Scatter;
    let x_values: Vec<_> = (0..data.len()).collect();

    // Create a plot
    let trace = Scatter::new(x_values, data.to_vec())
        .mode(plotly::common::Mode::Lines)
        .name("Data");

    let layout = Layout::new()
        .title("Interactive HDF5 Data Plot")
        .x_axis(Axis::new().title("Index"))
        .y_axis(Axis::new().title("Value"));

    let mut plot = Plot::new();
    plot.add_trace(trace);
    plot.set_layout(layout);

    // Save the plot as an HTML file
    let s = plot.to_html();
    fs::write("plot.html", s).unwrap();

    println!(
        "Plot saved to plot.html. Open this file in a web browser to view the interactive plot."
    );
}

#[cfg(features = "rplotters")]
pub mod rplotters {

    use std::error::Error;

    use plotters::prelude::*;
    pub fn plot_data(
        data: &[f32],
        rows: usize,
        cols: usize,
        min: f32,
        max: f32,
    ) -> Result<(), Box<dyn Error>> {
        let root = BitMapBackend::new("plot.png", (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("HDF5 Data Plot", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..rows, min..max)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            (0..)
                .zip(data.iter())
                .map(|(idx, &value)| (idx % rows, value)),
            &BLUE,
        ))?;

        root.present()?;
        println!("Plot saved to plot.png");

        plot_data_svg(data)?;

        Ok(())
    }

    fn plot_data_svg(data: &[f32]) -> Result<(), Box<dyn Error>> {
        let root_area = SVGBackend::new("plot.svg", (1280, 720)).into_drawing_area();
        root_area.fill(&WHITE)?;

        let (min, max) = (
            data.iter().cloned().fold(f32::INFINITY, f32::min),
            data.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
        );

        let mut chart = ChartBuilder::on(&root_area)
            .caption("HDF5 Data Plot", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..data.len(), min..max)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, *y)),
            &RED,
        ))?;

        root_area.present()?;
        println!("Plot saved to plot.svg");

        Ok(())
    }
}
