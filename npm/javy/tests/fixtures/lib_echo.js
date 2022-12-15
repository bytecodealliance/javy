import { readSync, writeSync } from "../../fs";

writeSync(1, readSync(0));
