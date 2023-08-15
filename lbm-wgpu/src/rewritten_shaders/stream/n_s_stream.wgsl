struct Dimensions{
    row: u32,
    col: u32,
    total: u32,
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read_write> n: array<f32>;
@group(1) @binding(1) var<storage, read_write> s: array<f32>;

@group(2) @binding(0) var<storage, read_write> post_n: array<f32>;
@group(2) @binding(1) var<storage, read_write> post_s: array<f32>;

@group(3) @binding(0) var<storage,read_write> barrier: array<u32>;

fn n_index(index: u32) -> u32{
   return index - dimensions.row;
}

fn s_index(index: u32) -> u32{
   return index + dimensions.row;
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

    let n_index = n_index(index);
    let s_index = s_index(index);

    //update n
    if(barrier[s_index] == 1u){
        post_n[index] = s[index];
    } else{
        post_n[index] = n[s_index];
    }

    //update s
    if(barrier[n_index] == 1u){
        post_s[index] = n[index];
    } else{
        post_s[index] = s[n_index];
    }
}