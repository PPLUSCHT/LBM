/* tslint:disable */
/* eslint-disable */
/**
* @param {number} pixel_ratio
* @param {number} res
* @param {number} width
* @param {number} height
*/
export function run(pixel_ratio: number, res: number, width: number, height: number): void;
/**
*/
export enum SummaryStat {
  Curl = 0,
  Ux = 1,
  Uy = 2,
  Rho = 3,
  Speed = 4,
}
/**
*/
export enum ColorMap {
  Inferno = 0,
  Viridis = 1,
  Jet = 2,
}
/**
*/
export enum Preset {
  Welcome = 0,
}
/**
*/
export enum ClickType {
  Line = 0,
  Erase = 1,
  Draw = 2,
  Inactive = 3,
}
/**
*/
export enum FluidPreset {
  Equilibrium = 0,
  CustomSpeed = 1,
  SingleNorth = 2,
  SingleOrigin = 3,
  SingleEast = 4,
  SingleNorthEast = 5,
}
/**
*/
export enum BarrierPreset {
  Welcome = 0,
  Tunnel = 1,
  Curl = 2,
  Chaos = 3,
}
/**
*/
export enum Resolution {
  TEST = 1000,
  NHD = 230400,
  HD = 921600,
  FHD = 2073600,
  UHD = 3686400,
}
/**
*/
export class WASMInteraction {
  free(): void;
/**
* @param {number} summary_stat
*/
  static set_output(summary_stat: number): void;
/**
* @param {number} draw_type
*/
  static set_draw_type(draw_type: number): void;
/**
* @param {number} color_map
*/
  static set_color_map(color_map: number): void;
/**
*/
  static toggle_pause(): void;
/**
* @param {number} rate
*/
  static update_compute_rate(rate: number): void;
/**
* @param {number} viscosity
*/
  static update_viscosity(viscosity: number): void;
/**
* @param {number} speed
*/
  static update_flow_speed(speed: number): void;
/**
*/
  static undo(): void;
/**
*/
  static set_step_mode(): void;
/**
*/
  static release_step_mode(): void;
/**
*/
  static take_step(): void;
/**
* @param {number} f
*/
  static change_fluid_preset(f: number): void;
/**
* @param {number} b
*/
  static change_barrier_preset(b: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasminteraction_free: (a: number) => void;
  readonly wasminteraction_set_output: (a: number) => void;
  readonly wasminteraction_set_draw_type: (a: number) => void;
  readonly wasminteraction_set_color_map: (a: number) => void;
  readonly wasminteraction_toggle_pause: () => void;
  readonly wasminteraction_update_compute_rate: (a: number) => void;
  readonly wasminteraction_update_viscosity: (a: number) => void;
  readonly wasminteraction_update_flow_speed: (a: number) => void;
  readonly wasminteraction_undo: () => void;
  readonly wasminteraction_set_step_mode: () => void;
  readonly wasminteraction_release_step_mode: () => void;
  readonly wasminteraction_take_step: () => void;
  readonly wasminteraction_change_fluid_preset: (a: number) => void;
  readonly wasminteraction_change_barrier_preset: (a: number) => void;
  readonly run: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1f4d768bc1e31376: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hed42f79d279dcf99: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h728fd1657b0a0d34: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h385fb9ebf94aef22: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h345b747b894e8bc3: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
