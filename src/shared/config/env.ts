/**
 * Typed access to build-time environment configuration.
 *
 * Vite natively reads `.env*` files and exposes `VITE_`-prefixed variables on
 * `import.meta.env`. We centralise that access here so the rest of the app
 * never touches `import.meta.env` directly and every variable has one typed
 * source of truth. See `.env.example` for the full list.
 */

export const env = {
  /** True during `vite dev` / `tauri dev`. */
  isDev: import.meta.env.DEV,
  /** Minimum log level to emit. Defaults to `debug` in dev, `info` otherwise. */
  logLevel: import.meta.env.VITE_LOG_LEVEL ?? (import.meta.env.DEV ? "debug" : "info"),
} as const;
