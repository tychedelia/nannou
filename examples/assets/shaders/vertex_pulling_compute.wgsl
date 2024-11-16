struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    color: vec4<f32>,
};

struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0) var<storage, read_write> vertices: array<Vertex>;
@group(0) @binding(1) var<uniform> sphere_center: vec4<f32>;
@group(0) @binding(2) var<uniform> sphere_radius: f32;
@group(0) @binding(3) var<uniform> cube_resolution: vec3<u32>;
@group(0) @binding(4) var<storage, read_write> indirect_args: DrawIndirectArgs;

fn sdf_sphere(point: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(point - center) - radius;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {

}

@compute @workgroup_size(64)
fn init(@builtin(global_invocation_id) global_id: vec3<u32>) {

}