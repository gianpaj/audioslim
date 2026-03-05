import { access, chmod, copyFile, mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";

const TARGET_PATH = resolve("vendors/ffmpeg/macos-arm64/ffmpeg");

async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function candidatePaths(): string[] {
  const fromEnv = process.env.AUDIOSLIM_FFMPEG_PATH;

  return [
    fromEnv,
    "/opt/homebrew/bin/ffmpeg",
    "/usr/local/bin/ffmpeg",
    "/opt/local/bin/ffmpeg",
    Bun.which("ffmpeg") ?? undefined,
  ].filter((value): value is string => Boolean(value));
}

async function resolveSource(): Promise<string> {
  for (const candidate of candidatePaths()) {
    if (await exists(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    [
      "Unable to locate ffmpeg for bundling.",
      "Install ffmpeg (for example with Homebrew) or set AUDIOSLIM_FFMPEG_PATH.",
      "Expected one of: /opt/homebrew/bin/ffmpeg, /usr/local/bin/ffmpeg, /opt/local/bin/ffmpeg, or ffmpeg in PATH.",
    ].join("\n"),
  );
}

async function main() {
  const source = await resolveSource();
  await mkdir(dirname(TARGET_PATH), { recursive: true });
  await copyFile(source, TARGET_PATH);
  await chmod(TARGET_PATH, 0o755);

  console.log(`[prepare-ffmpeg] Bundling ffmpeg from: ${source}`);
  console.log(`[prepare-ffmpeg] Copied to: ${TARGET_PATH}`);
}

await main();
