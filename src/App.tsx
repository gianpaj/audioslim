import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import DropZone from "./components/DropZone";
import FormatSelector from "./components/FormatSelector";
import OptionsPanel from "./components/OptionsPanel";
import FileList from "./components/FileList";
import ConvertButton from "./components/ConvertButton";

import {
  AudioFile,
  OutputFormat,
  ConversionOptions,
  ConversionProgress,
  FORMAT_CONFIGS,
} from "./types";

import "./App.css";

function App() {
  const [ffmpegStatus, setFfmpegStatus] = useState<string | null>(null);
  const [ffmpegError, setFfmpegError] = useState<string | null>(null);
  const [files, setFiles] = useState<AudioFile[]>([]);
  const [format, setFormat] = useState<OutputFormat>("mp3");
  const [options, setOptions] = useState<ConversionOptions>({
    format: "mp3",
    bitrate: FORMAT_CONFIGS.mp3.defaultBitrate,
  });
  const [isConverting, setIsConverting] = useState(false);

  // Check ffmpeg on mount
  useEffect(() => {
    invoke<string>("check_ffmpeg")
      .then((version) => setFfmpegStatus(version))
      .catch((err) => setFfmpegError(String(err)));
  }, []);

  // Listen for conversion progress events
  useEffect(() => {
    const unlistenPromise = listen<ConversionProgress>(
      "conversion-progress",
      (event) => {
        const progress = event.payload;
        setFiles((prev) =>
          prev.map((f) =>
            f.path === progress.filePath
              ? {
                  ...f,
                  status: progress.status as AudioFile["status"],
                  outputPath: progress.outputPath ?? undefined,
                  error: progress.error ?? undefined,
                  inputSize: progress.inputSize ?? f.inputSize,
                  outputSize: progress.outputSize ?? f.outputSize,
                }
              : f
          )
        );
      }
    );

    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, []);

  const handleFormatChange = (newFormat: OutputFormat) => {
    setFormat(newFormat);
    const config = FORMAT_CONFIGS[newFormat];
    setOptions({
      format: newFormat,
      bitrate: config.supportsBitrate ? config.defaultBitrate : undefined,
      quality: config.supportsQuality ? config.defaultQuality : undefined,
      sampleRate: undefined,
      channels: undefined,
    });
  };

  const handleFilesSelected = useCallback((newFiles: AudioFile[]) => {
    setFiles((prev) => {
      const existingPaths = new Set(prev.map((f) => f.path));
      const uniqueNew = newFiles.filter((f) => !existingPaths.has(f.path));
      return [...prev, ...uniqueNew];
    });
  }, []);

  const handleRemoveFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleClearFiles = () => {
    setFiles([]);
  };

  const handleConvert = async () => {
    if (files.length === 0) return;

    setIsConverting(true);
    setFiles((prev) =>
      prev.map((f) => ({
        ...f,
        status: "pending" as const,
        error: undefined,
        outputPath: undefined,
        inputSize: undefined,
        outputSize: undefined,
      }))
    );

    const inputPaths = files.map((f) => f.path);

    try {
      await invoke("convert_audio", {
        inputPaths,
        options: { ...options, format },
      });
    } catch (err) {
      console.error("Conversion error:", err);
    } finally {
      setIsConverting(false);
    }
  };

  if (ffmpegError) {
    return (
      <div className="container">
        <h1>AudioSlim</h1>
        <div className="error-banner">
          <p>ffmpeg is required but was not found.</p>
          <p className="error-detail">{ffmpegError}</p>
          <p>
            Please install ffmpeg and restart the application.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="container">
      <h1>AudioSlim</h1>
      {ffmpegStatus && <p className="ffmpeg-version">{ffmpegStatus}</p>}

      <DropZone onFilesSelected={handleFilesSelected} disabled={isConverting} />

      <FileList
        files={files}
        onRemoveFile={handleRemoveFile}
        onClearFiles={handleClearFiles}
      />

      <FormatSelector selectedFormat={format} onChange={handleFormatChange} />

      <OptionsPanel format={format} options={options} onChange={setOptions} />

      <ConvertButton
        fileCount={files.length}
        isConverting={isConverting}
        onConvert={handleConvert}
      />
    </div>
  );
}

export default App;
