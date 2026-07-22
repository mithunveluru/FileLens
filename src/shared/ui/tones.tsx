import {
  Archive,
  Clock,
  Code2,
  FileQuestion,
  FileText,
  HardDrive,
  Image,
  Music,
  Package,
  Trash2,
  Video,
} from "lucide-react";
import type { ComponentType } from "react";
import type { FileKind, FindingCategory } from "@/shared/types";

/** Named tones defined in global.css as `[data-tone="…"]`. */
export type Tone =
  | "slate"
  | "indigo"
  | "violet"
  | "blue"
  | "teal"
  | "emerald"
  | "amber"
  | "orange"
  | "rose";

interface Facet {
  tone: Tone;
  Icon: ComponentType;
}

/*
 * Colour and icon per finding category. Amber for the ones that cost the most
 * space, quiet slate for merely-stale files, so a glance down the column sorts
 * "act on this" from "probably fine".
 */
export const CATEGORY_FACETS: Record<FindingCategory, Facet> = {
  largeFile: { tone: "amber", Icon: HardDrive },
  oldFile: { tone: "slate", Icon: Clock },
  installer: { tone: "violet", Icon: Package },
  temporaryFile: { tone: "teal", Icon: Trash2 },
};

/** Colour and icon per destination folder in the organization plan. */
export const KIND_FACETS: Record<FileKind, Facet> = {
  documents: { tone: "blue", Icon: FileText },
  images: { tone: "violet", Icon: Image },
  videos: { tone: "rose", Icon: Video },
  audio: { tone: "amber", Icon: Music },
  archives: { tone: "orange", Icon: Archive },
  installers: { tone: "teal", Icon: Package },
  code: { tone: "emerald", Icon: Code2 },
  other: { tone: "slate", Icon: FileQuestion },
};
