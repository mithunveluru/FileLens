import { useState } from "react";
import "@/components/modal.css";
import { useModalA11y } from "@/components/useModalA11y";
import "./SettingsModal.css";
import type { Settings } from "@/shared/types";

interface SettingsModalProps {
  settings: Settings;
  onSave: (next: Settings) => Promise<void>;
  onClose: () => void;
}

const linesToArray = (text: string): string[] =>
  text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);

function SettingsModal({ settings, onSave, onClose }: SettingsModalProps) {
  const [form, setForm] = useState<Settings>(settings);
  const [saving, setSaving] = useState(false);
  const ref = useModalA11y(onClose);

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setForm((current) => ({ ...current, [key]: value }));

  const submit = async () => {
    setSaving(true);
    try {
      await onSave(form);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal-backdrop">
      <div
        className="modal settings-modal"
        ref={ref}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-label="Settings"
      >
        <h2>Settings</h2>

        <label>
          Downloads folder
          <input
            type="text"
            placeholder="Leave empty to use the system Downloads folder"
            value={form.downloadsFolder ?? ""}
            onChange={(e) => set("downloadsFolder", e.currentTarget.value || null)}
          />
        </label>

        <div className="settings-row">
          <label>
            Old after (days)
            <input
              type="number"
              min={1}
              value={form.ageThresholdDays}
              onChange={(e) => set("ageThresholdDays", Number(e.currentTarget.value) || 0)}
            />
          </label>
          <label>
            Large file size (MB)
            <input
              type="number"
              min={1}
              value={form.largeFileMinMb}
              onChange={(e) => set("largeFileMinMb", Number(e.currentTarget.value) || 0)}
            />
          </label>
        </div>

        <label>
          Ignored folders (one per line)
          <textarea
            rows={3}
            value={form.ignoredFolders.join("\n")}
            onChange={(e) => set("ignoredFolders", linesToArray(e.currentTarget.value))}
          />
        </label>

        <label>
          Ignored extensions (one per line)
          <textarea
            rows={2}
            value={form.ignoredExtensions.join("\n")}
            onChange={(e) => set("ignoredExtensions", linesToArray(e.currentTarget.value))}
          />
        </label>

        <div className="settings-row">
          <label>
            Theme
            <select
              value={form.theme}
              onChange={(e) => set("theme", e.currentTarget.value as Settings["theme"])}
            >
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </label>
          <label className="settings-checkbox">
            <input
              type="checkbox"
              checked={form.autoScanOnStartup}
              onChange={(e) => set("autoScanOnStartup", e.currentTarget.checked)}
            />
            Scan on startup
          </label>
        </div>

        <div className="modal-actions">
          <button type="button" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="danger" onClick={submit} disabled={saving}>
            {saving ? "Saving…" : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default SettingsModal;
