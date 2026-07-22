import * as Dialog from "@radix-ui/react-dialog";
import type { ReactNode } from "react";
import "./modal.css";

interface ModalProps {
  title: string;
  /** Rendered under the title; also announced as the dialog description. */
  description?: string;
  children?: ReactNode;
  footer: ReactNode;
  /** Widen past the default for form-heavy dialogs. */
  wide?: boolean;
  onClose: () => void;
}

/*
 * Radix owns the focus trap, scroll lock, Escape handling, and aria wiring —
 * the hand-rolled version focused the dialog but let Tab walk out of it.
 * Callers still mount/unmount this conditionally, so `open` is always true.
 */
function Modal({ title, description, children, footer, wide = false, onClose }: ModalProps) {
  return (
    <Dialog.Root open onOpenChange={(next) => !next && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="modal-backdrop" />
        <Dialog.Content
          className={`modal${wide ? " modal-wide" : ""}`}
          aria-describedby={description ? undefined : "undefined"}
        >
          <Dialog.Title>{title}</Dialog.Title>
          {description && <Dialog.Description>{description}</Dialog.Description>}
          {children}
          <div className="modal-actions">{footer}</div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export default Modal;
