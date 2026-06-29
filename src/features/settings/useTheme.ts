import { useEffect } from "react";
import type { Settings } from "@/shared/types";

/**
 * Applies the chosen theme to the document. "system" defers to the OS via the
 * `prefers-color-scheme` rules in global.css; "light"/"dark" force it through a
 * `data-theme` attribute and the `color-scheme` property (which also themes
 * native form controls and scrollbars).
 */
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
