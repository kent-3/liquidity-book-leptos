use leptos::prelude::*;
use leptos_chartistry::*;

#[derive(Clone)]
pub struct ReserveData {
    x: f64,
    y1: f64,
    y2: f64,
}

impl ReserveData {
    pub fn new(x: f64, y1: f64, y2: f64) -> Self {
        Self { x, y1, y2 }
    }
}

pub fn load_data() -> Signal<Vec<ReserveData>> {
    Signal::derive(|| {
        vec![
            ReserveData::new(10.0, 1.5, 0.0),
            ReserveData::new(11.0, 1.5, 0.0),
            ReserveData::new(12.0, 1.5, 0.0),
            ReserveData::new(13.0, 1.5, 0.0),
            ReserveData::new(14.0, 1.5, 0.0),
            ReserveData::new(15.0, 1.0, 0.0),
            ReserveData::new(16.0, 1.0, 0.0),
            ReserveData::new(17.0, 1.0, 0.0),
            ReserveData::new(18.0, 1.0, 0.0),
            ReserveData::new(19.0, 1.0, 0.0),
            ReserveData::new(20.0, 1.0, 0.0),
            ReserveData::new(21.0, 1.0, 0.0),
            ReserveData::new(22.0, 1.0, 0.0),
            ReserveData::new(23.0, 1.0, 0.0),
            ReserveData::new(24.0, 1.0, 0.0),
            ReserveData::new(25.0, 0.5, 0.0),
            ReserveData::new(26.0, 0.5, 0.0),
            ReserveData::new(27.0, 0.5, 0.0),
            ReserveData::new(28.0, 0.5, 0.0),
            ReserveData::new(29.0, 0.5, 0.0),
            ReserveData::new(30.0, 0.5, 0.0),
            ReserveData::new(31.0, 0.5, 0.0),
            ReserveData::new(32.0, 0.5, 0.0),
            ReserveData::new(33.0, 0.5, 0.0),
            ReserveData::new(34.0, 0.5, 0.0),
            ReserveData::new(35.0, 0.1, 0.0),
            ReserveData::new(36.0, 0.1, 0.0),
            ReserveData::new(37.0, 0.1, 0.0),
            ReserveData::new(38.0, 0.1, 0.0),
            ReserveData::new(39.0, 0.1, 0.0),
            ReserveData::new(40.0, 0.5, 0.0),
            ReserveData::new(41.0, 0.5, 0.0),
            ReserveData::new(42.0, 0.5, 0.0),
            ReserveData::new(43.0, 0.5, 0.0),
            ReserveData::new(44.0, 0.5, 0.0),
            ReserveData::new(45.0, 1.0, 0.0),
            ReserveData::new(46.0, 2.0, 0.0),
            ReserveData::new(47.0, 3.0, 0.0),
            ReserveData::new(48.0, 4.0, 0.0),
            ReserveData::new(49.0, 5.0, 0.0),
            ReserveData::new(50.0, 3.0, 3.0),
            ReserveData::new(51.0, 0.0, 5.0),
            ReserveData::new(52.0, 0.0, 4.0),
            ReserveData::new(53.0, 0.0, 3.0),
            ReserveData::new(54.0, 0.0, 2.0),
            ReserveData::new(55.0, 0.0, 1.0),
            ReserveData::new(56.0, 0.0, 0.5),
            ReserveData::new(57.0, 0.0, 0.5),
            ReserveData::new(58.0, 0.0, 0.5),
            ReserveData::new(59.0, 0.0, 0.5),
            ReserveData::new(60.0, 0.0, 0.5),
            ReserveData::new(61.0, 0.0, 0.1),
            ReserveData::new(62.0, 0.0, 0.1),
            ReserveData::new(63.0, 0.0, 0.1),
            ReserveData::new(64.0, 0.0, 0.1),
            ReserveData::new(65.0, 0.0, 0.1),
            ReserveData::new(66.0, 0.0, 0.0),
            ReserveData::new(67.0, 0.0, 0.0),
            ReserveData::new(68.0, 0.0, 0.0),
            ReserveData::new(69.0, 0.0, 0.0),
            ReserveData::new(70.0, 0.0, 3.0),
            ReserveData::new(71.0, 0.0, 3.0),
            ReserveData::new(72.0, 0.0, 3.0),
            ReserveData::new(73.0, 0.0, 3.0),
            ReserveData::new(74.0, 0.0, 3.0),
            ReserveData::new(75.0, 0.0, 3.0),
            ReserveData::new(76.0, 0.0, 3.0),
            ReserveData::new(77.0, 0.0, 3.0),
            ReserveData::new(78.0, 0.0, 3.0),
            ReserveData::new(79.0, 0.0, 3.0),
            ReserveData::new(80.0, 0.0, 3.0),
            ReserveData::new(81.0, 0.0, 3.1),
            ReserveData::new(82.0, 0.0, 3.1),
            ReserveData::new(83.0, 0.0, 3.1),
            ReserveData::new(84.0, 0.0, 3.1),
            ReserveData::new(85.0, 0.0, 3.1),
            ReserveData::new(86.0, 0.0, 3.1),
            ReserveData::new(87.0, 0.0, 3.1),
            ReserveData::new(88.0, 0.0, 3.1),
            ReserveData::new(89.0, 0.0, 3.1),
            ReserveData::new(90.0, 0.0, 3.1),
        ]
    })
}

#[component]
pub fn LiquidityChart(debug: Signal<bool>, data: Signal<Vec<ReserveData>>) -> impl IntoView {
    let series = Series::new(|data: &ReserveData| data.x)
        .with_min_y(0.00)
        .with_colours([
            Colour::from_rgb(246, 193, 119),
            Colour::from_rgb(49, 116, 143),
        ])
        .bar(
            Bar::new(|data: &ReserveData| data.y1).with_name("Token Y"), // .with_group_gap(0.05)
                                                                         // .with_gap(0.1),
        )
        .bar(
            Bar::new(|data: &ReserveData| data.y2).with_name("Token X"), // .with_group_gap(0.05)
                                                                         // .with_gap(0.1),
        );

    view! {
        <Chart
            aspect_ratio=AspectRatio::from_outer_height(330.0, 1.73)
            debug=debug
            series=series
            data=data
            font_width=14.0
            font_height=14.0

            // left=TickLabels::aligned_floats()
            inner=[]
            bottom=TickLabels::aligned_floats()
            tooltip=Tooltip::left_cursor()
        />
    }
}
