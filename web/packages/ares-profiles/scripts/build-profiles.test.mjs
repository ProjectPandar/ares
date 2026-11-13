import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const artifacts = process.env.ARES_PROFILE_TEST_ARTIFACTS;
assert.ok(artifacts, "set ARES_PROFILE_TEST_ARTIFACTS to a fresh artifact directory");
const fixtureRoot = fileURLToPath(new URL("../../../../tests/parity/profile-json-bom/", import.meta.url));
const provenance = JSON.parse(readFileSync(join(fixtureRoot, "provenance.json"), "utf8"));
const bom = Buffer.from([0xef, 0xbb, 0xbf]);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });

function workspace(name) {
  const root = mkdtempSync(join(artifacts, name + "-"));
  mkdirSync(join(root, "package/scripts"), { recursive: true });
  mkdirSync(join(root, "package/src"));
  copyFileSync(new URL("./build-profiles.mjs", import.meta.url), join(root, "package/scripts/build-profiles.mjs"));
  copyFileSync(new URL("../src/index.d.ts", import.meta.url), join(root, "package/src/index.d.ts"));
  save(join(root, "package/package.json"), { type: "module" });
  return root;
}

function build(root, source, sourceKind) {
  const script = join(root, "package/scripts/build-profiles.mjs");
  assert.equal(sha256(readFileSync(script)), sha256(readFileSync(new URL("./build-profiles.mjs", import.meta.url))));
  const result = spawnSync(process.execPath, [script], {
    env: { ...process.env, ORCA_SLICER_DIR: source }, maxBuffer: 64 * 1024 * 1024,
  });
  writeFileSync(join(root, "stdout"), result.stdout, { flag: "wx" });
  writeFileSync(join(root, "stderr"), result.stderr, { flag: "wx" });
  save(join(root, "RESULT.json"), {
    source, sourceKind, status: result.status, signal: result.signal,
    builderSha256: sha256(readFileSync(script)),
  });
  return result;
}

test("unchanged release resources build and expose Snapmaker U1 multi-bed support", async () => {
  const source = process.env.ORCA_SLICER_DIR;
  assert.ok(source, "set ORCA_SLICER_DIR to the release source checkout");
  const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: source, encoding: "utf8" });
  assert.equal(commit, provenance.commit + "\n");
  for (const input of provenance.inputs) {
    assert.equal(sha256(readFileSync(join(fixtureRoot, input.file))), input.sha256);
    const bytes = readFileSync(join(source, input.upstream));
    assert.equal(bytes.length, input.bytes);
    assert.equal(sha256(bytes), input.sha256);
  }
  const root = workspace("release-original");
  save(join(root, "INPUTS.json"), provenance);
  const result = build(root, source, "unchanged upstream release resource checkout");
  assert.equal(result.status, 0, result.stderr.toString("utf8"));
  const catalog = await import(pathToFileURL(join(root, "package/dist/index.js")));
  assert.equal(catalog.ORCA_SLICER_COMMIT, provenance.commit);
  const benfab = await catalog.loadVendor("BenFab");
  for (const input of provenance.inputs) {
    const bytes = readFileSync(join(fixtureRoot, input.file));
    const document = JSON.parse(bytes.subarray(3).toString("utf8"));
    assert.deepEqual(benfab.files["process/" + input.file], document);
    assert.equal(sha256(readFileSync(join(source, input.upstream))), input.sha256);
  }
  const snapmaker = await catalog.loadVendor("Snapmaker");
  assert.equal(snapmaker.files["machine/fdm_U1.json"].support_multi_bed_types, "1");
  save(join(root, "CATALOG-ASSERTIONS.json"), {
    commit: catalog.ORCA_SLICER_COMMIT, vendors: catalog.vendors.length,
    originalBenFabDocuments: provenance.inputs.length, supportMultiBedTypes: "1",
    benfab, snapmaker,
  });
});

async function controlled(name, prefix, succeeds) {
  const root = workspace(name);
  const source = join(root, "source");
  mkdirSync(join(source, "resources/profiles/Fixture/process"), { recursive: true });
  const index = { name: "Fixture", process_list: [{ sub_path: "process/fixture.json" }] };
  const document = {
    type: "process", name: "Fixture", version: "2.5.0", instantiation: "true",
    layer_height: "0.27", description: "\uFEFFkeep", inherits: "parent",
  };
  const bytes = Buffer.concat([prefix, Buffer.from(JSON.stringify(document))]);
  writeFileSync(join(source, "resources/profiles/Fixture.json"),
    Buffer.concat([prefix.equals(bom) ? bom : Buffer.alloc(0), Buffer.from(JSON.stringify(index))]), { flag: "wx" });
  writeFileSync(join(source, "resources/profiles/Fixture/process/fixture.json"), bytes, { flag: "wx" });
  execFileSync("git", ["init", "--quiet"], { cwd: source });
  execFileSync("git", ["add", "resources"], { cwd: source });
  execFileSync("git", ["-c", "core.hooksPath=/dev/null", "-c", "user.name=Profile fixture",
    "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "--no-gpg-sign",
    "-m", "test: record JSON byte fixture", "-m", "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"], { cwd: source });
  const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: source, encoding: "utf8" });
  save(join(root, "INPUTS.json"), {
    sourceKind: "synthetic controlled JSON fixture, not an upstream release producer",
    commit, prefixHex: prefix.toString("hex"), profileSha256: sha256(bytes),
  });
  const result = build(root, source, "synthetic controlled JSON fixture");
  assert.equal(sha256(readFileSync(join(source, "resources/profiles/Fixture/process/fixture.json"))), sha256(bytes));
  if (!succeeds) {
    assert.notEqual(result.status, 0, "invalid initial bytes must not be normalized into JSON");
    assert.match(result.stderr.toString("utf8"), /SyntaxError/);
    return;
  }
  assert.equal(result.status, 0, result.stderr.toString("utf8"));
  const catalog = await import(pathToFileURL(join(root, "package/dist/index.js")));
  assert.equal(catalog.ORCA_SLICER_COMMIT + "\n", commit);
  const vendor = await catalog.loadVendor("Fixture");
  assert.deepEqual(vendor.index, index);
  assert.deepEqual(vendor.files["process/fixture.json"], document);
  save(join(root, "PARSED-VENDOR.json"), vendor);
}

test("one initial BOM preserves strings, typed values and inheritance", () => controlled("single-bom", bom, true));
test("non-BOM profile remains unchanged", () => controlled("no-bom", Buffer.alloc(0), true));
for (const [name, prefix] of [
  ["repeated", Buffer.concat([bom, bom])],
  ["after-whitespace", Buffer.concat([Buffer.from(" "), bom])],
  ["truncated-one-byte", Buffer.from([0xef])],
  ["truncated-two-bytes", Buffer.from([0xef, 0xbb])],
  ["invalid-third-byte", Buffer.from([0xef, 0xbb, 0xbe])],
  ["utf16-le", Buffer.from([0xff, 0xfe])],
  ["utf16-be", Buffer.from([0xfe, 0xff])],
]) {
  test(name + " prefix is rejected", () => controlled(name, prefix, false));
}
