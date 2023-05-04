import { readFileSync, writeFileSync } from "../../src/fs/index.ts";

writeFileSync(1, readFileSync(0));
