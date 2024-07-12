use nannou::prelude::*;

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

fn main() {
    nannou::app(model).update_script("script.js").run();
}

#[derive(Reflect)]
struct Model {
    radius: f32,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let window = app
        .new_window()
        .title("Nannou + Egui")
        .size(WIDTH as u32, HEIGHT as u32)
        .view(view) // The function that will be called for presenting graphics to a frame.
        .build();

    Model {
        radius: 40f32,
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();

    draw.background().color(BLACK);

    draw.ellipse()
        .x_y(100.0, 100.0)
        .radius(model.radius);
}
