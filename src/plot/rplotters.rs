use plotters::prelude::*;
use std::error::Error;

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
