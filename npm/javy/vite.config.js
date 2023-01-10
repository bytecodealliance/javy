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
				fmt = fmt === "es" ? "" : `.${fmt}`;
				return `${name}${fmt}.js`;
			},
		},
	},
	plugins: [dts()],
};
