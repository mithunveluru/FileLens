import { basename } from "@/shared/format/path";
import type { FileKind } from "@/shared/types";

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

export function recomputeDestination(root: string, kind: FileKind, source: string): string {
  const separator = root.includes("\\") ? "\\" : "/";
  const trimmedRoot = root.replace(/[\\/]+$/, "");
  return [trimmedRoot, KIND_FOLDERS[kind], basename(source)].join(separator);
}
