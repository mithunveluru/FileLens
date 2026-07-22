/// <reference types="vitest/config" />
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  // `tsconfigPaths` keeps the `@/*` alias defined once (in tsconfig.json) and
  // shared across Vite, the type checker, and Vitest.
  plugins: [react(), tsconfigPaths()],

  // Keep Rust compiler errors visible.
  clearScreen: false,
  // Tauri expects this exact port and fails if it is taken.
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
}));
