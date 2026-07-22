import * as Tooltip from "@radix-ui/react-tooltip";
import type { ReactNode } from "react";

interface TipProps {
  content: string;
  children: ReactNode;
}

/** Styled, instant tooltip for truncated paths — `title` waits a second and
 *  renders in the OS chrome, where it cannot be themed or read reliably. */
function Tip({ content, children }: TipProps) {
  return (
    <Tooltip.Root>
      <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content className="tooltip" side="top" align="start" sideOffset={4}>
          {content}
          <Tooltip.Arrow className="tooltip-arrow" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}

export default Tip;
