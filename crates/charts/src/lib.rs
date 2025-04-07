use leptos::prelude::*;
use leptos_chartistry::*;

#[derive(Debug, Clone)]
pub struct ReserveData {
    pub id: f64,
    pub x: f64,
    pub y: f64,
}

impl ReserveData {
    pub fn new(id: f64, x: f64, y: f64) -> Self {
        Self { id, x, y }
    }

    pub fn from_bin(id: u32, x: u128, y: u128) -> Self {
        Self {
            id: id as f64,
            x: x as f64 / 1_000_000.0,
            y: y as f64 / 1_000_000.0,
        }
    }
}

pub fn load_data() -> Vec<ReserveData> {
    vec![
        ReserveData::new(8388559.0, 0.0, 0.0),
        ReserveData::new(8388560.0, 0.0, 0.0),
        ReserveData::new(8388561.0, 0.0, 0.0),
        ReserveData::new(8388562.0, 0.0, 0.0),
        ReserveData::new(8388563.0, 0.0, 0.0),
        ReserveData::new(8388564.0, 0.0, 0.0),
        ReserveData::new(8388565.0, 0.0, 0.0),
        ReserveData::new(8388566.0, 0.0, 0.0),
        ReserveData::new(8388567.0, 0.0, 0.0),
        ReserveData::new(8388568.0, 0.0, 0.0),
        ReserveData::new(8388569.0, 0.0, 0.0),
        ReserveData::new(8388570.0, 0.0, 0.0),
        ReserveData::new(8388571.0, 0.0, 0.0),
        ReserveData::new(8388572.0, 0.0, 0.0),
        ReserveData::new(8388573.0, 0.0, 0.0),
        ReserveData::new(8388574.0, 0.0, 0.0),
        ReserveData::new(8388575.0, 0.0, 0.0),
        ReserveData::new(8388576.0, 0.0, 0.0),
        ReserveData::new(8388577.0, 0.0, 0.0),
        ReserveData::new(8388578.0, 0.0, 0.0),
        ReserveData::new(8388579.0, 0.0, 0.0),
        ReserveData::new(8388580.0, 0.0, 0.0),
        ReserveData::new(8388581.0, 0.0, 0.0),
        ReserveData::new(8388582.0, 0.0, 0.0),
        ReserveData::new(8388583.0, 0.0, 0.0),
        ReserveData::new(8388584.0, 0.0, 0.0),
        ReserveData::new(8388585.0, 0.0, 0.0),
        ReserveData::new(8388586.0, 0.0, 0.0),
        ReserveData::new(8388587.0, 0.0, 0.0),
        ReserveData::new(8388588.0, 0.0, 0.0),
        ReserveData::new(8388589.0, 0.0, 0.0),
        ReserveData::new(8388590.0, 0.0, 0.0),
        ReserveData::new(8388591.0, 0.0, 0.0),
        ReserveData::new(8388592.0, 0.0, 0.0),
        ReserveData::new(8388593.0, 0.0, 0.0),
        ReserveData::new(8388594.0, 0.0, 0.0),
        ReserveData::new(8388595.0, 0.0, 0.0),
        ReserveData::new(8388596.0, 0.0, 0.0),
        ReserveData::new(8388597.0, 0.0, 0.0),
        ReserveData::new(8388598.0, 0.0, 0.0),
        ReserveData::new(8388599.0, 0.0, 0.0),
        ReserveData::new(8388600.0, 0.0, 0.0),
        ReserveData::new(8388601.0, 0.0, 0.0),
        ReserveData::new(8388602.0, 0.0, 0.0),
        ReserveData::new(8388603.0, 0.181818, 0.0),
        ReserveData::new(8388604.0, 0.181818, 0.0),
        ReserveData::new(8388605.0, 0.181818, 0.0),
        ReserveData::new(8388606.0, 0.181818, 0.0),
        ReserveData::new(8388607.0, 0.181818, 0.0),
        ReserveData::new(8388608.0, 0.090909, 0.090909),
        ReserveData::new(8388609.0, 0.0, 0.181818),
        ReserveData::new(8388610.0, 0.0, 0.181818),
        ReserveData::new(8388611.0, 0.0, 0.181818),
        ReserveData::new(8388612.0, 0.0, 0.181818),
        ReserveData::new(8388613.0, 0.0, 0.181818),
        ReserveData::new(8388614.0, 0.0, 0.0),
        ReserveData::new(8388615.0, 0.0, 0.0),
        ReserveData::new(8388616.0, 0.0, 0.0),
        ReserveData::new(8388617.0, 0.0, 0.0),
        ReserveData::new(8388618.0, 0.0, 0.0),
        ReserveData::new(8388619.0, 0.0, 0.0),
        ReserveData::new(8388620.0, 0.0, 0.0),
        ReserveData::new(8388621.0, 0.0, 0.0),
        ReserveData::new(8388622.0, 0.0, 0.0),
        ReserveData::new(8388623.0, 0.0, 0.0),
        ReserveData::new(8388624.0, 0.0, 0.0),
        ReserveData::new(8388625.0, 0.0, 0.0),
        ReserveData::new(8388626.0, 0.0, 0.0),
        ReserveData::new(8388627.0, 0.0, 0.0),
        ReserveData::new(8388628.0, 0.0, 0.0),
        ReserveData::new(8388629.0, 0.0, 0.0),
        ReserveData::new(8388630.0, 0.0, 0.0),
        ReserveData::new(8388631.0, 0.0, 0.0),
        ReserveData::new(8388632.0, 0.0, 0.0),
        ReserveData::new(8388633.0, 0.0, 0.0),
        ReserveData::new(8388634.0, 0.0, 0.0),
        ReserveData::new(8388635.0, 0.0, 0.0),
        ReserveData::new(8388636.0, 0.0, 0.0),
        ReserveData::new(8388637.0, 0.0, 0.0),
        ReserveData::new(8388638.0, 0.0, 0.0),
        ReserveData::new(8388639.0, 0.0, 0.0),
        ReserveData::new(8388640.0, 0.0, 0.0),
        ReserveData::new(8388641.0, 0.0, 0.0),
        ReserveData::new(8388642.0, 0.0, 0.0),
        ReserveData::new(8388643.0, 0.0, 0.0),
        ReserveData::new(8388644.0, 0.0, 0.0),
        ReserveData::new(8388645.0, 0.0, 0.0),
        ReserveData::new(8388646.0, 0.0, 0.0),
        ReserveData::new(8388647.0, 0.0, 0.0),
        ReserveData::new(8388648.0, 0.0, 0.0),
        ReserveData::new(8388649.0, 0.0, 0.0),
        ReserveData::new(8388650.0, 0.0, 0.0),
        ReserveData::new(8388651.0, 0.0, 0.0),
        ReserveData::new(8388652.0, 0.0, 0.0),
        ReserveData::new(8388653.0, 0.0, 0.0),
        ReserveData::new(8388654.0, 0.0, 0.0),
        ReserveData::new(8388655.0, 0.0, 0.0),
        ReserveData::new(8388656.0, 0.0, 0.0),
        ReserveData::new(8388657.0, 0.0, 0.0),
    ]
    // vec![
    //     ReserveData::new(8388560.0, 1.5, 0.0),
    //     ReserveData::new(8388561.0, 1.5, 0.0),
    //     ReserveData::new(8388562.0, 1.5, 0.0),
    //     ReserveData::new(8388563.0, 1.5, 0.0),
    //     ReserveData::new(8388564.0, 1.5, 0.0),
    //     ReserveData::new(8388565.0, 1.0, 0.0),
    //     ReserveData::new(8388566.0, 1.0, 0.0),
    //     ReserveData::new(8388567.0, 1.0, 0.0),
    //     ReserveData::new(8388568.0, 1.0, 0.0),
    //     ReserveData::new(8388569.0, 1.0, 0.0),
    //     ReserveData::new(8388570.0, 1.0, 0.0),
    //     ReserveData::new(8388571.0, 1.0, 0.0),
    //     ReserveData::new(8388572.0, 1.0, 0.0),
    //     ReserveData::new(8388573.0, 1.0, 0.0),
    //     ReserveData::new(8388574.0, 1.0, 0.0),
    //     ReserveData::new(8388575.0, 0.5, 0.0),
    //     ReserveData::new(8388576.0, 0.5, 0.0),
    //     ReserveData::new(8388577.0, 0.5, 0.0),
    //     ReserveData::new(8388578.0, 0.5, 0.0),
    //     ReserveData::new(8388579.0, 0.5, 0.0),
    //     ReserveData::new(8388580.0, 0.5, 0.0),
    //     ReserveData::new(8388581.0, 0.5, 0.0),
    //     ReserveData::new(8388582.0, 0.5, 0.0),
    //     ReserveData::new(8388583.0, 0.5, 0.0),
    //     ReserveData::new(8388584.0, 0.5, 0.0),
    //     ReserveData::new(8388585.0, 0.1, 0.0),
    //     ReserveData::new(8388586.0, 0.1, 0.0),
    //     ReserveData::new(8388587.0, 0.1, 0.0),
    //     ReserveData::new(8388588.0, 0.1, 0.0),
    //     ReserveData::new(8388589.0, 0.1, 0.0),
    //     ReserveData::new(8388590.0, 0.5, 0.0),
    //     ReserveData::new(8388591.0, 0.5, 0.0),
    //     ReserveData::new(8388592.0, 0.5, 0.0),
    //     ReserveData::new(8388593.0, 0.5, 0.0),
    //     ReserveData::new(8388594.0, 0.5, 0.0),
    //     ReserveData::new(8388595.0, 1.0, 0.0),
    //     ReserveData::new(8388596.0, 2.0, 0.0),
    //     ReserveData::new(8388597.0, 3.0, 0.0),
    //     ReserveData::new(8388598.0, 4.0, 0.0),
    //     ReserveData::new(8388599.0, 5.0, 0.0),
    //     ReserveData::new(8388600.0, 3.0, 3.0),
    //     ReserveData::new(8388601.0, 0.0, 5.0),
    //     ReserveData::new(8388602.0, 0.0, 4.0),
    //     ReserveData::new(8388603.0, 0.0, 3.0),
    //     ReserveData::new(8388604.0, 0.0, 2.0),
    //     ReserveData::new(8388605.0, 0.0, 1.0),
    //     ReserveData::new(8388606.0, 0.0, 0.5),
    //     ReserveData::new(8388607.0, 0.0, 0.5),
    //     ReserveData::new(8388608.0, 0.0, 0.5),
    //     ReserveData::new(8388609.0, 0.0, 0.5),
    //     ReserveData::new(8388610.0, 0.0, 0.5),
    //     ReserveData::new(8388611.0, 0.0, 0.1),
    //     ReserveData::new(8388612.0, 0.0, 0.1),
    //     ReserveData::new(8388613.0, 0.0, 0.1),
    //     ReserveData::new(8388614.0, 0.0, 0.1),
    //     ReserveData::new(8388615.0, 0.0, 0.1),
    //     ReserveData::new(8388616.0, 0.0, 0.0),
    //     ReserveData::new(8388617.0, 0.0, 0.0),
    //     ReserveData::new(8388618.0, 0.0, 0.0),
    //     ReserveData::new(8388619.0, 0.0, 0.0),
    //     ReserveData::new(8388620.0, 0.0, 3.0),
    //     ReserveData::new(8388621.0, 0.0, 3.0),
    //     ReserveData::new(8388622.0, 0.0, 3.0),
    //     ReserveData::new(8388623.0, 0.0, 3.0),
    //     ReserveData::new(8388624.0, 0.0, 3.0),
    //     ReserveData::new(8388625.0, 0.0, 3.0),
    //     ReserveData::new(8388626.0, 0.0, 3.0),
    //     ReserveData::new(8388627.0, 0.0, 3.0),
    //     ReserveData::new(8388628.0, 0.0, 3.0),
    //     ReserveData::new(8388629.0, 0.0, 3.0),
    //     ReserveData::new(8388630.0, 0.0, 3.0),
    //     ReserveData::new(8388631.0, 0.0, 3.1),
    //     ReserveData::new(8388632.0, 0.0, 3.1),
    //     ReserveData::new(8388633.0, 0.0, 3.1),
    //     ReserveData::new(8388634.0, 0.0, 3.1),
    //     ReserveData::new(8388635.0, 0.0, 3.1),
    //     ReserveData::new(8388636.0, 0.0, 3.1),
    //     ReserveData::new(8388637.0, 0.0, 3.1),
    //     ReserveData::new(8388638.0, 0.0, 3.1),
    //     ReserveData::new(8388639.0, 0.0, 3.1),
    //     ReserveData::new(8388640.0, 0.0, 3.1),
    // ]
}

