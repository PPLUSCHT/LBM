struct Update{
    location: u32,
    value: u32,
}

@group(0) @binding(0) var<uniform> num_updates: u32;
@group(0) @binding(1) var<storage, read> updates: array<Update>;

@group(1) @binding(0) var<storage, read_write> barrier: array<u32>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>){
    if(id.x >= num_updates){
        return;
    }
    barrier[updates[id.x].location] = updates[id.x].value;
}