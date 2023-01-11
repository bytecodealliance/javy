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
    // This plugin handles the WPT-invented `// META: script=`
    // directives.
    {
      name: "wpt-import-sytnax",
      async transform(chunk, id) {
        const imports = [];
        const newCode = chunk.replaceAll(MATCHER, (_match, ref) => {
          let base;
          if (!ref.startsWith("/")) {
            base = path.dirname(id);
          } else {
            base = projectRoot;
          }
          const refPath = path.join(base, ref);
          imports.push(refPath);
          return "";
        });

        // Originally, I just re-wrote the WPT import directives to ESM
        // imports. However, ES modules are isolated scopes, so top-level
        // variables would just get stripped/ignored as they were inaccessible
        // according to ESM semantics. WPT just inlines the code of the
        // referenced file, so I have to manually implement that behavior
        // here.
        const resolvedImports = await Promise.all(
          imports.map(async (importId) => {
            const resolution = await this.resolve(importId, id);
            const moduleInfo = await this.load(resolution);
            return moduleInfo.code;
          })
        );
        const transformedCode = `
          ${resolvedImports.join("\n")}
          ${newCode}
        `;
        return transformedCode;
      },
    },
    // This plugin handles the special import in `runner.js`.
    // It parses `test_spec.js` and concatenates all
    // the specified tests.
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
          spec.map(async ({ testFile, ignoredTests = [] }) => {
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
