import { basename } from "@/shared/format/path";
import type { FileKind } from "@/shared/types";

/** Display labels and destination folder names for each category. */
export const KIND_FOLDERS: Record<FileKind, string> = {
  documents: "Documents",
  images: "Images",
  videos: "Videos",
  audio: "Audio",
  archives: "Archives",
  installers: "Installers",
  code: "Code",
  other: "Other",
};

export const KIND_ORDER: FileKind[] = [
  "documents",
  "images",
  "videos",
  "audio",
  "archives",
  "installers",
  "code",
  "other",
];

/**
 * Recomputes a destination when the user changes a file's target category,
 * keeping it inside the same Downloads root and preserving the OS separator.
 */
export function recomputeDestination(root: string, kind: FileKind, source: string): string {
  const separator = root.includes("\\") ? "\\" : "/";
  const trimmedRoot = root.replace(/[\\/]+$/, "");
  return [trimmedRoot, KIND_FOLDERS[kind], basename(source)].join(separator);
}
