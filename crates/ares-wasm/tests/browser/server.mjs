import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const browserRoot = fileURLToPath(new URL(".", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../../../", import.meta.url));
const routes = new Map([
  ["/", [join(browserRoot, "index.html"), "text/html; charset=utf-8"]],
  [
    "/index.html",
    [join(browserRoot, "index.html"), "text/html; charset=utf-8"],
  ],
  [
    "/target/wasm-browser/ares_wasm.js",
    [
      join(repositoryRoot, "target", "wasm-browser", "ares_wasm.js"),
      "text/javascript; charset=utf-8",
    ],
  ],
  [
    "/target/wasm-browser/ares_wasm_bg.wasm",
    [
      join(repositoryRoot, "target", "wasm-browser", "ares_wasm_bg.wasm"),
      "application/wasm",
    ],
  ],
  [
    "/tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf",
    [
      join(
        repositoryRoot,
        "tests",
        "ksr_fdmtest_v4",
        "ksr_fdmtest_v4.project.3mf",
      ),
      "application/vnd.ms-package.3dmanufacturing-3dmodel+xml",
    ],
  ],
]);

createServer(async (request, response) => {
  const route = routes.get(new URL(request.url, "http://127.0.0.1").pathname);
  if (!route) {
    response.writeHead(404).end("not found");
    return;
  }
  try {
    const content = await readFile(route[0]);
    response.writeHead(200, { "Content-Type": route[1] });
    response.end(content);
  } catch {
    response.writeHead(404).end("not found");
  }
}).listen(4173, "127.0.0.1");
