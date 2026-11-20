/// <reference types="vitest" />
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const PORTAL = "http://127.0.0.1:8090";
const proxied = ["/api", "/login", "/logout", "/password", "/assets", "/oauth2", "/staging"];

// The Rust server serves the built SPA at /app/. In dev, Vite serves it and forwards the rest.
export default defineConfig({
  base: "/app/",
  plugins: [react()],
  server: {
    port: 5173,
    proxy: Object.fromEntries(proxied.map((p) => [p, { target: PORTAL, changeOrigin: false }])),
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    sourcemap: false,
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
    css: false,
  },
});
