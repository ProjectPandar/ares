import { copyFileSync, cpSync } from "node:fs";

const destination = new URL("../app/dist/", import.meta.url);
copyFileSync(new URL("../../THIRD_PARTY_NOTICES.md", import.meta.url), new URL("THIRD_PARTY_NOTICES.md", destination));
cpSync(new URL("../../LICENSES/", import.meta.url), new URL("LICENSES/", destination), { recursive: true });
