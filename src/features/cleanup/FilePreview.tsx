import { CalendarClock, CalendarPlus, FileType2, FolderOpen, HardDrive, Tag } from "lucide-react";
import { type ComponentType, useEffect, useState } from "react";
import Modal from "@/components/Modal";
import { formatBytes } from "@/shared/format/bytes";
import { getFileInfo } from "@/shared/ipc/commands";
import type { FileEntry } from "@/shared/types";

interface FilePreviewProps {
  path: string;
  onClose: () => void;
}

function formatDate(ms: number | null): string {
  return ms === null ? "—" : new Date(ms).toLocaleString();
}

function FilePreview({ path, onClose }: FilePreviewProps) {
  const [info, setInfo] = useState<FileEntry | null>(null);

  useEffect(() => {
    getFileInfo(path)
      .then(setInfo)
      .catch(() => {
        // Logged by the IPC client; the modal shows what it can.
      });
  }, [path]);

  const rows: Array<[ComponentType, string, string]> = info
    ? [
        [Tag, "Name", info.name],
        [FileType2, "Type", info.mimeType],
        [HardDrive, "Size", formatBytes(info.sizeBytes)],
        [CalendarPlus, "Created", formatDate(info.createdMs)],
        [CalendarClock, "Modified", formatDate(info.modifiedMs)],
        [FolderOpen, "Path", info.path],
      ]
    : [];

  return (
    <Modal
      title="File information"
      onClose={onClose}
      footer={
        <button type="button" onClick={onClose}>
          Close
        </button>
      }
    >
      {info === null ? (
        <div className="file-preview">
          {[70, 55, 80].map((width) => (
            <div key={width} className="skeleton" style={{ width: `${width}%` }} />
          ))}
        </div>
      ) : (
        <dl className="file-preview">
          {rows.map(([Icon, label, value]) => (
            <div key={label}>
              <dt>
                <Icon />
                {label}
              </dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      )}
    </Modal>
  );
}

export default FilePreview;
