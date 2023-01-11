import dts from "vite-plugin-dts";

export default {
	build: {
		lib: {
			entry: {
				main: "index.ts",
				fs: "fs/index.ts",
			},
			fileName: (fmt, name) => {
				name = name === "main" ? "index" : `${name}/index`;
				const ext = fmt === "es" ? "js" : "cjs";
				return `${name}.${ext}`;
			},
		},
	},
	plugins: [dts()],
};
