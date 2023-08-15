@group(0) @binding(0) var<storage, read_write> n: array<f32>;
@group(0) @binding(1) var<storage, read_write> s: array<f32>;

@group(1) @binding(0) var<storage, read_write> e: array<f32>;
@group(1) @binding(1) var<storage, read_write> w: array<f32>;

@group(2) @binding(0) var<storage, read_write> ux: array<f32>;
@group(2) @binding(1) var<storage, read_write> uy: array<f32>;
@group(2) @binding(2) var<storage, read_write> rho: array<f32>;

@group(3) @binding(0) var<uniform> size: u32;
@group(3) @binding(1) var<uniform> omega: f32;
@group(3) @binding(2) var<storage, read_write> origin: array<f32>;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    if (global_invocation_id.x > size - 1u){
        return;
    }

    let i = global_invocation_id.x;

    let thisrho = rho[i];
	let thisux = ux[i] / thisrho;
	let thisuy = uy[i] / thisrho;

	let one9thrho = 1.0/9.0 * thisrho;

	let ux3 = 3.0 * thisux;
	let uy3 = 3.0 * thisuy;
	let ux2 = thisux * thisux;
	let uy2 = thisuy * thisuy;
	let uxuy2 = 2.0 * thisux * thisuy;
	let u2 = ux2 + uy2;
	let u215 = 1.5 * u2;

	origin[i]  += omega * (4.0/9.0*thisrho * (1.0                        - u215) - origin[i]);
	e[i]  += omega * (   one9thrho * (1.0 + ux3       + 4.5*ux2        - u215) - e[i]);
	w[i]  += omega * (   one9thrho * (1.0 - ux3       + 4.5*ux2        - u215) - w[i]);
	n[i]  += omega * (   one9thrho * (1.0 + uy3       + 4.5*uy2        - u215) - n[i]);
	s[i]  += omega * (   one9thrho * (1.0 - uy3       + 4.5*uy2        - u215) - s[i]);

}
