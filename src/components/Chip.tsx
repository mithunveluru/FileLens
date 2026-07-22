import type { ComponentType } from "react";
import type { Tone } from "@/shared/ui/tones";

interface ChipProps {
  tone: Tone;
  Icon?: ComponentType;
  children: React.ReactNode;
}

/** Tinted status pill. The tone drives colour through CSS custom properties,
 *  so the palette lives in global.css rather than in every call site. */
function Chip({ tone, Icon, children }: ChipProps) {
  return (
    <span className="chip" data-tone={tone}>
      {Icon && <Icon />}
      {children}
    </span>
  );
}

export default Chip;
