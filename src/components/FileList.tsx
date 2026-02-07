import { AudioFile, formatFileSize } from "../types";

interface FileListProps {
  files: AudioFile[];
  onRemoveFile: (index: number) => void;
  onClearFiles: () => void;
}

function FileList({ files, onRemoveFile, onClearFiles }: FileListProps) {
  if (files.length === 0) return null;

  return (
    <div className="file-list">
      <div className="file-list-header">
        <span>
          {files.length} file{files.length !== 1 ? "s" : ""} selected
        </span>
        <button className="clear-btn" onClick={onClearFiles}>
          Clear All
        </button>
      </div>
      <ul>
        {files.map((file, index) => (
          <li
            key={`${file.path}-${index}`}
            className={`file-item file-status-${file.status}`}
          >
            <span className="file-name">{file.name}</span>
            <span className="file-status-text">
              {file.status === "pending" && "Ready"}
              {file.status === "converting" && "Converting..."}
              {file.status === "done" && (
                <>
                  Done
                  {file.inputSize != null && file.outputSize != null && (
                    <span className="file-sizes">
                      {" "}
                      ({formatFileSize(file.inputSize)} &rarr;{" "}
                      {formatFileSize(file.outputSize)})
                    </span>
                  )}
                </>
              )}
              {file.status === "error" && (
                <span title={file.error}>Error</span>
              )}
            </span>
            {file.status === "pending" && (
              <button
                className="remove-btn"
                onClick={() => onRemoveFile(index)}
              >
                &times;
              </button>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

export default FileList;
