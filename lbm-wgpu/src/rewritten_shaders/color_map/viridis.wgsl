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
    
    let color = clamp(15.0 * value[global_invocation_id.x],-2.0, 2.0);
    let color_block = i32(floor(color));

    switch color_block{
        case -2: {
            let right_weight = 2.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.9921875, 0.90625, 0.1484375) + right_weight * vec3(0.3671875, 0.7890625, 0.3828125);
        }
        case -1: {
            let right_weight = 1.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.3671875, 0.7890625, 0.3828125) + right_weight * vec3(0.1328125, 0.56640625, 0.55078125);
        }
        case 0: {
            let right_weight = 0.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.1328125, 0.56640625, 0.55078125) + right_weight * vec3(0.23046875, 0.32421875, 0.546875);
        }
        case 1: {
            let right_weight = -1.0 + color;
            colors[global_invocation_id.x] = (1 - right_weight)  * vec3(0.23046875, 0.32421875, 0.546875) + right_weight * vec3(0.265625, 0.0078125, 0.33203125);
        }
        default: {
            colors[global_invocation_id.x] = vec3(0.265625, 0.0078125, 0.33203125);
        }
    }
    if(barrier[global_invocation_id.x] == 1u){
        colors[global_invocation_id.x] = vec3(0.0,0.0,0.0);
    }
}