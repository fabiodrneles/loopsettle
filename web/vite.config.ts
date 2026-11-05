import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Relative base so the built site works from any folder (and later GitHub Pages).
export default defineConfig({
  base: "./",
  plugins: [react()],
  worker: { format: "es" },
  build: { target: "es2020" },
});
