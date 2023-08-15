@group(0) @binding(0) var<storage, read_write> bad_nw: array<f32>;
@group(0) @binding(1) var<storage, read_write> bad_se: array<f32>;

@group(1) @binding(0) var<storage, read_write> nw: array<f32>;
@group(1) @binding(1) var<storage, read_write> ne: array<f32>;
@group(1) @binding(2) var<storage, read_write> se: array<f32>;
@group(1) @binding(3) var<storage, read_write> sw: array<f32>;

@group(2) @binding(0) var<storage, read_write> output: array<f32>;

@group(3) @binding(0) var<uniform> size: u32;

@compute
@workgroup_size(256)
fn main(builtin(global_invocation_id): global_invocation_id){
    if global_invocation_id.x >= size){
        return;
    }
    output[global_invocation_id.x] = bad_nw[global_invocation_id.x] - se[global_invocation_id.x];
}