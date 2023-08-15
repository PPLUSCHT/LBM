@group(0) @binding(0) var<storage, read_write> colors: array<vec3<f32>>;

@group(1) @binding(0) var<storage, read_write> value: array<f32>;

@group(2) @binding(0) var<storage, read_write> barrier: array<u32>;

@group(3) @binding(0) var<uniform> size: u32;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>){
    if (global_invocation_id.x >= size) {
        return;
    }
    
    let color = clamp(20.0 * value[global_invocation_id.x],-4.0, 4.0);
    let color_block = i32(floor(color));

    switch color_block{
        case -4: {
            let right_weight = 4.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight) * vec3(0.0,0.0,0.5) + right_weight * vec3(0.0, 0.0, 1.0);
        }
        case -3: {
            let right_weight = 3.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.0,0.0,1.0) + right_weight * vec3(0.0, 0.5, 1.0);
        }
        case -2: {
            let right_weight = 2.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.0, 0.5, 1.0) + right_weight * vec3(0.0, 1.0, 1.0);
        }
        case -1: {
            let right_weight = 1.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.0, 1.0, 1.0) + right_weight * vec3(0.5, 1.0, 0.5);
        }
        case 0: {
            let right_weight = 0.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.5, 1.0, 0.5) + right_weight * vec3(1.0, 1.0, 0.0);
        }
        case 1: {
            let right_weight = -1.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(1.0, 1.0, 0.0) + right_weight * vec3(1.0, 0.5, 0.0);
        }
        case 2: {
            let right_weight = -2.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(1.0, 0.5, 0.0) + right_weight * vec3(1.0, 0.0, 0.0);
        }
        case 3: {
            let right_weight = -3.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(1.0, 0.0, 0.0) + right_weight * vec3(0.5, 0.0, 0.0);
        }
        default: {
            colors[global_invocation_id.x] = vec3(0.5, 0.0, 0.0);
        }
    }
    if(barrier[global_invocation_id.x] == 1u){
        colors[global_invocation_id.x] = vec3(0.0,0.0,0.0);
    }
}