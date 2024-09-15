use nannou::prelude::bevy_render::renderer::RenderDevice;
use nannou::prelude::bevy_render::storage::ShaderStorageBuffer;
use nannou::prelude::*;
use std::sync::Arc;

const NUM_PARTICLES: u32 = 100000;
const WORKGROUP_SIZE: u32 = 64;

fn main() {
    nannou::app(model)
        .compute(compute)
        .update(update)
        .shader_model::<ShaderModel>()
        .run();
}

pub enum Shape {
    Circle,
    Square,
    Triangle,
}

struct Model {
    particles: Handle<ShaderStorageBuffer>,
    shape: Shape,
    attract_strength: f32,
}

impl Model {
    fn material(&self) -> ShaderModel {
        ShaderModel {
            particles: self.particles.clone(),
        }
    }
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: Vec4,
}

#[derive(Default, Debug, Eq, PartialEq, Hash, Clone)]
enum State {
    #[default]
    Init,
    Update,
}

#[derive(AsBindGroup, Clone)]
struct ComputeModel {
    #[storage(0, visibility(compute))]
    particles: Handle<ShaderStorageBuffer>,
    #[uniform(1)]
    mouse: Vec2,
    #[uniform(2)]
    attract_strength: f32,
    #[uniform(3)]
    particle_count: u32,
}

impl Compute for ComputeModel {
    type State = State;

    fn shader() -> ShaderRef {
        "shaders/particle_mouse_compute.wgsl".into()
    }

    fn entry(state: &Self::State) -> &'static str {
        match state {
            State::Init => "init",
            State::Update => "update",
        }
    }

    fn dispatch_size(_state: &Self::State) -> (u32, u32, u32) {
        ((NUM_PARTICLES + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE, 1, 1)
    }
}

#[shader_model(
    fragment = "shaders/particle_mouse_material.wgsl",
    vertex = "shaders/particle_mouse_material.wgsl"
)]
struct ShaderModel {
    #[storage(0, read_only, visibility(vertex))]
    particles: Handle<ShaderStorageBuffer>,
}

fn model(app: &App) -> Model {
    let _window_id = app
        .new_window()
        .primary()
        .size(1024, 768)
        .view(view)
        .build();

    // Create a buffer to store the particles.
    let particle_size = Particle::min_size().get() as usize;
    let mut particles = ShaderStorageBuffer::with_size(
        NUM_PARTICLES as usize * particle_size * 2,
        RenderAssetUsages::RENDER_WORLD,
    );
    particles.buffer_description.label = Some("particles");
    particles.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::VERTEX;

    let particles = app.assets_mut().add(particles);

    Model {
        particles,
        shape: Shape::Circle,
        attract_strength: 1.0,
    }
}

fn update(app: &App, model: &mut Model) {
    if app.keys().just_pressed(KeyCode::ArrowLeft) {
        match model.shape {
            Shape::Circle => model.shape = Shape::Square,
            Shape::Square => model.shape = Shape::Triangle,
            Shape::Triangle => model.shape = Shape::Circle,
        }
    }
    if app.keys().just_pressed(KeyCode::ArrowRight) {
        match model.shape {
            Shape::Circle => model.shape = Shape::Triangle,
            Shape::Square => model.shape = Shape::Circle,
            Shape::Triangle => model.shape = Shape::Square,
        }
    }
    if app.keys().just_pressed(KeyCode::ArrowUp) {
        model.attract_strength += 1.0;
    }
    if app.keys().just_pressed(KeyCode::ArrowDown) {
        model.attract_strength -= 1.0;
    }
}

fn compute(app: &App, model: &Model, state: State, view: Entity) -> (State, ComputeModel) {
    let window = app.main_window();
    let window_rect = window.rect();

    let mouse_pos = app.mouse();
    let mouse_norm = Vec2::new(
        mouse_pos.x / window_rect.w() * 2.0,
        mouse_pos.y / window_rect.h() * 2.0,
    );

    let compute_model = ComputeModel {
        particles: model.particles.clone(),
        mouse: mouse_norm,
        attract_strength: model.attract_strength,
        particle_count: NUM_PARTICLES,
    };

    match state {
        State::Init => (State::Update, compute_model),
        State::Update => (State::Update, compute_model),
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(GRAY);

    let draw = draw.material(model.material());
    match model.shape {
        Shape::Circle => {
            draw_particles_circle(&draw);
        }
        Shape::Square => {
            draw_particles_square(&draw);
        }
        Shape::Triangle => {
            draw_particles_triangle(&draw);
        }
    }
}

fn draw_particles_circle(draw: &Draw<ShaderModel>) {
    draw.instanced()
        .primitive(draw.ellipse().w_h(5.0, 5.0))
        .range(0..NUM_PARTICLES);
}

fn draw_particles_square(draw: &Draw<ShaderModel>) {
    draw.instanced()
        .primitive(draw.rect().w_h(5.0, 5.0))
        .range(0..NUM_PARTICLES);
}

fn draw_particles_triangle(draw: &Draw<ShaderModel>) {
    draw.instanced()
        .primitive(draw.tri().w_h(5.0, 5.0))
        .range(0..NUM_PARTICLES);
}