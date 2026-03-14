import type { ElectrobunRPCSchema } from "electrobun/bun";
import type { ConversionOptions, ConversionProgress } from "./types";

export type AudioSlimRpcSchema = ElectrobunRPCSchema & {
  bun: {
    requests: {
      checkFfmpeg: { params: undefined; response: string };
      pickAudioFiles: { params: undefined; response: string[] };
      convertAudio: {
        params: { inputPaths: string[]; options: ConversionOptions };
        response: string[];
      };
      openOutputPath: { params: { path: string }; response: boolean };
    };
    messages: {};
  };
  webview: {
    requests: {};
    messages: {
      conversionProgress: ConversionProgress;
    };
  };
};
