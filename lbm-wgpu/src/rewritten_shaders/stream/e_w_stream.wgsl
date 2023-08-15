struct Dimensions{
    row: u32,
    col: u32,
    total: u32,
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read_write> e: array<f32>;
@group(1) @binding(1) var<storage, read_write> w: array<f32>;

@group(2) @binding(0) var<storage, read_write> post_e: array<f32>;
@group(2) @binding(1) var<storage, read_write> post_w: array<f32>;

@group(3) @binding(0) var<storage,read_write> barrier: array<u32>;

fn w_index(index: u32) -> u32{
   return index - 1u;
}

fn e_index(index: u32) -> u32{
   return index + 1u;
}

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let index = global_invocation_id.x;

    if (barrier[index] == 1u){
        return;
    }

    if(index > dimensions.total - 1u){
        return;
    }

    if(index % dimensions.row == 0u){
        return;
    }

    if(index / dimensions.row >= dimensions.col - 1u){
        return;
    }

    let e_index = e_index(index);
    let w_index = w_index(index);

    //update w
    if(barrier[e_index] == 1u){
        post_w[index] = e[index];
    } else{
        post_w[index] = w[e_index];
    }

    //update e
    if(barrier[w_index] == 1u){
        post_e[index] = w[index];
    } else{
        post_e[index] = e[w_index];
    }

}