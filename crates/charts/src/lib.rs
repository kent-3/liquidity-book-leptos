use leptos::prelude::*;
use leptos_chartistry::*;

#[derive(Clone)]
pub struct MyData {
    x: f64,
    y1: f64,
    y2: f64,
}

impl MyData {
    fn new(x: f64, y1: f64, y2: f64) -> Self {
        Self { x, y1, y2 }
    }
}

pub fn load_data() -> Signal<Vec<MyData>> {
    Signal::derive(|| {
        vec![
            MyData::new(10.0, 1.5, 0.0),
            MyData::new(11.0, 1.5, 0.0),
            MyData::new(12.0, 1.5, 0.0),
            MyData::new(13.0, 1.5, 0.0),
            MyData::new(14.0, 1.5, 0.0),
            MyData::new(15.0, 1.0, 0.0),
            MyData::new(16.0, 1.0, 0.0),
            MyData::new(17.0, 1.0, 0.0),
            MyData::new(18.0, 1.0, 0.0),
            MyData::new(19.0, 1.0, 0.0),
            MyData::new(20.0, 1.0, 0.0),
            MyData::new(21.0, 1.0, 0.0),
            MyData::new(22.0, 1.0, 0.0),
            MyData::new(23.0, 1.0, 0.0),
            MyData::new(24.0, 1.0, 0.0),
            MyData::new(25.0, 0.5, 0.0),
            MyData::new(26.0, 0.5, 0.0),
            MyData::new(27.0, 0.5, 0.0),
            MyData::new(28.0, 0.5, 0.0),
            MyData::new(29.0, 0.5, 0.0),
            MyData::new(30.0, 0.5, 0.0),
            MyData::new(31.0, 0.5, 0.0),
            MyData::new(32.0, 0.5, 0.0),
            MyData::new(33.0, 0.5, 0.0),
            MyData::new(34.0, 0.5, 0.0),
            MyData::new(35.0, 0.1, 0.0),
            MyData::new(36.0, 0.1, 0.0),
            MyData::new(37.0, 0.1, 0.0),
            MyData::new(38.0, 0.1, 0.0),
            MyData::new(39.0, 0.1, 0.0),
            MyData::new(40.0, 0.5, 0.0),
            MyData::new(41.0, 0.5, 0.0),
            MyData::new(42.0, 0.5, 0.0),
            MyData::new(43.0, 0.5, 0.0),
            MyData::new(44.0, 0.5, 0.0),
            MyData::new(45.0, 1.0, 0.0),
            MyData::new(46.0, 2.0, 0.0),
            MyData::new(47.0, 3.0, 0.0),
            MyData::new(48.0, 4.0, 0.0),
            MyData::new(49.0, 5.0, 0.0),
            MyData::new(50.0, 3.0, 3.0),
            MyData::new(51.0, 0.0, 5.0),
            MyData::new(52.0, 0.0, 4.0),
            MyData::new(53.0, 0.0, 3.0),
            MyData::new(54.0, 0.0, 2.0),
            MyData::new(55.0, 0.0, 1.0),
            MyData::new(56.0, 0.0, 0.5),
            MyData::new(57.0, 0.0, 0.5),
            MyData::new(58.0, 0.0, 0.5),
            MyData::new(59.0, 0.0, 0.5),
            MyData::new(60.0, 0.0, 0.5),
            MyData::new(61.0, 0.0, 0.1),
            MyData::new(62.0, 0.0, 0.1),
            MyData::new(63.0, 0.0, 0.1),
            MyData::new(64.0, 0.0, 0.1),
            MyData::new(65.0, 0.0, 0.1),
            MyData::new(66.0, 0.0, 0.0),
            MyData::new(67.0, 0.0, 0.0),
            MyData::new(68.0, 0.0, 0.0),
            MyData::new(69.0, 0.0, 0.0),
            MyData::new(70.0, 0.0, 3.0),
            MyData::new(71.0, 0.0, 3.0),
            MyData::new(72.0, 0.0, 3.0),
            MyData::new(73.0, 0.0, 3.0),
            MyData::new(74.0, 0.0, 3.0),
            MyData::new(75.0, 0.0, 3.0),
            MyData::new(76.0, 0.0, 3.0),
            MyData::new(77.0, 0.0, 3.0),
            MyData::new(78.0, 0.0, 3.0),
            MyData::new(79.0, 0.0, 3.0),
            MyData::new(80.0, 0.0, 3.0),
            MyData::new(81.0, 0.0, 3.1),
            MyData::new(82.0, 0.0, 3.1),
            MyData::new(83.0, 0.0, 3.1),
            MyData::new(84.0, 0.0, 3.1),
            MyData::new(85.0, 0.0, 3.1),
            MyData::new(86.0, 0.0, 3.1),
            MyData::new(87.0, 0.0, 3.1),
            MyData::new(88.0, 0.0, 3.1),
            MyData::new(89.0, 0.0, 3.1),
            MyData::new(90.0, 0.0, 3.1),
        ]
    })
}

#[component]
pub fn LiquidityChart(debug: Signal<bool>, data: Signal<Vec<MyData>>) -> impl IntoView {
    let series = Series::new(|data: &MyData| data.x)
        .with_min_y(0.00)
        .with_colours([
            Colour::from_rgb(246, 193, 119),
            Colour::from_rgb(49, 116, 143),
        ])
        .bar(
            Bar::new(|data: &MyData| data.y1).with_name("Token Y"), // .with_group_gap(0.05)
                                                                    // .with_gap(0.1),
        )
        .bar(
            Bar::new(|data: &MyData| data.y2).with_name("Token X"), // .with_group_gap(0.05)
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
