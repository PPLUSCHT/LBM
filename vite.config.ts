import { defineConfig } from 'vite'
import wasmPack from 'vite-plugin-wasm-pack'

export default defineConfig({
  plugins: [
    wasmPack('./lbm-wgpu')
  ],
  base: '/LBM'
})