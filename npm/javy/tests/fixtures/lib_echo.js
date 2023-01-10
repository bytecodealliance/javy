import { readFileSync, writeFileSync } from "../../fs/index.ts";

writeFileSync(1, readFileSync(0));
