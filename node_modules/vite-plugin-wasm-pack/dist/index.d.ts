import { PluginOption } from 'vite';
/**
 *   return a Vite plugin for handling wasm-pack crate
 *
 *   only use local crate
 *
 *   import wasmPack from 'vite-plugin-wasm-pack';
 *
 *   plugins: [wasmPack(['./my-local-crate'])]
 *
 *   only use npm crate, leave the first param to an empty array
 *
 *   plugins: [wasmPack([],['test-npm-crate'])]
 *
 *   use both local and npm crate
 *
 *   plugins: [wasmPack(['./my-local-crate'],['test-npm-crate'])]
 *
 * @param crates local crates paths, if you only use crates from npm, leave an empty array here.
 * @param moduleCrates crates names from npm
 */
declare function vitePluginWasmPack(crates: string[] | string, moduleCrates?: string[] | string): PluginOption;
declare namespace vitePluginWasmPack {
    var _a: typeof vitePluginWasmPack;
    export { _a as default };
}
export default vitePluginWasmPack;
