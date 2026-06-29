import { useEffect, useRef } from "react";

/**
 * Shared modal accessibility: moves focus into the dialog on open and closes it
 * on Escape. Attach the returned ref to the dialog element (give it
 * `tabIndex={-1}` so it can receive focus).
 */
export function useModalA11y(onClose: () => void) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    ref.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return ref;
}
