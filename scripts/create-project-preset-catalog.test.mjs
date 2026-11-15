import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

assert(process.env.ARES_CATALOG_TEST_ARTIFACTS, "set ARES_CATALOG_TEST_ARTIFACTS to a fresh directory");
assert(process.env.ORCA_SLICER_DIR, "set ORCA_SLICER_DIR to the original release checkout");
const artifacts = resolve(process.env.ARES_CATALOG_TEST_ARTIFACTS);
const upstream = resolve(process.env.ORCA_SLICER_DIR);
const script = fileURLToPath(new URL("./create-project-preset-catalog.mjs", import.meta.url));
const lists = ["machine_model_list", "machine_list", "process_list", "filament_list"];
const bom = Buffer.from([0xef, 0xbb, 0xbf]);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const index = { process_list: [{ name: "Raw", sub_path: "process/raw.json" }], note: "\uFEFFvendor" };
const profile = { name: "Raw", inherits: "Base", note: "\uFEFFkeep", scalar: 0.2, vector: ["0.2", 2], flag: true, empty: null, legacy: "1" };
const indexBytes = Buffer.from(JSON.stringify(index));
const profileBytes = Buffer.from(JSON.stringify(profile));
mkdirSync(artifacts);
mkdirSync(join(artifacts, "fixtures"));
mkdirSync(join(artifacts, "runs"));

function fixture(name, manifest, document) {
  const root = join(artifacts, "fixtures", name);
  mkdirSync(join(root, "Boundary", "process"), { recursive: true });
  writeFileSync(join(root, "Boundary.json"), manifest);
  writeFileSync(join(root, "Boundary", "process", "raw.json"), document);
  return root;
}

function invoke(name, source) {
  const record = join(artifacts, "runs", name);
  mkdirSync(record);
  const output = join(record, "catalog");
  const args = [script, "--profiles", source, "--out", output];
  const result = spawnSync(process.execPath, args, { maxBuffer: 64 * 1024 * 1024 });
  writeFileSync(join(record, "stdout"), result.stdout);
  writeFileSync(join(record, "stderr"), result.stderr);
  writeFileSync(join(record, "launch.json"), JSON.stringify({ executable: process.execPath, args, status: result.status, signal: result.signal }, null, 2) + "\n");
  return { ...result, output, record };
}

function verifyOriginals(source, { output, record }) {
  const expected = [], hashes = {}, documents = [];
  let prefixed = 0;
  const readOriginal = (relative) => {
    const bytes = readFileSync(join(source, relative));
    const offset = bytes.subarray(0, 3).equals(bom) ? 3 : 0;
    prefixed += Number(offset !== 0);
    const value = JSON.parse(bytes.subarray(offset).toString("utf8"));
    return { bytes, value };
  };
  const verifyCopy = (relative, bytes) => {
    const copy = readFileSync(join(output, "native", "system", relative));
    assert(copy.equals(bytes), relative + " copied bytes differ");
    assert(readFileSync(join(source, relative)).equals(bytes), relative + " original bytes changed");
    hashes[relative] = hash(bytes);
    documents.push({ path: relative, bytes: bytes.length, sha256: hash(bytes) });
  };
  for (const entry of readdirSync(source, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name, "en"))) {
    if (!entry.isFile() || !entry.name.endsWith(".json")) continue;
    const { bytes, value: manifest } = readOriginal(entry.name);
    if (!lists.some((list) => Array.isArray(manifest[list]))) continue;
    const id = entry.name.slice(0, -5), files = {};
    verifyCopy(entry.name, bytes);
    const paths = new Set(lists.flatMap((list) => (manifest[list] ?? []).map((item) => item.sub_path)));
    for (const path of [...paths].sort()) {
      const relative = `${id}/${path}`;
      const original = readOriginal(relative);
      verifyCopy(relative, original.bytes);
      files[path] = original.value;
    }
    expected.push({ id, index: manifest, files });
  }
  const catalog = readFileSync(join(output, "catalog.json"));
  assert(catalog.equals(Buffer.from(JSON.stringify({ vendors: expected, imports: [] }) + "\n")), "complete catalog bytes differ from original source values");
  assert.deepEqual(JSON.parse(readFileSync(join(output, "source-hashes.json"), "utf8")), {
    catalog_sha256: hash(catalog), vendors: expected.length, documents: documents.length, sha256: hashes,
  });
  writeFileSync(join(record, "verified-originals.json"), JSON.stringify({ vendors: expected.length, documents, bomDocuments: prefixed, catalogBytes: catalog.length, catalogSHA256: hash(catalog) }, null, 2) + "\n");
  return { prefixed, vendors: expected };
}

