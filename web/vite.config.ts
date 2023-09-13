import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [sveltekit()],
    build: {
        rollupOptions: {
            plugins: [],
            output: {
                manualChunks: {
                    '/pkg/interpreter.js': ['src/pkg/interpreter.js'],
                },
            },
        },
    },
    resolve: {
        alias: {
            // If Vite's root is your project root, then this path should work:
            '/pkg': '/static/pkg',
        },
    },
});

