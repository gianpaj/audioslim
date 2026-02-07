export type OutputFormat = "mp3" | "wav" | "aac" | "ogg" | "flac" | "m4a" | "mp4";

export interface ConversionOptions {
  format: OutputFormat;
  bitrate?: string;
  sampleRate?: number;
  channels?: number;
  quality?: string;
}

export type FileStatus = "pending" | "converting" | "done" | "error";

export interface AudioFile {
  path: string;
  name: string;
  status: FileStatus;
  outputPath?: string;
  error?: string;
  inputSize?: number;
  outputSize?: number;
}

export interface ConversionProgress {
  filePath: string;
  status: string;
  outputPath: string | null;
  error: string | null;
  index: number;
  total: number;
  inputSize: number | null;
  outputSize: number | null;
}

export interface FormatConfig {
  supportsBitrate: boolean;
  bitrateOptions: string[];
  defaultBitrate: string;
  supportsQuality: boolean;
  qualityLabel: string;
  qualityRange: [number, number];
  defaultQuality: string;
}

export const FORMAT_CONFIGS: Record<OutputFormat, FormatConfig> = {
  mp3: {
    supportsBitrate: true,
    bitrateOptions: ["64k", "128k", "192k", "256k", "320k"],
    defaultBitrate: "192k",
    supportsQuality: true,
    qualityLabel: "VBR Quality (0=best, 9=worst)",
    qualityRange: [0, 9],
    defaultQuality: "2",
  },
  wav: {
    supportsBitrate: false,
    bitrateOptions: [],
    defaultBitrate: "",
    supportsQuality: false,
    qualityLabel: "",
    qualityRange: [0, 0],
    defaultQuality: "",
  },
  aac: {
    supportsBitrate: true,
    bitrateOptions: ["64k", "128k", "192k", "256k", "320k"],
    defaultBitrate: "192k",
    supportsQuality: false,
    qualityLabel: "",
    qualityRange: [0, 0],
    defaultQuality: "",
  },
  ogg: {
    supportsBitrate: true,
    bitrateOptions: ["64k", "128k", "192k", "256k", "320k"],
    defaultBitrate: "192k",
    supportsQuality: true,
    qualityLabel: "Quality (-1 to 10, higher=better)",
    qualityRange: [-1, 10],
    defaultQuality: "5",
  },
  flac: {
    supportsBitrate: false,
    bitrateOptions: [],
    defaultBitrate: "",
    supportsQuality: true,
    qualityLabel: "Compression Level (0=fast, 8=small)",
    qualityRange: [0, 8],
    defaultQuality: "5",
  },
  m4a: {
    supportsBitrate: true,
    bitrateOptions: ["64k", "128k", "192k", "256k", "320k"],
    defaultBitrate: "192k",
    supportsQuality: false,
    qualityLabel: "",
    qualityRange: [0, 0],
    defaultQuality: "",
  },
  mp4: {
    supportsBitrate: true,
    bitrateOptions: ["64k", "128k", "192k", "256k", "320k"],
    defaultBitrate: "192k",
    supportsQuality: false,
    qualityLabel: "",
    qualityRange: [0, 0],
    defaultQuality: "",
  },
};

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
