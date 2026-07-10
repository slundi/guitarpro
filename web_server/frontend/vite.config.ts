import { defineConfig } from "vite";
import { alphaTab } from "@coderline/alphatab-vite";

export default defineConfig({
  // alphaTab's worker is emitted to dist/assets/, and at runtime it resolves
  // font/ and soundfont/ URLs relative to itself — so the assets must land
  // under dist/assets/ too, not at dist/ root.
  plugins: [alphaTab({ assetOutputDir: "dist/assets" })],
  build: {
    outDir: "dist",
    emptyOutDir: false,
  },
  server: {
    proxy: {
      "/api": "http://localhost:3000",
    },
  },
});