#[component]
pub fn LiquidityChart(
    debug: Signal<bool>,
    data: Signal<Vec<ReserveData>>,
    token_labels: Signal<(String, String)>,
) -> impl IntoView {
    let muted_foreground = Colour::from_rgb(156, 163, 175);

    let (token_x, token_y) = token_labels.get();

    let series = Series::new(|data: &ReserveData| data.id)
        .with_min_y(0.00)
        .with_colours([
            Colour::from_rgb(246, 193, 119),
            Colour::from_rgb(49, 116, 143),
        ])
        .bar(Bar::new(|data: &ReserveData| data.x).with_name(token_y))
        .bar(Bar::new(|data: &ReserveData| data.y).with_name(token_x));

    view! {
        <Chart
            aspect_ratio=AspectRatio::from_outer_height(330.0, 1.73)
            debug=debug
            series=series
            data=data
            font_width=10.0
            font_height=14.0
            tooltip=Tooltip::left_cursor()

            inner=[
                AxisMarker::bottom_edge()
                    .with_arrow(false)
                    .with_colour(muted_foreground)
                    .into_inner(),
                XGuideLine::over_data().with_colour(muted_foreground).into_inner(),
            ]
            bottom=TickLabels::aligned_floats()
        />
    }
}

#[component]
pub fn PoolDistributionChart(
    debug: Signal<bool>,
    data: Signal<Vec<ReserveData>>,
    token_labels: Signal<(String, String)>,
) -> impl IntoView {
    let muted_foreground = Colour::from_rgb(156, 163, 175);

    let (token_x, token_y) = token_labels.get();

    let series = Series::new(|data: &ReserveData| data.id)
        .with_min_y(0.0)
        .with_colours([
            Colour::from_rgb(246, 193, 119),
            Colour::from_rgb(49, 116, 143),
        ])
        .bar(Bar::new(|data: &ReserveData| data.x).with_name(token_y))
        .bar(Bar::new(|data: &ReserveData| data.y).with_name(token_x));

    view! {
        <Chart
            aspect_ratio=AspectRatio::from_env_width(160.0)
            debug=debug
            series=series
            data=data
            font_width=9.0
            font_height=14.0
            tooltip=Tooltip::left_cursor()

            inner=[
                AxisMarker::bottom_edge()
                    .with_arrow(false)
                    .with_colour(muted_foreground)
                    .into_inner(),
                XGuideLine::over_data().with_colour(muted_foreground).into_inner(),
            ]
            bottom=TickLabels::aligned_floats()
        />
    }
}
