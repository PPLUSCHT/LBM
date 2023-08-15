struct Dimensions{
    row: u32,
    col: u32,
    total: u32,
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read_write> ux: array<f32>;
@group(1) @binding(1) var<storage, read_write> uy: array<f32>;
@group(1) @binding(2) var<storage, read_write> rho: array<f32>;

@group(2) @binding(0) var<storage, read_write> output: array<f32>;

@compute
@workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

   let index = global_invocation_id.x;

   if(index > dimensions.total - 1u){
        return;
   }
   output[index] =  4.0 * clamp(0.15 * rho[index], 0.0, 1.0) - 0.5;
}