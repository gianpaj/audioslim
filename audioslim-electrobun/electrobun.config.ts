import type { ElectrobunConfig } from "electrobun";

export default {
	app: {
		name: "audioslim",
		identifier: "it.gianpaolo.audioslim.electrobun",
		version: "0.0.1",
	},
	build: {
		targets: "macos-arm64",
		// Vite builds to dist/, we copy from there
		copy: {
			"dist/index.html": "views/mainview/index.html",
			"dist/assets": "views/mainview/assets",
			"vendors/ffmpeg/macos-arm64/ffmpeg": "bin/ffmpeg",
		},
		// Ignore Vite output in watch mode — HMR handles view rebuilds separately
		watchIgnore: ["dist/**"],
		mac: {
			bundleCEF: false,
		},
		linux: {
			bundleCEF: false,
		},
		win: {
			bundleCEF: false,
		},
	},
	scripts: {
		preBuild: "scripts/prepare-ffmpeg.ts",
	},
} satisfies ElectrobunConfig;
