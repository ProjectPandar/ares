#!/usr/bin/env bun
// LPT timing-balanced Playwright shard planner, completeness prover, and timing recorder.
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const DEFAULT_DURATION_MS = 10_000;
const SPLIT_THRESHOLD_MS = 20_000;

function parseArgs(argv) {
  const [command, ...rest] = argv;
  const opts = {};
  for (let i = 0; i < rest.length; i += 1) {
    const arg = rest[i];
    if (arg.startsWith("--")) {
      const key = arg.slice(2);
      const next = rest[i + 1];
      if (next !== undefined && !next.startsWith("--")) {
        opts[key] = next;
        i += 1;
      } else {
        opts[key] = "true";
      }
    }
  }
  return { command, opts };
}

function matchesRule(pathStr, rule) {
  if (!rule) return false;
  if (Array.isArray(rule)) return rule.some((r) => matchesRule(pathStr, r));
  if (rule instanceof RegExp) return rule.test(pathStr);
  return pathStr.endsWith(String(rule));
}

async function discoverSpecs(webDir, projectName) {
  const cfgMod = await import(pathToFileURL(join(webDir, "playwright.config.ts")).href);
  const cfg = cfgMod.default;
  const project = (cfg.projects ?? []).find((p) => p.name === projectName);
  if (!project) throw new Error(`Unknown Playwright project: ${projectName}`);
  const testDir = join(webDir, cfg.testDir ?? "e2e");
  const allSpecs = readdirSync(testDir)
    .filter((f) => f.endsWith(".spec.ts"))
    .sort();
  return allSpecs.filter((file) => {
    const full = join(testDir, file);
    if (project.testMatch && !matchesRule(full, project.testMatch)) return false;
    if (project.testIgnore && matchesRule(full, project.testIgnore)) return false;
    return true;
  });
}

function loadTimings(timingsPath) {
  if (!timingsPath || !existsSync(timingsPath)) return {};
  return JSON.parse(readFileSync(timingsPath, "utf8"));
}

function assignLpt(specs, total, timings) {
  const ranked = specs
    .map((file) => ({ file, ms: Number(timings[file] ?? DEFAULT_DURATION_MS) }))
    .sort((a, b) => b.ms - a.ms || a.file.localeCompare(b.file));
  const shards = Array.from({ length: total }, () => ({ totalMs: 0, files: [] }));
  for (const item of ranked) {
    let best = 0;
    for (let i = 1; i < shards.length; i += 1) {
      if (shards[i].totalMs < shards[best].totalMs) best = i;
    }
    shards[best].files.push(item.file);
    shards[best].totalMs += item.ms;
  }
  for (const s of shards) s.files.sort();
  return shards;
}

function proveCompleteness(specs, shards) {
  const seen = new Map();
  for (let idx = 0; idx < shards.length; idx += 1) {
    for (const file of shards[idx].files) {
      if (seen.has(file)) {
        throw new Error(`Shard completeness violation: duplicate file ${file} in shards ${seen.get(file)} and ${idx + 1}`);
      }
      seen.set(file, idx + 1);
    }
  }
  const missing = specs.filter((f) => !seen.has(f));
  const extra = [...seen.keys()].filter((f) => !specs.includes(f));
  if (missing.length > 0 || extra.length > 0) {
    throw new Error(`Shard completeness violation: missing=${JSON.stringify(missing)} extra=${JSON.stringify(extra)}`);
  }
  return { totalSpecs: specs.length, shardCount: shards.length };
}

function extractReportTimings(reportPath) {
  const report = JSON.parse(readFileSync(reportPath, "utf8"));
  const perFile = {};
  const walk = (suite, inheritedFile) => {
    const file = suite.file || inheritedFile;
    for (const spec of suite.specs ?? []) {
      const f = spec.file || file;
      for (const test of spec.tests ?? []) {
        for (const result of test.results ?? []) {
          perFile[f] = (perFile[f] ?? 0) + Number(result.duration ?? 0);
        }
      }
    }
    for (const child of suite.suites ?? []) walk(child, file);
  };
  for (const suite of report.suites ?? []) walk(suite, "");
  return perFile;
}

const { command, opts } = parseArgs(process.argv.slice(2));
const webDir = resolve(opts["web-dir"] ?? "web");
const project = opts.project ?? "desktop";
const total = Number(opts.total ?? 4);
const timingsPath = opts.timings ? resolve(opts.timings) : "";

if (command === "select" || command === "verify") {
  if (!Number.isInteger(total) || total < 1) throw new Error(`Invalid --total: ${opts.total}`);
  const specs = await discoverSpecs(webDir, project);
  if (specs.length === 0) throw new Error(`No specs discovered for project ${project}`);
  const timings = loadTimings(timingsPath);
  const shards = assignLpt(specs, total, timings);
  const proof = proveCompleteness(specs, shards);

  if (command === "verify") {
    const summary = shards.map((s, i) => `shard ${i + 1}/${total}: ${s.files.length} files (~${Math.round(s.totalMs / 1000)}s)`);
    console.log(`Shard completeness PROVEN: ${proof.totalSpecs} unique specs partitioned across ${proof.shardCount} disjoint shards`);
    for (const line of summary) console.log(`  ${line}`);
    process.exit(0);
  }

  const shard = Number(opts.shard);
  if (!Number.isInteger(shard) || shard < 1 || shard > total) throw new Error(`Invalid --shard: ${opts.shard}`);
  const selected = shards[shard - 1].files;
  console.error(`shard ${shard}/${total}: ${selected.length}/${specs.length} specs (~${Math.round(shards[shard - 1].totalMs / 1000)}s)`);
  for (const file of selected) console.log(file);
} else if (command === "record") {
  const reportPath = resolve(opts.report ?? "");
  const outPath = resolve(opts.out ?? "");
  if (!existsSync(reportPath)) throw new Error(`Missing Playwright JSON report: ${reportPath}`);
  const existing = loadTimings(timingsPath);
  const measured = extractReportTimings(reportPath);
  const merged = { ...existing, ...measured };
  mkdirSync(dirname(outPath), { recursive: true });
  writeFileSync(outPath, `${JSON.stringify(merged, null, 2)}\n`);
  const slow = Object.entries(measured)
    .filter(([, ms]) => ms > SPLIT_THRESHOLD_MS)
    .sort((a, b) => b[1] - a[1]);
  console.log(`Recorded timings for ${Object.keys(measured).length} spec files -> ${outPath}`);
  if (slow.length > 0) {
    console.log(`Spec files > ${SPLIT_THRESHOLD_MS / 1000}s split threshold (${slow.length}):`);
    for (const [file, ms] of slow) console.log(`  ${(ms / 1000).toFixed(1)}s  ${file}`);
  }
} else {
  console.error("Usage: balance-web-shards.mjs <select|verify|record> [options]");
  process.exit(2);
}
