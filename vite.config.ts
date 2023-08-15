import { defineConfig } from 'vite'
import wasmPack from 'vite-plugin-wasm-pack'

export default defineConfig({
  plugins: [
    wasmPack('/Users/pluscht/Desktop/LanguageResources/Rust/lbm-wgpu')
  ],
})