use nannou::prelude::mesh::cube::Cube;
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run()
}

struct Model {
    cube: Entity,
    camera: Entity,
    light: Entity,
}

fn model(app: &App) -> Model {
    let camera = app.new_camera().build();
    let light = app.new_light().color(WHITE).build();
    let _window = app
        .new_window::<Model>()
        .primary()
        .light(light)
        .size(800, 800)
        .camera(camera)
        .build();

    let cube = app.geom().cuboid();
    Model { cube, camera, light }
}

fn update(app: &App, model: &mut Model) {
    let cube = app.geom().get::<Cube>(model.cube);
    let camera = app.camera(model.camera);

    // Animate Cube
    let x = app.time().sin() * 2.0;
    let hue = app.time() % 1.0;
    let color = Color::hsl(hue * 360.0, 1.0, 0.5);

    cube.x(x)
        .x_length(app.time().sin().abs() * 2.0)
        .turns(Vec3::splat(app.time().sin()))
        .base_color(color);

    // Animate Camera
    let radius = 10.0;
    let speed = 2.0;

    let cam_x = radius * (app.time() * speed).cos();
    let cam_z = radius * (app.time() * speed).sin();
    let cam_y = 4.5;

    camera
        .x_y_z(cam_x, cam_y, cam_z)
        .look_at(Vec3::ZERO, Vec3::Y);

    // Animate light
    let light = app.light(model.light);
    light.x_y_z(cam_z, cam_x, cam_y);
}