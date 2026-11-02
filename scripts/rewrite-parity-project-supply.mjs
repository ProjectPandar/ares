#!/usr/bin/env node
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const apply = process.argv.slice(2).includes("--apply");
const base = join(root, "crates/ares-cli/tests/orca_parity");
const implementation = "runner/application/";
const names = [
  "slice_project_plate_processed_validated", "slice_project_plate_processed",
  "slice_project_plate_validated", "slice_project_plate", "slice_project", "slice_plater_plate",
];

// Mask comments and Rust literals without changing source offsets. Lifetimes are not literals.
function codeOnly(text, literals = []) {
  const masked = text.split("");
  function blank(start, end) {
    for (let at = start; at < end; at++) if (masked[at] !== "\n") masked[at] = " ";
  }
  for (let at = 0; at < text.length;) {
    const start = at;
    let literal = false;
    if (text.startsWith("//", at)) {
      at = text.indexOf("\n", at);
      if (at < 0) at = text.length;
    } else if (text.startsWith("/*", at)) {
      at += 2;
      let depth = 1;
      while (at < text.length && depth) {
        if (text.startsWith("/*", at)) { depth++; at += 2; }
        else if (text.startsWith("*/", at)) { depth--; at += 2; }
        else at++;
      }
      if (depth) throw new Error("unterminated Rust comment");
    } else {
      const raw = /^(?:br|r)(#*)"/.exec(text.slice(at));
      const character = /^'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'/u.exec(text.slice(at));
      if (raw && (at === 0 || !/[A-Za-z0-9_]/.test(text[at - 1]))) {
        const end = text.indexOf('"' + raw[1], at + raw[0].length);
        if (end < 0) throw new Error("unterminated Rust raw string");
        at = end + 1 + raw[1].length;
        literal = true;
      } else if (text[at] === '"') {
        at++;
        while (at < text.length && text[at] !== '"') at += text[at] === "\\" ? 2 : 1;
        if (at === text.length) throw new Error("unterminated Rust string");
        at++;
        literal = true;
      } else if (character) {
        at += character[0].length;
        literal = true;
      } else {
        at++;
        continue;
      }
    }
    if (literal) literals.push(text.slice(start, at));
    blank(start, at);
  }
  return masked.join("");
}

function paths(directory) {
  return readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name, "en"))
    .flatMap((entry) => entry.isDirectory() ? paths(join(directory, entry.name))
      : entry.name.endsWith(".rs") ? [join(directory, entry.name)] : []);
}

function edits(text) {
  const code = codeOnly(text);
  const result = [];
  const callee = new RegExp("\\bares_core::(" + names.join("|") + ")(?=\\s*\\()", "g");
  for (const match of code.matchAll(callee)) {
    result.push({ at: match.index, old: text.slice(match.index, match.index + match[0].length),
      replacement: "crate::runner::application::" + match[1], kind: "callee" });
  }
  const constructor = /(?<![A-Za-z0-9_:])(?:std::process::)?Command::new\s*\(\s*env!\s*\(\s*"CARGO_BIN_EXE_ares"\s*\)\s*\)/g;
  for (const match of text.matchAll(constructor)) {
    if (!code.slice(match.index, match.index + match[0].indexOf("env!")).includes("Command::new")) continue;
    result.push({ at: match.index, old: match[0], replacement: "crate::runner::application::cli_command()", kind: "constructor" });
  }
  return result.sort((a, b) => b.at - a.at);
}

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.status !== 0) throw new Error(`${command} failed (${result.status})`);
}

function format(text, path) {
  const result = spawnSync("rustfmt", ["--edition", "2024", "--emit", "stdout", "--config", "skip_children=true"], {
    cwd: root, input: text, encoding: "utf8",
  });
  if (result.status !== 0) throw new Error(`cannot normalize source: ${relative(root, path)}: ${result.stderr}`);
  return result.stdout;
}

function skeleton(text) {
  let normalized = text;
  for (const span of edits(text).filter((edit) => edit.kind === "constructor")) {
    normalized = normalized.slice(0, span.at) + "CLI_CONSTRUCTOR" + normalized.slice(span.at + span.old.length);
  }
  normalized = normalized.replace(/crate::runner::application::cli_command\s*\(\s*\)/g, "CLI_CONSTRUCTOR");
  const literals = [];
  let code = codeOnly(normalized, literals);
  code = code.replace(/\buse\b[^;]*;/g, "");
  code = code.replace(/crate::runner::application::(slice_[a-z_]+)/g, "ares_core::$1");
  return JSON.stringify([code.replace(/\s+/g, ""), literals]);
}

const selected = [];
for (const path of paths(base)) {
  if (relative(base, path).replaceAll("\\", "/").startsWith(implementation)) continue;
  const before = readFileSync(path, "utf8");
  const replacements = edits(before);
  if (!replacements.length) continue;
  let after = before;
  for (const edit of replacements) {
    after = after.slice(0, edit.at) + edit.replacement + after.slice(edit.at + edit.old.length);
  }
  selected.push({ path, before, after, replacements });
}
const summary = {
  files: selected.length,
  callees: selected.flatMap((file) => file.replacements).filter((edit) => edit.kind === "callee").length,
  constructors: selected.flatMap((file) => file.replacements).filter((edit) => edit.kind === "constructor").length,
  root_touched: false,
};
console.log(JSON.stringify({ ...summary, apply, paths: selected.map((file) => relative(root, file.path)) }, null, 2));
if (apply) {
  run("git", ["diff", "--quiet"]);
  for (const file of selected) {
    file.expected = format(file.after, file.path);
    writeFileSync(file.path, file.expected);
  }
  // rustc's own fixes remove orphaned Command imports, including grouped imports.
  run("cargo", ["fix", "-p", "ares-cli", "--test", "orca_parity", "--allow-dirty", "--allow-staged"]);
  run("cargo", ["fmt", "--all"]);
  for (const file of selected) {
    const after = format(readFileSync(file.path, "utf8"), file.path);
    writeFileSync(file.path, after);
    if (skeleton(file.expected) !== skeleton(after)) {
      throw new Error(`non-mechanical code change: ${relative(root, file.path)}`);
    }
    if (after.split("\n").length - 1 >= 400) {
      throw new Error(`line limit reached; stop for review: ${relative(root, file.path)}`);
    }
  }
  const changed = spawnSync("git", ["diff", "--name-only"], { cwd: root, encoding: "utf8" });
  if (changed.status !== 0) throw new Error("cannot inspect generated diff");
  const allowed = new Set(selected.map((file) => relative(root, file.path).replaceAll("\\", "/")));
  for (const path of changed.stdout.trim().split("\n").filter(Boolean)) {
    if (!allowed.has(path)) throw new Error(`unexpected changed path: ${path}`);
  }
}