test("catalog preserves complete original release values, copied bytes and hashes", () => {
  const git = spawnSync("git", ["-C", upstream, "rev-parse", "HEAD"], { maxBuffer: 64 * 1024 * 1024 });
  assert.equal(git.status, 0, git.stderr.toString());
  assert.equal(git.stdout.toString(), "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19\n");
  const source = join(upstream, "resources", "profiles");
  const result = invoke("release-originals", source);
  assert.equal(result.status, 0, result.stderr.toString());
  const verified = verifyOriginals(source, result);
  assert.equal(verified.prefixed, 5);
});

test("unprefixed controls preserve JSON types and in-string U+FEFF", () => {
  const source = fixture("plain", indexBytes, profileBytes);
  const result = invoke("plain", source);
  assert.equal(result.status, 0, result.stderr.toString());
  verifyOriginals(source, result);
  const actual = JSON.parse(readFileSync(join(result.output, "catalog.json"), "utf8"));
  assert.deepEqual(actual, { vendors: [{ id: "Boundary", index, files: { "process/raw.json": profile } }], imports: [] });
});

for (const boundary of ["manifest", "profile"]) {
  test(`one initial UTF-8 BOM at ${boundary} preserves values, original bytes and hashes`, () => {
    const name = boundary + "-bom";
    const source = fixture(name, boundary === "manifest" ? Buffer.concat([bom, indexBytes]) : indexBytes,
      boundary === "profile" ? Buffer.concat([bom, profileBytes]) : profileBytes);
    const result = invoke(name, source);
    assert.equal(result.status, 0, result.stderr.toString());
    const verified = verifyOriginals(source, result);
    assert.equal(verified.prefixed, 1);
    assert.deepEqual(verified.vendors, [{ id: "Boundary", index, files: { "process/raw.json": profile } }]);
  });
}

const rejected = [
  ["repeated", (bytes) => Buffer.concat([bom, bom, bytes])],
  ["truncated-ef", (bytes) => Buffer.concat([Buffer.from([0xef]), bytes])],
  ["truncated-ef-bb", (bytes) => Buffer.concat([Buffer.from([0xef, 0xbb]), bytes])],
  ["displaced", (bytes) => Buffer.concat([Buffer.from(" "), bom, bytes])],
  ["utf16le", (bytes) => Buffer.concat([Buffer.from([0xff, 0xfe]), Buffer.from(bytes.toString("utf8"), "utf16le")])],
  ["utf16be", (bytes) => Buffer.concat([Buffer.from([0xfe, 0xff]), Buffer.from(bytes.toString("utf8"), "utf16le").swap16()])],
  ["bom-only", () => bom],
];
for (const boundary of ["manifest", "profile"]) {
  for (const [prefix, encode] of rejected) {
    test(`rejects ${prefix} at ${boundary} without modifying the source`, () => {
      const name = `${boundary}-${prefix}`;
      const manifest = boundary === "manifest" ? encode(indexBytes) : indexBytes;
      const document = boundary === "profile" ? encode(profileBytes) : profileBytes;
      const source = fixture(name, manifest, document);
      const result = invoke(name, source);
      assert.equal(result.status, 1);
      assert.match(result.stderr.toString(), /Unexpected token|Unexpected end of JSON input/);
      assert(!existsSync(join(result.output, "catalog.json")));
      assert(readFileSync(join(source, "Boundary.json")).equals(manifest));
      assert(readFileSync(join(source, "Boundary", "process", "raw.json")).equals(document));
    });
  }
}
