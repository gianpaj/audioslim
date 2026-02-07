import { ConversionOptions, OutputFormat, FORMAT_CONFIGS } from "../types";

const SAMPLE_RATES = [22050, 44100, 48000, 96000];
const CHANNELS = [
  { value: 1, label: "Mono" },
  { value: 2, label: "Stereo" },
];

interface OptionsPanelProps {
  format: OutputFormat;
  options: ConversionOptions;
  onChange: (options: ConversionOptions) => void;
}

function OptionsPanel({ format, options, onChange }: OptionsPanelProps) {
  const config = FORMAT_CONFIGS[format];

  return (
    <div className="options-panel">
      {config.supportsBitrate && (
        <div className="option-group">
          <label>Bitrate:</label>
          <select
            value={options.bitrate ?? config.defaultBitrate}
            onChange={(e) => onChange({ ...options, bitrate: e.target.value })}
          >
            {config.bitrateOptions.map((br) => (
              <option key={br} value={br}>
                {br}
              </option>
            ))}
          </select>
        </div>
      )}

      <div className="option-group">
        <label>Sample Rate:</label>
        <select
          value={options.sampleRate ?? ""}
          onChange={(e) =>
            onChange({
              ...options,
              sampleRate: e.target.value ? Number(e.target.value) : undefined,
            })
          }
        >
          <option value="">Default</option>
          {SAMPLE_RATES.map((sr) => (
            <option key={sr} value={sr}>
              {sr} Hz
            </option>
          ))}
        </select>
      </div>

      <div className="option-group">
        <label>Channels:</label>
        <select
          value={options.channels ?? ""}
          onChange={(e) =>
            onChange({
              ...options,
              channels: e.target.value ? Number(e.target.value) : undefined,
            })
          }
        >
          <option value="">Default</option>
          {CHANNELS.map((ch) => (
            <option key={ch.value} value={ch.value}>
              {ch.label}
            </option>
          ))}
        </select>
      </div>

      {config.supportsQuality && (
        <div className="option-group">
          <label>{config.qualityLabel}:</label>
          <div className="quality-control">
            <input
              type="range"
              min={config.qualityRange[0]}
              max={config.qualityRange[1]}
              value={options.quality ?? config.defaultQuality}
              onChange={(e) =>
                onChange({ ...options, quality: e.target.value })
              }
            />
            <span className="quality-value">
              {options.quality ?? config.defaultQuality}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

export default OptionsPanel;
