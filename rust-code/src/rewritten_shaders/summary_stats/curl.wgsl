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

fn n_index(index: u32) -> u32{
   return index - dimensions.row;
}

fn w_index(index: u32) -> u32{
   return index - 1u;
}

fn e_index(index: u32) -> u32{
   return index + 1u;
}

fn s_index(index: u32) -> u32{
   return index + dimensions.row;
}

@compute
@workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

   let index = global_invocation_id.x;

    if(index > dimensions.total - 1u){
        return;
    }

    if(index % dimensions.row == 0u){
        return;
    }

    if(index / dimensions.row >= dimensions.col - 1u){
        return;
    } 

   output[index] = 10.0 * (uy[index + 1u] - uy[index - 1u] - ux[n_index(index)] + ux[s_index(index)])/rho[index];
}