import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The interface talks to the engine through Tauri commands in the desktop app
// and through the development server (`bm serve`) in a browser.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    proxy: { "/api": "http://127.0.0.1:8787" },
  },
  build: { target: "es2022", chunkSizeWarningLimit: 900 },
});
