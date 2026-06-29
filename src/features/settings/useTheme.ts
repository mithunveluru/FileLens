import { useEffect } from "react";
import type { Settings } from "@/shared/types";

// "system" defers to prefers-color-scheme; "light"/"dark" force it via data-theme and color-scheme.
export function useTheme(theme: Settings["theme"]): void {
  useEffect(() => {
    const root = document.documentElement;
    if (theme === "light" || theme === "dark") {
      root.dataset.theme = theme;
      root.style.colorScheme = theme;
    } else {
      delete root.dataset.theme;
      root.style.colorScheme = "";
    }
  }, [theme]);
}
