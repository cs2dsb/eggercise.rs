use asciimath_rs::format::mathml::ToMathML;
use leptos::{
    component, create_rw_signal, ev::KeyboardEvent, event_target_value, view, IntoView, RwSignal,
    Signal, SignalGet, SignalUpdate, SignalWith,
};
use leptos_chartistry::{
    AspectRatio, AxisMarker, Chart as Chart_, IntoInner, Legend, Line, RotatedLabel, Series,
    TickLabels, Tooltip, XGridLine, XGuideLine, YGridLine, YGuideLine,
};
use meval::Error as MevalError;
use shared::api::error::{FrontendError, ResultContext};

use crate::components::FrontendErrorBoundary;

#[component]
pub fn EquationForm(equation: RwSignal<String>) -> impl IntoView {
    view! {
        <section>
            <form on:submit=|ev| ev.prevent_default()>
                <input
                    type="text"
                    placeholder="Equation"
                    // Attribute only sets the initial value
                    value={ equation.get() }
                    // TODO: Is it possible to dedupe these?
                    on:keyup=move |ev: KeyboardEvent| {
                        let val = event_target_value(&ev);
                        equation.update(|v| *v = val);
                    }
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        equation.update(|v| *v = val);
                    }
                />
            </form>
        </section>
    }
}

#[component]
pub fn EquationDisplay(#[prop(into)] equation: Signal<String>) -> impl IntoView {
    let output = Signal::derive(move || {
        let expr: meval::Expr = equation
            .with(|e| e.parse())
            .map_err(FrontendError::map_display)
            .with_context(|| format!("Error parsing \"{}\"", equation.get()))?;

        let eval = expr
            .eval()
            .map_err(FrontendError::map_display)
            .with_context(|| format!("Error evaluating \"{}\"", equation.get()))?;

        let mathml = asciimath_rs::parse(equation.get()).to_mathml();

        Ok::<_, FrontendError<meval::Error>>((expr, eval, mathml))
    });

    view! {
        <section>
            <FrontendErrorBoundary<MevalError>>
            { move || output.get().map(|(expr, eval, mathml)| {
                view! {
                    <p>{ format!("Parsed expression: \"{:?}\"", expr) }</p>
                    <p>{ format!("Evaluated expression: \"{:?}\"", eval) }</p>
                    <math inner_html=mathml />
                }.into_view()
            }) }
            </FrontendErrorBoundary<MevalError>>
        </section>
    }
}

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

    let equation = create_rw_signal(String::new());

    view! {
        <h1>"Chart"</h1>
        <section>
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
        </section>
        <EquationForm equation />
        <EquationDisplay equation />
    }
}
