// Typed access to VITE_ build-time env so the app never reads import.meta.env directly.
export const env = {
  // Defaults to debug in dev, info otherwise.
  logLevel: import.meta.env.VITE_LOG_LEVEL ?? (import.meta.env.DEV ? "debug" : "info"),
} as const;
