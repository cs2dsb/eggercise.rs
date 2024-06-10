use leptos::{component, view, IntoView, Signal};
use leptos_chartistry::{
    AspectRatio, AxisMarker, Chart as Chart_, IntoInner, Legend, Line, RotatedLabel, Series,
    TickLabels, Tooltip, XGridLine, XGuideLine, YGridLine, YGuideLine,
};

#[derive(Clone)]
pub struct MyData {
    x: f64,
    y: Vec<f64>,
}

impl MyData {
    fn new(x: f64, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
        }
    }
}

#[component]
pub fn Chart() -> impl IntoView {
    let mut series = Series::new(|data: &MyData| data.x);
    for y in 0..3 {
        series = series
            .line(Line::new(move |data: &MyData| data.y[y]).with_name(format!("Yyyyyyyy {y}")));
    }

    let data = Signal::derive(|| {
        let mut data = Vec::new();
        for x in 0..10 {
            let mut ys = Vec::new();
            for y in 0..3 {
                let q = if x % 2 == 0 { -3. } else { x as f64 * 1.5 };
                ys.push((x + y) as f64 * q);
            }
            data.push(MyData::new(x as f64, ys));
        }
        data
    });

    view! {
        <h1>"Chart"</h1>
        <Chart_
            aspect_ratio=AspectRatio::from_outer_height(800.0, 1.2)
            debug=false
            series=series
            data=data

            // Decorate our chart
            top=RotatedLabel::middle("My Chart")
            left=TickLabels::aligned_floats()
            bottom=Legend::middle()
            inner=[
                // Standard set of inner layout options
                AxisMarker::left_edge().into_inner(),
                AxisMarker::bottom_edge().into_inner(),
                XGridLine::default().into_inner(),
                YGridLine::default().into_inner(),
                YGuideLine::over_mouse().into_inner(),
                XGuideLine::over_data().into_inner(),
            ]
            tooltip=Tooltip::left_cursor().show_x_ticks(false)
        />
    }
}
