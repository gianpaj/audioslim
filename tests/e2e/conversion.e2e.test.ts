import { expect, test } from "bun:test";
import { mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import {
  buildFfmpegArgs,
  checkFfmpegInstalled,
  convertAudioBatch,
  findOverwriteTargets,
} from "../../src/bun/converter";

async function createSampleWav(path: string): Promise<void> {
  const process = Bun.spawn({
    cmd: [
      "ffmpeg",
      "-f",
      "lavfi",
      "-i",
      "sine=frequency=440:duration=1",
      "-c:a",
      "pcm_s16le",
      "-y",
      path,
    ],
    stdout: "ignore",
    stderr: "pipe",
  });

  const [stderr, code] = await Promise.all([
    new Response(process.stderr).text(),
    process.exited,
  ]);

  if (code !== 0) {
    throw new Error(`failed to create sample audio: ${stderr}`);
  }
}

test("buildFfmpegArgs maps options for mp3", () => {
  const args = buildFfmpegArgs("/tmp/in.wav", "/tmp/out.mp3", {
    format: "mp3",
    bitrate: "192k",
    sampleRate: 44100,
    channels: 2,
    quality: "2",
  });

  expect(args).toEqual([
    "-i",
    "/tmp/in.wav",
    "-y",
    "-b:a",
    "192k",
    "-ar",
    "44100",
    "-ac",
    "2",
    "-q:a",
    "2",
    "/tmp/out.mp3",
  ]);
});

test("e2e converts wav to mp3", async () => {
  let ffmpegVersion: string;
  try {
    ffmpegVersion = await checkFfmpegInstalled();
  } catch {
    return;
  }

  expect(ffmpegVersion.toLowerCase()).toContain("ffmpeg");

  const workdir = await mkdtemp(join(tmpdir(), "audioslim-e2e-"));
  const input = join(workdir, "sample.wav");

  try {
    await createSampleWav(input);

    const events: string[] = [];
    const outputPaths = await convertAudioBatch(
      [input],
      {
        format: "mp3",
        bitrate: "192k",
      },
      (progress) => {
        events.push(progress.status);
      },
    );

    expect(outputPaths.length).toBe(1);
    expect(outputPaths[0]).toEndWith(".mp3");

    const outputFile = Bun.file(outputPaths[0]);
    expect(await outputFile.exists()).toBe(true);
    expect(events).toContain("converting");
    expect(events).toContain("done");
  } finally {
    await rm(workdir, { recursive: true, force: true });
  }
});

test("detects overwrite targets before conversion", async () => {
  const workdir = await mkdtemp(join(tmpdir(), "audioslim-overwrite-"));
  const input = join(workdir, "track.wav");
  const existingOutput = join(workdir, "track.mp3");

  try {
    await Bun.write(input, "fake-wav");
    await Bun.write(existingOutput, "fake-mp3");

    const targets = await findOverwriteTargets([input], "mp3");
    expect(targets).toEqual([existingOutput]);
  } finally {
    await rm(workdir, { recursive: true, force: true });
  }
});
