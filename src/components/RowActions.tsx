import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { MoreHorizontal } from "lucide-react";
import type { ComponentType } from "react";

export interface RowAction {
  label: string;
  Icon: ComponentType;
  onSelect: () => void;
  danger?: boolean;
}

interface RowActionsProps {
  actions: RowAction[];
  /** Used for the trigger's accessible name, e.g. the file being acted on. */
  label: string;
}

/*
 * One menu per row instead of a button per action — the findings table was
 * rendering four buttons across fifty rows, which is 200 tab stops to cross.
 */
function RowActions({ actions, label }: RowActionsProps) {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button type="button" className="btn-sm btn-ghost" aria-label={`Actions for ${label}`}>
          <MoreHorizontal />
        </button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content className="menu" align="end" sideOffset={4}>
          {actions.map(({ label: text, Icon, onSelect, danger }, index) => (
            <div key={text}>
              {danger && index > 0 && <DropdownMenu.Separator className="menu-separator" />}
              <DropdownMenu.Item
                className={`menu-item${danger ? " menu-item-danger" : ""}`}
                onSelect={onSelect}
              >
                <Icon />
                {text}
              </DropdownMenu.Item>
            </div>
          ))}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}

export default RowActions;
