use std::fs;
use serde::Serialize;

pub(crate) fn plotly<T>(data: Vec<T>)
    where T: Serialize + Clone + 'static,
{
    use plotly::layout::{Axis, Layout};
    use plotly::Plot;
    use plotly::Scatter;
    let x_values: Vec<_> = (0..data.len()).collect();

    // Create a plot
    let trace = Scatter::new(x_values, data)
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
    if s.len() > 200_000_000 {
        log::warn!("The produced plot exceed 200 MB and will be hard for any browser to display, consider limiting the sample count");
    } else if s.len() > 150_000_000 {
        log::warn!("The produced plot exceeds 150 MB and will be difficult to load and interact with");
    } else if s.len() > 100_000_000 {
        log::warn!("The produced plot exceeds 100 MB, it should load at a reasonable speed in most cases but interaction will likely be sluggish");
    }
    fs::write("plot.html", s).unwrap();

    println!(
        "Plot saved to plot.html. Open this file in a web browser to view the interactive plot."
    );
}
