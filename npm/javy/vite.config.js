import dts from "vite-plugin-dts";

export default {
	build: {
		lib: {
			entry: {
				main: "src/index.ts",
				fs: "src/fs/index.ts",
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
