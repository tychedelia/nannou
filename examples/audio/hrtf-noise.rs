//! A minimal example to demonstrate the effect of HRTF (head-related transfer function) on a
//! real-time stream of audio. HRTF is a popular approach to achieving binaural audio.
//!
//! The example uses generated white noise as the sound source. The source position can be rotated
//! around the user by moving the mouse along the *x* axis.
//!
//! The example will fail if the default cpal output device under the default host offers less than
//! two channels.
//!
//! This effect is best experienced with headphones!

use hrtf::{HrirSphere, HrtfContext, HrtfProcessor};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

use nannou::prelude::*;
use nannou::rand::{rngs::SmallRng, Rng, SeedableRng};
use nannou_audio as audio;
use nannou_audio::Buffer;

fn main() {
    nannou::app(model).exit(exit).run();
}

pub enum AudioCommand {
    Play,
    Pause,
    SourcePosition(Point3),
    Exit,
}

struct Model {
    audio_thread: JoinHandle<()>,
    audio_tx: Sender<AudioCommand>,
    source_position: Point3,
    is_paused: bool,
}

// HRTF requires a fixed sample rate and "block length" (i.e. buffer length in frames).
const SAMPLE_RATE: u32 = 44_100;
const BUFFER_LEN_FRAMES: usize = 64;

// Taken from rg3d-sound.
const HRTF_BLOCK_LEN: usize = 513;
const HRTF_INTERPOLATION_STEPS: usize = 8;
const HRTF_BUFFER_LEN: usize = HRTF_BLOCK_LEN * HRTF_INTERPOLATION_STEPS;

// The radius in which we can hear the sound in points.
const LISTENING_RADIUS: f32 = 300.0;
const WINDOW_SIDE: u32 = (LISTENING_RADIUS * 2.0) as u32;

struct Audio {
    rng: SmallRng,
    hrtf_data: HrtfData,
    hrtf_processor: HrtfProcessor,
    source_position: Point3,
    prev_source_position: Point3,
}

// A set of buffers to re-use for HRTF processing.
struct HrtfData {
    source: Vec<f32>,
    output: Box<[(f32, f32); HRTF_BUFFER_LEN]>,
    prev_left_samples: Vec<f32>,
    prev_right_samples: Vec<f32>,
}

