import { sveltekit } from '@sveltejs/kit/vite';
import VitePluginWasm from 'vite-plugin-wasm';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [sveltekit(), VitePluginWasm()],
    server: {
        fs: {
            allow: ['.', 'public/pkg']
        }
    }
});

