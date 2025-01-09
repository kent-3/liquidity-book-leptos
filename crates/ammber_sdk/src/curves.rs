use crate::constants::liquidity_config::LiquidityConfigurations;

// TODO: decide on which curves to use

pub fn uniform_radius_distribution(radius: u32) -> LiquidityConfigurations {
    let radius = radius as i64;

    let delta_ids: Vec<i64> = (-radius..=radius).collect();
    let len = delta_ids.len();

    let weight = 1.0 / (radius as f64 + 0.5); // 0.5 for the delta_id == 0 case
    let truncated_weight = (weight * 1_000_000.0).trunc() / 1_000_000.0;
    let half_weight = ((weight / 2.0) * 1_000_000.0).trunc() / 1_000_000.0;

    let mut distribution_x = vec![0.0; len];
    let mut distribution_y = vec![0.0; len];

    for (i, &delta_id) in delta_ids.iter().enumerate() {
        if delta_id > 0 {
            // Positive delta_ids contribute to distribution_x
            distribution_x[i] = truncated_weight;
        } else if delta_id < 0 {
            // Negative delta_ids contribute to distribution_y
            distribution_y[i] = truncated_weight;
        } else {
            // delta_id == 0, so split the weight equally
            distribution_x[i] = half_weight;
            distribution_y[i] = half_weight;
        }
    }

    LiquidityConfigurations::new(delta_ids, distribution_x, distribution_y)
}

// Function to calculate the y-values using the exponential function directly on the index
fn exponential_growth_curve(index: usize, b: f64) -> f64 {
    (index as f64 * b).exp() // Exponential growth: e^(b * index)
}

use std::f64::consts::E;

fn piecewise_linear(x: f64, x_index: f64, slope: f64, intercept: f64) -> f64 {
    if x <= x_index {
        // Positive slope until x_index
        slope * x + intercept
    } else {
        // Reverse slope after x_index
        -slope * (x - 2.0 * x_index) + intercept
    }
}
fn growth_curve(x: f64, b: f64) -> f64 {
    E.powf(b * x) // Exponential growth: e^(bx)
}

fn decay_curve(x: f64, b: f64) -> f64 {
    E.powf(-b * x) // Exponential decay: e^(-bx)
}

fn logistic_curve(x: f64, x0: f64, k: f64) -> f64 {
    1.0 / (1.0 + E.powf(-k * (x - x0)))
}

// Derivative of the logistic function (bell-shaped curve)
fn logistic_derivative(x: f64, k: f64) -> f64 {
    let exp_kx = E.powf(k * x);
    exp_kx / ((1.0 + exp_kx).powi(2))
}

