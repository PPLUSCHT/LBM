struct Dimensions{
    row: u32,
    col: u32,
    total: u32,
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read_write> nw: array<f32>;
@group(1) @binding(1) var<storage, read_write> se: array<f32>;

@group(2) @binding(0) var<storage, read_write> post_nw: array<f32>;
@group(2) @binding(1) var<storage, read_write> post_se: array<f32>;

@group(3) @binding(0) var<storage,read_write> barrier: array<u32>;

fn nw_index(index: u32) -> u32{
   return index - 1u - dimensions.row;
}

fn se_index(index: u32) -> u32{
   return index + 1u + dimensions.row; 
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

    let nw_index = nw_index(index);
    let se_index = se_index(index);

    //update nw
    if(barrier[se_index] == 1u){
        post_nw[index] = se[index];
    } else{
        post_nw[index] = nw[se_index];
    }

    //update se
    if(barrier[nw_index] == 1u){
        post_se[index] = nw[index];
    } else{
        post_se[index] = se[nw_index];
    }
}