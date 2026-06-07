import { defineConfig } from "vite";

export default defineConfig({
  // alphaTab ships its own worker bundle; exclude it from Vite's dep optimiser
  // so it can load its internal assets (wasm, fonts, workers) correctly.
  optimizeDeps: {
    exclude: ["@coderline/alphatab"],
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
  server: {
    proxy: {
      "/api": "http://localhost:3000",
    },
  },
});
