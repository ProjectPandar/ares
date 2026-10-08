// Lighthouse audit of the production build served by `vite preview`: the mobile
// (default) and desktop presets, RUNS runs each, the median of every score and
// metric, JSON and HTML reports and a summary. Exits non-zero when a median
// score misses its target. Run `npm run build` first.
// Usage: node e2e/lighthouse.mjs [outDir]   (ARES_E2E_CHROME: Chrome binary)
import { spawn } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import * as chromeLauncher from "chrome-launcher";
import lighthouse, { desktopConfig } from "lighthouse";
import { computeMedianRun } from "lighthouse/core/lib/median-run.js";

const APP_URL = "http://127.0.0.1:4174/";
const RUNS = 3;
const TARGETS = { accessibility: 95, "best-practices": 95, seo: 90 };
const CATEGORIES = ["performance", "accessibility", "best-practices", "seo"];
const METRICS = {
  lcpMs: "largest-contentful-paint",
  tbtMs: "total-blocking-time",
  cls: "cumulative-layout-shift",
  fcpMs: "first-contentful-paint",
  transferBytes: "total-byte-weight",
};
const PRESETS = { mobile: undefined, desktop: desktopConfig };

const web = fileURLToPath(new URL("../", import.meta.url));
const out = process.argv[2] ?? fileURLToPath(new URL("../../target/web-lighthouse/", import.meta.url));
rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

const median = (values) => values.toSorted((a, b) => a - b)[Math.floor(values.length / 2)];

async function startPreview() {
  const server = spawn("npm", ["run", "preview", "-w", "ares-app", "--", "--host", "127.0.0.1", "--strictPort"],
    { cwd: web, detached: true, stdio: "ignore" });
  const exited = new Promise((resolve) => server.once("exit", resolve));
  for (let attempt = 0; attempt < 120; attempt++) {
    const up = await fetch(APP_URL).then((response) => response.ok, () => false);
    if (up) return { stop: () => process.kill(-server.pid, "SIGTERM"), exited };
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  process.kill(-server.pid, "SIGTERM");
  throw new Error(`vite preview did not serve ${APP_URL}`);
}

async function audit(preset, run) {
  const chrome = await chromeLauncher.launch({
    chromePath: process.env.ARES_E2E_CHROME || undefined,
    chromeFlags: ["--headless=new", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
  });
  try {
    const result = await lighthouse(APP_URL, { port: chrome.port, output: ["json", "html"], logLevel: "error" },
      PRESETS[preset]);
    const [json, html] = result.report;
    writeFileSync(`${out}${preset}-${run}.report.json`, json);
    writeFileSync(`${out}${preset}-${run}.report.html`, html);
    return result.lhr;
  } finally {
    chrome.kill();
  }
}

function summarize(lhrs) {
  const scores = Object.fromEntries(CATEGORIES.map((id) =>
    [id, median(lhrs.map((lhr) => Math.round(lhr.categories[id].score * 100)))]));
  const metrics = Object.fromEntries(Object.entries(METRICS).map(([name, id]) =>
    [name, median(lhrs.map((lhr) => lhr.audits[id].numericValue))]));
  const runs = lhrs.map((lhr) => Object.fromEntries(CATEGORIES.map((id) => [id, Math.round(lhr.categories[id].score * 100)])));
  return { scores, metrics, runs, medianRun: lhrs.indexOf(computeMedianRun(lhrs)) + 1 };
}

const preview = await startPreview();
const summary = { url: APP_URL, runs: RUNS, lighthouse: "", targets: TARGETS, presets: {} };
try {
  for (const preset of Object.keys(PRESETS)) {
    const lhrs = [];
    for (let run = 1; run <= RUNS; run++) lhrs.push(await audit(preset, run));
    summary.lighthouse = lhrs[0].lighthouseVersion;
    summary.presets[preset] = summarize(lhrs);
  }
} finally {
  preview.stop();
  await preview.exited;
}

const misses = Object.entries(summary.presets).flatMap(([preset, { scores }]) =>
  Object.entries(TARGETS).filter(([id, target]) => scores[id] < target).map(([id, target]) =>
    `${preset} ${id} ${scores[id]} < ${target}`));
summary.misses = misses;
writeFileSync(`${out}summary.json`, `${JSON.stringify(summary, null, 2)}\n`);

const rows = Object.entries(summary.presets).map(([preset, { scores, metrics, medianRun }]) =>
  `| ${preset} | ${CATEGORIES.map((id) => scores[id]).join(" | ")} | ${Math.round(metrics.lcpMs)} | ${Math.round(metrics.tbtMs)} | `
  + `${metrics.cls.toFixed(3)} | ${Math.round(metrics.fcpMs)} | ${Math.round(metrics.transferBytes)} | ${medianRun} |`);
writeFileSync(`${out}summary.md`, [
  `# Lighthouse ${summary.lighthouse}: ${APP_URL}, median of ${RUNS} runs`,
  "",
  "| preset | Performance | Accessibility | Best Practices | SEO | LCP ms | TBT ms | CLS | FCP ms | transfer B | median run |",
  "|---|---|---|---|---|---|---|---|---|---|---|",
  ...rows,
  "",
  `Targets: ${Object.entries(TARGETS).map(([id, target]) => `${id} >= ${target}`).join(", ")}.`,
  misses.length ? `Missed: ${misses.join("; ")}.` : "All targets met.",
  "",
].join("\n"));
console.log(`${out}summary.md`);
if (misses.length) {
  console.error(`Lighthouse targets missed: ${misses.join("; ")}`);
  process.exitCode = 1;
}
