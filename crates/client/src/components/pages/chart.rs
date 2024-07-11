use std::{collections::HashMap, num::ParseFloatError};

use asciimath_rs::format::mathml::ToMathML;
use leptos::{
    component, create_rw_signal, create_signal, event_target_value, view, with_owner, For,
    IntoView, Owner, RwSignal, Signal, SignalGet, SignalUpdate, SignalWith,
};
use leptos_chartistry::{
    AspectRatio, AxisMarker, Chart as Chart_, IntoInner, Legend, Line, RotatedLabel, Series,
    TickLabels, Tooltip, XGridLine, XGuideLine, YGridLine, YGuideLine,
};
use meval::Error as MevalError;
use shared::api::error::{FrontendError, ResultContext};
use wasm_bindgen::JsCast;

use crate::components::FrontendErrorBoundary;

#[component]
pub fn EquationForm(equation: RwSignal<String>) -> impl IntoView {
    fn on_change<T: JsCast>(ev: T, signal: RwSignal<String>) {
        let val = event_target_value(&ev);
        signal.update(|v| *v = val)
    }
    view! {
        <section>
            <input
                type="text"
                placeholder="Equation"
                // Attribute only sets the initial value
                value={ equation.get() }
                on:keyup=move |ev| on_change(ev, equation)
                on:change=move |ev| on_change(ev, equation)
            />
        </section>
    }
}

fn get_vars(expr: &meval::Expr) -> Vec<String> {
    use meval::tokenizer::Token;

    expr.iter().filter_map(|t| if let Token::Var(v) = t { Some(v.clone()) } else { None }).collect()
}
// TODO: rejig so binds are in the data entry form
#[component]
pub fn EquationDisplay(#[prop(into)] equation: Signal<String>) -> impl IntoView {
    let (binds, set_binds) = create_signal(HashMap::new());

    // TODO: Is there a neater way of doing this? The counters example doesn't seem
    // to do this and has a similar setup
    let owner = Owner::current().unwrap();

    let update_binds = move |vars: &Vec<String>| {
        set_binds.update(|binds| {
            binds.retain(|k: &String, _| vars.contains(k));
            for v in vars.iter() {
                if !binds.contains_key(v) {
                    let signal = with_owner(owner, || create_rw_signal(Ok(0_f64)));
                    binds.insert((*v).to_owned(), signal);
                }
            }
        });
    };

    let output = Signal::derive(move || {
        let expr: meval::Expr = equation
            .with(|e| e.parse())
            .map_err(FrontendError::map_display)
            .with_context(|| format!("Error parsing \"{}\"", equation.get()))?;

        let vars = get_vars(&expr);

        update_binds(&vars);

        let values = binds.with(|b| {
            vars.iter()
                .map(|v| {
                    let signal = b[v];
                    signal.get().unwrap_or_default()
                })
                .collect::<Vec<f64>>()
        });

        let vars = vars.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
        let bind = expr
            .bindn(&vars)
            .map_err(FrontendError::map_display)
            .with_context(|| format!("Error binding \"{:?}\" in \"{}\"", vars, equation.get()))?;

        let eval = bind(&values);
        Ok::<_, FrontendError<meval::Error>>(eval)
    });

    let mathml = Signal::derive(move || equation.with(|e| asciimath_rs::parse(e).to_mathml()));

    view! {
        <section>
            { move || view! { <math inner_html=mathml() /> }}
            <For
                each=move || binds.get()
                key=|(label, _)| label.to_owned()
                children=move |(label, signal)| view!{ <Bind label signal /> }
            />
            <FrontendErrorBoundary<MevalError>>
                { move || output.get().map(|eval| {
                    view! {
                        <p>{ format!("Result: {:?}", eval) }</p>
                    }.into_view()
                }) }
            </FrontendErrorBoundary<MevalError>>
        </section>
    }
}

#[component]
pub fn Bind(label: String, signal: RwSignal<Result<f64, ParseFloatError>>) -> impl IntoView {
    fn on_change<T: JsCast>(ev: T, signal: RwSignal<Result<f64, ParseFloatError>>) {
        let val = event_target_value(&ev);
        signal.update(|v| *v = val.parse())
    }

    view! {
        <div class="block-wrap" style="padding-block-end: var(--size-1);">
            <div>{ label }</div>
            <input
                type="number"
                // Attribute only sets the initial value
                value={ move || signal.get().unwrap_or_default() }
                on:keyup=move |ev| on_change(ev, signal)
                on:change=move |ev| on_change(ev, signal)
            />
        </div>
        { move || match signal.get() {
            Ok(_) => view!{}.into_view(),
            Err(_) => view!{ <p>"Enter a valid floating point number"</p> }.into_view(),
        }}
    }
}

#[derive(Clone)]
pub struct MyData {
    x: f64,
    y: Vec<f64>,
}

impl MyData {
    fn new(x: f64, y: Vec<f64>) -> Self {
        Self { x, y }
    }
}

#[component]
pub fn Chart() -> impl IntoView {
    let equation = create_rw_signal(String::new());
    let expr: Signal<meval::Expr> = Signal::derive(move || {
        if let Ok(expr) = equation.with(|v| v.parse()) {
            // Can contain multiple xs but no other vars
            if get_vars(&expr).iter().all(|v| v == "x") {
                return expr;
            }
        }

        return "x".parse().unwrap();
    });

    let mut series = Series::new(|data: &MyData| data.x);
    for y in 0..1 {
        series = series.line(Line::new(move |data: &MyData| data.y[y]).with_name(format!("Y {y}")));
    }

    let data = Signal::derive(move || {
        let expr = expr.get();
        let vars = get_vars(&expr);
        let x_count = vars.len();

        let var_refs = vars.iter().map(String::as_ref).collect::<Vec<_>>();

        let bind = expr.bindn(&var_refs).expect("Binding x failed");

        let mut data = Vec::new();
        for x in 0..10 {
            let x = vec![x as f64];
            let ys = vec![bind(&x.iter().cycle().take(x_count).cloned().collect::<Vec<_>>())];
            data.push(MyData::new(x[0], ys));
        }
        data
    });

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

        <div class="block-wrap">
            <EquationForm equation />
            <EquationDisplay equation />
        </div>

    }
}