pub fn curve_radius_distribution(radius: u32) -> LiquidityConfigurations {
    let radius = radius as i64;
    let delta_ids: Vec<i64> = (-radius..=radius).collect();
    let len = delta_ids.len();

    // Initialize the distributions
    let mut distribution_x = vec![0.0; len];
    let mut distribution_y = vec![0.0; len];

    let slope = 1.0; // Positive slope before the midpoint
    let intercept = 0.7; // Starting y-value (at x = -radius)

    for (i, &delta_id) in delta_ids.iter().enumerate() {
        if delta_id > 0 {
            // Positive delta_ids contribute to distribution_x
            distribution_x[i] = piecewise_linear(i as f64, radius as f64, slope, intercept);
        } else if delta_id < 0 {
            // Negative delta_ids contribute to distribution_y
            distribution_y[i] = piecewise_linear(i as f64, radius as f64, slope, intercept);
        } else {
            // delta_id == 0, so split the weight equally
            distribution_x[i] = piecewise_linear(i as f64, radius as f64, slope, intercept) / 2.0;
            distribution_y[i] = piecewise_linear(i as f64, radius as f64, slope, intercept) / 2.0;
        }
    }

    // let midpoint = radius as f64; // Set the center of the curve to `radius`
    // let k = 0.7_f64;
    //
    // Apply the logistic derivative, shifting by the midpoint
    // for i in 0..len {
    //     let x = i as f64 - midpoint;
    //     curve[i] = logistic_derivative(x, k);
    // }
    // println!("{:?}", curve);
    //
    // for (i, &delta_id) in delta_ids.iter().enumerate() {
    //     let x = i as f64 - midpoint;
    //     if delta_id > 0 {
    //         // Positive delta_ids contribute to distribution_x
    //         distribution_x[i] = logistic_derivative(x, k);
    //     } else if delta_id < 0 {
    //         // Negative delta_ids contribute to distribution_y
    //         distribution_y[i] = logistic_derivative(x, k);
    //     } else {
    //         // delta_id == 0, so split the weight equally
    //         distribution_x[i] = logistic_derivative(x, k) / 2.0;
    //         distribution_y[i] = logistic_derivative(x, k) / 2.0;
    //     }
    // }

    // Normalize both distributions to sum to 1
    let x_total: f64 = distribution_x.iter().sum();
    let y_total: f64 = distribution_y.iter().sum();
    distribution_x.iter_mut().for_each(|x| *x /= x_total);
    distribution_x
        .iter_mut()
        .for_each(|x| *x = (*x * 1_000_000.0).trunc() / 1_000_000.0);
    distribution_y.iter_mut().for_each(|y| *y /= y_total);
    distribution_y
        .iter_mut()
        .for_each(|y| *y = (*y * 1_000_000.0).trunc() / 1_000_000.0);

    LiquidityConfigurations::new(delta_ids, distribution_x, distribution_y)
}

// fn main() {
//     let (delta_ids, distribution_x, distribution_y) = curve_radius_distribution(5);
//
//     let x_sum: f64 = distribution_x.iter().sum();
//     let y_sum: f64 = distribution_y.iter().sum();
//     println!("Distribution X: {:?}", x_sum);
//     println!("Distribution Y: {:?}", y_sum);
//
//     println!("Delta IDs: {:?}", delta_ids);
//     println!("Distribution X: {:?}", distribution_x);
//     println!("Distribution Y: {:?}", distribution_y);
// }

fn simple_symmetric_curve(radius: usize) -> Vec<f64> {
    let len = 2 * radius + 1;
    let mut distribution = vec![0.0; len];
    let shift = 0.15; // This will shift the curve up so that the starting and ending values are non-zero

    // Create increasing values up to the midpoint, and then mirror for the decreasing part
    for i in 0..=radius {
        let value = (i as f64 + 1.0) / (radius as f64 + 1.0); // Linearly increasing
        distribution[i] = value + shift;
        distribution[len - 1 - i] = value; // Mirror the values for the second half
    }

    // Normalize the distribution to sum to 2
    let total: f64 = distribution.iter().sum();
    distribution.iter_mut().for_each(|x| *x *= 2.0 / total);

    distribution
}

fn quadratic_symmetric_curve(radius: usize) -> Vec<f64> {
    let len = 2 * radius + 1;
    let mut distribution = vec![0.0; len];
    let midpoint = radius as f64;
    let shift = 0.15; // This will shift the curve up so that the starting and ending values are non-zero

    // Create a symmetrical quadratic distribution and shift the values up
    for i in 0..len {
        let x = i as f64;
        let value = 1.0 - ((x - midpoint) / midpoint).powi(2); // Quadratic shape
        distribution[i] = value + shift; // Shift the curve upwards
    }

    // Normalize the distribution to sum to 2
    let total: f64 = distribution.iter().sum();
    distribution.iter_mut().for_each(|y| *y *= 2.0 / total);
    distribution
        .iter_mut()
        .for_each(|y| *y = (*y * 1_000_000.0).trunc() / 1_000_000.0);

    distribution
}

fn main() {
    let radius = 5;
    let distribution = quadratic_symmetric_curve(radius);
    println!("quadratic_symmetric_curve: {:?}", distribution);

    let distribution = simple_symmetric_curve(radius);
    println!("simple_symmetric_curve: {:?}", distribution);

    let distribution = curve_radius_distribution(radius as u32);
    println!("curve_radius_distribution: {:?}", distribution);
}
