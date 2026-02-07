import { OutputFormat } from "../types";

const FORMATS: OutputFormat[] = ["mp3", "wav", "aac", "ogg", "flac", "m4a", "mp4"];

interface FormatSelectorProps {
  selectedFormat: OutputFormat;
  onChange: (format: OutputFormat) => void;
}

function FormatSelector({ selectedFormat, onChange }: FormatSelectorProps) {
  return (
    <div className="format-selector">
      <label>Output Format:</label>
      <div className="format-buttons">
        {FORMATS.map((fmt) => (
          <button
            key={fmt}
            className={`format-btn ${selectedFormat === fmt ? "format-btn-active" : ""}`}
            onClick={() => onChange(fmt)}
          >
            {fmt.toUpperCase()}
          </button>
        ))}
      </div>
    </div>
  );
}

export default FormatSelector;
