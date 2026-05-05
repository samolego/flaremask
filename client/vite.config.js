import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'path';
import { copyFileSync, cpSync, mkdirSync, existsSync, readFileSync, writeFileSync } from 'fs';

const isExt = process.env.BUILD_TARGET === 'extension';

/**
 * Builds background.js + content.js as IIFEs, then copies manifest.json, popup.html,
 * and the icons directory to dist-ext.
 * Firefox MV3 background/content scripts can't use ES module imports.
 */
function extensionAssets() {
  return {
    name: 'extension-assets',
    async closeBundle() {
      const { build } = await import('vite');

      // Build background as a self-contained IIFE
      await build({
        configFile: false,
        plugins: [svelte()],
        build: {
          outDir: resolve(__dirname, 'dist-ext'),
          emptyOutDir: false,
          lib: {
            entry: resolve(__dirname, 'src/extension/background/index.js'),
            formats: ['iife'],
            name: 'flaremaskBackground',
            fileName: () => 'background.js',
          },
        },
      });

      // Build content script as a self-contained IIFE
      await build({
        configFile: false,
        plugins: [svelte({ emitCss: false })],
        build: {
          outDir: resolve(__dirname, 'dist-ext'),
          emptyOutDir: false,
          lib: {
            entry: resolve(__dirname, 'src/extension/content/index.js'),
            formats: ['iife'],
            name: 'flaremaskContent',
            fileName: () => 'content.js',
          },
        },
      });

      // Copy icons to dist-ext/icons/
      const iconsOut = resolve(__dirname, 'dist-ext/icons');
      if (!existsSync(iconsOut)) mkdirSync(iconsOut, { recursive: true });
      cpSync(resolve(__dirname, 'public/icons'), iconsOut, { recursive: true });

      copyFileSync(
        resolve(__dirname, 'src/extension/manifest.json'),
        resolve(__dirname, 'dist-ext/manifest.json'),
      );
      // Flatten nested popup HTML to dist-ext/popup.html, fixing relative paths
      const src = resolve(__dirname, 'dist-ext/src/extension/popup/index.html');
      const html = readFileSync(src, 'utf8').replace(/\.\.\/\.\.\/\.\.\//g, './');
      writeFileSync(resolve(__dirname, 'dist-ext/popup.html'), html);
    },
  };
}

export default defineConfig({
  plugins: [svelte(), tailwindcss(), ...(isExt ? [extensionAssets()] : [])],
  base: isExt ? './' : '/',
  server: {
    proxy: {
      '/api': 'http://localhost:8787',
      '/auth': 'http://localhost:8787',
    },
  },
  build: isExt ? {
    outDir: 'dist-ext',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        popup: resolve(__dirname, 'src/extension/popup/index.html'),
      },
      output: {
        entryFileNames: '[name].js',
        assetFileNames: '[name][extname]',
      },
    },
  } : {
    rollupOptions: {
      output: {
        // Stable filenames so worker can embed them with include_bytes!
        entryFileNames: 'assets/index.js',
        assetFileNames: 'assets/[name][extname]',
      },
    },
  },
});

