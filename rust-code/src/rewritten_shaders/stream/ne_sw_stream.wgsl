struct Dimensions{
    row: u32,
    col: u32,
    total: u32,
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read_write> ne: array<f32>;
@group(1) @binding(1) var<storage, read_write> sw: array<f32>;

@group(2) @binding(0) var<storage, read_write> post_ne: array<f32>;
@group(2) @binding(1) var<storage, read_write> post_sw: array<f32>;

@group(3) @binding(0) var<storage,read_write> barrier: array<u32>;
fn ne_index(index: u32) -> u32{
   return index + 1u - dimensions.row;
}

fn sw_index(index: u32) -> u32{
   return index - 1u + dimensions.row;
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

    let ne_index = ne_index(index);
    let sw_index = sw_index(index);


    //update ne
    if(barrier[sw_index] == 1u){
        post_ne[index] = sw[index];
    } else{
        post_ne[index] = ne[sw_index];
    }

    //update sw
    if(barrier[ne_index] == 1u){
        post_sw[index] = ne[index];
    } else{
        post_sw[index] = sw[ne_index];
    }
}