import { basename, dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { access, stat } from "node:fs/promises";
import type {
  ConversionOptions,
  ConversionProgress,
  OutputFormat,
} from "../shared/types";

function formatExtension(format: OutputFormat): string {
  return format;
}

export function buildFfmpegArgs(
  inputPath: string,
  outputPath: string,
  options: ConversionOptions,
): string[] {
  const args: string[] = ["-i", inputPath, "-y"];

  if (options.bitrate) {
    args.push("-b:a", options.bitrate);
  }

  if (options.sampleRate) {
    args.push("-ar", String(options.sampleRate));
  }

  if (options.channels) {
    args.push("-ac", String(options.channels));
  }

  if (options.quality) {
    if (options.format === "mp3" || options.format === "ogg") {
      args.push("-q:a", options.quality);
    } else if (options.format === "flac") {
      args.push("-compression_level", options.quality);
    }
  }

  if (options.format === "aac" || options.format === "m4a") {
    args.push("-c:a", "aac");
  }

  if (options.format === "mp4") {
    args.push("-c:a", "aac", "-vn");
  }

  if (options.format === "ogg") {
    args.push("-c:a", "libvorbis");
  }

  args.push(outputPath);
  return args;
}

async function getFileSize(path: string): Promise<number | undefined> {
  try {
    const metadata = await stat(path);
    return metadata.size;
  } catch {
    return undefined;
  }
}

async function runCommand(cmd: string[]): Promise<{
  success: boolean;
  stdout: string;
  stderr: string;
}> {
  const process = Bun.spawn({ cmd, stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, code] = await Promise.all([
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
    process.exited,
  ]);

  return {
    success: code === 0,
    stdout,
    stderr,
  };
}

async function isExecutableAvailable(path: string): Promise<boolean> {
  if (!path) {
    return false;
  }

  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

const BUN_DIR = dirname(fileURLToPath(import.meta.url));
const BUNDLED_FFMPEG_PATH = resolve(BUN_DIR, "../bin/ffmpeg");
const LOCAL_VENDOR_FFMPEG_PATH = resolve("vendors/ffmpeg/macos-arm64/ffmpeg");

async function resolveFfmpegBinary(): Promise<string> {
  const fromEnv = process.env.AUDIOSLIM_FFMPEG_PATH;
  if (fromEnv && (await isExecutableAvailable(fromEnv))) {
    return fromEnv;
  }

  if (await isExecutableAvailable(BUNDLED_FFMPEG_PATH)) {
    return BUNDLED_FFMPEG_PATH;
  }

  if (await isExecutableAvailable(LOCAL_VENDOR_FFMPEG_PATH)) {
    return LOCAL_VENDOR_FFMPEG_PATH;
  }

  if (process.platform === "darwin") {
    const macCandidates = [
      "/opt/homebrew/bin/ffmpeg",
      "/usr/local/bin/ffmpeg",
      "/opt/local/bin/ffmpeg",
    ];

    for (const candidate of macCandidates) {
      if (await isExecutableAvailable(candidate)) {
        return candidate;
      }
    }
  }

  return "ffmpeg";
}

export async function checkFfmpegInstalled(): Promise<string> {
  const ffmpegPath = await resolveFfmpegBinary();
  const result = await runCommand([ffmpegPath, "-version"]);

  if (!result.success) {
    throw new Error(
      `ffmpeg is not installed or not found. Tried: ${ffmpegPath}. You can also set AUDIOSLIM_FFMPEG_PATH.`,
    );
  }

  return result.stdout.split("\n")[0] ?? "ffmpeg found";
}

export function buildOutputPath(inputPath: string, format: OutputFormat): string {
  const fileName = basename(inputPath);
  const parent = dirname(inputPath);
  const ext = extname(fileName);
  const stem = ext ? fileName.slice(0, -ext.length) : fileName;
  const outputName = `${stem}.${formatExtension(format)}`;
  const initialPath = join(parent, outputName);

  if (initialPath === inputPath) {
    return join(parent, `${stem}_converted.${formatExtension(format)}`);
  }

  return initialPath;
}

async function pathExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

export async function findOverwriteTargets(
  inputPaths: string[],
  format: OutputFormat,
): Promise<string[]> {
  const targets = inputPaths.map((inputPath) => buildOutputPath(inputPath, format));
  const existing: string[] = [];

  for (const target of targets) {
    if (await pathExists(target)) {
      existing.push(target);
    }
  }

  return existing;
}

export async function convertAudioBatch(
  inputPaths: string[],
  options: ConversionOptions,
  onProgress: (progress: ConversionProgress) => void,
): Promise<string[]> {
  const ffmpegPath = await resolveFfmpegBinary();
  const outputPaths: string[] = [];
  const total = inputPaths.length;

  for (const [index, inputPath] of inputPaths.entries()) {
    const inputSize = await getFileSize(inputPath);

    onProgress({
      filePath: inputPath,
      status: "converting",
      outputPath: null,
      error: null,
      index,
      total,
      inputSize: inputSize ?? null,
      outputSize: null,
    });

    const outputPath = buildOutputPath(inputPath, options.format);
    const args = buildFfmpegArgs(inputPath, outputPath, options);
    const result = await runCommand([ffmpegPath, ...args]);

    if (result.success) {
      const outputSize = await getFileSize(outputPath);

      onProgress({
        filePath: inputPath,
        status: "done",
        outputPath,
        error: null,
        index,
        total,
        inputSize: inputSize ?? null,
        outputSize: outputSize ?? null,
      });

      outputPaths.push(outputPath);
      continue;
    }

    onProgress({
      filePath: inputPath,
      status: "error",
      outputPath: null,
      error: result.stderr || "ffmpeg returned an unknown error",
      index,
      total,
      inputSize: inputSize ?? null,
      outputSize: null,
    });
  }

  return outputPaths;
}
