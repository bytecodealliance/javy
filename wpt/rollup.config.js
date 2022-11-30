import * as path from "path";

const projectRoot = path.join(
  path.dirname(new URL(import.meta.url.toString()).pathname),
  "upstream"
);
const MATCHER = /^\/\/\s*META:\s+script=(.+)\s*$/gm;
const PRIVATE = Symbol();
export default {
  output: {
    file: "bundle.js",
    format: "es",
  },
  plugins: [
    // This plugin injects an import for the test harness
    // into the top-level file and fixes up the global scope with
    // stuff we need.
    {
      name: "harness-injector",
      async buildStart(options) {
        const resolvedInputs = await Promise.all(
          options.input.map((id) => this.resolve(id))
        );
        this[PRIVATE] = new Set(resolvedInputs.map((e) => e.id));
      },
      resolveId(id) {
        if (id !== "custom:globalFix") return;
        return id;
      },
      load(id) {
        if (id !== "custom:globalFix") return;
        return `
					globalThis.self = globalThis;
					global.location = {};
				`;
      },
      transform(chunk, id) {
        if (!this[PRIVATE].has(id)) return;
        return `
					import "custom:globalFix";
					import "${path.join(projectRoot, "/resources/testharness.js")}";
					import reporter from "${path.join(projectRoot, "/../reporter.js")}";
					add_completion_callback(reporter);
					${chunk}
				`;
      },
    },
    // This plugin transforms the WPT-invented `// META: script=`
    // directives into ESM imports.
    {
      name: "wpt-import-sytnax",
      transform(chunk, id) {
        return chunk.replaceAll(MATCHER, (_match, ref) => {
          let base;
          if (ref.startsWith(".")) {
            base = path.dirname(id);
          } else {
            base = projectRoot;
          }
          const refPath = path.join(base, ref);
          return `import ${JSON.stringify(refPath)};`;
        });
      },
    },
  ],
};
