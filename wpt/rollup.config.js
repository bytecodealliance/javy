import * as fs from "fs/promises";
import * as path from "path";

const projectRoot = path.join(
  path.dirname(new URL(import.meta.url.toString()).pathname),
  "upstream"
);
const MATCHER = /^\/\/\s*META:\s+script=(.+)\s*$/gm;
export default {
  output: {
    file: "bundle.js",
    format: "es",
  },
  plugins: [
    // This plugin transforms the WPT-invented `// META: script=`
    // directives into ESM imports.
    {
      name: "wpt-import-sytnax",
      transform(chunk, id) {
        const imports = [];
        const newCode = chunk.replaceAll(MATCHER, (_match, ref) => {
          let base;
          if (ref.startsWith(".")) {
            base = path.dirname(id);
          } else {
            base = projectRoot;
          }
          const refPath = path.join(base, ref);
          imports.push(refPath);
          return "";
        });
        return `
          ${imports.map((i) => `import ${JSON.stringify(i)};`).join("\n")}
          ${newCode}
        `;
      },
    },
    // This plugin injects an import for the test harness
    // into the top-level file and fixes up the global scope with
    // stuff we need.
    {
      name: "test-spec",
      resolveId(id) {
        if (id !== "custom:test_spec") return;
        return id;
      },
      async load(id) {
        if (id !== "custom:test_spec") return;
        const { default: spec } = await import("./test_spec.js");
        const modules = await Promise.all(
          spec.map(async ({ testFile, ignoredTests }) => {
            const { id } = await this.resolve(testFile);
            const module = await this.load({ id, resolveDependencies: true });
            const [imports, other] = splitOffImports(module.code);
            return [
              imports,
              `
              (function () {
                globalThis.ignoredTests = ${JSON.stringify(ignoredTests)};
                ${other}
              })();
            `,
            ];
          })
        );

        return `
          ${modules.map(([imports, other]) => imports).join("\n")}
          
          export default function() {
            ${modules.map(([imports, other]) => other).join("\n")}
          }
        `;
      },
    },
  ],
};

// This is brittle and should be improved.
function splitOffImports(code) {
  const lines = code.split("\n");
  const imports = lines
    .filter((line) => line.trim().startsWith("import "))
    .join("\n");
  const other = lines
    .filter((line) => !line.trim().startsWith("import "))
    .join("\n");
  return [imports, other];
}
