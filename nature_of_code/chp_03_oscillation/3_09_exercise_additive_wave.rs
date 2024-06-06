// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Additive Wave
// Create a more complex wave by adding two waves together.

// Maybe better for this answer to be OOP???.

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    x_spacing: f32,      // How far apart should each horizontal position be spaced
    max_waves: usize,    // total # of waves to add together
    theta: f32,          // Start angle at 0
    amplitude: Vec<f32>, // Height of the wave
    dx: Vec<f32>, // Value for incementing X, to be calculated as a function of period and x_spacing
    y_values: Vec<f32>, // Using a vector to store the height values for the wave (not entirely necessary)
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();

    let x_spacing = 8.0;
    let w = app.window_rect().w() + 16.0; // Width of entire wave
    let max_waves = 5;
    let mut amplitude = Vec::new();
    let mut dx = Vec::new();
    let y_values = vec![0.0; (w / x_spacing) as usize];

    for _ in 0..max_waves {
        amplitude.push(random_range(10.0, 30.0));
        let period = random_range(100.0, 300.0); // How many pixels before the wave repeats
        dx.push(((PI * 2.0) / period) * x_spacing);
    }
    Model {
        x_spacing,
        max_waves,
        theta: 0.0,
        amplitude,
        dx,
        y_values,
    }
}

fn update(_app: &App, m: &mut Model) {
    // Increment theta (try different values for 'angular velocity' here
    m.theta += 0.02;

    // Set all height values to zero
    for i in 0..m.y_values.len() {
        m.y_values[i] = 0.0;
    }

    // Accumulate wave height values
    for j in 0..m.max_waves {
        let mut x = m.theta;
        for i in 0..m.y_values.len() {
            // Every other wave is cosine instead of sine
            if j % 2 == 0 {
                m.y_values[i] += x.sin() * m.amplitude[j];
            } else {
                m.y_values[i] += x.cos() * m.amplitude[j];
            }
            x += m.dx[j];
        }
    }
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(BLACK);

    // A simple way to draw the wave with an ellipse at each position
    for x in 0..m.y_values.len() {
        draw.ellipse()
            .x_y(
                app.window_rect().left() + x as f32 * m.x_spacing,
                m.y_values[x],
            )
            .w_h(16.0, 16.0)
            .rgba(1.0, 1.0, 1.0, 0.25);
    }



}
