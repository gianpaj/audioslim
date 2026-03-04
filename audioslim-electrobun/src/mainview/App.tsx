import { useCallback, useEffect, useState } from "react";
import { Electroview } from "electrobun/view";

import DropZone from "./components/DropZone";
import FormatSelector from "./components/FormatSelector";
import OptionsPanel from "./components/OptionsPanel";
import FileList from "./components/FileList";
import ConvertButton from "./components/ConvertButton";

import type { AudioSlimRpcSchema } from "../shared/rpc";
import {
  AudioFile,
  ConversionOptions,
  ConversionProgress,
  FORMAT_CONFIGS,
  OutputFormat,
} from "../shared/types";

import "./App.css";

const rpc = Electroview.defineRPC<AudioSlimRpcSchema>({
  maxRequestTime: Infinity,
  handlers: {
    requests: {},
    messages: {
      conversionProgress(progress) {
        window.dispatchEvent(
          new CustomEvent<ConversionProgress>("conversion-progress", {
            detail: progress,
          }),
        );
      },
    },
  },
});

new Electroview({ rpc });

function pathToName(path: string): string {
  return path.split("/").pop() ?? path.split("\\").pop() ?? path;
}

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

  useEffect(() => {
    rpc.request
      .checkFfmpeg()
      .then((version) => setFfmpegStatus(version))
      .catch((error) => setFfmpegError(String(error)));
  }, []);

  useEffect(() => {
    const handler = (event: Event) => {
      const detail = (event as CustomEvent<ConversionProgress>).detail;

      setFiles((previous) =>
        previous.map((file) =>
          file.path === detail.filePath
            ? {
                ...file,
                status: detail.status as AudioFile["status"],
                outputPath: detail.outputPath ?? undefined,
                error: detail.error ?? undefined,
                inputSize: detail.inputSize ?? file.inputSize,
                outputSize: detail.outputSize ?? file.outputSize,
              }
            : file,
        ),
      );
    };

    window.addEventListener("conversion-progress", handler);
    return () => window.removeEventListener("conversion-progress", handler);
  }, []);

  const handleFormatChange = (newFormat: OutputFormat) => {
    const config = FORMAT_CONFIGS[newFormat];

    setFormat(newFormat);
    setOptions({
      format: newFormat,
      bitrate: config.supportsBitrate ? config.defaultBitrate : undefined,
      quality: config.supportsQuality ? config.defaultQuality : undefined,
      sampleRate: undefined,
      channels: undefined,
    });
  };

  const handleFilesSelected = useCallback((selectedFiles: AudioFile[]) => {
    setFiles((previous) => {
      const existing = new Set(previous.map((file) => file.path));
      const uniqueFiles = selectedFiles.filter((file) => !existing.has(file.path));
      return [...previous, ...uniqueFiles];
    });
  }, []);

  const handlePickFiles = async () => {
    try {
      const selected = await rpc.request.pickAudioFiles();
      const pickedFiles: AudioFile[] = selected.map((path) => ({
        path,
        name: pathToName(path),
        status: "pending",
      }));
      handleFilesSelected(pickedFiles);
    } catch (error) {
      console.error("Failed to pick files", error);
    }
  };

  const handleRemoveFile = (index: number) => {
    setFiles((previous) => previous.filter((_, current) => current !== index));
  };

  const handleClearFiles = () => {
    setFiles([]);
  };

  const handleConvert = async () => {
    if (files.length === 0) {
      return;
    }

    setIsConverting(true);
    setFiles((previous) =>
      previous.map((file) => ({
        ...file,
        status: "pending",
        outputPath: undefined,
        error: undefined,
        inputSize: undefined,
        outputSize: undefined,
      })),
    );

    try {
      await rpc.request.convertAudio({
        inputPaths: files.map((file) => file.path),
        options: { ...options, format },
      });
    } catch (error) {
      console.error("Conversion error", error);
    } finally {
      setIsConverting(false);
    }
  };

  return (
    <div className="container">
      <h1>AudioSlim</h1>
      {ffmpegError ? (
        <div className="error-banner">
          <p>ffmpeg is required but was not found.</p>
          <p className="error-detail">{ffmpegError}</p>
          <p>Please install ffmpeg and restart the application.</p>
        </div>
      ) : (
        <>
          {ffmpegStatus && <p className="ffmpeg-version">{ffmpegStatus}</p>}

          <DropZone
            onFilesSelected={handleFilesSelected}
            onPickFiles={handlePickFiles}
            disabled={isConverting}
          />

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
        </>
      )}
    </div>
  );
}

export default App;
