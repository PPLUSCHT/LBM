@group(0) @binding(0) var<storage, read_write> bad_n: array<f32>;
@group(0) @binding(1) var<storage, read_write> bad_s: array<f32>;

@group(1) @binding(0) var<storage, read_write> n: array<f32>;
@group(1) @binding(1) var<storage, read_write> e: array<f32>;
@group(1) @binding(2) var<storage, read_write> s: array<f32>;
@group(1) @binding(3) var<storage, read_write> w: array<f32>;

@group(2) @binding(0) var<storage, read_write> output: array<f32>;

@group(3) @binding(0) var<uniform> size: u32;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>){
    if (global_invocation_id.x >= size){
        return;
    }
    output[global_invocation_id.x] = bad_n[global_invocation_id.x] - n[global_invocation_id.x];
}