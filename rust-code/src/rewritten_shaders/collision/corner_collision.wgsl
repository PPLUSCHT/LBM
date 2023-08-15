@group(0) @binding(0) var<storage, read_write> ne: array<f32>;
@group(0) @binding(1) var<storage, read_write> sw: array<f32>;

@group(1) @binding(0) var<storage, read_write> nw: array<f32>;
@group(1) @binding(1) var<storage, read_write> se: array<f32>;

@group(2) @binding(0) var<storage, read_write> ux: array<f32>;
@group(2) @binding(1) var<storage, read_write> uy: array<f32>;
@group(2) @binding(2) var<storage, read_write> rho: array<f32>;

@group(3) @binding(0) var<uniform> size: u32;
@group(3) @binding(1) var<uniform> omega: f32;
@group(3) @binding(2) var<storage, read_write> origin: array<f32>;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>){
    if (global_invocation_id.x > size - 1u){
        return;
    }

    let i = global_invocation_id.x;

	rho[i] = rho[i] + origin[i];

    let thisrho = rho[i];
	let thisux = ux[i] / thisrho;
	let thisuy = uy[i] / thisrho;
		
	let one36thrho = 1.0/36.0 * thisrho;
	let ux3 = 3.0 * thisux;
	let uy3 = 3.0 * thisuy;
	let ux2 = thisux * thisux;
	let uy2 = thisuy * thisuy;
	let uxuy2 = 2.0 * thisux * thisuy;
	let u2 = ux2 + uy2;
	let u215 = 1.5 * u2;

	ne[i] += omega * (  one36thrho * (1.0 + ux3 + uy3 + 4.5*(u2+uxuy2) - u215) - ne[i]);
    se[i] += omega * (  one36thrho * (1.0 + ux3 - uy3 + 4.5*(u2-uxuy2) - u215) - se[i]);
	nw[i] += omega * (  one36thrho * (1.0 - ux3 + uy3 + 4.5*(u2-uxuy2) - u215) - nw[i]);
	sw[i] += omega * (  one36thrho * (1.0 - ux3 - uy3 + 4.5*(u2+uxuy2) - u215) - sw[i]);
}