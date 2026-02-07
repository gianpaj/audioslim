import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { AudioFile } from "../types";

const AUDIO_EXTENSIONS = [
  "wav", "mp3", "ogg", "flac", "aac", "m4a", "mp4", "wma", "aiff",
];

interface DropZoneProps {
  onFilesSelected: (files: AudioFile[]) => void;
  disabled: boolean;
}

function pathToName(p: string): string {
  return p.split("/").pop() ?? p.split("\\").pop() ?? p;
}

function isAudioFile(path: string): boolean {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  return AUDIO_EXTENSIONS.includes(ext);
}

function DropZone({ onFilesSelected, disabled }: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        setIsDragging(true);
      } else if (event.payload.type === "drop") {
        setIsDragging(false);
        const paths = event.payload.paths.filter(isAudioFile);
        if (paths.length > 0) {
          const files: AudioFile[] = paths.map((p) => ({
            path: p,
            name: pathToName(p),
            status: "pending" as const,
          }));
          onFilesSelected(files);
        }
      } else {
        setIsDragging(false);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [onFilesSelected]);

  const handleOpenDialog = async () => {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Audio Files",
          extensions: AUDIO_EXTENSIONS,
        },
      ],
    });

    if (selected === null) return;

    const paths = Array.isArray(selected) ? selected : [selected];
    const files: AudioFile[] = paths.map((p) => ({
      path: p,
      name: pathToName(p),
      status: "pending" as const,
    }));
    onFilesSelected(files);
  };

  return (
    <div className={`dropzone ${isDragging ? "dropzone-active" : ""}`}>
      <div className="dropzone-content">
        <p className="dropzone-icon">&#9835;</p>
        <p>Drag and drop audio files here</p>
        <p className="dropzone-or">or</p>
        <button onClick={handleOpenDialog} disabled={disabled}>
          Choose Files
        </button>
      </div>
    </div>
  );
}

export default DropZone;
