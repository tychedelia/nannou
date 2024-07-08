use nannou::noise::NoiseFn;
use nannou::prelude::*;

fn main() {
    nannou::app(model)
        // Register our custom material to make it available for use in our drawing
        .init_custom_material::<VideoMaterial>()
        .run();
}

#[derive(Reflect)]
struct Model {
    window: Entity,
    camera: Entity,
    video: Handle<Video>,
}

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
struct VideoMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl Material for VideoMaterial {
    fn fragment_shader() -> ShaderRef {
        "draw_video_material.wgsl".into()
    }
}

fn model(app: &App) -> Model {
    let camera = app.new_camera().build();
    let window = app
        .new_window()
        .camera(camera)
        .primary()
        .size_pixels(1024, 512)
        .view(view)
        .build();

    let video = app.asset_server().load_with_settings(
        "video/file_example_MP4_640_3MG.mp4",
        |settings: &mut VideoLoaderSettings| {
            settings
                .options
                .insert("hwaccel".to_string(), "sasfd".to_string());
        },
    );
    Model {
        window,
        camera,
        video,
    }
}

fn view(app: &App, model: &Model) {
    let Some(video) = app.assets().get(&model.video) else {
        return;
    };

    let draw = app
        .draw()
        // Initialize our draw instance with our custom material
        .material(VideoMaterial { texture: video.texture.clone() });

    draw.rect()
        .w_h(640.0, 400.0);
}