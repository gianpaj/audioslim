import {
  ApplicationMenu,
  BrowserView,
  BrowserWindow,
  Updater,
  Utils,
} from "electrobun/bun";
import type { AudioSlimRpcSchema } from "../shared/rpc";
import { AUDIO_EXTENSIONS } from "../shared/types";
import {
  checkFfmpegInstalled,
  convertAudioBatch,
  findOverwriteTargets,
} from "./converter";

const DEV_SERVER_PORT = 5173;
const DEV_SERVER_URL = `http://localhost:${DEV_SERVER_PORT}`;

async function getMainViewUrl(): Promise<string> {
  const channel = await Updater.localInfo.channel();

  if (channel === "dev") {
    try {
      await fetch(DEV_SERVER_URL, { method: "HEAD" });
      return DEV_SERVER_URL;
    } catch {
      console.log("Vite dev server not running. Run 'bun run dev:hmr' for HMR support.");
    }
  }

  return "views://mainview/index.html";
}

function isAudioPath(path: string): boolean {
  const extension = path.split(".").pop()?.toLowerCase() ?? "";
  return AUDIO_EXTENSIONS.includes(extension);
}

const rpc = BrowserView.defineRPC<AudioSlimRpcSchema>({
  maxRequestTime: Infinity,
  handlers: {
    requests: {
      async checkFfmpeg() {
        return checkFfmpegInstalled();
      },
      async pickAudioFiles() {
        const selected = await Utils.openFileDialog({
          startingFolder: "~/",
          allowedFileTypes: AUDIO_EXTENSIONS.join(","),
          canChooseFiles: true,
          canChooseDirectory: false,
          allowsMultipleSelection: true,
        });

        return selected.filter((path) => path && isAudioPath(path));
      },
      async convertAudio({ inputPaths, options }) {
        const overwriteTargets = await findOverwriteTargets(inputPaths, options.format);
        if (overwriteTargets.length > 0) {
          const preview = overwriteTargets.slice(0, 3).map((path) => `• ${path}`).join("\n");
          const remaining = overwriteTargets.length - Math.min(3, overwriteTargets.length);
          const detail = remaining > 0 ? `${preview}\n• ...and ${remaining} more` : preview;

          const result = await Utils.showMessageBox({
            type: "warning",
            title: "Overwrite Existing Files?",
            message: `AudioSlim is about to overwrite ${overwriteTargets.length} existing file${overwriteTargets.length === 1 ? "" : "s"}.`,
            detail,
            buttons: ["Cancel", "Overwrite"],
            defaultId: 0,
            cancelId: 0,
          });

          if (result.response !== 1) {
            return [];
          }
        }

        return convertAudioBatch(inputPaths, options, (progress) => {
          rpc.send.conversionProgress(progress);
        });
      },
      async openOutputPath({ path }) {
        return Utils.openPath(path);
      },
    },
    messages: {},
  },
});

const url = await getMainViewUrl();

ApplicationMenu.setApplicationMenu([
  {
    label: "AudioSlim",
    submenu: [
      { role: "about" },
      { type: "separator" },
      { role: "hide" },
      { role: "hideOthers" },
      { role: "showAll" },
      { type: "separator" },
      { role: "quit", accelerator: "Cmd+Q" },
    ],
  },
  {
    label: "File",
    submenu: [{ role: "quit", accelerator: "Cmd+Q" }],
  },
  {
    label: "Edit",
    submenu: [
      { role: "undo" },
      { role: "redo" },
      { type: "separator" },
      { role: "cut" },
      { role: "copy" },
      { role: "paste" },
      { role: "selectAll" },
    ],
  },
]);

new BrowserWindow({
  title: "AudioSlim",
  url,
  rpc,
  frame: {
    width: 900,
    height: 700,
    x: 200,
    y: 200,
  },
});
