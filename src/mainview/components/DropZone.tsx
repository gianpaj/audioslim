import { DragEvent, useState } from "react";
import { AUDIO_EXTENSIONS, AudioFile } from "../../shared/types";

interface DropZoneProps {
  onFilesSelected: (files: AudioFile[]) => void;
  onPickFiles: () => Promise<void>;
  disabled: boolean;
}

function pathToName(path: string): string {
  return path.split("/").pop() ?? path.split("\\").pop() ?? path;
}

function toAudioFile(path: string): AudioFile {
  return {
    path,
    name: pathToName(path),
    status: "pending",
  };
}

function isAudioPath(path: string): boolean {
  const extension = path.split(".").pop()?.toLowerCase() ?? "";
  return AUDIO_EXTENSIONS.includes(extension);
}

function fileUriToPath(uri: string): string | null {
  if (!uri.startsWith("file://")) {
    return null;
  }

  try {
    const decoded = decodeURIComponent(uri.slice("file://".length));
    return decoded || null;
  } catch {
    return null;
  }
}

function extractDropPaths(event: DragEvent<HTMLDivElement>): string[] {
  const dropped: string[] = [];

  for (const file of Array.from(event.dataTransfer.files)) {
    const maybePath = (file as unknown as { path?: string }).path;
    if (maybePath) {
      dropped.push(maybePath);
    }
  }

  if (dropped.length === 0) {
    const uriList = event.dataTransfer.getData("text/uri-list");
    if (uriList) {
      for (const line of uriList.split("\n")) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith("#")) {
          continue;
        }

        const path = fileUriToPath(trimmed);
        if (path) {
          dropped.push(path);
        }
      }
    }
  }

  return Array.from(new Set(dropped));
}

function DropZone({ onFilesSelected, onPickFiles, disabled }: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDrop = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setIsDragging(false);

    if (disabled) {
      return;
    }

    const paths = extractDropPaths(event);
    if (paths.length === 0) {
      return;
    }

    onFilesSelected(paths.filter(isAudioPath).map(toAudioFile));
  };

  return (
    <div
      className={`dropzone ${isDragging ? "dropzone-active" : ""}`}
      onDragEnter={(event) => {
        event.preventDefault();
        setIsDragging(true);
      }}
      onDragOver={(event) => {
        event.preventDefault();
      }}
      onDragLeave={(event) => {
        event.preventDefault();
        setIsDragging(false);
      }}
      onDrop={handleDrop}
    >
      <div className="dropzone-content">
        <p className="dropzone-icon">&#9835;</p>
        <p>Drag and drop audio files here</p>
        <p className="dropzone-or">or</p>
        <button type="button" onClick={() => void onPickFiles()} disabled={disabled}>
          Choose Files
        </button>
      </div>
    </div>
  );
}

export default DropZone;
