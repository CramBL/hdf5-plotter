use std::fs;

pub(crate) fn plotly(data: &[f32]) {
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
