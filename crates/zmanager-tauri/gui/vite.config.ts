import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { createSvgIconsPlugin } from "vite-plugin-svg-icons";
import { resolve } from "node:path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    createSvgIconsPlugin({
      // Specify icon directories to be cached
      iconDirs: [
        resolve(__dirname, "public/icons/dev_icons"),
        resolve(__dirname, "public/icons/filetypes"),
        resolve(__dirname, "public/icons/ui"),
      ],
      // Symbol ID format: icon-{dir}-{name}
      // e.g., icon-typescript-typescript-original, icon-folder, icon-copy
      symbolId: "icon-[dir]-[name]",
      // Inject sprite at end of body
      inject: "body-last",
      customDomId: "__svg_icons__",
    }),
  ],
  resolve: {
    alias: {
      "@": resolve(__dirname, "./src"),
    },
  },
  // Tauri expects a fixed port and will fail if it's not available
  server: {
    port: 5173,
    strictPort: true,
  },
  // Tauri expects the build output in the dist folder
  build: {
    outDir: "dist",
    target: "esnext",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  // Prevent vite from obscuring Rust errors
  clearScreen: false,
});