impl HrtfData {
    fn new() -> Self {
        HrtfData {
            source: vec![0.0; HRTF_BUFFER_LEN],
            output: Box::new([(0.0, 0.0); HRTF_BUFFER_LEN]),
            prev_left_samples: vec![0.0; HRTF_BUFFER_LEN],
            prev_right_samples: vec![0.0; HRTF_BUFFER_LEN],
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(WINDOW_SIDE, WINDOW_SIDE)
        .key_pressed(key_pressed)
        .mouse_moved(mouse_moved)
        .view(view)
        .build();

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Load a HRIR sphere and initialise the processor.
    let assets = app.assets_path();
    let hrir_sphere_path = assets.join("hrir").join("IRC_1002_C").with_extension("bin");
    let hrir_sphere = HrirSphere::from_file(hrir_sphere_path, SAMPLE_RATE)
        .expect("failed to load HRIR sphere from file");
    let hrtf_processor = HrtfProcessor::new(hrir_sphere, HRTF_INTERPOLATION_STEPS, HRTF_BLOCK_LEN);

    // Initialise the state that we want to live on the audio thread.
    let source_position = [0.0; 3].into();
    let audio_model = Audio {
        rng: SmallRng::seed_from_u64(0),
        hrtf_data: HrtfData::new(),
        hrtf_processor,
        source_position,
        prev_source_position: [0.0; 3].into(),
    };

    // Kick off the audio thread.
    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let audio_thread = thread::spawn(move || {
        let stream = audio_host
            .new_output_stream(audio_model)
            .render(audio)
            .channels(2)
            .sample_rate(SAMPLE_RATE)
            .frames_per_buffer(BUFFER_LEN_FRAMES)
            .build()
            .unwrap();

        stream.play().unwrap();

        loop {
            match audio_rx.recv() {
                Ok(AudioCommand::Play) => {
                    stream.play().unwrap();
                }
                Ok(AudioCommand::Pause) => {
                    stream.pause().unwrap();
                }
                Ok(AudioCommand::Exit) => {
                    stream.pause().ok();
                    break;
                }
                Ok(AudioCommand::SourcePosition(new_source_position)) => {
                    stream
                        .send(move |audio| {
                            audio.source_position = new_source_position;
                        })
                        .unwrap();
                }
                Err(_) => break,
            }
        }
    });

    Model {
        audio_thread,
        audio_tx,
        source_position,
        is_paused: false,
    }
}

// A function that renders the given `Audio` to the given `Buffer`.
fn audio(audio: &mut Audio, output: &mut Buffer) {
    // Silence the output buffers.
    for sample in output.iter_mut() {
        *sample = 0.0;
    }
    for sample in audio.hrtf_data.output.iter_mut() {
        *sample = (0.0, 0.0);
    }

    // Fill the source buffer with new noise.
    audio.hrtf_data.source.drain(..BUFFER_LEN_FRAMES);
    for _ in 0..BUFFER_LEN_FRAMES {
        let sample = audio.rng.gen::<f32>() * 2.0 - 1.0;
        audio.hrtf_data.source.push(sample);
    }

    // Calculate the distance based gain.
    let new_distance_gain = dist_gain(&audio.source_position);
    let prev_distance_gain = dist_gain(&audio.prev_source_position);

    // Apply the HRTF.
    let hrtf_ctxt = HrtfContext {
        source: &audio.hrtf_data.source[..],
        output: &mut audio.hrtf_data.output[..],
        new_sample_vector: (-audio.source_position).into(),
        prev_sample_vector: (-audio.prev_source_position).into(),
        prev_left_samples: &mut audio.hrtf_data.prev_left_samples,
        prev_right_samples: &mut audio.hrtf_data.prev_right_samples,
        new_distance_gain,
        prev_distance_gain,
    };
    audio.hrtf_processor.process_samples(hrtf_ctxt);

    // Update `prev` data with current data.
    audio.prev_source_position = audio.source_position;

    // Write the result to the output buffer.
    let hrtf_out = &audio.hrtf_data.output[HRTF_BUFFER_LEN - BUFFER_LEN_FRAMES..];
    for (out_f, &(l, r)) in output.frames_mut().zip(hrtf_out) {
        // Try not to scare the bajeezus out of anyone running the example.
        let volume = 0.1;
        out_f[0] = l * volume;
        out_f[1] = r * volume;
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    if key == KeyCode::Space {
        if model.is_paused {
            model.audio_tx.send(AudioCommand::Play).ok();
            model.is_paused = false;
        } else {
            model.audio_tx.send(AudioCommand::Pause).ok();
            model.is_paused = true;
        }
    }
}

fn mouse_moved(_app: &App, model: &mut Model, p: Point2) {
    let (x, y) = p.into();
    // Use the y axis of the mouse position for the z axis in space.
    let new_source_position = pt3(x, 0.0, y) / LISTENING_RADIUS;
    model.source_position = new_source_position;
    model
        .audio_tx
        .send(AudioCommand::SourcePosition(new_source_position))
        .ok();
}

fn exit(_app: &App, model: Model) {
    model.audio_tx.send(AudioCommand::Exit).ok();
    model.audio_thread.join().ok();
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(Color::srgb(0.1, 0.12, 0.13));

    // Listenable area.
    draw.ellipse().radius(LISTENING_RADIUS).srgb(0.1, 0.1, 0.1);

    // Draw the head.
    draw.ellipse().color(BLUE);
    draw.arrow()
        .color(BLACK)
        .weight(5.0)
        .points(pt2(0.0, -30.0), pt2(0.0, 30.0));
    draw.text("HEAD").color(WHITE);

    // Draw the source.
    let (x, y, z) = model.source_position.into();
    let text = format!("Noise Source:\n[{:.2}, {:.2}, {:.2}]", x, y, z);
    draw.text(&text).xy(app.mouse() + vec2(0.0, 20.0));
}

// Simple function for determining a gain based on the distance from the listener.
fn dist_gain(p: &Point3) -> f32 {
    let m = p.length();
    if m == 0.0 {
        1.0
    } else if m > 1.0 {
        0.0
    } else {
        1.0 - m
    }
    .powf(1.6)
    .min(1.0)
}
