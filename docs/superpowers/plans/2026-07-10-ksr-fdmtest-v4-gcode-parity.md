# KSR FDM Test V4 G-code Parity Implementation Plan

> **For agentic workers:** REQUIRED WORKFLOW SKILLS: use both `sdd-workflow` and `superpowers:subagent-driven-development` for every implementation task; every fresh implementation subagent also follows `superpowers:test-driven-development`. `superpowers:executing-plans` is not a substitute for either required workflow. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Ares slice the committed `ksr_fdmtest_v4.project.3mf`
entirely from its embedded model and options into production G-code whose
layer, deposition, lifecycle, control, template, timing, and material semantics
match the committed OrcaSlicer 2.4.2 reference.

**Architecture:** Add a byte-oriented project path beside the existing
explicit-option STL API. The project path is a source-cited Rust rewrite of the
fixed OrcaSlicer 2.4.2 `libslic3r` import, configuration, slicing, G-code, and
post-processing boundaries; it owns typed 3MF/config data and never retries
through the existing approximate STL pipeline. The final seam canonicalizes
independent-island ordering because upstream
`TriangleMeshSlicer.cpp:521-529` appends TBB results in scheduler order. It
never canonicalizes production output or reads golden data.

**Tech Stack:** Rust 1.91.0, edition 2024; `serde` concrete structs; `zip = 8.6.0` with only `deflate-flate2-zlib-rs`; `quick-xml = 0.41.0` with only `serialize`; in-repository Rust ports of the fixed upstream geometry algorithms; Tokio only at public async adapters; `cargo-nextest`; GitHub Actions Tier-1 matrix.

## Global Constraints

- The only upstream baseline is OrcaSlicer tag `v2.4.2`, commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
- The local `OrcaSlicer` checkout is not assumed to be at that commit. Every source inspection command uses `git -C OrcaSlicer show 8500fcdccaa10b5099ac20d252af3a7c560046f1:<path>` or a detached temporary worktree at that exact commit.
- `ares-core` stays byte-oriented, in-memory, and free of filesystem, clock, UI, terminal, OpenGL, executable invocation, and native-only dependencies.
- Tier 1 remains WASM, Windows, macOS, and Linux. New core dependencies must compile for `wasm32-unknown-unknown` with native/default features disabled where necessary.
- Starting when the stable `sliceProject` export lands, Tier-1 WASM evidence includes a committed headless-Chromium harness that imports the generated `wasm-bindgen` web module, passes the real 3MF as a browser `Uint8Array`, and observes either the exact typed `ProjectSlicingIncomplete` error (before parity) or the normalized byte-exact G-code (at parity). Rust-only `wasm32` checks do not replace this JavaScript boundary test.
- The first task adding each pinned dependency must resolve it through Cargo, inspect `cargo tree -e features` for the named feature set, commit the resulting `Cargo.lock`, and run the relevant native/WASM checks in that same task. A missing version/feature is a RED dependency failure to correct there, never an instruction to float to an unreviewed version.
- Known production JSON/XML documents and all known options use concrete serde structs and concrete field types. Production `serde_json::Value`, `Map`, `RawValue`, erased payloads, DOM-based type dispatch, and catch-all option-value enums are forbidden.
- Tests may use `serde_json::Value` and `json!`. A genuinely open production field needs the separately reviewed source-cited allowlist described by the spec; none is expected for this fixture.
- The 653 project keys are owned once in raw input: Printer 132, Process 352, Filament 122, and project/runtime residual 47. `ObjectOptions` (126), `RegionOptions` (153), and `GCodeOptions` (149) are typed effective projections across those raw scopes, not duplicate raw stores.
- The authoritative v2.4.2 option-type inventory is 650 real options plus metadata `from`, `name`, and `version`: `coBool=105`, `coBools=22`, `coEnum=44`, `coEnums=9`, `coFloat=160`, `coFloatOrPercent=36`, `coFloats=90`, `coInt=41`, `coInts=45`, `coPercent=25`, `coPercents=5`, `coPoint=4`, `coPoints=6`, `coPointsGroups=1`, `coString=30`, and `coStrings=27`.
- A field is not behaviorally complete merely because it deserializes. Each option read by a ported branch gets a focused RED/GREEN test that changes the option and observes typed effective state, geometry, ordered toolpaths, or emitted bytes. Inactive options may be retained-only with exact config serialization.
- Within Tasks 6-14, add inventory rows one option at a time: first a failing concrete type/default/wire-shape/canonical-serialization assertion for that key, then its field and direct typed dispatch arm, then focused GREEN before advancing. A task groups the reviewed commit; it does not authorize a fixture-shaped bulk struct or untested generated field dump.
- Production project slicing never reads the reference G-code, fixture path/name/hash, invokes Orca, uses FFI, copies pre-rendered toolpaths, or falls back to the STL `pipeline.rs` path.
- The project loader enforces the exact archive, path, XML, and JSON limits in the approved spec before semantic parsing.
- Modified Rust files stay at or below 400 physical lines. Files currently at exactly 400 LOC are split before modification.
- No new workspace crate is added. The existing `ares-core`, `ares-cli`, `ares-wasm`, and `ares-vgcode` ownership boundaries remain intact.
- The committed fixture hashes remain `698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9` for the 3MF and `10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3` for the reference G-code.
- The generator-normalized reference SHA-256
  `c61202df3fa26ffcb3064f2dbc02e06a89f95565b8325b31029ec4ed6cedcdc4`
  remains a fixture-integrity fact, not a parity oracle.
- Intermediate RED tests use expected vectors fixed before implementation from pinned upstream tests (`tests/libslic3r/test_{clipper_offset,clipper_utils,geometry,polygon,elephant_foot_compensation,placeholder_parser}.cpp` and `tests/fff_print/test_{trianglemesh,fill,gcode,gcode_timing,gcodewriter}.cpp`) or from a small hand-calculated cited formula. They never bless an Ares-generated snapshot. Fixture-level pre-G-code assertions are limited to facts independently available from the 3MF/reference contract (mesh, transforms, typed config, layer count/Z, exact config block); internal contour/surface/pre-seam snapshots are not reconstructed from final G-code. The committed G-code is post-filter/post-processor output and is never used as a byte oracle for raw rewrite stages.
- No active test is weakened to make an increment green. The CLI golden remains explicitly ignored until Task 31B, when it becomes an active semantic comparison. Progress evidence follows the stage-specific boundary/error/internal-diff/core-browser/CLI contract below.
- Complete G-code comparisons report bounded structural diagnostics. Tests never `assert_eq!`/`assert_ne!` multi-megabyte byte arrays.

## Completed Preconditions

Baseline commit `a0eec942f` is already pushed on
`codex/ksr-fdmtest-v4-parity`. It committed the two hash-pinned fixtures,
ARD-0023, the reviewed parity spec, and the active program near the top of
`docs/roadmap.md`. That roadmap section already lists all eight rewrite
increments and records that the one-source-line `PrintConfig.hpp` and
`PrintConfig.cpp` milestones and documents were removed, while staged
`PrintApply` milestones remain superseded. This satisfies the spec's
pre-implementation roadmap requirement; do not add a duplicate section or a
second baseline commit.

Before dispatching Task 1A, the primary agent verifies that `HEAD` descends
from `a0eec942f` and that the roadmap section still contains those eight items
and the supersession rule. A missing or weakened section is repaired and
reviewed as documentation work before Task 1A writes implementation code.

## Mandatory Gate After Every Task

Each task is one review/commit unit. The primary agent applies `sdd-workflow` and `superpowers:subagent-driven-development`; a fresh implementation subagent reads and follows `superpowers:test-driven-development` before its RED/GREEN cycle. Before staging, a fresh Codex reviewer and OpenCode's default model independently review the entire task diff against this plan, the spec, ARD-0023, and the fixed upstream commit. Both must return the exact literal `VERDICT: APPROVE`; any other outcome is fixed and re-reviewed by both reviewers. The primary agent uses `superpowers:verification-before-completion` before every success claim and commit.

Before diff review or verification, register only task-owned new files with `git add --intent-to-add -- <exact task paths>` so `git diff`, `git diff --check`, and both reviewers see their full contents. Intent-to-add is not approval to stage unrelated workspace files; the real staging command still occurs only after dual approval and documentation updates.

After dual approval:

1. Update `docs/roadmap.md` and, when option behavior changed, `docs/architecture/option-parity-v4.md` with completed behavior and remaining gaps.
2. Run fresh focused tests and the full local matrix:

   ```powershell
   cargo fmt --check
   cargo nextest run --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo check -p ares-core --target wasm32-unknown-unknown
   cargo check -p ares-wasm --target wasm32-unknown-unknown
   cargo nextest run -p ares-core --test no_unapproved_dynamic_values
   git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
   Get-ChildItem crates -Recurse -Filter *.rs | ForEach-Object {
       $lines = (Get-Content $_.FullName).Count
       if ($lines -gt 400) { throw "$($_.FullName): $lines LOC" }
   }
   ```

   Starting with Task 4, also build the `wasm-bindgen` web package and run the
   committed `crates/ares-wasm/tests/browser/` harness in headless Chromium.
   Tasks 4 through 30E pass the real fixture and assert the exact mapped
   `ProjectSlicingIncomplete` error. Task 31A changes the same harness to the
   single-generator-line normalized byte comparison; no task may substitute a
   Rust-only wasm test for this gate.

   Progress evidence is stage-specific. Through Task 3, run the ignored CLI
   golden once per task and retain its nonzero exit status plus bounded CLI
   boundary error. From Task 4 through Task 30E, the active browser assertion
   above replaces that CLI probe and must observe exactly
   `ProjectSlicingIncomplete`; do not claim a byte diff because no project
   G-code exists. Task 30F owns the first complete post-processed internal
   document and records the bounded byte/line difference. Task 31A activates
   exact core and browser equality. Task 31B removes the CLI ignore and
   activates exact CLI equality.

   For Tasks 1A through 3, use:

   ```powershell
   cargo nextest run -p ares-cli --test ksr_fdmtest_v4 --run-ignored all
   $goldenProgressExit = $LASTEXITCODE
   if ($goldenProgressExit -eq 0) {
       throw 'the CLI golden passed before the project API/browser boundary exists'
   }
   ```

   Task 31B removes the ignore; from that commit onward the active CLI golden runs inside `cargo nextest run --workspace` and must succeed.

3. Stage only the approved task files, create the task's listed Conventional Commit, and push `codex/ksr-fdmtest-v4-parity`.
4. Wait for the pushed commit's `tier1.yml` Windows/macOS/Linux/WASM jobs to succeed before starting a dependent task.

## Rollback Procedure

Record the SHA of every approved task commit in the SDD progress ledger. If a
pushed task must be rolled back, revert that whole SHA with `git revert`; do
not add a feature flag, retry path, alternate slicer, or runtime fallback. Run
the task's focused checks, the full Mandatory Gate, and the stage-appropriate
CLI-boundary/browser-incomplete/internal-diff/core-browser/CLI parity evidence
defined above. The revert diff receives fresh Codex and OpenCode review, then
is pushed and must reach green Tier-1 CI before any dependent task resumes. If
the revert itself needs a corrective change, that fix re-enters the same dual
review, verification, commit, push, and CI gate.

## Locked File and Interface Map

The following ownership is fixed for this plan. Additional sibling files are allowed only to keep a named owner below 400 LOC.

| Boundary | Ares files | Primary interfaces |
| --- | --- | --- |
| Golden/audits | `crates/ares-cli/tests/ksr_fdmtest_v4.rs`, `crates/ares-cli/tests/ksr_fdmtest_v4/golden.rs`, `crates/ares-core/tests/no_unapproved_dynamic_values.rs` | `normalize_generator_line`, `first_difference`, syntax fingerprint audit |
| Project package | `crates/ares-core/src/project.rs`, `project/archive.rs`, `project/content_types.rs`, `project/relationships.rs` | `load_project`, `ProjectArchive`, `PackagePath`, `ArchiveLimits` |
| Project documents | `project/xml.rs`, `project/model_xml.rs`, `project/model_settings.rs`, `project/slice_info.rs`, `project/filament_sequence.rs`, `project/plate.rs` | bounded XML validation plus named concrete serde wire structs from the spec |
| Project domain | `project/transform.rs`, `generation.rs` | `Project`, `ProjectObject`, `ProjectVolume`, `ProjectInstance`, `Transform3d`, `GenerationMetadata` |
| Configuration | `options/config_types.rs`, raw group files, effective projection files, `project_deserialize.rs`, `project_normalize.rs`, `config_export.rs`, `full_print_config.rs` | `ProjectSettings`, `SliceOptions`, `FullPrintConfig`, typed group structs |
| Geometry | `geometry.rs`, `geometry/coord.rs`, `geometry/clipper/*`, `geometry/polygon_ops.rs`, `mesh_slicer.rs`, `mesh_slicer/*`, `layer.rs` | `Coord`, `Point`, `Polygon`, `ExPolygon`, `Layer`, `LayerSlice` |
| Print geometry | `print_object.rs`, `print_object/slice.rs`, `print_object/surfaces.rs`, `surface.rs`, `perimeters/classic.rs`, `fill.rs`, `fill/*`, `brims.rs` | upstream-compatible surface, perimeter, fill, brim results |
| Motion/G-code | `print.rs`, `print_order.rs`, `extrusion_entity.rs`, `gcode_writer.rs`, `gcode_writer/*` | `Print`, ordered extrusion entities, stateful writer |
| Templates/blocks | `placeholder_parser.rs`, focused children, existing project-owned G-code block files | typed placeholder AST/context and exact block serializers |
| Processor/adapters | `project_slice.rs`, `gcode_processor.rs`, `gcode_processor/*`, CLI/WASM adapters, `crates/ares-wasm/tests/browser/` | `slice_project`, stable JavaScript `sliceProject`, `GCodeProcessorResult`, byte-oriented adapters |

---

### Task 1A: Establish the Tier-1 Workflow

**Upstream boundary:** No runtime port; this task creates the portability gate required for every later source-cited rewrite increment.

**Files:**
- Create: `.github/workflows/tier1.yml`

**Interfaces:**
- Produces a native matrix on `windows-latest`, `macos-latest`, and `ubuntu-latest`, one Linux formatting job, and one Linux WASM job.

- [ ] **Step 1: Add the workflow and make its first run RED if any existing platform fails**

  Native jobs install Rust 1.91.0 and `cargo-nextest`, then run `cargo nextest run --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`. The formatting job runs `cargo fmt --check`. The WASM job installs `wasm32-unknown-unknown` and checks `ares-core` and `ares-wasm`. Use `taiki-e/install-action@nextest`; do not download a platform-specific binary by hand.

- [ ] **Step 2: Fix only baseline portability failures and run the mandatory task gate**

  Task 1A omits only the explicit dynamic-value audit command because that test is introduced in Task 1C. Obtain dual approval, update the roadmap, commit, push, and wait for this new workflow itself to turn green:

  ```powershell
  git commit -m "ci: add Tier-1 Rust verification matrix"
  git push
  ```

---

### Task 1B: Byte-Exact Golden Harness

**Upstream boundary:** `Format/bbs_3mf.*` establishes project input identity; `GCodeProcessor.cpp` establishes the single generator line exception. No slicer implementation is added.

**Files:**
- Create: `crates/ares-cli/tests/ksr_fdmtest_v4.rs`
- Create: `crates/ares-cli/tests/ksr_fdmtest_v4/golden.rs`
- Create: `docs/architecture/option-parity-v4.md`
- Modify: workspace and `crates/ares-cli/Cargo.toml` with exact dev dependencies `sha2 = 0.11.0`, `regex = 1.13.0`, and `zip = { version = 8.6.0, default-features = false, features = ["deflate-flate2-zlib-rs"] }`

**Interfaces:**
- Produces `golden::normalize_one_generator_line(bytes, GeneratorKind) -> Result<Vec<u8>, String>`.
- Produces `golden::first_difference(expected, actual) -> Option<Difference>` with at most three context lines in its `Display` output.

- [ ] **Step 1: Write the golden helper and fixture-contract tests**

  The helper validates exactly one complete UTF-8 line before replacing it; it does not normalize line endings or any other bytes:

  ```rust
  pub(crate) enum GeneratorKind { Orca, Ares }

  pub(crate) fn normalize_one_generator_line(
      bytes: &[u8],
      kind: GeneratorKind,
  ) -> Result<Vec<u8>, String> {
      let name = match kind { GeneratorKind::Orca => "OrcaSlicer", GeneratorKind::Ares => "Ares" };
      let pattern = regex::Regex::new(&format!(
          r"(?m)^; generated by {name} 2\.4\.2 on \d{{4}}-\d{{2}}-\d{{2}} at \d{{2}}:\d{{2}}:\d{{2}}$"
      )).unwrap();
      let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
      let matches = pattern.find_iter(text).collect::<Vec<_>>();
      if matches.len() != 1 { return Err(format!("expected one {name} generator line, found {}", matches.len())); }
      Ok(pattern.replace(text, "; generated by <SLICER> 2.4.2 on <TIMESTAMP>").as_bytes().to_vec())
  }
  ```

  `ksr_fdmtest_v4.rs` asserts both hashes, 15 package entries, 269,330 reference lines, 460 layer markers, the normalized reference hash, and bounded diff formatting. Add the complete E2E test with exactly:

  ```rust
  #[test]
  #[ignore = "full project parity incomplete"]
  fn project_matches_orca_242_except_generator_line() {
      let temp = tempfile::tempdir().unwrap();
      let output = temp.path().join("actual.gcode");
      assert_cmd::Command::cargo_bin("ares").unwrap()
          .args(["slice", "-o", output.to_str().unwrap(), fixture_path("ksr_fdmtest_v4.project.3mf").to_str().unwrap()])
          .assert()
          .success();
      let actual = std::fs::read(output).unwrap();
      let expected = normalize_one_generator_line(&reference(), GeneratorKind::Orca).unwrap();
      let actual = normalize_one_generator_line(&actual, GeneratorKind::Ares).unwrap();
      if expected != actual {
          panic!("{}", first_difference(&expected, &actual).unwrap());
      }
  }
  ```

- [ ] **Step 2: Run the full golden once to establish RED**

  Run:

  ```powershell
  cargo nextest run -p ares-cli --test ksr_fdmtest_v4 --run-ignored all
  ```

  Expected: runtime failure because the current CLI still requires `--options`. Record this exact RED reason in the roadmap; do not weaken the comparison.

- [ ] **Step 3: Run active harness GREEN and the mandatory task gate**

  Task 1B also omits only the explicit dynamic-value audit command because Task 1C has not landed. The fixture/hash/helper tests are active and green; the full golden remains intentionally ignored. Obtain dual approval, update docs, run the remaining global matrix, then commit and push:

  ```powershell
  git commit -m "test(parity): add byte-exact project golden"
  git push
  ```

---

### Task 1C: Syntax-Aware Dynamic-Value Migration Audit

**Upstream boundary:** This audit protects the typed rewrite boundaries in `Config.*`, `PrintConfig.*`, `Format/bbs_3mf.*`, and their Ares destinations; it adds no slicing behavior.

**Files:**
- Create: `crates/ares-core/tests/no_unapproved_dynamic_values.rs`
- Create: `scripts/dynamic_value_baseline.txt`
- Create: `scripts/dynamic_value_allowlist.toml`
- Modify: workspace and `crates/ares-core/Cargo.toml` with exact dev dependencies `syn = { version = 2.0.118, features = ["full", "visit", "extra-traits"] }`, `walkdir = 2.5.0`, and `toml = 1.1.2`

**Interfaces:**
- Produces a normalized AST fingerprint set across production Rust in `ares-core`, `ares-cli`, and `ares-wasm` whose migration baseline may only shrink.
- Produces concrete serde structs for parsing the reviewed allowlist TOML.

- [ ] **Step 1: Write RED classifier tests for every forbidden category**

  Synthetic Rust snippets cover direct, grouped, renamed, glob-assisted, and re-exported `serde_json::{Value,Map,value::RawValue}` imports and paths; type aliases; `serde_json::from_value`; production `json!`; generic/custom `ConfigValue`; `Box<dyn Any>` and equivalent erased payloads; and configured XML/JSON DOM or runtime-type-test patterns. Tests prove `#[cfg(test)]` modules and `#[test]` items are excluded while a production item in the same file is still scanned.

- [ ] **Step 2: Implement the AST walk and initial baseline**

  Parse every production file with `syn::parse_file`, visit imports, paths, types, macros, aliases, re-exports, and trait objects, and normalize findings as:

  ```text
  crates/ares-core/src/options.rs|use|serde_json::Value
  crates/ares-core/src/profiles/fragment.rs|type|BTreeMap<String,serde_json::Value>
  ```

  The active test rejects any fingerprint not in `scripts/dynamic_value_baseline.txt`, permits removals, and rejects baseline growth. The allowlist parser requires `path`, `containing_struct`, `field`, `upstream_source`, and `rationale`; it also rejects an allowlisted field used by type or slicing dispatch. Add an ignored print-only baseline test; it never writes files.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core --test no_unapproved_dynamic_values
  git commit -m "test(config): audit untyped production values"
  git push
  ```

---

### Task 1D: Remove PrintConfig Source-Line Pinning

**Upstream boundary:** Runtime concepts remain owned by `PrintConfig.hpp`, `PrintConfig.cpp`, and `Config.*`; only unreachable raw-line/token metadata representations are removed.

**Files:**
- Modify/split: `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`
- Removed by the Option cleanup: `crates/ares-core/src/options/tests/print_config_hpp_modules.rs`
- Delete: option modules/tests satisfying all four source-pinning deletion predicates in the approved spec
- Modify: `scripts/dynamic_value_baseline.txt` only for fingerprints removed with deleted files

**Interfaces:**
- Preserves all runtime registry entries, parsers, normalizers, accessors, and behavior tests.

- [ ] **Step 1: Establish behavioral GREEN before deletion**

  ```powershell
  cargo nextest run -p ares-core options
  ```

- [ ] **Step 2: Remove pinning tests by intent and delete only unreachable no-behavior modules**

  Build the test candidate set from raw `source_file`, `source_lines`, `raw`, milestone/dependency/deferred assertions and remove those tests by intent even when they share a file with retained runtime code. After test removal, delete a private production module only if it both implements no retained runtime behavior and has no non-test runtime reference proven with `rg`; split mixed modules as needed instead of deleting runtime code. Remove the matching declarations and `include!`/module registration from `options/tests.rs` and `options/tests/print_config_hpp_modules.rs`; delete that aggregate file if no behavioral test remains. Split the exactly-400-LOC option roots before adding retained declarations. Re-run the same option suite and the AST audit. A final `rg` over module declarations and `include!` paths must prove that no deleted source-pinning module is orphaned or registered; remaining matches may contain concise runtime source citations but no staged raw-line record.

- [ ] **Step 3: Run the mandatory task gate**

  ```powershell
  git commit -m "refactor(config): remove PrintConfig source pinning"
  git push
  ```

---

### Task 1E: Remove Staged PrintApply Source-Line Pinning

**Upstream boundary:** `PrintApply.cpp::Print::apply` runtime state and behavior remain; only unreachable staged one-line representations/tests are removed.

**Files:**
- Modify/split: `crates/ares-core/src/print_apply.rs`, `crates/ares-core/src/print_apply/tests.rs`, `print_apply/staged_modules.rs`, `print_apply/staged_modules_legacy.rs`, `print_apply/staged_modules_legacy_older.rs`
- Delete: staged `print_apply` modules/tests satisfying all four deletion predicates
- Modify: `scripts/dynamic_value_baseline.txt` only for fingerprints removed with deleted files

**Interfaces:**
- Preserves real `print_apply` config, transform, volume-cache, region, invalidation, and status behavior.

- [ ] **Step 1: Establish behavioral GREEN before deletion**

  ```powershell
  cargo nextest run -p ares-core print_apply
  ```

- [ ] **Step 2: Delete the staged corpus and prove behavior is unchanged**

  Apply Task 1D's split rule to `print_apply`: remove source-line/token tests by test intent, then delete only private production modules that have no retained behavior and no non-test runtime reachability. Retain mixed/runtime modules and tests that vary inputs and observe state. Run `cargo nextest run -p ares-core print_apply` and the AST audit after deletion.

- [ ] **Step 3: Run the mandatory task gate**

  ```powershell
  git commit -m "refactor(print): remove staged PrintApply pinning"
  git push
  ```

---

### Task 2: Bounded In-Memory OPC/ZIP Package Reader

**Upstream boundary:** `Format/bbs_3mf.hpp::load_bbs_3mf`; `Format/bbs_3mf.cpp::_BBS_3MF_Importer::{_extract_from_archive,_extract_xml_from_archive,_handle_start_relationship}`; OPC package path semantics.

**Files:**
- Modify: workspace and `crates/ares-core/Cargo.toml` (`zip` with default features disabled and only pure-Rust stored/deflate support)
- Create: `crates/ares-core/src/project.rs`
- Create: `crates/ares-core/src/project/archive.rs`
- Create: `crates/ares-core/src/project/archive/limits.rs`
- Create: `crates/ares-core/src/project/archive/path.rs`
- Create: `crates/ares-core/src/project/tests/archive.rs`
- Create: `crates/ares-core/src/project/tests/path.rs`
- Modify: `crates/ares-core/src/lib.rs`

**Interfaces:**
- Produces: `ArchiveLimits`, `PackagePath`, and crate-private `ProjectArchive<'a>`.
- `ProjectArchive::open(&[u8], ArchiveLimits) -> Result<ProjectArchive<'_>, SliceError>` validates central-directory metadata and normalized names without expanding entries.
- `ProjectArchive::read(&mut self, &PackagePath) -> Result<Vec<u8>, SliceError>` expands one entry under limits and forces CRC verification.

- [ ] **Step 1: Write path and archive-limit RED tests**

  Use small synthetic ZIPs for exactly the rejection classes in the spec: 4,097 entries, declared/actual per-entry and total size overflow, ratio over 1,000:1, encrypted and unsupported compression metadata where constructible, corrupt CRC/local size, duplicate normalized path, drive/UNC/backslash/NUL, empty segment, literal and percent-decoded dot traversal, encoded separator, and fragments. Positive tests cover root-absolute and owner-relative relationship targets.

- [ ] **Step 2: Define exact boundary types**

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub(crate) struct ArchiveLimits {
      pub max_entries: usize,
      pub max_entry_size: u64,
      pub max_total_size: u64,
      pub max_expansion_ratio: u64,
  }

  impl ArchiveLimits {
      pub(crate) const PROJECT: Self = Self {
          max_entries: 4_096,
          max_entry_size: 256 * 1024 * 1024,
          max_total_size: 1024 * 1024 * 1024,
          max_expansion_ratio: 1_000,
      };
  }

  #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub(crate) struct PackagePath(Box<str>);

  impl PackagePath {
      pub(crate) fn entry(raw: &[u8]) -> Result<Self, SliceError>;
      pub(crate) fn resolve(&self, target: &str) -> Result<Self, SliceError>;
      pub(crate) fn as_str(&self) -> &str;
  }
  ```

- [ ] **Step 3: Implement metadata-first archive validation and bounded reads**

  Accept only Stored and Deflated methods enabled in the manifest. Check counts, declared sizes, ratios, encryption flags, and duplicate normalized paths before allocating entry buffers. On read, use a limit of `declared_size + 1`, require the number of expanded bytes to equal trusted metadata, and consume to EOF so the ZIP reader validates CRC. Map all external errors into a project-specific `SliceError` message that names the entry but never includes its payload.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core project_archive
  cargo nextest run -p ares-core package_path
  git commit -m "feat(project): add bounded in-memory 3mf archive reader"
  git push
  ```

---

### Task 3: Typed OPC Relationships and Project Metadata Documents

**Upstream boundary:** `Format/bbs_3mf.cpp` relationship/content handling, `_extract_project_config_from_archive`, `_extract_filament_sequence_from_archive`, model-config and plate-data handlers.

**Files:**
- Modify: workspace and `crates/ares-core/Cargo.toml` (`quick-xml` with serde support and default features disabled where possible)
- Create: `project/xml.rs`
- Create: `project/content_types.rs`
- Create: `project/relationships.rs`
- Create: `project/model_settings.rs`
- Create: `project/slice_info.rs`
- Create: `project/filament_sequence.rs`
- Create: `project/plate.rs`
- Create: `project/tests/documents.rs`
- Create: `project/tests/xml_limits.rs`

**Interfaces:**
- Produces all concrete serde wire structs named in the approved spec.
- Produces `deserialize_xml<T: DeserializeOwned>(&[u8], XmlRole) -> Result<T, SliceError>` and `deserialize_json<T: DeserializeOwned>(&[u8], JsonRole) -> Result<T, SliceError>` after bounded pre-validation.
- Produces `Relationships::resolve_required(owner, relationship_type) -> Result<PackagePath, SliceError>`.

- [ ] **Step 1: Write typed-document and hostile-document RED tests**

  Parse the committed `[Content_Types].xml`, both relationship documents, `model_settings.config`, `slice_info.config`, `filament_sequence.json`, and `plate_1.json`. Assert concrete fields including OrcaSlicer version `2.4.2`, plate ID 1, plate bbox, object ID 147, first-layer time, and empty typed filament sequences. Assert that all five PNG entries identified by package content type are enumerated even though only two are relationship targets. Add malformed-type tests and XML depth 257, 1,025 attributes, decoded text over 64 MiB, DTD, external/general entity declaration/reference, JSON over 64 MiB, and a synthetic archive whose unreferenced preview has bad CRC/size metadata.

- [ ] **Step 2: Define concrete serde wire types**

  Representative definitions (all remaining fields follow the same concrete pattern):

  ```rust
  #[derive(Debug, Deserialize, PartialEq)]
  #[serde(rename = "Types")]
  pub(crate) struct ContentTypes {
      #[serde(rename = "Default", default)]
      pub defaults: Vec<DefaultContentType>,
      #[serde(rename = "Override", default)]
      pub overrides: Vec<OverrideContentType>,
  }

  #[derive(Debug, Deserialize, PartialEq)]
  pub(crate) struct Relationship {
      #[serde(rename = "@Target")]
      pub target: String,
      #[serde(rename = "@Id")]
      pub id: String,
      #[serde(rename = "@Type")]
      pub relationship_type: String,
  }

  #[derive(Debug, Deserialize, PartialEq)]
  pub(crate) struct FilamentSequences(
      pub std::collections::BTreeMap<PlateId, PlateFilamentSequence>
  );
  ```

  `PlateId` implements a serde visitor accepting only canonical `plate_<positive integer>` member names. `PlateJson` and its bounds use `f64`, never `f32` or a dynamic JSON value.

- [ ] **Step 3: Implement streaming XML pre-validation followed by direct serde**

  Use `quick_xml::NsReader` to validate namespaces, required namespace URIs, depth, per-element attribute counts, decoded text accumulation, and forbidden declaration/entity events. Do not construct a DOM. After validation, deserialize the same bounded byte slice directly into the named struct. JSON goes directly from the bounded slice into its named struct. Validate content types and relationship roles against the required model/thumbnail roles. Enumerate every PNG entry recognized by `[Content_Types].xml`, including `plate_no_light_1.png`, `top_1.png`, and `pick_1.png` that are not relationship targets; pass each normalized path through `ProjectArchive::read` to force local/central size agreement, expanded-size limits, and CRC validation, then immediately discard bytes not needed by typed metadata. Retain no decoded pixels and do not make preview presence control slicing behavior.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core project_documents
  cargo nextest run -p ares-core project_xml_limits
  git commit -m "feat(project): deserialize typed 3mf metadata documents"
  git push
  ```

---

### Task 4: 3MF Model, Mesh, Transform, and Public Project Domain

**Upstream boundary:** `Format/bbs_3mf.cpp` model XML handlers at the fixed tag, `_create_object_instance`, `_apply_transform`, volume generation; `Model.*`; `TriangleMesh.*`.

**Files:**
- Create: `project/model_xml.rs`
- Create: `project/transform.rs`
- Create: `project/domain.rs`
- Create: `project/load.rs`
- Create: `project/tests/model.rs`
- Create: `project/tests/transform.rs`
- Create: `crates/ares-core/src/generation.rs`
- Create: `crates/ares-core/src/project_slice.rs`
- Modify: `project.rs`, `lib.rs`
- Modify: workspace and `crates/ares-wasm/Cargo.toml` (add the lock-compatible `js-sys = 0.3.98`)
- Modify: `crates/ares-wasm/src/lib.rs`
- Create: `crates/ares-wasm/tests/browser/package.json` and the generated `package-lock.json` with `@playwright/test = 1.57.0`
- Create: `crates/ares-wasm/tests/browser/{index.html,project-slice.spec.mjs,playwright.config.mjs,server.mjs}`
- Modify: `.github/workflows/tier1.yml` to build the generated web package and run the browser harness

**Interfaces:**
- Produces public `Project`, `ProjectModel`, `ProjectObject`, `ProjectVolume`, `ProjectInstance`, and `PlateMetadata`.
- `BuildItem` and `ProjectInstance` preserve typed `printable`, `auto_drop`, loaded object/instance identity, and the model-settings `identify_id` used to derive label 133; no runtime-generated object ID substitutes for that loaded label.
- Produces `pub fn load_project(input: impl AsRef<[u8]>) -> Result<Project, SliceError>`.
- Produces deterministic `GenerationMetadata` and the public async `slice_project` signature. At this increment `slice_project` loads/validates the project and returns the typed `ProjectSlicingIncomplete` error; it never emits approximate G-code.
- Produces the stable `#[wasm_bindgen(js_name = sliceProject)]` JavaScript export. It obtains local calendar fields from `js_sys::Date`, calls only the core project API, and maps the core error without fallback.

- [ ] **Step 1: Write model/transform RED tests**

  Assert the two `.model` documents deserialize into concrete `ModelDocument` structs; the object model has 6,109 vertices and 12,234 triangles; local bounds are `[-37.5,-35,-46]..[37.5,35,46]`; the build/component composition yields translation `(133.039205,115.992105,46)` and the expected world bounds; model-settings object/part/assembly references resolve; the fixture instance preserves `printable=true`, `auto_drop=true`, and derives loaded label ID 133 from the model-settings identity rather than mesh/root/plate/runtime IDs; triangle indices and object references are checked. Add non-finite coordinate/transform, invalid index, missing relationship target, unsupported required extension, transform-order, and synthetic absent/`0`/`1` printable/auto-drop tests that lock the fixed-tag defaults and false semantics.

- [ ] **Step 2: Define lossless model and transform types**

  ```rust
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub struct Point3d { pub x: f64, pub y: f64, pub z: f64 }

  #[derive(Clone, Debug, PartialEq)]
  pub struct ProjectMesh {
      vertices: Vec<Point3d>,
      triangles: Vec<[u32; 3]>,
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct ProjectInstance {
      object_id: u32,
      instance_id: u32,
      loaded_label_id: u32,
      printable: bool,
      auto_drop: bool,
      transform: Transform3d,
  }

  #[derive(Clone, Copy, Debug, PartialEq)]
  pub struct Transform3d([[f64; 4]; 4]);

  impl Transform3d {
      pub const IDENTITY: Self;
      pub fn parse_3mf(value: &str) -> Result<Self, SliceError>;
      pub fn then(self, rhs: Self) -> Self;
      pub fn transform_point(self, point: Point3d) -> Point3d;
  }
  ```

  Keep these separate from the legacy STL `Model`/`Point3(f32)` so project precision cannot be lost through the old API.

- [ ] **Step 3: Implement relationship-driven project assembly**

  Start at `_rels/.rels`, resolve the root model, follow its model relationships, create volumes from mesh objects, resolve component object IDs, compose transforms in Orca order, then apply build-item transforms to instances. Preserve build-item `printable`/`auto_drop`, resolve model-settings object and instance identity, and carry the imported label ID into `ProjectInstance`. Attach typed model/part/plate/slice/filament metadata and embedded project-settings bytes. Reject unreferenced required parts and invalid ownership at this public untrusted boundary.

  Delete the old `InputFormat::ThreeMf => Ok(Model::new(InputFormat::ThreeMf, Vec::new()))` branch. The separate `load_model`/STL API returns a clear boundary error for 3MF bytes directing callers to `load_project`; it never flattens a project into the legacy `f32` STL model and never retries project slicing through the STL pipeline.

- [ ] **Step 4: Add deterministic generation metadata and project API**

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct GenerationMetadata {
      year: u16, month: u8, day: u8,
      hour: u8, minute: u8, second: u8,
  }

  impl GenerationMetadata {
      pub fn new_local(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8)
          -> Result<Self, SliceError>;
      pub const fn deterministic(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8)
          -> Self;
  }

  pub async fn slice_project(
      project: impl AsRef<[u8]>,
      metadata: GenerationMetadata,
  ) -> Result<Vec<u8>, SliceError>;
  ```

  The generator compatibility version is a core constant `2.4.2`, not the crate package version. The metadata type carries no options, file identifiers, or output bytes.

- [ ] **Step 5: Run focused GREEN and the mandatory task gate**

  A focused core API test now reaches `ProjectSlicingIncomplete`. Build
  `ares-wasm` for `wasm32-unknown-unknown`, run `wasm-bindgen --target web`,
  serve the repository through the committed local-only Node HTTP server, and
  run Playwright in headless Chromium. The page imports the generated
  `ares_wasm.js`, fetches the committed 3MF, constructs a `Uint8Array`, calls
  `sliceProject`, and asserts the exact mapped `ProjectSlicingIncomplete`
  error. The browser test may not call a Rust helper directly. The CLI golden
  remains RED at the old required-`--options` boundary until Task 31B changes
  the adapter.

  ```powershell
  cargo nextest run -p ares-core project_model
  cargo nextest run -p ares-core project_transform
  cargo nextest run -p ares-core project_import
  cargo build -p ares-wasm --target wasm32-unknown-unknown --release
  cargo install --locked wasm-bindgen-cli --version 0.2.121
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser ci
  npx --prefix crates/ares-wasm/tests/browser playwright install chromium
  npm --prefix crates/ares-wasm/tests/browser test
  git commit -m "feat(project): load 3mf mesh and project domain"
  git push
  ```

---

### Task 5: Typed Option Codecs and Fixed v2.4.2 Inventory

**Upstream boundary:** `Config.hpp/cpp::ConfigOption*`, `ConfigDef`, `set_deserialize*`; `PrintConfig.cpp` option registration; `Preset.cpp` raw scope lists, all read with `git show` at the fixed commit.

**Files:**
- Create: `options/config_types.rs` and focused children when needed
- Create: `options/option_group.rs`
- Create: `options/project_settings.rs`
- Create: `options/tests/config_types.rs`
- Create: `options/tests/project_inventory.rs`
- Create: `crates/ares-core/tests/option_inventory.rs`
- Create: `tests/ksr_fdmtest_v4/options-v242.json`
- Modify/split: `options.rs`, `options/tests.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces typed wrappers `OrcaBool`, `Millimeters`, `Percent`, `FloatOrPercent`, `Point2d`, `Nullable<T>`, and actual Orca enum types.
- Produces internal typed group-field declaration/dispatch support that generates a private `Option<T>` builder/patch field and a public/default-resolved concrete group field for every declaration, without erased values.
- Produces a committed test-only 653-row fixed-commit inventory proving each key's raw scope, static owner, concrete option type, nullable/default semantics, effective projections, legacy inputs, config-export disposition, upstream definition, and upstream consumers.

- [ ] **Step 1: Write codec and inventory RED tests**

  Cover embedded string booleans `0`/`1`, the already-supported explicit-STL API's native boolean/number forms where applicable, signed/unsigned integers, finite floats, percentages, float-or-percent, scalar/vector forms, `nil` nullable elements, `x,y` points, `x`-separated point vectors, point groups, empty string versus empty array, multi-line strings, variant strides, flat matrices, AMS/ramming/CSV/space-tuple opaque typed newtypes, and invalid lexical forms. Load `options-v242.json` into a concrete test-only `OptionInventoryRow` serde struct and assert 653 unique rows, exact fixture-key equality, 448 scalar strings, 205 arrays, five empty arrays, Printer 132 + Process 352 + Filament 122 + residual 47, 31 nullable fields, effective Object 126/Region 153/GCode 149 projections, and the exact v2.4.2 type histogram in Global Constraints.

- [ ] **Step 2: Commit a reproducible per-key inventory**

  Store rows in bytewise key order. Every JSON row has these required concrete fields:

  ```rust
  #[derive(serde::Deserialize)]
  struct OptionInventoryRow {
      key: String,
      raw_scope: RawScope,
      static_owner: StaticOwner,
      option_type: OrcaOptionType,
      nullable: bool,
      default_serialized: String,
      wire_shape: WireShape,
      effective_projections: Vec<EffectiveProjection>,
      legacy_inputs: Vec<LegacyInput>,
      config_export: ConfigExportRule,
      upstream_definition: SourceCitation,
      upstream_consumers: Vec<SourceCitation>,
  }
  ```

  `LegacyInput` is a concrete rename/value-conversion record with its `handle_legacy` citation. `ConfigExportRule` is a concrete enum/struct describing ordinary canonical export, omit-when-nil, metadata exclusion, or a named fixed-tag special rule; it is not a runtime value container. The active test needs only the committed artifact and fixture, so a fresh Ares clone and CI do not depend on an Orca checkout. An ignored provenance test accepts `ORCA_SLICER_REPO`, runs only `git -C <repo> show 8500fcdccaa10b5099ac20d252af3a7c560046f1:<path>`, reconstructs raw scopes/types/defaults/nullable/static owners/legacy inputs/export rules with line-anchored parsers, verifies every cited consumer symbol exists at that commit, and compares all 653 reconstructed rows to the artifact. The deterministic verification command is:

  ```powershell
  $env:ORCA_SLICER_REPO = (Resolve-Path OrcaSlicer)
  cargo nextest run -p ares-core --test option_inventory --run-ignored all
  Remove-Item Env:ORCA_SLICER_REPO
  ```

- [ ] **Step 3: Implement concrete serde wrappers**

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct OrcaBool(pub bool);

  #[derive(Clone, Copy, Debug, PartialEq)]
  pub enum FloatOrPercent { Float(f64), Percent(Percent) }

  #[derive(Clone, Debug, PartialEq)]
  pub enum Nullable<T> { Nil, Value(T) }

  #[derive(Clone, Copy, Debug, PartialEq)]
  pub struct Point2d { pub x: f64, pub y: f64 }
  ```

  Each wrapper implements serde directly from Orca's actual JSON string/array wire form, validates finite/range invariants at deserialization, and serializes with Orca lexical rules. `Nullable<T>` is the actual upstream nullable union, not a catch-all value type.

- [ ] **Step 4: Add typed group dispatch support**

  A group builder receives a key and `serde::de::MapAccess`; it calls `next_value::<ConcreteType>()` only in the matching key arm and returns `false` without consuming the value otherwise. The builder stores presence as `Option<ConcreteType>`, then resolves missing fields through the fixed upstream typed default into a group struct whose fields are concrete `ConcreteType`. This permits the final top-level visitor to try raw groups in fixed order and reject the still-unmatched key by name. It never deserializes an intermediate value.

- [ ] **Step 5: Establish the behavioral option ledger**

  `docs/architecture/option-parity-v4.md` records, for each key added by later tasks: raw scope, concrete type, effective projection(s), fixed upstream consumer, state (`retained-only` or `consumed`), and focused behavioral test. This is a behavior ledger, not raw source-line pinning. Task 5 seeds the document with the verified totals and codec rules rather than 653 unimplemented claims.

- [ ] **Step 6: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core config_types
  cargo nextest run -p ares-core project_inventory
  cargo nextest run -p ares-core --test option_inventory
  git commit -m "feat(config): add typed Orca option codecs"
  git push
  ```

---

### Task 6: Printer Machine-Envelope Options (28 Fields)

**Upstream boundary:** Fixed-tag `Preset.cpp` printer key list; `PrintConfig.hpp::MachineEnvelopeConfig`; the 12 axis keys registered by loops in `PrintConfig.cpp` plus the remaining machine-envelope fields.

**Files:**
- Create: `options/printer_options.rs`
- Create: `options/printer_options/machine_envelope.rs`
- Create: `options/tests/printer_machine_envelope.rs`
- Modify: `options/project_settings.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `MachineEnvelopeOptions` with 28 concrete resolved fields plus a private typed builder with `deserialize_known_field`/ordered serialization.
- Starts `PrinterOptions` with its completed `machine: MachineEnvelopeOptions` child; later printer child groups are added only by their own tasks, and no untyped remainder map exists.

- [ ] **Step 1: Add a RED inventory and per-codec field test**

  Select the exact 28 inventory rows formed by the union of fixed-tag `MachineEnvelopeConfig` fields and `Preset.cpp::s_Preset_machine_limits_options`, intersected with the fixture. This includes `emit_machine_limits_to_gcode` and the templated `input_shaping_type` declaration. Assert pairwise uniqueness, exact count, concrete type, default, fixture wire shape, parsed semantic value, and fixed-tag canonical serialization for every key. Add behavioral tests for the machine-limit fields already consumed by existing G-code tests by changing one typed field and observing the affected limit command or typed state.

- [ ] **Step 2: Add all 28 typed declarations**

  Use the Task 5 group dispatcher so each key arm directly invokes the correct concrete deserializer. The four `machine_max_speed_*`, four `machine_max_acceleration_*`, and four `machine_max_jerk_*` loop-generated upstream names are individual Rust fields; there is no suffix-driven runtime value typing. Preserve upstream default and declaration ordering separately from lexicographic export ordering.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core printer_machine_envelope
  git commit -m "feat(config): type printer machine envelope options"
  git push
  ```

---

### Task 7: Printer G-code-Owned Options (62 Fields)

**Upstream boundary:** Fixed-tag intersection of the 132 printer raw keys with `PrintConfig.hpp::GCodeConfig`; associated enum declarations and defaults in `PrintConfig.cpp`.

**Files:**
- Create: `options/printer_options/gcode_source.rs`
- Create: `options/tests/printer_gcode_source.rs`
- Modify: `options/printer_options.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `PrinterGCodeSourceOptions` with 62 concrete resolved fields plus its private typed builder.
- Extends `PrinterOptions` raw dispatch without changing the 28 machine fields.

- [ ] **Step 1: Add RED coverage for the exact 62-key intersection**

  Select the 62 committed inventory rows whose raw scope is Printer and static owner is `GCodeConfig`. Assert none overlap `MachineEnvelopeOptions`, and every fixture value round-trips through its declared type. Add focused cases for the sole `coPoints` field `wrapping_exclude_area`, printer/extruder enums, vector cardinality, multi-line machine G-code, and invalid enum spelling. This boundary contains no `coPointsGroups` field.

- [ ] **Step 2: Add typed fields one inventory row at a time**

  For each row, first add the failing field test, then add its concrete field and match arm. Do not infer type from whether the fixture value is a string or array. Machine templates, `wrapping_exclude_area`, nozzle/extruder vectors, and firmware flavor use their explicit upstream types and enums. `extruder_printable_area` belongs to Task 8's printer/`PrintConfig` intersection, `extruder_offset` is a residual/`PrintConfig` key, and `bed_shape` is absent from this fixture; none belongs to Task 7.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core printer_gcode_source
  git commit -m "feat(config): type printer gcode options"
  git push
  ```

---

### Task 8: Remaining Printer Raw Options (42 Fields)

**Upstream boundary:** Fixed-tag printer raw scope intersected with `PrintConfig` ownership (27) plus the 15 printer rows classified `unowned` by the committed FFF raw inventory, including variant and hardware metadata used by `PrintApply.cpp`.

**Files:**
- Create: `options/printer_options/remaining.rs`
- Create: `options/printer_options/remaining/enums.rs`
- Create: `options/printer_options/remaining/wire.rs`
- Create: `options/tests/printer_remaining.rs`
- Modify: `options/printer_options.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Completes `PrinterOptions` at exactly 132 raw fields.
- Produces one `PrinterRemainingOptions` child with the exact 42 rows plus typed hardware/variant helpers used later by effective normalization.

- [ ] **Step 1: Add RED coverage for 27 + 15 fields and whole-printer completeness**

  Select exactly the 42 rows where `raw_scope=printer` and `static_owner` is `print_config` or `unowned`. Assert the 28/62/42 printer child sets are pairwise disjoint and their union equals the inventory's 132-key printer set. Exercise the already-loaded 8-entry `MachineEnvelope` stride vectors as a cross-child regression, the two physical-extruder forms, four expanded `nozzle_volume` values, both nullable float-vector types, structured `extruder_variant_list`, and all explicit empty-area arrays. Assert that `extruder_ams_count` is rejected here because it belongs to Task 14 residual options.

- [ ] **Step 2: Implement the remaining concrete fields and typed special encodings**

  Define dedicated newtypes for structured encodings such as per-extruder variant lists and thumbnail definitions when their internal components are consumed later. Preserve an opaque `String` only for an upstream `coString` whose content is not parsed to make a slicing decision. Keep `AmsCounts` ownership deferred to Task 14. Complete direct serde serialization and raw field dispatch for all 132 printer keys.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core printer_options
  git commit -m "feat(config): complete typed printer project options"
  git push
  ```

---

### Task 9: Process Object-Owned Raw Options (126 Fields)

**Upstream boundary:** Fixed-tag process raw scope intersected with `PrintConfig.hpp::PrintObjectConfig`; object defaults and enums in `PrintConfig.cpp`.

**Files:**
- Create: `options/process_options.rs`
- Create: `options/process_options/object_source.rs`
- Create: `options/process_options/object_source/enums.rs`
- Create: `options/process_options/object_source/wire.rs`
- Create: `options/tests/process_object_source.rs`
- Create: `options/tests/process_object_source/expected.rs`
- Create: `options/tests/process_object_source/enums.rs`
- Create: `options/tests/process_object_source/type_assertions.rs`
- Create: `options/tests/process_object_source/direct_dispatch.rs`
- Modify: `options/project_settings.rs`, `options/option_group.rs`, `options.rs`, `lib.rs`, `options/tests.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `ProcessObjectSourceOptions` with 126 concrete resolved fields plus its private typed builder.
- Starts `ProcessOptions` raw ownership; it does not yet create the effective `ObjectOptions` projection.

- [ ] **Step 1: Add RED inventory and complete typed-state tests**

  Select the inventory's exact 126 Process/`PrintObjectConfig` rows using active line-anchored macro entries (not a cross-line/loose regex). Explicitly exclude the two commented-out tuple-shaped lines (`independent_support_layer_height` and `adaptive_layer_height`) that make an unanchored textual count appear as 128. Prove all 126 fields are scalar strings with the exact histogram of 22 bool, 12 enum, 63 float, six float-or-percent, 13 int, and ten percent fields. Cover layer height, slicing mode, closing radius, brim and elephant-foot options, raft, support/interface/tree settings, XY compensation, bridge policies, object acceleration/jerk, wall-generator transitions, interlocking/MMU segmentation, and calibration state. Assert that first-layer height and resolution are deferred to Task 11, while wall count, sparse fill density/pattern, and top/bottom shell fields are deferred to Task 10. Because 108 fixture values equal upstream defaults, add a valid non-default single-field typed-state test for every one of the 126 keys rather than relying on fixture round-trip alone.

- [ ] **Step 2: Implement all 126 fields through direct typed dispatch**

  Keep `ProcessObjectSourceOptions` as the single public child group; split its raw enum definitions and direct lexicographic serializer into private sibling modules so every production file remains below 400 LOC. Preserve exact upstream defaults and canonical enum spellings. `support_ironing_pattern` uses the complete fixed 28-token `InfillPattern` raw map, not its two-value UI subset or Ares's effective infill enum. Update the shared typed group decoder so invalid field values report the Option key, with regression coverage for existing printer groups. Record the 108 current dynamic-consumer name collisions but do not migrate them in Task 9; effective typed projection and consumer migration remain Task 15.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core process_object_source
  git commit -m "feat(config): type process object options"
  git push
  ```

---

### Task 10: Process Region-Owned Raw Options (149 Fields)

**Upstream boundary:** Fixed-tag process raw scope intersected with `PrintConfig.hpp::PrintRegionConfig`; flow, perimeter, infill, bridge, and ironing declarations in `PrintConfig.cpp`.

**Files:**
- Create: `options/process_options/region_source.rs`
- Create: `options/process_options/region_source/enums.rs`
- Create: `options/process_options/region_source/wire.rs`
- Create: `options/process_options/wire.rs`
- Create: `options/process_options/wire/early.rs`
- Create: `options/process_options/wire/middle.rs`
- Create: `options/process_options/wire/late.rs`
- Create: `options/tests/process_region_source.rs`
- Create: `options/tests/process_region_source/expected.rs`
- Create: `options/tests/process_region_source/enums.rs`
- Create: `options/tests/process_region_source/type_assertions.rs`
- Create: `options/tests/process_region_source/direct_dispatch.rs`
- Modify: `options/process_options.rs`, `options.rs`, `lib.rs`, `options/tests.rs`, `options/tests/process_object_source.rs`, `options/tests/process_object_source/direct_dispatch.rs`, `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`

**Interfaces:**
- Produces `ProcessRegionSourceOptions` with 149 concrete resolved fields plus its private typed builder.

- [ ] **Step 1: Add RED inventory and lexical/enum tests for all 149 keys**

  Select the inventory's exact 149 Process/`PrintRegionConfig` rows from the 155 active HPP tuples. Exclude the four filament-scope nullable ironing overrides and the two legacy-only shells `ironing_direction` and `wall_infill_order`. Assert no overlap with the 126 object-source rows and prove the exact histogram: 31 bool, 14 enum, 49 float, 24 float-or-percent, 15 int, one integer vector, 11 percent, three string, and one string-vector field. The wire boundary is 147 scalar strings plus the two non-nullable vectors `print_extruder_id` and `print_extruder_variant`; preserve fixture cardinality four but accept and round-trip other valid lengths. Check widths, flows, speeds, `seam_gap`, bridge settings, sparse/solid/top/bottom fill patterns, fuzzy/ironing forms, and the two variant vectors. Because 119 fixture values equal defaults, exercise a valid non-default typed state for every one of the 149 keys. Add targeted invalid scalar/array/element-shape tests and exact 149-child plus global 275-parent lexicographic byte-round-trip tests.

- [ ] **Step 2: Implement fields in fixed declaration order**

  Keep `ProcessRegionSourceOptions` as one public region child and preserve the exact filtered HPP declaration order in production. Perimeter/flow, fill/bridge, and ironing/fuzzy/speed may be implementation batches only. Reuse the complete 28-token raw `ProcessInfillPattern` for all five pattern fields and define dedicated raw enums for the other nine domains. Canonical serde accepts only Orca's machine-readable tokens; UI labels and `handle_legacy` conversions remain Task 19A. Preserve the two arrays as raw typed vectors without active-extruder normalization. Add `region` to direct parent dispatch, but do not migrate the 109 current dynamic consumers; effective region projection remains Task 16. Replace the one-child parent serializer with a direct globally lexicographic 275-entry serializer split across contiguous helper modules that share one `SerializeMap`; do not delegate child maps, flatten, or buffer through a DOM.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core process_region_source
  git commit -m "feat(config): type process region options"
  git push
  ```

---

### Task 11: Remaining Process Raw Options (77 Fields)

**Upstream boundary:** Fixed-tag process scope intersected with `GCodeConfig` (17), `PrintConfig` ownership (59), and the one unowned process key.

**Files:**
- Create: `options/process_options/gcode_source.rs`
- Create: `options/process_options/gcode_source/wire.rs`
- Create: `options/process_options/print_source.rs`
- Create: `options/process_options/print_source/enums.rs`
- Create: `options/process_options/print_source/wire.rs`
- Create: `options/tests/process_remaining.rs`
- Create: `options/tests/process_remaining/direct_dispatch.rs`
- Create: `options/tests/process_remaining/enums.rs`
- Create: `options/tests/process_remaining/expected.rs`
- Create: `options/tests/process_remaining/type_assertions.rs`
- Create: `options/tests/process_remaining/vectors.rs`
- Modify: `options/process_options.rs`, `options/process_options/wire.rs`, `options/process_options/wire/early.rs`, `options/process_options/wire/middle.rs`, `options/process_options/wire/late.rs`, `options.rs`, `lib.rs`, `options/tests.rs`, `options/tests/process_object_source/direct_dispatch.rs`, `options/tests/process_region_source.rs`, `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`

**Interfaces:**
- Completes `ProcessOptions` at exactly 352 raw fields.

- [ ] **Step 1: Add RED whole-process set proof**

  Select the remaining Process rows from the inventory and assert the 126 object, 149 region, 17 `GCodeConfig`, 59 `PrintConfig`, and one unowned key sets are pairwise disjoint and union to its exact 352-key process scope. Prove the remaining histogram is 25 bool, six enum, 24 float, six float-or-percent, one float-vector, six int, four percent, three string, and two string-vector fields; all 77 are non-nullable, with 74 scalar-string and three array wire shapes. The only arrays are `post_process`, `small_area_infill_flow_compensation_model`, and `wiping_volumes_extruders`; accept and byte-round-trip arbitrary valid lengths, including empty arrays, and reject scalar, null, and invalid element shapes without encoding the fixture's cardinality. Assert the exact 15 fixture/default differences and exercise a valid non-default typed state for every key because 62 fixture values equal fixed defaults. For both standalone children, pass every one of their fields through both standalone and flat-parent non-default dispatch. For all 77 fields, prove `null` fails with the key; for all 74 scalars, prove array and object shapes fail with the key. Exercise the direct parent `ironing_expansion` path separately for a valid non-default, duplicate, null, array, and object value. Cover output/post-process, print sequence, brim/skirt, prime tower, timelapse, the extrusion-role-change G-code, all six canonical enum domains, strict child ownership, and exact standalone-child plus flat 352-parent lexical bytes.

- [ ] **Step 2: Implement 77 concrete fields and complete process dispatch**

  Add public `gcode: ProcessGCodeSourceOptions` and `print: ProcessPrintSourceOptions` children with crate-private builders. Preserve each child's exact filtered HPP declaration order: `GCodeConfig` only within fixed `PrintConfig.hpp:1299-1476`, and FFF `PrintConfig` only within `PrintConfig.hpp:1479-1660` so the unrelated SLA `filename_format` declaration is not counted. Keep the unique unowned `ironing_expansion` scalar directly on `ProcessOptions`, sourced from fixed `PrintConfig.cpp:4368`; do not create an invented effective-runtime group for it. G-code and other strings remain raw typed strings, and the three arrays remain raw typed containers without matrix interpretation or cardinality normalization. Define strict raw enums for draft shield, print order, print sequence, skirt type, timelapse, and wipe-tower wall type; canonical serde accepts only machine tokens. UI labels, `draft_shield=limited`, `timelapse_type=2`, and the 13 recorded legacy canonical targets remain Task 19A work. Record the 63 current production literal collisions and exact 14-key complement without migrating them; `prime_volume` is legacy-parser-only, so the behavioral-consumer union is 62. The 17 G-code-source fields project in Task 17, full-print normalization remains Task 19B, behavioral consumers migrate across Tasks 20A-20D, and the legacy compatibility parser is removed only in Task 20E.

  Extend direct parent dispatch to the two new children and the unowned scalar. Replace the 275-entry parent output with one globally lexicographic, directly streamed 352-entry map across the existing contiguous `early`/`middle`/`late` helpers. The new entries distribute 23/32/22, producing 115/124/113 entries per helper; do not delegate child maps, use serde flattening, or buffer through a DOM. Keep every changed production and test module below 400 physical LOC.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  In addition to the standard workspace nextest, warning-denying Clippy,
  rustfmt, WASM, dynamic-value, and diff gates, run a physical-LOC audit over
  every changed production and test Rust module and fail the task at 400 lines.

  ```powershell
  cargo nextest run -p ares-core process_remaining
  git commit -m "feat(config): complete typed process project options"
  git push
  ```

---

### Task 12: Filament G-code-Owned Raw Options (53 Fields)

**Upstream boundary:** Select the 52 live fixed-tag filament preset names from
`Preset.cpp:1309-1346` that intersect fixed
`PrintConfig.hpp:1299-1476` `GCodeConfig`, then add the separately project-owned
`filament_colour` from `PresetBundle.cpp:43-58,2652-2658,2795-2802`. The latter
is commented out of the preset list at `Preset.cpp:1309` but remains a
`GCodeConfig` field at `PrintConfig.hpp:1333` with its definition at
`PrintConfig.cpp:2455`. The resulting exact 53 declarations are at
`PrintConfig.hpp:1308-1464` and their definitions are in `PrintConfig.cpp`. Raw
nullable and non-nullable float, int, string, and bool
vector grammar is owned by generic `Config.hpp:624-663` plus the exact float,
int, string, and bool boundaries at `Config.hpp:812-952,995-1085,1118-1163`
and `Config.hpp:1857-1967`; JSON array loading is at
`Config.cpp:830-870,950-1004` and 3MF JSON array emission is at
`Config.cpp:1464-1496`. The exact ten variant-stride fields are named by
`PrintConfig.cpp:8375-8415`; lookup and 8-to-active mapping remain deferred from
this raw slice to Task 19B (`PrintConfig.cpp:9004-9054,9805-10023`,
`PrintApply.cpp:1164-1173`, and `Print.cpp:3166-3175`).

**Files:**
- Create: `options/filament_options.rs`
- Create: `options/filament_options/wire.rs`
- Create: `options/filament_options/gcode_source.rs`
- Create: `options/filament_options/gcode_source/wire.rs`
- Create: `options/tests/filament_gcode_source.rs`
- Create: `options/tests/filament_gcode_source/direct_dispatch.rs`
- Create: `options/tests/filament_gcode_source/expected.rs`
- Create: `options/tests/filament_gcode_source/type_assertions.rs`
- Create: `options/tests/filament_gcode_source/vectors.rs`
- Modify: `options.rs`, `lib.rs`, `options/tests.rs`,
  `options/project_settings.rs`, `docs/architecture/option-parity-v4.md`,
  `docs/roadmap.md`

**Interfaces:**
- Produces `FilamentGCodeSourceOptions` with 53 concrete raw typed vector
  fields plus its private typed builder; it does not produce resolved or active
  filament values.
- Starts `FilamentOptions` raw ownership with a public `gcode` child and adds a
  public `filament: FilamentOptions` aggregate to `ProjectSettings`.
- Both standalone and flat-parent serialization stream the same 53 entries in
  global lexical order; the parent does not emit a nested `gcode` object or
  buffer through a DOM.

- [ ] **Step 1: Add RED exact-inventory, raw-vector, and dispatch tests**

  Prove the exact 53-key inventory is unique, belongs to filament raw scope and
  `GCodeConfig`, and has the exact histogram of eight `coBools`, 27 `coFloats`,
  seven `coInts`, and 11 `coStrings`. All 53 wire values are arrays. The exact
  seven element-nullable fields are `filament_adaptive_volumetric_speed`,
  `filament_cooling_before_tower`, `filament_flow_ratio`,
  `filament_flush_temp`, `filament_flush_volumetric_speed`,
  `long_retractions_when_ec`, and `retraction_distances_when_ec`; test their
  concrete `Nullable<T>` element types and exact `"nil"` round-trip. Non-nullable
  numeric and boolean arrays reject `"nil"`; string arrays retain it as an
  ordinary raw string.

  Assert the fixed HPP declaration order separately from lexical wire order and
  the exact singleton-vector defaults, including the single-space G-code
  defaults and embedded newlines. Fixture tests must preserve 43 two-element
  vectors and the exact ten eight-element variant vectors without shrinking or
  expansion. All 53 fixture vectors differ from singleton defaults by source
  cardinality, while exactly 17 also differ after cardinality is ignored; add a
  valid non-default typed-state test through both child and flat parent dispatch
  for every field so repeated fixture defaults cannot conceal a bad type or
  dispatch arm.

  Accept and byte-round-trip arbitrary valid raw lengths, including empty,
  one-, three-, five-, and eight-element vectors. Reject a scalar, object, or
  null top-level value and invalid element lexical shapes with an error naming
  the key; do not reject a vector merely because its length differs from the
  fixture. Cover duplicate and unknown keys, strict child ownership, globally
  lexical 53-key bytes, the exact multiline start/end G-code bytes, empty
  strings, and raw structured-string bytes. `filament_type` is an open string
  suggestion domain, `filament_extruder_variant` remains raw strings carrying
  four normalization tokens, and `filament_printable` is an integer bitmask;
  none is a Task 12 enum. Add an explicit aggregate-boundary RED assertion that
  `ProjectSettings::default().filament == FilamentOptions::default()` and that
  its public `.filament.gcode` child has the concrete default type, so omitting
  the `ProjectSettings` integration cannot leave focused tests green.

- [ ] **Step 2: Implement the 53 raw typed vectors without normalization**

  Preserve production field layout in exact filtered HPP declaration order and
  use the existing raw semantic wrappers for
  `adaptive_pressure_advance_model` (`CsvTable`),
  `filament_extruder_variant` (`VariantStride`),
  `filament_ramming_parameters` (`RammingParameters`), and
  `volumetric_speed_coefficients` (`SpaceTuple`). These wrappers preserve raw
  string contents in Task 12; they do not select active entries, expand a
  stride, or interpret a slicing model. The other fields use the concrete Orca
  vector wrappers. Represent the seven nullable arrays directly as
  `Vec<Nullable<OrcaBool>>`, `Vec<Nullable<OrcaFloat>>`, or
  `Vec<Nullable<OrcaInt>>`; do not couple filament ownership to printer-owned
  nullable wrapper names or create one-use filament wrapper types. Keep all
  defaults as singleton vectors and deserialize arbitrary valid lengths
  directly.

  Record but do not apply open-enum suggestions, the four canonical extruder
  variant tokens, the `Normal`/`Big Traffic` and `ASA-Aero` legacy conversions,
  or the seven `omit_when_nil` export rules. The exact fixture variant mapping
  selects raw indices `[0,4]`, but active selection and cross-field cardinality
  validation belong to Task 19B; nullable export belongs to Task 19C. Record the
  51 current production literal collisions and exact two-key complement
  (`adaptive_pressure_advance_model`,
  `adaptive_pressure_advance_overhangs`) without migrating consumers; consumer
  migration remains Tasks 20A and 20D and legacy parser removal remains Task
  20E. Keep every changed production and test module below 400 physical LOC.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  In addition to the standard workspace nextest, warning-denying Clippy,
  rustfmt, WASM, dynamic-value, and diff gates, run a physical-LOC audit over
  every changed production and test Rust module and fail the task at 400 lines.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core filament_gcode_source
  git commit -m "feat(config): type filament gcode options"
  git push
  ```

---

### Task 13: Remaining Filament Raw Options and Nullable Overrides (69 Fields)

**Upstream boundary:** The remaining fixed-tag filament raw scope consists of
48 FFF `PrintConfig` declarations at `PrintConfig.hpp:1484-1650`, four
filament ironing overrides declared by `PrintRegionConfig` at
`PrintConfig.hpp:1153-1156`, the exact 16-entry
`filament_extruder_override_keys` list and `add_nullable` construction loop at
`PrintConfig.cpp:63-84,7287-7318`, and the uniquely unowned
`pellet_flow_coefficient` definition at `PrintConfig.cpp:2639`. Fixed preset
serialization force-preserves nil retract overrides at
`Preset.cpp:1861-1878`. After stripping comments, the fixed live filament
preset list at `Preset.cpp:1309-1346` has 126 unique names and intersects the
fixture in 121; every Task 13 field is live, while the sole filament fixture
exception is Task 12's project-owned `filament_colour`. Raw vector/nullable
JSON grammar remains the fixed
`Config.hpp`/`Config.cpp` boundary cited by Task 12, and the exact 27
eight-entry variant fields are the intersection with
`PrintConfig.cpp:8375-8415`. Active selection and resizing remain Task 19B;
all-nil effective export omission at `GCode.cpp:5632-5640` remains Task 19C.

**Files:**
- Create: `options/filament_options/print_source.rs`
- Create: `options/filament_options/print_source/enums.rs`
- Create: `options/filament_options/print_source/wire.rs`
- Create: `options/filament_options/region_source.rs`
- Create: `options/filament_options/region_source/wire.rs`
- Create: `options/filament_options/retract_overrides.rs`
- Create: `options/filament_options/retract_overrides/wire.rs`
- Create: `options/filament_options/wire/early.rs`
- Create: `options/filament_options/wire/middle.rs`
- Create: `options/filament_options/wire/late.rs`
- Create: `options/tests/filament_remaining.rs`
- Create: `options/tests/filament_remaining/expected.rs`
- Create: `options/tests/filament_remaining/expected/keys.rs`
- Create: `options/tests/filament_remaining/expected/orders.rs`
- Create: `options/tests/filament_remaining/expected/sets.rs`
- Create: `options/tests/filament_remaining/inventory_defaults.rs`
- Create: `options/tests/filament_remaining/type_assertions.rs`
- Create: `options/tests/filament_remaining/fixture.rs`
- Create: `options/tests/filament_remaining/direct_dispatch.rs`
- Create: `options/tests/filament_remaining/nullable.rs`
- Create: `options/tests/filament_remaining/enums.rs`
- Create: `options/tests/filament_remaining/invalid.rs`
- Create: `options/tests/filament_remaining/wire.rs`
- Modify: `options/filament_options.rs`, `options/filament_options/wire.rs`,
  `options.rs`, `lib.rs`, `options/tests.rs`,
  `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`

**Interfaces:**
- Completes `FilamentOptions` at exactly 122 raw keys with public `print`,
  `region`, and `retract_overrides` children plus direct
  `pellet_flow_coefficient`; no
  invented runtime projection owns the unowned vector.
- Adds exactly 20 element-nullable filament fields in this task: four region
  ironing overrides and 16 generated retract overrides. Completed
  `FilamentOptions` owns 27 nullable fields including Task 12's seven; the
  cumulative project-wide count is 31 only after including Printer's four.
- Standalone children preserve their fixed declaration/list order and
  serialize lexically; the flat parent directly streams all 122 entries in
  one global lexical order without nesting, serde flattening, or a DOM.

- [ ] **Step 1: Add RED exact-inventory, nullable, and 122-parent tests**

  Prove the remaining exact 69-key set is unique, all-array, disjoint from the
  Task 12 child, and partitioned as 48 `PrintConfig`, four
  `PrintRegionConfig`, 16 generated retract, and one direct pellet field. The
  exact histogram is 11 `coBools`, three `coEnums`, 20 `coFloats`, 30
  `coInts`, four `coPercents`, and one `coStrings`; subgroup histograms are
  Print 8/1/6/30/2/1, Region 0/0/3/0/1/0, retract 3/2/10/0/1/0, and pellet one
  float vector. Assert the fixed HPP order for the first two children, the
  fixed `filament_extruder_override_keys` order for the third, exact singleton
  defaults for every field, and concrete public field types.

  Assert exactly 20 Task 13 element-nullable arrays. All four region defaults
  are singleton nil, while all 16 generated defaults clone a concrete
  singleton from the corresponding extruder option. Exercise mixed
  `["nil", value, "nil"]` payloads for every nullable field. The fixture is
  fully nil for exactly 15 fields: the four region fields plus 11 generated
  overrides. Its five nullable fields with concrete values are
  `filament_retraction_distances_when_cut`, `filament_retraction_length`,
  `filament_wipe`, `filament_wipe_distance`, and `filament_z_hop_types`.
  Exactly 48 non-string, non-nullable arrays reject `"nil"`, while raw
  `filament_notes` retains it as ordinary text.

  Prove exact fixture cardinality of 42 two-entry and 27 eight-entry vectors;
  the latter set must equal the fixed `filament_options_with_variant`
  intersection. Every fixture value differs from its singleton default by raw
  cardinality. After cardinality is ignored, assert the exact 36 semantic
  overrides and 33 repeated defaults. The override set is
  `additional_cooling_fan_speed`, `close_additional_fan_first_x_layers`,
  `complete_print_exhaust_fan_speed`, `during_print_exhaust_fan_speed`,
  `eng_plate_temp`, `eng_plate_temp_initial_layer`,
  `fan_cooling_layer_time`, `fan_min_speed`,
  `filament_deretraction_speed`, `filament_long_retractions_when_cut`,
  `filament_retract_before_wipe`, `filament_retract_lift_above`,
  `filament_retract_lift_below`, `filament_retract_lift_enforce`,
  `filament_retract_restart_extra`,
  `filament_retract_when_changing_layer`,
  `filament_retraction_distances_when_cut`, `filament_retraction_length`,
  `filament_retraction_minimum_travel`, `filament_retraction_speed`,
  `filament_wipe`, `filament_z_hop`, `filament_z_hop_types`,
  `first_x_layer_fan_speed`, `hot_plate_temp`,
  `hot_plate_temp_initial_layer`, `nozzle_temperature`,
  `nozzle_temperature_initial_layer`, `overhang_fan_threshold`,
  `reduce_fan_stop_start_freq`, `slow_down_layer_time`,
  `slow_down_min_speed`, `supertack_plate_temp`,
  `supertack_plate_temp_initial_layer`, `textured_plate_temp`, and
  `textured_plate_temp_initial_layer`. Raw
  child and parent parsing must accept and byte-round-trip arbitrary valid
  empty, one-, three-, five-, and eight-entry arrays without active selection,
  resizing, or cross-field cardinality validation.

  Cover all three strict canonical enum domains:
  `overhang_fan_threshold` accepts `0%`, `10%`, `25%`, `50%`, `75%`, and
  `95%`; nullable `filament_retract_lift_enforce` accepts `All Surfaces`,
  `Top Only`, `Bottom Only`, and `Top and Bottom`; nullable
  `filament_z_hop_types` accepts `Auto Lift`, `Normal Lift`, `Slope Lift`, and
  `Spiral Lift`. Reject unknown/case variants and the legacy `5%` value.
  Assert their singleton defaults `95%`, `All Surfaces`, and `Slope Lift`, and
  the fixture's `50%`, nil, and `Spiral Lift` payloads respectively.
  `filament_notes` is the only remaining string field and must preserve empty,
  multiline, UTF-8, and literal `"nil"` strings without a parsing newtype.

  For each of the 68 child-owned fields, pass a valid non-default value through
  its owning standalone child and flat `FilamentOptions` direct dispatch.
  Exercise direct `pellet_flow_coefficient` separately through the parent for
  valid non-default, duplicate, scalar, object, null, and invalid-element
  cases. Through every applicable child and the parent, reject bad shapes with
  an error naming the key. Cover representative duplicate keys per child,
  unknown and cross-child keys, nested group objects, exact standalone lexical
  bytes, the exact fixture's full flat 122-key bytes, and the public
  `ProjectSettings::default().filament` child/direct-field boundary.

- [ ] **Step 2: Implement the source-owned children and direct pellet field**

  Implement the 48-field `FilamentPrintSourceOptions`, four-field
  `FilamentRegionSourceOptions`, and 16-field
  `FilamentRetractOverrideOptions` with crate-private typed builders. Each
  dynamically generated upstream name becomes an explicit Rust field and
  match arm; no prefix/suffix reflection chooses a type at runtime. Keep
  `pellet_flow_coefficient: OrcaFloats` directly on `FilamentOptions`, with a
  singleton `0.4157` default and direct duplicate handling. Because the
  concrete vector's derived default is empty, implement parent default through
  its builder resolution rather than deriving the wrong pellet default.

  Define a strict raw enum for the six overhang-threshold tokens. Reuse the
  existing source-equivalent `RetractLiftEnforce` and `ZHopType` enums inside
  direct `Vec<Nullable<T>>` fields. The other nullable fields likewise use
  direct `Vec<Nullable<OrcaBool>>`, `Vec<Nullable<OrcaFloat>>`, or
  `Vec<Nullable<Percent>>`; do not couple them to printer-owned wrappers or
  create one-use nullable wrappers. Preserve `filament_notes` as raw
  `OrcaStrings`; there is no structured grammar in this slice. Do not reuse
  the existing behavioral overhang-threshold type, which collapses the six
  raw tokens into two runtime states.

  Replace the 53-entry parent serializer with one directly streamed,
  globally lexical 122-entry map split into contiguous 41/41/40-entry
  `early`/`middle`/`late` helpers. Do not delegate nested child maps, use serde
  flattening, or buffer through a DOM. Preserve arbitrary raw cardinality and
  exact nil elements. Record but do not apply the four legacy targets
  (`bridge_fan_speed`, `cooling`, `overhang_fan_threshold=5%`, and
  `chamber_temperatures`), the 20 `omit_when_nil` exports, or the 66 current
  production literal collisions; the exact three-key complement is
  `chamber_minimal_temperature`, `filament_long_retractions_when_cut`, and
  `filament_retraction_distances_when_cut`; none of the 66 is legacy-only.
  Legacy conversion remains Task
  19A, region projection remains Task 16, active sizing/normalization remains
  Task 19B together with nullable retract inheritance, nil export remains Task
  19C, consumer migration remains Tasks 20A and 20D, and compatibility-parser
  removal remains Task 20E. Keep every
  changed production and test Rust module below 400 physical LOC.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  In addition to the standard workspace nextest, warning-denying Clippy,
  rustfmt, WASM, dynamic-value, and diff gates, run a physical-LOC audit over
  every changed production and test Rust module and fail the task at 400 lines.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core filament_remaining
  git commit -m "feat(config): complete typed filament project options"
  git push
  ```

---

### Task 14: Typed Project/Runtime Residual Options (47 Fields)

**Upstream boundary:** The exact difference between the fixed-tag 653-key
fixture and the already typed Printer 132 + Process 352 + Filament 122 union;
17 residual `GCodeConfig` declarations, 19 residual FFF `PrintConfig`
declarations, eight project/preset registrations, and three project JSON
provenance strings.

This task is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The exact source slices are:

- `PrintConfig.hpp:1299-1476`, with the 17 filtered `GCodeConfig`
  declarations at `:1304-1475` and their definitions in `PrintConfig.cpp`;
- `PrintConfig.hpp:1479-1660`, with the 19 filtered FFF `PrintConfig`
  declarations at `:1501-1626` and their definitions in `PrintConfig.cpp`;
- the eight registrations at `PrintConfig.cpp:1832,1874,2473,2477,2933,
  5111,5116,5432`, with project/preset composition at
  `PresetBundle.cpp:43-58,102-239,2652-2672,3881-4161`;
- `extruder_ams_count` raw structured-string behavior at
  `PrintConfig.cpp:617-653` and its empty-vector default at `:5379-5383`;
- metadata load/save at `Config.cpp:887-911,1464-1496` and the 3MF save call
  at `Format/bbs_3mf.cpp:7726`; and
- strict raw enum maps at `PrintConfig.cpp:469-478,573-584`, distinct from
  adjacent UI-facing enum lists.

Do not define the residual as the literal complement of the fixed Preset
lists. That complement contains 48 keys because `filament_colour` is commented
out at `Preset.cpp:1309`; Task 12 already assigns that project-owned
`GCodeConfig` field to Filament using
`PresetBundle.cpp:43-58,2652-2658,2795-2802`. The exact typed equation is:

```text
fixture 653 - printer 132 - process 352 - filament 122 = residual 47
```

**Files:**
- Create: `options/project_runtime_options.rs` with `gcode_source`,
  `print_source`, `preset_source`, enum, and wire siblings as required by the
  under-400-physical-LOC gate
- Create: `options/preset_metadata.rs`
- Create: `options/tests/project_runtime_options.rs` with focused inventory,
  defaults/types, fixture, dispatch, enum, invalid-shape, wire, metadata, and
  expected-data siblings
- Modify: `options.rs`, `lib.rs`, `options/project_settings.rs`,
  `options/tests.rs`, `docs/architecture/option-parity-v4.md`,
  `docs/roadmap.md`

**Interfaces:**
- Produces public `ProjectRuntimeOptions { gcode, print, preset }` for 44 real
  raw options and `PresetMetadata { from, name, version }` for the three
  provenance strings.
- Adds concrete public `project` and `metadata` fields to `ProjectSettings`.
- Each source child preserves fixed upstream declaration/registration order and
  serializes independently in lexical order. `ProjectRuntimeOptions` streams a
  flat lexical 44-key map directly through one shared `SerializeMap`; it emits
  no nested child objects, serde flattening, remainder map, or DOM buffer.
- Completes the pairwise-disjoint typed 653-key fixture union in tests. Task 18
  still owns the production top-level `ProjectSettings` visitor, cross-group
  duplicate/unknown handling, and strict project loading. This program does not
  add a production top-level `ProjectSettings` serializer.

**Exact ownership:**

- `ProjectGCodeSourceOptions` owns these 17 fields in fixed HPP order:
  `deretraction_speed`, `filament_ids`, `filament_map_mode`, `filament_map`,
  `retract_before_wipe`, `retraction_length`, `retract_length_toolchange`,
  `z_hop`, `retract_lift_above`, `retract_lift_below`,
  `retract_restart_extra`, `retract_restart_extra_toolchange`,
  `retraction_speed`, `nozzle_volume_type`, `extruder_ams_count`,
  `bbl_calib_mark_logo`, `has_scarf_joint_seam`.
- `ProjectPrintSourceOptions` owns these 19 fields in fixed HPP order:
  `curr_bed_type`, `first_layer_print_sequence`,
  `other_layers_print_sequence`, `other_layers_print_sequence_nums`,
  `extruder_colour`, `extruder_offset`, `max_layer_height`,
  `min_layer_height`, `nozzle_diameter`, `retraction_minimum_travel`,
  `retract_when_changing_layer`, `wipe`, `wipe_distance`, `wipe_tower_x`,
  `wipe_tower_y`, `flush_volumes_matrix`, `flush_volumes_vector`,
  `flush_multiplier`, `start_end_points`.
- `ProjectPresetSourceOptions` owns these eight real fields in fixed
  registration order: `print_compatible_printers`,
  `default_filament_profile`, `filament_multi_colour`,
  `filament_colour_type`, `filament_settings_id`, `print_settings_id`,
  `printer_settings_id`, `filament_self_index`.
- `PresetMetadata` owns exactly `from`, `name`, and `version`; they are not
  `PrintConfig` options and never enter `ProjectRuntimeOptions`.

The exact 44-real-option histogram is:

```text
coBool=2, coBools=2, coEnum=2, coEnums=1, coFloats=19,
coInt=1, coInts=4, coPercents=1, coPoints=2, coString=2, coStrings=8
```

All 47 fields are non-nullable. The fixture and canonical-save boundary is 37
JSON arrays plus ten scalar strings. The six singleton vectors remain arrays:
`default_filament_profile`, `first_layer_print_sequence`,
`other_layers_print_sequence`, `print_compatible_printers`, `wipe_tower_x`,
and `wipe_tower_y`. The exact array-length histogram is
`{1:6, 2:14, 4:15, 8:2}`. Raw vectors accept arbitrary valid cardinality; no
fixture-length, matrix-dimension, AMS-topology, or active-extruder rule enters
this task.

- [ ] **Step 1: Establish genuine aggregate RED and exact inventory tests**

  First require the missing `ProjectSettings::project: ProjectRuntimeOptions`
  and `ProjectSettings::metadata: PresetMetadata` interfaces and record a
  focused nonzero result caused only by those absent production interfaces.
  Then assert the exact 17/19/8/3 partition, fixed source orders, unique 47-key
  inventory, histogram, zero nullable fields, 37/10 wire shapes, and all fixed
  defaults/concrete public types using expected constants independent of
  production declarations.

  Prove Printer 132, Process 352, Filament 122, and Residual 47 are pairwise
  disjoint and merge to the exact 653 fixture keys. The whole 650-real-option
  histogram must remain:

  ```text
  coBool105/coBools22/coEnum44/coEnums9/coFloat160/
  coFloatOrPercent36/coFloats90/coInt41/coInts45/coPercent25/
  coPercents5/coPoint4/coPoints6/coPointsGroups1/coString30/coStrings27
  ```

  This union is test evidence only; do not implement Task 18's production
  top-level map visitor early.

- [ ] **Step 2: Add RED defaults, enum, fixture-shape, and dispatch tests**

  Exercise every one of the 44 real fields through its child and flat parent
  with a valid non-default value before adding production support. Assert all
  singleton/vector/scalar defaults, including `extruder_ams_count == []`, and
  exactly these strict raw enum domains:

  - `curr_bed_type`: `Default Plate`, `Supertack Plate`, `Cool Plate`,
    `Engineering Plate`, `High Temp Plate`, `Textured PEI Plate`,
    `Textured Cool Plate`;
  - `filament_map_mode`: `Auto For Flush`, `Auto For Match`, `Manual`; UI-only
    `Default` is not a canonical raw token; and
  - `nozzle_volume_type` elements: `Standard`, `High Flow`.

  Reject unknown, case-variant, numeric, UI-only, and legacy enum spellings;
  legacy conversions remain Task 19A. For every vector, accept and byte-round
  trip arbitrary valid empty, one-, and three-element values. Preserve `[]`
  versus `[""]`, structured AMS strings, point encodings, percent suffixes,
  and finite numeric values without interpretation.

  Reject wrong top-level shapes, invalid elements, JSON null, duplicates,
  unknown keys, cross-child keys, and nested child objects with bounded errors
  naming the field. Test declaration order separately from exact standalone
  child lexical bytes, flat 44-key parent bytes, and `from,name,version`
  metadata bytes.

  Use the real 3MF through the existing bounded test loader to prove the exact
  six singleton arrays, `{1:6,2:14,4:15,8:2}` cardinalities, metadata values,
  and lossless typed round trip. Exactly seven real fixture fields equal their
  fixed defaults: `bbl_calib_mark_logo`, `filament_map_mode`,
  `first_layer_print_sequence`, `has_scarf_joint_seam`,
  `other_layers_print_sequence`, `other_layers_print_sequence_nums`, and
  `start_end_points`; the other 37 differ.

- [ ] **Step 3: Implement the 47 concrete fields option-by-option**

  For each field, add its failing valid/default/wire assertion first, then only
  its concrete field, default, direct dispatch arm, and serializer entry before
  rerunning focused GREEN. Reuse existing `Orca*`, `Point2dList`,
  `FlatMatrix`, `AmsCounts`, and `NozzleVolumeTypes` codecs. Add strict
  `ProjectBedType` and `ProjectFilamentMapMode` enums from the raw key maps;
  do not reuse the open-string `DefaultBedType` wrapper.

  `AmsCounts` remains a raw string vector and `FlatMatrix` remains a finite
  float vector. Do not parse AMS structure, enforce square matrices, resize
  vectors, select active entries, or validate cross-field cardinality. Keep
  metadata in its concrete sibling and add only the two aggregate fields to
  `ProjectSettings`; do not add production top-level serialization or project
  loader wiring.

  Every changed production and test Rust file must remain below 400 physical
  lines. Split only the named semantic siblings needed to satisfy that gate.
  Add no filesystem, terminal, FFI, native-only, UI, fallback, fixture-name,
  or reference-G-code behavior to `ares-core`.

  Record without migrating the 31 real-name production literal collisions.
  The exact 13-key no-collision complement is `bbl_calib_mark_logo`,
  `extruder_offset`, `filament_self_index`, `first_layer_print_sequence`,
  `flush_multiplier`, `flush_volumes_matrix`, `flush_volumes_vector`,
  `has_scarf_joint_seam`, `other_layers_print_sequence`,
  `other_layers_print_sequence_nums`, `retract_length_toolchange`,
  `retract_restart_extra_toolchange`, and `start_end_points`.

  Explicitly defer all 17 residual effective G-code projections to Task 17;
  strict top-level dispatch/persistence to Task 18; legacy key/value conversion
  to Task 19A; active sizing, AMS/self-index interpretation, vector/matrix
  normalization, and cross-field validation to Task 19B; metadata exclusion,
  `extruder_colour` substitution, scaled `flush_volumes_matrix`, duplicate
  plate-indexed `wipe_tower_x/y`, and exact config-block export to Task 19C;
  and consumer migration/parser removal to Tasks 20A-20E.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  Run the focused test throughout TDD, then the complete local matrix:

  ```powershell
  cargo +1.91.0 fmt --all -- --check
  cargo +1.91.0 nextest run -p ares-core project_runtime_options
  cargo +1.91.0 nextest run --workspace
  cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
  cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
  cargo +1.91.0 check -p ares-core
  cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
  cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
  cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser ci
  npm --prefix crates/ares-wasm/tests/browser test
  git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
  ```

  The browser harness must import the generated web binding, pass the real 3MF
  as a `Uint8Array`, and observe exact `ProjectSlicingIncomplete`. Also run
  no-index whitespace checks for untracked files, physical LOC checks failing
  at 400 or more, an exact changed-file audit, and affected adjacent typed
  option tests.

  Freeze the complete diff and obtain independent literal
  `SPEC VERDICT: APPROVE`, `QUALITY VERDICT: APPROVE`, and after tracked docs
  are updated, `DOCS VERDICT: APPROVE`. The user-approved OpenCode bypass
  applies. Any finding requires correction and fresh affected gates/review.
  Rerun the complete final local matrix on the exact bytes to commit, then:

  ```powershell
  git commit -m "feat(config): type project runtime options"
  git push
  ```

  Identify the Tier 1 run for the exact pushed SHA and require all five jobs--
  Windows, macOS, Ubuntu/Linux, format, and WASM--to succeed before Task 15.

---

### Task 15: Effective Object Options (126-Field Projection)

**Fixed upstream:** OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` only. Task 14's prerequisite is
satisfied by pushed commit `dc47e069ede1caa307411d63ba29f78784630494`
and five-job Tier 1 run `29253342315`; Task 15 production implementation may
start only while that exact pushed-SHA evidence remains recorded.

**Upstream boundary:**

- `PrintConfig.hpp:917-1071` owns the exact 126 active
  `PrintObjectConfig` fields, after which `PrintRegionConfig` begins;
  `PrintConfig.cpp` owns their concrete types, serialized defaults, and enum
  domains. Commented `independent_support_layer_height` and
  `adaptive_layer_height` declarations are excluded.
- `Model.hpp:72-102,354-370`, `PrintConfig.hpp:2053-2128`,
  `Format/bbs_3mf.cpp:2119-2132,4389-4399`, and `Config.cpp:573-654` own
  `ModelConfigObject`/global `ModelObject::config`, ordered object metadata,
  and canonical lexical deserialization.
- `PrintConfig.cpp:2200-2213` owns model-only canonical `extruder`, used by the
  fixture and later region normalization.
- `PrintObject.cpp:3555-3579` owns
  `PrintObject::object_config_from_model_object`; `Config.cpp:461-500` owns
  static projection with `ignore_nonexistent=true`.
- `PrintApply.cpp:1130-1133,1190-1194,1273-1283,1468-1482,1539-1548,
  1646-1656` owns normalized default-object snapshots, recomputation, and the
  `num_extruders` input, including recomputation on `num_extruders_changed`.
  Cache invalidation, timestamps, workers, and geometry remain outside this
  task.
- `PrintConfig.cpp:8520-8741` owns monolithic and split FDM normalization.
  Their fixed write sets have zero intersection with these 126 fields; this
  task must prove that fact before deferring the general port to Task 19B.

**Files:**
- Create: `crates/ares-core/src/options/object_fields.rs` as the one private
  compile-time field/type/default inventory shared by raw, sparse, and
  effective object structs
- Create: `crates/ares-core/src/options/object_options.rs` and only the private
  `object_options/*` semantic siblings needed to stay below 400 physical LOC
- Create: `crates/ares-core/src/options/tests/object_options.rs` and focused
  inventory, metadata, overlay, clamp, normalization, and fixture siblings
- Create: `crates/ares-core/src/project/model_settings/object_metadata.rs`
- Modify: `crates/ares-core/src/options/process_options/object_source.rs`,
  `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`,
  `crates/ares-core/src/lib.rs`, `crates/ares-core/src/project.rs`,
  `crates/ares-core/src/project/model_settings.rs`, and focused
  `crates/ares-core/src/project/tests/*` modules
- Modify: `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, and the
  ignored SDD progress/brief artifacts

**Interfaces:**
- Produces distinct concrete `ObjectOptions` and sparse
  `ObjectOptionOverrides` structs with the same 126 concrete scalar types as
  `ProcessObjectSourceOptions`; neither may be an alias/newtype, erased enum,
  dynamic map, or JSON-backed representation. `ObjectOptions` has no
  independent `Default`; its only base/default source is the supplied process
  object snapshot, while `ObjectOptionOverrides::default()` means all fields
  are absent.
- `ProcessObjectSourceOptions`, `ObjectOptionOverrides`, and `ObjectOptions`
  expand the same compile-time field inventory. Defaults and enum domains have
  one production source; sparse presence uses `Option<T>` and is never inferred
  by comparing with a default. The Task 9 declaration-order inventory and
  fixed-source table are normative, including all 12 strict raw enum domains
  and the complete 28-token `ProcessInfillPattern` domain used by
  `support_ironing_pattern`. Independently select the same rows from committed
  `tests/ksr_fdmtest_v4/options-v242.json` where `raw_scope=process` and
  `static_owner=print_object_config`; tests must not use the production macro
  as their oracle.
- External object metadata decoding returns keyed `Result` errors. Once values
  are typed, effective resolution is infallible:

  ```rust,ignore
  ObjectOptions::resolve(
      base: &ProcessObjectSourceOptions,
      overrides: &ObjectOptionOverrides,
      num_extruders: usize,
  ) -> ObjectOptions
  ```

- Document-layer `ObjectSettings` retains its existing ID, last assigned
  `name`/`module`, typed sparse 126-field overrides, and the ordered canonical
  non-object key/value entries needed by later projections in a concrete
  `retained_config: Vec<Metadata>` field. Task 15 does not have the complete
  fixed global `PrintConfigDef` key universe, so every non-126 entry is retained
  in XML source order without classifying it as known or unknown and is ignored
  by `ObjectOptions`. Part `matrix` metadata stays on the part path.
  Task 16 performs concrete lexical decoding for retained `extruder`/region
  entries. Task 19A performs reviewed legacy rewrites, then Task 19B's named
  `options/model_config_deserialize.rs` boundary ports the complete fixed
  `PrintConfigDef` canonical key/type registry, directly validates and routes
  every remaining entry, and rejects a still-unknown key with its exact name
  before full resolution. This staged deviation and error timing are explicit;
  the fixture's 653 rows are never used as a global registry. Task 15 validates
  lexical/type/enum shape for its 126 owned fields only.
- Task 15 does not attach effective values to `ProjectObject`, add a
  `project.settings()` API, or perform object lookup in production. Task 18
  owns top-level typed project storage and Task 19B owns final association by
  source-model path/object ID.
- Resolution order is fixed: copy the supplied normalized default/base
  snapshot; copy the ordered model config; rely only on the separately tested
  zero-intersection result for the upstream normalization call; apply present
  supported object fields with `ignore_nonexistent=true` semantics; then run
  both support-filament clamps and return.

- [ ] **Step 1: Freeze the shared inventory and base-identity RED**

  This sequential slice owns `object_fields.rs`, the refactor of
  `process_options/object_source.rs`, the initial `object_options.rs`, and only
  the inventory/base test siblings. Its genuine RED is the absent concrete
  `ObjectOptions`/shared inventory interface.

  Independently assert the exact 126 declaration-order keys, scalar-string
  wire shape, zero nullable fields, and histogram
  `coBool=22/coEnum=12/coFloat=63/coFloatOrPercent=6/coInt=13/coPercent=10`.
  Require the missing concrete
  effective and sparse interfaces first. Then prove a base containing a
  non-default value for every field projects byte-for-byte into
  `ObjectOptions`, with no second defaults or enum table.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core object_options_inventory
  cargo +1.91.0 nextest run -p ares-core process_object_source
  ```

- [ ] **Step 2: RED/GREEN ordered external metadata decoding**

  After Step 1 is GREEN, this slice owns `object_options/overrides.rs`,
  `project/model_settings/object_metadata.rs`, the minimal
  `project/model_settings.rs` wiring, and metadata/project test siblings. Its
  genuine RED is canonical object metadata that still remains untyped or uses
  the old duplicate-rejecting lookup.

  Use synthetic XML to exercise one non-default value for every primitive
  wrapper and enum and table-drive all 126 canonical keys. XML values decode
  directly from lexical strings into their concrete fields with errors naming
  malformed bool/int/float/percent/float-or-percent/enum keys. `name` and
  `module` are named object strings; `matrix` remains part metadata. Retain all
  non-object entries, including `extruder`, without premature global-key
  classification. Process metadata in XML order and prove repeated option,
  `name`, and `module` assignments are last-write-wins; do not reuse the
  duplicate-rejecting `optional_value` helper for this path.

  A later malformed duplicate must fail at that later assignment instead of
  being hidden by the earlier valid value. Task 15 adds lexical, concrete-type,
  and enum-domain decoding only; it adds no new option-range or cross-field
  validation.

  Keep the retained non-object handoff as ordered boundary key/value text; do
  not parse it through `serde_json::Value`, a generic dynamic
  option, a raw effective-option map, or a serialization round trip. Legacy
  aliases are retained as noncanonical text and do not enter typed overrides
  until Task 19A's reviewed rewrite; Task 15 adds no legacy fallback.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core object_settings_metadata
  ```

- [ ] **Step 3: RED/GREEN sparse overlay**

  After Step 2 is GREEN, this slice owns the effective projection helper in
  `object_options.rs` and only overlay tests. Its genuine RED is the missing
  presence-preserving field application across all 126 slots.

  Prove absent overrides inherit all 126 fields, each present field changes
  only itself, and every field can override independently. With a non-default
  base, a present override equal to that field's raw default must replace the
  base; absence alone means inheritance. Duplicate metadata uses the final
  value. Implement field application only; clamps remain RED in Step 4.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core object_options_projection
  ```

- [ ] **Step 4: RED/GREEN exact support-filament clamps**

  After Step 3 is GREEN, this slice owns the two post-overlay clamp statements
  and only clamp tests. Its genuine RED is an over-limit override remaining
  unchanged after otherwise-correct sparse projection.

  For both `support_filament` and `support_interface_filament`, prove `0`, a
  codec-admitted negative, `1`, and `num_extruders` remain unchanged while
  `num_extruders + 1` becomes `1`; changing the count recomputes the result.
  Include over-limit sparse overrides so tests observe that both clamps run
  after overlay.
  Implement only the fixed strict-`>` clamp from `PrintObject.cpp:3555-3560`,
  not the adjacent `<= 0 || > count` feature-filament rule.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core object_options_clamps
  ```

- [ ] **Step 5: Verify normalization zero-intersection**

  After Step 4 is GREEN, this verification-only slice owns the normalization
  expected sets and intersection tests. It may start GREEN because it records
  fixed-source evidence rather than new production behavior; do not manufacture
  a failing test to satisfy RED ceremony.

  Freeze the complete fixed write sets of both `normalize_fdm` and
  `normalize_fdm_1`/`normalize_fdm_2` independently of production declarations
  and prove zero intersection with the 126 object keys. Exercise `extruder`
  and at least one other normalization-driving registered key from each path:
  they remain available to later projections and cannot change
  `ObjectOptions` in this task.

  The independently expected union is `extruder`, the six region filament-ID
  keys, `retract_when_changing_layer`,
  `filament_retract_when_changing_layer`, `wall_loops`,
  `alternate_extra_wall`, `top_shell_layers`, `sparse_infill_density`,
  `resolution`, `enable_prime_tower`, and
  `independent_support_layer_height`; tests retain separate monolithic and
  split sets before checking both intersections.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core object_options_normalization
  ```

- [ ] **Step 6: Verify the document-layer real fixture**

  After Step 5 is verified, this verification-only slice owns the real-3MF
  fixture tests and any minimal document-test registration not completed in
  Step 2. It may already be GREEN because Step 2 owns the document parser; do
  not manufacture a failure. It must not change effective projection
  production code or add a fixture branch.

  Through the bounded real-3MF loader, locate object ID 2 generically, retain
  `name=ksr_fdmtest_v4.drc`, accept `extruder=1`, and prove there are zero
  126-key overrides. Decode the process object-source base using the existing
  typed fixture path and assert complete effective equality, exact
  `108 default-equal / 18 process-overridden` counts, and these exact fixture
  differences from the fixed defaults:

  ```text
  brim_object_gap=0.1, brim_width=5, default_acceleration=10000,
  elefant_foot_compensation=0.15, initial_layer_acceleration=500,
  inner_wall_acceleration=0, line_width=0.42, max_bridge_length=0,
  outer_wall_acceleration=5000, support_interface_bottom_layers=2,
  support_interface_top_layers=2, support_line_width=0.42,
  support_speed=150, support_type=tree(auto),
  top_surface_acceleration=2000, tree_support_branch_angle=45,
  tree_support_branch_diameter=2, wall_generator=classic
  ```

  Also assert representative layer/shell/support/seam values. Keep existing part-transform
  and model-settings tests green; fixture-only evidence does not replace the
  synthetic override matrix.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core object_options_fixture
  cargo +1.91.0 nextest run -p ares-core project_documents
  ```

- [ ] **Step 7: Run the mandatory review, verification, commit, and Tier 1 gate**

  ```powershell
  cargo +1.91.0 fmt --all -- --check
  cargo +1.91.0 nextest run -p ares-core object_options
  cargo +1.91.0 nextest run -p ares-core -E 'test(/(object_options|process_object_source|project)/)'
  cargo +1.91.0 nextest run --workspace
  cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
  cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
  cargo +1.91.0 check -p ares-core
  cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
  cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
  cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser ci
  npm --prefix crates/ares-wasm/tests/browser test
  git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
  ```

  Also require no-index whitespace checks for every untracked file, an exact
  changed-file audit, physical LOC below 400 for every changed Rust production
  and test module, and proof that no JSON/dynamic intermediate, option pinning,
  fixture-name branch, native I/O, terminal, FFI, or platform-specific code was
  added. Freeze the bytes and obtain independent literal
  `SPEC VERDICT: APPROVE`, `QUALITY VERDICT: APPROVE`, and after docs update
  `DOCS VERDICT: APPROVE`; the user-approved OpenCode bypass applies. Report
  the dynamic-value audit's configured ignored/skip count exactly. Only after
  all approvals, rerun the complete local matrix on the frozen bytes, then:

  ```powershell
  git commit -m "feat(config): resolve effective object options"
  ```

  Verify the created commit tree matches the frozen reviewed byte manifest,
  then `git push`. Require all five Tier 1 jobs green for that exact pushed SHA
  before Task 16.

  Execute Steps 1-6 sequentially; do not assign overlapping production files
  to concurrent implementers. For each production Step 1-4 slice, first run
  its named pinned focused filter and preserve a genuine nonzero RED
  attributable only to that step's stated reason, then rerun that filter to
  GREEN before beginning the next slice or widening the gate. Run Steps 5-6
  with their named normalization and document filters as verification evidence.
  Do not hide oversized Rust with `include!`;
  do not put fixture IDs, names, values, or G-code fragments in production;
  canonical option-key expansion comes only from the shared inventory. Add no
  legacy fallback or Option Pinning.

**Explicitly deferred:** region/extruder propagation outcome and
volume/material/layer-range precedence to corrected Task 16; G-code projection
to Task 17; strict top-level project loading to Task 18; aliases and legacy
conversion on both top-level and model-settings paths to Task 19A; general FDM
normalization, active sizing, and per-object association to Task 19B;
config-block export to Task 19C; dynamic consumer migration/removal to Tasks
20A-20E; and `PrintApply` lifecycle, geometry, G-code, GUI, and SLA behavior.

---

### Task 16: Effective Region Options (153-Field Projection)

**Fixed upstream boundary:** `PrintConfig.hpp:1074-1249::PrintRegionConfig`,
`PrintObject.cpp:3582-3709::apply_to_print_region_config` and
`region_config_from_model_volume`, the model-part path at
`PrintApply.cpp:786-795,1021-1042`, the selected filament ironing reads at
`Fill/Fill.cpp:1591-1604`, and object/volume metadata loading at
`Format/bbs_3mf.cpp:2119-2132,4894-5117`, with exact metadata string codecs at
`Config.cpp:123-144::unescape_string_cstyle` and
`Config.cpp:146-215::unescape_strings_cstyle`, all at fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

The exact effective inventory is 153 real fields: the 149 typed process-region
fields plus the four nullable filament vectors `filament_ironing_flow`,
`filament_ironing_spacing`, `filament_ironing_inset`, and
`filament_ironing_speed`, selected into concrete scalars. Active HPP members
`ironing_direction` and `wall_infill_order` are legacy-only shells, not two
additional effective fields.

**Files:**
- Create: `options/region_fields.rs` as the single compile-time 149-field
  inventory shared by the existing process source and the effective projection
- Create: `options/region_options.rs` with private `overrides` and `merge`
  siblings
- Create: `options/tests/region_options.rs` with focused inventory,
  precedence, filament, normalization, and fixture siblings
- Create or modify: `project/model_settings/part_metadata.rs`
- Modify: `project/model_settings.rs`, the Task 15
  `project/model_settings/object_metadata.rs` handoff,
  `project/load/metadata.rs`, `options/process_options/region_source.rs`,
  `options/tests.rs`, `options.rs`, `lib.rs`, focused project-document,
  model-settings, process-region, and matrix/import tests,
  `docs/architecture/option-parity-v4.md`, and `docs/roadmap.md`

**Interfaces:**
- Produces public non-raw `RegionOptions` with 153 concrete fields and
  crate-private sparse `RegionOptionOverrides`. Each sparse source contains
  presence-preserving slots for the 149 region fields plus model-only
  `extruder: Option<OrcaInt>` used by that scope's fallback/mask logic.
- The exact pure resolution boundary is:

  ```rust,ignore
  pub(crate) enum RegionBase<'a> {
      ModelPart {
          process: &'a ProcessRegionSourceOptions,
          object: Option<&'a RegionOptionOverrides>,
          layer_range: Option<&'a RegionOptionOverrides>,
      },
      Modifier {
          parent: &'a RegionOptions,
      },
  }

  pub(crate) struct RegionOverrideSources<'a> {
      pub base: RegionBase<'a>,
      pub volume: &'a RegionOptionOverrides,
      pub material: Option<&'a RegionOptionOverrides>,
  }

  impl RegionOptions {
      pub(crate) fn resolve(
          filament: &FilamentRegionSourceOptions,
          sources: RegionOverrideSources<'_>,
          num_extruders: usize,
      ) -> RegionOptions;
  }
  ```

  The bundle preserves source identity/order and may not collapse into a map or
  erased value. XML lexical and active-vector cardinality errors are handled at
  the external typed document/full-config boundary; trusted concrete merge,
  clamps, and selection are infallible.
- `RegionBase::ModelPart` projects the 149 process fields and seeds its feature
  mask from positive process/default feature IDs before applying object,
  volume, material, and layer-range sources. `RegionBase::Modifier` clones the
  already-resolved parent's 149 region fields, starts the feature mask entirely
  clear even when parent feature IDs are positive, and applies only volume and
  material. Both branches recompute the four selected ironing scalars after
  final clamps and normalization. This directly represents upstream's
  `default_or_parent_region_config` parameter without reconstructing a parent
  through `ProcessRegionSourceOptions` or carrying a contradictory boolean.
- The crate-private resolver has a trusted precondition that
  `num_extruders > 0` and each of the four filament vectors has exactly that
  cardinality. Task 19B owns the active-vector sizing/validation boundary before
  the first production resolver call. Task 16 tests use only inputs satisfying
  that precondition; the internal merge does not add a fallback or duplicate
  boundary validation.
- Task 16 decodes Task 15's ordered `ObjectSettings::retained_config` into the
  object `RegionOptionOverrides`, including `extruder`, before effective
  resolution. Consumed region entries are not duplicated in retained raw
  metadata, while every remaining non-Task-16 entry stays in XML order for
  Tasks 18/19A/19B. It never recovers region state from `ObjectOptions`.
- Part metadata follows the same split: all 149 canonical region keys and
  model-only `extruder` become typed sparse overrides with repeated keys applied
  in XML order and last write winning. The non-Task-16 structural keys `name`,
  `volume_type`, `part_type`, `matrix`, `mesh_shared`, `source_file`,
  `source_object_id`, `source_volume_id`, `source_offset_x`, `source_offset_y`,
  `source_offset_z`, `source_in_inches`, and `source_in_meters` remain in an
  ordered retained document field for their existing project loader/later
  owners; the typed `mesh_stat` element remains named. Region/extruder entries
  are removed from that retained field rather than duplicated. Existing
  structural duplicate validation remains at the project metadata boundary.
- Metadata decoding uses the fixed `Config.hpp:994-1067,1087-1158` and
  `Config.cpp:123-215` lexical forms: comma-separated non-null integer vectors,
  C-style escaped scalar strings, and the exact quoted/escaped semicolon vector
  parser. The vector codec must preserve Orca's treatment of quoted semicolons,
  spaces/tabs between entries, consecutive/trailing separators, empty input,
  and malformed quotes/escapes; it may not unescape the whole input and then
  split on semicolons. The three scalar string region fields,
  `print_extruder_id`, and `print_extruder_variant` receive explicit codecs;
  the remaining concrete scalar/enum wrappers use keyed direct lexical
  decoding. No metadata path round-trips through JSON or an erased value.
- The four ironing fields are selected with the final clamped
  `top_surface_filament_id - 1`; there is no independent caller-provided active
  filament that can disagree with the resolved region. Selected nil inherits
  the ordinary region `ironing_*` value.

- [ ] **Step 1: RED/GREEN inventory, handoff, and source precedence**

  Assert 149 + 4, the fixed 149-field type histogram, unique keys, and concrete
  selected ironing outputs. Freeze the resolved 153-field histogram as 31
  bool, 14 enum, 52 float, 24 float-or-percent, 15 int, one integer vector, 12
  percent, three string, and one string-vector field. Prove direct sparse
  metadata dispatch for all 149 keys, including the two vector codecs,
  last-write-wins region/extruder duplicates, keyed malformed-value errors, and
  ordered retention of every remaining object/part entry. For model parts,
  prove exact precedence:

  Freeze scalar string cases for trailing-backslash rejection, `\r`, `\n`, and
  generic escaped characters. Freeze string-vector cases for quoted semicolons,
  quoted escapes, leading/inter-item spaces and tabs, consecutive and trailing
  separators, empty input, and malformed quote/escape rejection.

  ```text
  process/default region -> ModelObject -> volume -> material -> layer range
  -> feature clamps/final normalization -> selected ironing values
  ```

  Preserve the upstream feature-override mask across scopes: a positive
  feature ID is explicit; any nonpositive feature ID clears that flag without
  assigning the nonpositive value; a positive same-scope `extruder` fallback
  then applies only to clear features. Without such a fallback the prior field
  value remains while its mask stays clear, allowing a later scope's extruder
  fallback to replace it. A positive process/default feature ID starts explicit
  only for model parts. Synthetic sources must use different values at every
  stage so an omission or swap fails, and must distinguish negative/no-extruder
  mask clearing from naive assign-then-final-clamp behavior. The real fixture
  must consume object `extruder=1`; with its six process feature IDs at zero,
  all six effective IDs become one. A modifier begins with its already-resolved
  parent region, starts with all feature-mask bits clear, applies volume then
  material, and never reapplies object or layer-range sources;
  modifier-parent graph construction is deferred.

- [ ] **Step 2: RED/GREEN six clamps, final normalization, and ironing selection**

  Cover all six feature IDs from `PrintObject.cpp:3583-3590`. After overlays,
  each `<= 0` or `> num_extruders` value becomes one while `1..=num_extruders`
  stays unchanged. Do not reuse Task 15's strict-`>` helper.

  `Percent` stores Orca's percentage number directly, so prove density values
  below `0.00011` become `Percent(0.0)`, equality remains unchanged, and values
  above 100 cap at `Percent(100.0)`. For fuzzy skin, every
  `ProcessFuzzySkinType` other than `None` (including `Disabled`) enters the
  fixed guard; it becomes `None`, not `Disabled`, when point distance is below
  `0.01` or thickness below `0.001`. Test both strict thresholds at equality.
  For all four filament vectors, selected non-nil overrides the ordinary region
  scalar and selected nil inherits it. The fixture selects index zero and
  inherits `10%`, `0.15`, `0.21`, and `30` respectively.

- [ ] **Step 3: Implement direct typed metadata and merge dispatch**

  Decode canonical region keys from Task 15-retained object entries and
  part/volume metadata directly into concrete sparse slots. Structural part
  metadata (`name`, `matrix`, source IDs/offsets, mesh data) remains named.
  Apply fields with concrete assignments or compile-time-generated concrete
  code, preserving the feature mask and stages; add no JSON/DOM, generic option
  value, raw map, native I/O, terminal/UI, or platform-specific dependency.

  This task includes pure typed optional material/layer-range inputs. The
  current fixture contains neither. Task 19B owns source-supported bounded
  import/association for optional `Metadata/layer_config_ranges.xml`. Fixed
  BBS 3MF has no material-config document ingestion path: material remains an
  optional typed model input for the pure resolver, and no Ares archive reader
  may be invented for it. Task 19B records that fixed-source absence while
  associating material only if a source-supported model boundary supplies one.

- [ ] **Step 4: Run the mandatory task gate**

  ```powershell
  cargo +1.91.0 fmt --all -- --check
  cargo +1.91.0 nextest run -p ares-core region_options
  cargo +1.91.0 nextest run -p ares-core -E 'test(/(region_options|object_options|process_region_source|project)/)'
  cargo +1.91.0 nextest run --workspace
  cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
  cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
  cargo +1.91.0 check -p ares-core
  cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
  cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
  cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser ci
  npm --prefix crates/ares-wasm/tests/browser test
  git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
  ```

  Require the same no-index whitespace, changed-file ownership, under-400-LOC,
  forbidden dynamic/JSON/Option-Pinning, independent frozen-byte spec/quality,
  docs, fresh frozen-byte matrix, commit-tree-match, push, exact-pushed-SHA, and
  five-job Tier 1 gates as Task 15. Only after approvals and fresh verification,
  commit with `feat(config): resolve effective region options`, verify the
  commit tree, and push. Explicitly defer
  modifier graph construction, region deduplication, `PrintApply` lifecycle,
  project-wide active sizing/association, consumers, geometry, and G-code.

---

### Task 17: Registered Pre-normalization GCodeConfig Projection (149 Fields)

**Fixed upstream boundary:** OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, specifically
`PrintConfig.hpp:759-776::StaticPrintConfig::StaticCache::finalize`,
`PrintConfig.hpp:838-865::PRINT_CONFIG_CLASS_DEFINE`,
`PrintConfig.hpp:1299-1476::GCodeConfig`,
`PrintConfig.hpp:1479-1482::PrintConfig`,
`PrintConfig.hpp:1662-1666::FullPrintConfig`, and cache initialization at
`PrintConfig.cpp:10571-10585`.

`GCodeConfig` contains 151 active C++ members. Only 149 have a registered
`PrintConfigDef` entry and therefore enter the static runtime key set:

```text
151 active members
- thumbnail_size            (unregistered legacy input)
- bbl_bed_temperature_gcode (unregistered temporary placeholder)
= 149 registered GCodeConfig keys
```

`thumbnail_size` canonicalization to `thumbnails` remains Task 19A.
`bbl_bed_temperature_gcode` is not an Option; its later placeholder behavior
belongs to the template/document rewrite. Task 17 creates no field for either.
The exact registered ownership already typed by Tasks 7, 11, 12, and 14 is
Printer 62 + Process 17 + Filament 53 + residual/project 17 = 149. The
committed 653-row inventory is the reproducible key/type/default evidence; no
active test pins raw Orca source lines or needs an Orca checkout.

**Files:**
- Create: `crates/ares-core/src/options/gcode_fields.rs` as the single
  compile-time effective 149-field ledger, carrying each Rust field, canonical
  wire key, concrete Rust type, and its unique typed source owner
- Create: `crates/ares-core/src/options/gcode_options.rs`
- Create: `crates/ares-core/src/options/tests/gcode_options.rs` with
  `gcode_options/{inventory,types,projection,templates,fixture}.rs` siblings
- Modify: `crates/ares-core/src/options.rs`,
  `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/lib.rs`
- Modify only after final implementation approval:
  `docs/architecture/option-parity-v4.md` and `docs/roadmap.md`

Do not modify the four already-approved raw source groups merely to change
their structure. Their declarations remain the one raw owner of each value;
`gcode_fields.rs` owns only the effective projection definition. Split test
siblings further if required to keep every changed Rust file below 400
physical lines.

**Interfaces:**

```rust,ignore
#[derive(Clone, Debug, PartialEq)]
pub struct GCodeOptions {
    // 149 public concrete fields generated from the compile-time ledger.
}

impl GCodeOptions {
    pub(crate) fn from_sources(
        printer: &PrinterGCodeSourceOptions,
        process: &ProcessGCodeSourceOptions,
        filament: &FilamentGCodeSourceOptions,
        project: &ProjectGCodeSourceOptions,
    ) -> Self;
}
```

This boundary is infallible. It receives no active-filament parameter, returns
no `SliceError`, implements no `Default`, `Deserialize`, or `Serialize`, and
performs only direct typed clone/copy from four pairwise-disjoint owners. Task
18 owns production top-level 3MF-to-`ProjectSettings` parsing. Task 19B owns
active sizing, printer variant indices `[0,2]`, filament variant indices
`[0,4]`, nullable retract inheritance, `has_scarf_joint_seam` recomputation,
normalization, and final reprojection. Task 19C owns config-block export.

Freeze the exact registered option histogram:

```text
coBool=27, coBools=9, coEnum=6, coEnums=5,
coFloat=14, coFloats=38, coFloatOrPercent=3,
coInt=5, coInts=11, coPercent=1, coPercents=1,
coPoints=1, coString=13, coStrings=15
```

The raw wire boundary represented by these fields is 69 scalars plus 80
arrays. The exact nine nullable-element arrays are
`nozzle_flush_dataset`, `nozzle_type`,
`filament_adaptive_volumetric_speed`, `filament_cooling_before_tower`,
`filament_flow_ratio`, `filament_flush_temp`,
`filament_flush_volumetric_speed`, `long_retractions_when_ec`, and
`retraction_distances_when_ec`. The concrete enum/newtype domains remain the
already-approved `GCodeFlavor`, `BedTemperatureFormula`,
`PowerLossRecoveryMode`, `PrinterStructure`, `WipeTowerType`,
`ExtruderTypes`, `NullableNozzleTypes`, `RetractLiftEnforces`, `ZHopTypes`,
`ProjectFilamentMapMode`, `NozzleVolumeTypes`, and existing opaque wrappers.
There is no catch-all effective value type.

- [ ] **Step 1: RED/GREEN registered inventory and concrete structure**

  Start with a genuine missing-`GCodeOptions` compiler RED. Independently read
  the committed inventory in test code and assert exactly 149 unique `g_code`
  rows, the pairwise-disjoint 62/17/53/17 source partition, the complete type
  histogram, 69/80 wire-shape split, and exact nullable set above. Assert the
  union equals the four existing source declaration sets and excludes both
  unregistered active C++ members. Add compile-time concrete type assertions
  for every field, including the wire/Rust spelling pair
  `required_nozzle_HRC` / `required_nozzle_hrc`.

  Implement only the single compile-time effective ledger and public concrete
  `GCodeOptions` structure. Generate its test-only canonical key inventory from
  that ledger. Do not create a runtime registry, deserialize an effective
  struct, serialize it, parse the upstream checkout, or add a source-line
  pinning test.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core gcode_options_inventory
  cargo +1.91.0 nextest run -p ares-core gcode_options_types
  ```

- [ ] **Step 2: RED/GREEN direct four-source projection**

  Start with a genuine missing-`from_sources` RED. Use independent explicit
  test assertions for all 149 fields and distinct typed source values so an
  omission, same-typed field swap, or default substitution fails. Prove each
  field has exactly one owning source and all four default source groups also
  project field-for-field. There is no precedence between the disjoint groups.

  Implement `from_sources` through the compile-time ledger so each destination
  field can only clone the identically named field from its fixed source
  owner. Add no string-key lookup, serde round trip, generic value, fallback,
  validation, resizing, selection, override, or normalization.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core gcode_options_projection
  ```

- [ ] **Step 3: Verify template, opaque-string, vector, and nullable fidelity**

  This is a verification-only slice and may begin GREEN after Step 2. Prove
  byte preservation for all 16 template fields: the twelve printer strings
  `before_layer_change_gcode`, `printing_by_object_gcode`,
  `machine_end_gcode`, `layer_change_gcode`, `time_lapse_gcode`,
  `wrapping_detection_gcode`, `file_start_gcode`, `machine_start_gcode`,
  `change_filament_gcode`, `change_extrusion_role_gcode`,
  `machine_pause_gcode`, and `template_custom_gcode`; process
  `process_change_extrusion_role_gcode`; and filament vectors
  `filament_end_gcode`, `filament_start_gcode`, and
  `filament_change_extrusion_role_gcode`. Cover LF, CRLF, backslashes,
  placeholder expressions, UTF-8, empty strings, and trailing newlines.
  Expression parsing remains Task 28.

  Prove `adaptive_pressure_advance_model`,
  `volumetric_speed_coefficients`, `filament_ramming_parameters`, and
  `small_area_infill_flow_compensation_model` remain their existing opaque
  typed strings. Preserve every raw vector and nullable element exactly,
  including empty, singleton, unequal, three-, four-, and eight-element inputs.
  Tests must explicitly kill any `[0,2]`/`[0,4]` selection, resize, nil
  inheritance, or cardinality check in Task 17.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core gcode_options_templates
  cargo +1.91.0 nextest run -p ares-core gcode_options_shapes
  ```

- [ ] **Step 4: Verify the real 3MF through typed sources**

  This is a test-only verification slice. Load the real project through the
  bounded in-memory project reader, read only its project-settings bytes, and
  use the committed inventory in test code to split those bytes into the four
  typed source structs. Production top-level parsing remains Task 18. Assert
  all 149 effective fields equal their unique typed source value and template
  bytes remain exact. Freeze the fixture's raw 80-array length histogram as
  one empty, 49 length-two, 19 length-four, ten length-eight, and one
  length-ten array. The 19 printer-variant fields remain length four, the ten
  filament-variant fields remain length eight, and the other 43 filament
  G-code arrays remain length two. Do not read the reference G-code or add a
  production fixture name, ID, value, path, or hash branch.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core gcode_options_fixture
  ```

- [ ] **Step 5: Run review, documentation, release, and exact-SHA gates**

  Execute Steps 1-2 sequentially with fresh TDD implementers; no production
  writers overlap. Steps 3-4 use fresh bounded test implementers. Each slice
  receives fresh independent specification and quality review before the next
  begins. After Step 4, freeze the complete implementation bytes and obtain
  independent whole-diff `VERDICT: APPROVE` for specification and quality.
  The user-approved OpenCode bypass remains in force.

  Only after implementation approval, update architecture and roadmap,
  including the already-released Task 16 SHA/Tier-1 evidence and Task 17's
  included/deferred behavior. Obtain independent documentation approval, then
  run the complete frozen-byte matrix:

  ```powershell
  cargo +1.91.0 fmt --all -- --check
  cargo +1.91.0 nextest run -p ares-core gcode_options
  cargo +1.91.0 nextest run -p ares-core -E 'test(/(gcode_options|printer_gcode_source|filament_gcode_source|process_remaining|project_runtime_options|project_inventory|project)/)'
  cargo +1.91.0 nextest run --workspace
  cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
  cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
  cargo +1.91.0 check -p ares-core
  cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
  cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
  cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser ci
  npm --prefix crates/ares-wasm/tests/browser test
  git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
  ```

  Also require per-added-file no-index whitespace checks, exact changed-file
  ownership, every changed Rust file below 400 physical lines, unchanged
  fixture hashes, and production guards against dynamic/JSON/erased values,
  runtime option-key lookup, Option Pinning, fixture/reference reads or
  branches, active selection/normalization, native I/O, terminal/UI, FFI, and
  platform-specific code. Stage only the frozen approved manifest, prove
  index/workspace and commit-tree byte equality, commit
  `feat(config): resolve effective gcode options`, push, and require all five
  Tier 1 jobs green for that exact pushed SHA before Task 18.

**Explicitly deferred:** production flat project-settings parsing to Task 18;
legacy names/value conversion to Task 19A; active sizing, printer/filament
variant selection, nullable retract inheritance, model-driven recomputation,
normalization, and final reprojection to Task 19B; config export to Task 19C;
consumer migration/removal to Tasks 20A-20E; templates to Task 28; document
assembly to Task 29; and all geometry/G-code byte generation later in the
approved program.

---

### Task 18: Strict Typed `ProjectSettings` Load from the 3MF Package

**Fixed upstream rewrite boundary:** OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, specifically
`Config.cpp:573-685::set_deserialize_nothrow/set_deserialize/set_deserialize_raw`,
`Config.cpp:820-1100::ConfigBase::load_from_json`,
`Config.hpp:2763-2963::DynamicConfig`, and
`Format/bbs_3mf.cpp:210,1569-1573,1923-1926,2632-2653`
for the `Metadata/project_settings.config` load boundary. The Rust
destination is the existing five concrete groups under `options/`, an explicit
`ProjectSettings` serde visitor, and typed storage in `project::Project`.

Fixed Orca materializes the entire JSON object before dispatch, classifies
`from`/`name`/`version` separately, handles scalar strings and vector arrays,
then applies complete-document composite legacy conversion once. Its
`DynamicConfig` stores present options in a `std::map`. Per-entry legacy
rewrite, aliases/shortcuts, obsolete inputs, the JSON-specific mutations at
`Config.cpp:930-1089`, and the complete-document composite call are Task 19A,
not Task 18. Fixed project-settings JSON save at
`Config.cpp:1464-1502::ConfigBase::save_to_json` and
`Format/bbs_3mf.cpp:6351-6355,7722-7728` is an adjacent 3MF-export boundary,
not part of this load task or the current G-code parity program.

**Intentional strict Ares boundary decisions:** nlohmann object materialization
collapses duplicate member names, while fixed Orca ignores still-unregistered
keys after `handle_legacy`. At the untrusted 3MF JSON boundary Ares instead
rejects a duplicate canonical assignment and rejects a still-unknown canonical
key. Errors use compact key-specific text such as
`unknown Orca project option <key>`; do not render a 653-key `unknown_field`
registry. These stricter decisions, and the real fixture's lack of duplicate or
unknown keys, are tested explicitly. Fixed Orca dispatches project scalar
strings and vector arrays rather than native JSON booleans/numbers. The already
approved concrete Ares codecs accept native booleans/numbers where applicable;
Task 18 intentionally preserves that typed API compatibility behavior without
widening it further. The real 3MF fixture remains constrained to Orca's
string/array shape, and only that canonical shape is required for parity.

**Included:** canonical direct dispatch of all 653 fixture members into
`PrinterOptions` (132), `ProcessOptions` (352), `FilamentOptions` (122),
`ProjectRuntimeOptions` (44), and `PresetMetadata` (3); aggregate defaults for
omitted keys; bounded project JSON loading; and typed `Project::settings()`
access. The 650 option types retain
their existing concrete scalar/vector wrappers; the three metadata values are
strings.

**Explicitly deferred:** legacy key/value conversion and complete-document
composites to Task 19A; active filament sizing, variant selection, inheritance,
normalization, and recomputation to Task 19B; exact effective G-code
config-block export and metadata exclusion to Task 19C; dynamic consumer
migration/removal to Tasks 20A-20E; and all geometry/G-code behavior to later
approved rewrite slices. No top-level `Serialize for ProjectSettings` is added
or planned in this program. Tests may merge the
five groups' already-existing standalone serialized maps as a test-only
semantic oracle. Task 19C writes `FullPrintConfig`'s effective G-code
`CONFIG_BLOCK`; it does not serialize `ProjectSettings` or rewrite a 3MF.

**Production files:**
- Create: `crates/ares-core/src/options/project_deserialize.rs`
- Modify: `crates/ares-core/src/options.rs`
- Modify: `crates/ares-core/src/options/project_settings.rs`
- Modify: `crates/ares-core/src/options/printer_options.rs`
- Modify: `crates/ares-core/src/options/process_options.rs`
- Modify: `crates/ares-core/src/options/filament_options.rs`
- Modify: `crates/ares-core/src/options/project_runtime_options.rs`
- Modify: `crates/ares-core/src/options/preset_metadata.rs`
- Modify: `crates/ares-core/src/project/domain.rs`
- Modify: `crates/ares-core/src/project/load.rs`
- Modify: `crates/ares-core/src/project/xml/role.rs`

The four aggregate option modules expose only the crate-private builder
dispatch and resolution functions needed by the top-level visitor.
`ProcessOptionsBuilder` and `FilamentOptionsBuilder` become crate-private and
gain aggregate `deserialize_known_field`/`resolve` methods matching the already
usable printer/project builders. `PresetMetadata` gains the same small concrete
builder boundary. Each aggregate's existing standalone strict serde behavior
remains unchanged. No `flatten`, `serde_json::Value`, raw parked value,
`BTreeMap`, runtime option registry, or all-key sort is permitted in the new
production path.

**Test files:**
- Create: `crates/ares-core/src/options/tests/project_deserialize.rs` and split
  focused children before any Rust file reaches 400 physical lines
- Create: `crates/ares-core/src/options/tests/project_fixture.rs`, a test-only
  bounded archive reader for the raw fixture JSON oracle
- Modify: `crates/ares-core/src/options/tests.rs`
- Modify the twelve existing option-test callers of
  `Project::project_settings_bytes()` under `options/tests/` to use the shared
  test-only archive oracle, including the nested G-code and region fixtures
- Modify: `crates/ares-core/src/project/tests/model/import.rs`

The raw fixture bytes remain a test oracle read directly through
`ProjectArchive`; they are not retained in the production `Project`, and no
`#[cfg(test)]` raw field is added to that domain type.

- [ ] **Slice 18.1: RED/GREEN canonical five-group visitor**

  First add a genuine compile/test RED for missing `Deserialize<ProjectSettings>`
  and top-level dispatch. Then implement a single streaming visitor backed by
  the five concrete builders. Prove arbitrary input member order, defaults for
  omitted canonical keys, exact 132/352/122/44/3 ownership, 653 unique consumed
  fixture members, and representative values from every concrete wire family.
  Leave one explicit typed insertion point before the compact unknown branch
  for Task 19A; do not implement legacy behavior.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core project_deserialize
  cargo +1.91.0 nextest run -p ares-core project_inventory
  ```

  Freeze the slice diff and require independent spec-compliance and code-quality
  `APPROVE` before continuing.

- [ ] **Slice 18.2: RED/GREEN strict canonical boundary and semantic oracle**

  Add failures before their production branches for compact exact-key unknown
  and duplicate diagnostics, null/wrong containers, and invalid scalar, enum,
  array, and vector lexemes. For the bounded malformed cases in this slice,
  assert each diagnostic is below 1,024 bytes and contains its exact key; this
  is not a universal promise for an arbitrarily long unknown key inside the
  existing 64 MiB document limit. In test code, prove that direct native
  bool/number values already accepted by the
  approved concrete Ares codecs remain accepted and canonicalize through those
  groups' existing serializers; Task 18 neither widens nor narrows those codec
  contracts. The real Orca fixture boundary remains scalar strings and arrays.
  Also in test code only, serialize the five concrete groups independently,
  merge their
  object entries, and assert exact 653-member semantic equality with the raw
  fixture, no nested scope objects, all fixture scalar values are strings, and
  all fixture arrays contain only strings. Do not add top-level production
  serialization or a project-settings JSON writer.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core project_deserialize
  cargo +1.91.0 nextest run -p ares-core project_inventory
  ```

  Freeze the slice diff and require independent spec-compliance and code-quality
  `APPROVE` before continuing.

- [ ] **Slice 18.3: RED/GREEN typed project load and raw-API removal**

  Add a RED for missing `Project::settings()` and strict project-settings JSON
  diagnostics. Add `JsonRole::ProjectSettings`; parse the bounded archive bytes
  through `deserialize_json::<ProjectSettings>`; store the concrete value in
  `Project`; expose `settings(&self) -> &ProjectSettings`; and delete the raw
  field plus `project_settings_bytes()`. Migrate existing raw-oracle tests to
  the shared test-only archive reader. Synthetic project packages must prove
  canonical partial settings load with defaults and that unknown, duplicate,
  and ill-typed settings fail as `invalid project settings JSON: ...` before
  project slicing begins.

  ```powershell
  cargo +1.91.0 nextest run -p ares-core project_import
  cargo +1.91.0 nextest run -p ares-core project_deserialize
  cargo +1.91.0 nextest run -p ares-core -E 'test(/(filament_gcode_source|filament_remaining|gcode_options|printer_gcode_source|printer_machine_envelope|printer_remaining|process_object_source|process_region_source|process_remaining|project_inventory|project_runtime_options|region_options)/)'
  ```

  Freeze the slice diff and require independent spec-compliance and code-quality
  `APPROVE` before continuing.

- [ ] **Slice 18.4: Real-fixture isolation and whole-spec review**

  Load `ksr_fdmtest_v4.project.3mf` through the public byte API and prove
  `Project::settings()` is semantically equal to the independently extracted
  653-member raw JSON oracle. Prove the project loader/project slicer do not
  construct, deserialize, or inspect the temporary dynamic `SliceOptions` map;
  the explicit STL `slice(input, SliceOptions)` compatibility shell and all its
  existing consumers remain baseline-covered and unchanged for Tasks 20A-20E.
  Browser project slicing must still reach the existing
  `ProjectSlicingIncomplete` boundary only after typed project loading succeeds.
  Freeze the complete implementation manifest and require independent whole-spec
  and code-quality `APPROVE`; any `REVISE` restarts review on a new frozen diff.

- [ ] **Task 18 documentation and release gate**

  Only after whole-spec `APPROVE`, update
  `docs/architecture/option-parity-v4.md` and `docs/roadmap.md`, then require an
  independent documentation `APPROVE`. Run the complete frozen release matrix:

  ```powershell
  cargo +1.91.0 fmt --all -- --check
  cargo +1.91.0 nextest run -p ares-core project_deserialize
  cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_deserialize|project_import|project_inventory|gcode_options|printer_gcode_source|filament_gcode_source|process_remaining|project_runtime_options|region_options)/)'
  cargo +1.91.0 nextest run --workspace
  cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
  cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
  cargo +1.91.0 check -p ares-core
  cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
  cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
  cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser ci
  npm --prefix crates/ares-wasm/tests/browser test
  git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
  ```

  Also require exact changed-file ownership, per-added-file no-index whitespace
  checks, every changed Rust file below 400 physical lines, unchanged fixture
  hashes, and production scans forbidding dynamic/JSON/erased values, runtime
  key registries, raw fixture/reference access, Option Pinning, native I/O,
  terminal/UI, FFI, platform-specific code, and geometry/G-code changes in the
  Task 18 path. Stage only the frozen approved manifest, prove
  index/workspace/commit-tree byte equality, commit
  `feat(config): deserialize strict typed project settings`, push, and require
  all five Tier 1 jobs green for that exact pushed SHA before Task 19A.

---

### Task 19A: Typed Legacy Conversion Across Project Inputs

**Frozen upstream boundary:** every citation in this task is against OrcaSlicer
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, not the mutable checkout HEAD:

- `PrintConfig.cpp:8033-8285::PrintConfigDef::handle_legacy` owns the per-entry
  key/value rewrite and the final registered-key/obsolete-key decision.
- `Config.cpp:573-685` owns lexical typed decode, including canonical alias
  lookup at `Config.cpp:603-626`; JSON iteration begins at `Config.cpp:885` and
  its string/array value branches are `Config.cpp:927-1000`.
  `Config.cpp:1008-1088` owns the two slicing-state side effects plus
  profile-difference bookkeeping.
- `PrintConfig.cpp:8290-8339::handle_legacy_composite` and
  `GCode/Thumbnails.cpp:530-577` own the post-load thumbnail conversion.
- `Config.cpp:1092-1095,1184-1186,1273-1275,1455-1457` calls the composite only
  after a complete top-level config load. Object and volume XML metadata use
  per-entry dispatch at `Format/bbs_3mf.cpp:2119-2132,5088-5117` and never call
  the composite.

#### Reviewed source inventory and scope

The committed 653-field fixture inventory is a coverage oracle, not the rule
truth. Its 88 `LegacyInput` rows cover 73 distinct input names. The fixed source
has 76 explicit conditional input names and 44 unconditional obsolete names:

`76 = (73 fixture-ledger names - perimeter_feed_rate) + 4 source-only names`.

The four source-only names are `inherits_cummulative`,
`compatible_printers_condition_cummulative`,
`compatible_prints_condition_cummulative`, and
`different_settings_to_system`. Their targets are not among the current 653
typed fields. The first three rename to `inherits_group`,
`compatible_machine_expression_group`, and
`compatible_process_expression_group`. The fourth strips quotes, splits and
deduplicates semicolon-separated names, recursively rewrites each name, and
replaces changed substrings. All four are profile/UI metadata, not slicing
state, and are source-cited deferred: Task 19A records their exact disposition
in the compile-time audit inventory but does not accept, store, or invent fields
or dynamic state for them. Therefore the executable Task 19A boundary is the
remaining 72 fixed explicit names plus all 44 obsolete names.

The compile-time action inventory must encode the following exact inputs and
parameters. It is an exhaustive source ledger, not a runtime string registry:

| Action | Exact fixed inputs and parameters |
| --- | --- |
| Direct rename | `enable_wipe_tower -> enable_prime_tower`; `wipe_tower_width -> prime_tower_width`; `wiping_volume -> prime_volume`; `wipe_tower_brim_width -> prime_tower_brim_width`; `tool_change_gcode -> change_filament_gcode`; `bridge_fan_speed -> overhang_fan_speed`; `wipe_tower_extruder -> wipe_tower_filament`; `support_material_extruder -> support_filament`; `support_material_interface_extruder -> support_interface_filament`; `support_material_angle -> support_angle`; `support_material_enforce_layers -> enforce_support_layers`; `cooling -> slow_down_for_layer_cooling`; `timelapse_no_toolhead -> timelapse_type`; `sparse_infill_anchor -> infill_anchor`; `sparse_infill_anchor_max -> infill_anchor_max`; `chamber_temperatures -> chamber_temperature`; `thumbnail_size -> thumbnails`; `initial_layer_flow_ratio -> bottom_solid_infill_flow_ratio`; `ironing_direction -> ironing_angle`; `counterbole_hole_bridging -> counterbore_hole_bridging`; `prime_tower_extra_rib_length -> wipe_tower_extra_rib_length`; `prime_tower_rib_width -> wipe_tower_rib_width`; `prime_tower_fillet_wall -> wipe_tower_fillet_wall`; `extruder_clearance_max_radius -> extruder_clearance_radius`; `machine_switch_extruder_time -> machine_tool_change_time`. |
| Feature-filament rename | `infill_extruder` and `sparse_infill_filament -> sparse_infill_filament_id`; `solid_infill_extruder` and `solid_infill_filament -> internal_solid_filament_id`; `top_solid_infill_filament -> top_surface_filament_id`; `bottom_solid_infill_filament -> bottom_surface_filament_id`; `perimeter_extruder`, `wall_filament`, and `wall_filament_id -> outer_wall_filament_id`; `inner_wall_filament -> inner_wall_filament_id`; `outer_wall_filament -> outer_wall_filament_id`. For every row, exact legacy value `"1"` becomes canonical inherit `"0"`; all other values are retained. |
| Conditional consume | For `initial_layer_print_height`, `initial_layer_speed`, `internal_solid_infill_speed`, `top_surface_speed`, `support_interface_speed`, `outer_wall_speed`, and `support_object_xy_distance`, consume the whole assignment only when the lexical value contains `%`; otherwise decode the canonical key normally. `top_one_wall_type` consumes `none`, otherwise writes `only_one_wall_top=true`. `prime_tower_rib_wall` writes `wipe_tower_wall_type=rib` only for `1`, otherwise consumes the assignment. |
| Exact value rewrite | `curr_bed_type`: `SuperTack Plate -> Supertack Plate`; `timelapse_type`: `2 -> 0`; `support_type`: `normal -> normal(manual)`, `tree -> tree(manual)`, `hybrid(auto) -> tree(auto)`; `support_base_pattern`: `none -> hollow`; `overhang_fan_threshold`: `5% -> 10%`; `enable_power_loss_recovery`: case-insensitive `true` or `1 -> enable`, case-insensitive `false` or `0 -> disable`; `ensure_vertical_shell_thickness`: `1 -> ensure_all`, `0 -> ensure_moderate`; `rotate_solid_infill_direction -> solid_infill_rotate_template` with `1 -> 0,90`, `0 -> 0`; `ironing_angle`: any leading `- -> 0`; `draft_shield`: `limited -> disabled`; `filament_map_mode`: `Auto -> Auto For Flush`; `wall_direction`: `auto -> ccw`. Nonmatching values retain the source key/value unless a conditional-consume rule above says otherwise. The upstream `else-if` chain is intentionally non-recursive, so `ironing_direction=-45` renames without reapplying the `ironing_angle` branch in that call. |
| Wall order | `wall_infill_order` always renames to `wall_sequence`: the two inner-first spellings become `inner wall/outer wall`; the two outer-first spellings become `outer wall/inner wall`; `inner-outer-inner wall/infill` becomes `inner-outer-inner wall`; any other value is retained under `wall_sequence`. Only the JSON project path additionally writes `is_infill_first=true` for `infill/outer wall/inner wall` and `infill/inner wall/outer wall`. |
| Global replacement | For `nozzle_volume_type`, `default_nozzle_volume_type`, `printer_extruder_variant`, `print_extruder_variant`, `filament_extruder_variant`, and `extruder_variant_list`, replace every `Normal` with `Standard` and every `Big Traffic` with `High Flow`. For `extruder_type`, replace every `DirectDrive` with `Direct Drive`. |
| Pattern rewrite | For `sparse_infill_pattern`, `top_surface_pattern`, `bottom_surface_pattern`, `internal_solid_infill_pattern`, `ironing_pattern`, and `support_ironing_pattern`, exact `zig-zag -> rectilinear`. |
| Filament token rebuild | For `filament_type`, split on `;`, strip one surrounding quote pair per token, replace exact token `ASA-Aero -> ASA-AERO`, and only when a token changed rebuild every token quoted and joined by `;`. |
| JSON-only derived side effects | `support_type=hybrid(auto)` writes canonical `support_type=tree(auto)` and schedules typed `support_style=tree_hybrid`. The two infill-first `wall_infill_order` spellings schedule `is_infill_first=true`. Fixed `Config.cpp:1008-1017` applies both after the complete JSON iteration, so each derived value overwrites an explicit canonical target regardless of whether that target appeared before or after the trigger. A derived write is not an alias/canonical duplicate. XML model dispatch performs only the per-entry key/value rewrite. |
| Source-only deferred | The three cumulative renames and recursive `different_settings_to_system` behavior described above remain explicit `DeferredProfileBookkeeping` inventory rows. Ares stores neither their source nor target values in Task 19A. |

The 44 exact obsolete inputs are consumed without decoding:
`acceleration`, `scale`, `rotate`, `duplicate`, `duplicate_grid`, `bed_size`,
`print_center`, `g0`, `wipe_tower_per_color_wipe`, `support_sharp_tails`,
`support_remove_small_overhangs`, `support_with_sheath`,
`tree_support_collision_resolution`, `tree_support_with_infill`,
`max_volumetric_speed`, `max_print_speed`, `support_closing_radius`,
`remove_freq_sweep`, `remove_bed_leveling`,
`remove_extrusion_calibration`, `support_transition_line_width`,
`support_transition_speed`, `bed_temperature`,
`bed_temperature_initial_layer`, `can_switch_nozzle_type`,
`can_add_auxiliary_fan`, `extra_flush_volume`, `spaghetti_detector`,
`adaptive_layer_height`, `z_hop_type`, `z_lift_type`,
`bed_temperature_difference`, `long_retraction_when_cut`,
`retraction_distance_when_cut`, `internal_bridge_support_thickness`,
`top_area_threshold`, `reduce_wall_solid_infill`, `filament_load_time`,
`filament_unload_time`, `smooth_coefficient`, `overhang_totally_speed`,
`silent_mode`, `overhang_speed_classic`, and `filament_prime_volume`.

Two audited fixed branches are expressly not implemented:

- `perimeter_feed_rate -> inner_wall_speed` is only an option alias at
  `PrintConfig.cpp:5046`, not one of the 76 `handle_legacy` inputs. Project/model
  loaders run `handle_legacy` before alias lookup, and the unregistered key is
  cleared at `PrintConfig.cpp:8281-8283`. The fixture ledger row is excluded.
- `wiping_volumes_matrix` and `wiping_volumes_use_custom_matrix` occur only in
  `handle_legacy_composite` at `PrintConfig.cpp:8320-8338`; neither is
  registered or written in the fixed `PrintConfigDef`, so per-entry handling
  clears them before the composite. They are absent from the 653 typed fields.
  Task 19A must not add them or confuse them with `flush_volumes_matrix`.

Committed behavioral tests prove these inputs remain unavailable/unknown and
that the fixture/source set difference is exact. Review evidence separately
re-runs fixed-commit `git grep` for the cited definitions and call order; no
committed test reads or pins Orca source text.

#### Ares ownership and interfaces

**Create production files:**

- `crates/ares-core/src/options/typed_legacy.rs`: private facade and typed
  outcomes only.
- `crates/ares-core/src/options/typed_legacy/actions.rs`: the complete 76-name
  source action inventory, exact parameters, per-rule string/array wire shape,
  array empty-value first-pass outcome, and four deferred dispositions.
- `crates/ares-core/src/options/typed_legacy/obsolete.rs`: the exact 44-name
  compile-time obsolete set.
- `crates/ares-core/src/options/typed_legacy/convert.rs`: pure concrete lexical
  transformations for the 72 executable names; no erased value.
- `crates/ares-core/src/options/typed_legacy/project.rs`: top-level typed
  dispatch and JSON-only `support_style` / `is_infill_first` writes.
- `crates/ares-core/src/options/typed_legacy/model.rs`: XML per-entry
  canonicalization and typed owner handoff.
- `crates/ares-core/src/options/typed_legacy/thumbnails.rs`: presence-aware
  thumbnail composite only.

**Create test files:**

- `crates/ares-core/src/options/tests/typed_legacy.rs` with split
  `typed_legacy/inventory.rs`, `typed_legacy/convert.rs`,
  `typed_legacy/project.rs`, and `typed_legacy/thumbnails.rs` siblings.
- `crates/ares-core/src/project/tests/documents/object_settings_metadata/legacy.rs`
  for ordered object/part XML integration.

**Modify only these existing ownership points:**

- `crates/ares-core/src/options.rs` and
  `crates/ares-core/src/options/tests.rs` for module wiring.
- `crates/ares-core/src/options/option_group.rs` to generate a crate-private
  typed-value assignment entry point alongside canonical `MapAccess` decode;
  it accepts a concrete deserializer and never a dynamic option value.
- The five aggregate builders in
  `options/printer_options.rs`, `options/process_options.rs`,
  `options/filament_options.rs`, `options/project_runtime_options.rs`, and
  `options/preset_metadata.rs` to delegate that typed assignment entry point.
- `options/project_settings.rs` to add one crate-private
  `ProjectSettingsBuilder` owning the five aggregate builders, target presence,
  strict duplicate detection, the two typed JSON side effects, and composite
  before final resolution; `options/project_deserialize.rs` becomes the thin
  streaming visitor that delegates canonical and reviewed legacy entries.
- `project/model_settings.rs`,
  `project/model_settings/object_metadata.rs`, and
  `project/model_settings/part_metadata.rs` for ordered XML dispatch, plus the
  existing object-settings test module only to wire its new `legacy` sibling.
- After whole implementation approval only,
  `docs/architecture/option-parity-v4.md` and `docs/roadmap.md`.

Every changed Rust file must remain below 400 physical lines. If any named file
would cross that limit, split only its stated responsibility into a same-named
private submodule before review; do not move unrelated code.

The per-rule inventory has an exhaustive wire/action contract matching fixed
`Config.cpp:573-685,885,927-1000`:

- A top-level JSON string and an XML metadata lexical string each execute the
  rule once with the actual lexical value. The resulting canonical concrete
  target is decoded directly. Every non-deferred, non-obsolete rule accepts
  this string path; nonmatching conditional values follow the exact preserve or
  consume action recorded in the table above.
- A top-level JSON array first executes the rule with `value=""`, exactly as
  `Config.cpp:950-956` does. The action inventory records that empty-value
  outcome per input; it is not reduced to a key-only rewrite. In particular,
  `top_one_wall_type` maps the empty value to
  `only_one_wall_top="1"`, while `prime_tower_rib_wall` consumes the assignment.
  Obsolete and other conditional-consume behavior also follows its exact empty
  predicate before any array decode.
- If that first pass retains an input, only these registered array targets are
  array-capable in the current typed boundary: `overhang_fan_speed` and
  `chamber_temperature` as `coInts`; `slow_down_for_layer_cooling` as `coBools`;
  `nozzle_volume_type`, `default_nozzle_volume_type`, `extruder_type`, and
  `overhang_fan_threshold` as `coEnums`; and `extruder_variant_list`,
  `filament_extruder_variant`, `filament_type`, `print_extruder_variant`, and
  `printer_extruder_variant` as `coStrings`. Their legacy source aliases inherit
  the target's specific vector shape. Every other rule/target is string-only;
  a JSON array for it is an invalid option value unless the exact empty-value
  rule consumed it first.
- For an allowed array, reproduce `Config.cpp:831-872,975-1000`: require
  homogeneous JSON element kinds at each level and string leaves; flatten
  `coInts`, `coBools`, and `coEnums` with `,`; flatten `coStrings` as C-style
  escaped, quoted strings separated by `;`; reject unsupported depth/kind or a
  value the specific typed vector cannot decode. The flattened lexical string
  is appended to any first-pass value, then the canonicalized key and complete
  flattened string execute `handle_legacy` semantics a second time before typed
  assignment. Do not rewrite array elements independently.
- Exhaustive tests cover the second-pass distinctions: global replacements for
  `nozzle_volume_type`, `default_nozzle_volume_type`, and `extruder_type` run
  over the complete comma-flattened `coEnums` string; the four variant-list
  inputs run replacements over the complete quoted/semicolon `coStrings`
  string; `filament_type` tokenizes that complete flattened string and rebuilds
  only when exact `ASA-Aero` changes; exact whole-string predicates such as
  `overhang_fan_threshold="5%"` trigger only when the flattened value itself
  equals the predicate. Tests include singleton and multi-item vectors,
  escaping, mixed/non-string/nested invalid kinds, and first-pass
  append/consume cases.

The top-level visitor consumes each JSON value once into either the declared
lexical string or the declared specific typed-vector path; no generic dynamic
value is parked. The canonical target presence bit is shared by canonical and
legacy spellings, so any input assignment collision is Task 18's strict
duplicate error after canonicalization. The two derived slicing-state writes
are stored separately until JSON iteration finishes, then applied over any
explicit `support_style` / `is_infill_first` target without raising duplicate,
which makes the result independent of input order. Without a trigger, an
explicit target is unchanged. Unknown and the four deferred profile/UI names
error with the exact input name; obsolete assignments are the only reviewed
inputs silently consumed.

Object and part metadata preserve XML order. A canonicalized target owned by
`ObjectOptionOverrides` or `RegionOptionOverrides` is parsed and written
directly to that typed owner; later canonical or legacy assignments win.
Canonicalized non-owner targets are retained under the canonical key at the
same ordered position for Task 19B. Unclassified non-legacy entries remain
ordered and unchanged. Structural `name`, `module`, `matrix`, `source_*`, and
mesh metadata bypass option dispatch entirely; `mesh_stat` remains an element.
XML never receives JSON-only side effects or a composite.

Thumbnail conversion runs on `ProjectSettingsBuilder` before `Option` presence
is discarded, and only when canonical `thumbnails` or legacy `thumbnail_size`
was present. A missing per-item format uses present `thumbnails_format`, or PNG
when that option was absent; explicit per-item formats win. Invalid dimensions,
range, or format return the typed project-option error. Valid items normalize to
fixed `WIDTHxHEIGHT/FORMAT` spelling joined by comma-space. A missing thumbnails
input does nothing even though the resolved printer struct has defaults.

The Task 19A production path forbids `serde_json::Value`, `serde_json::Map`,
`RawValue`, `Box<dyn Any>`, equivalent erased values, a runtime option registry,
fixture-name/hash branches, reference-G-code reads, native I/O, terminal/UI,
FFI, Option Pinning, and calls into the temporary dynamic `SliceOptions` legacy
path. Existing `options/legacy.rs`, its submodules, and legacy tests remain
unchanged and baseline-covered until Task 20E.

- [ ] **Slice 19A.1: RED/GREEN exhaustive source action inventory**

  First add failing inventory tests proving 76 distinct explicit source names,
  44 distinct obsolete names, no overlap, exact action parameters, exactly four
  `DeferredProfileBookkeeping` rows, and the exact fixture delta
  `{source-only four}` / `{ledger-only perimeter_feed_rate}`. Add table-driven
  RED cases for every direct rename and feature-filament parameter, including
  non-recursive ordering, per-rule JSON string/XML string/JSON array allowance,
  target vector type, and exact empty-string first-pass action. GREEN only the
  compile-time action/obsolete modules and pure typed outcomes. Freeze this
  slice and require independent spec-compliance and code-quality `APPROVE`.

- [ ] **Slice 19A.2: RED/GREEN all conditional transformations and consumes**

  Add failing table-driven tests for every value-rewrite branch and its
  nonmatching branch, the seven percentage consumes, all wall-order spellings,
  global replacements, six pattern inputs, filament-token rebuilding, all 44
  obsolete inputs, and the two conditional consumes. Add genuine RED array
  cases for all twelve allowed targets, both first-pass special outcomes,
  comma/coStrings flattening, second-pass whole-string transformations,
  singleton versus multi-item enum predicates, escaping, mixed/non-string or
  unsupported nested kinds, and arrays rejected for string-only targets. GREEN
  the concrete transformer only; prove there is no element-wise migration,
  JSON/erased value, or runtime registry. Freeze and independently approve both
  spec compliance and code quality.

- [ ] **Slice 19A.3: RED/GREEN strict typed JSON project integration**

  Add failing streaming-deserialization tests for all executable action families
  through `ProjectSettings`, canonical/legacy and legacy/legacy collisions,
  unknown/deferred exact-name errors, and JSON-only effects:
  `support_type=hybrid(auto)` also sets `support_style=tree_hybrid`, while the
  two infill-first wall spellings also set `is_infill_first=true`. For each
  derived target, add genuine RED cases with an explicit conflicting canonical
  target before and after the trigger; both orders must end with the derived
  value and must not report a duplicate. Cover both infill-first spellings and
  prove a non-trigger preserves its explicit target. Prove no
  `different_settings_to_system` state appears. GREEN the option-group typed
  assignment entry point, five aggregate delegates, `ProjectSettingsBuilder`,
  deferred post-iteration derived writes, and thin visitor. Freeze and
  independently approve both reviews.

- [ ] **Slice 19A.4: RED/GREEN ordered object and part XML integration**

  Add failing object/part tests for owner-target direct writes, non-owner
  canonical retention, both canonical/legacy orders, feature-filament `1 -> 0`,
  obsolete consumption, structural bypass, unclassified retention, and absence
  of JSON-only effects/composite. GREEN only model dispatch and the two metadata
  loops while preserving XML last-write-wins. Freeze and independently approve
  both reviews.

- [ ] **Slice 19A.5: RED/GREEN presence-aware thumbnail composite and boundary proofs**

  Add failing tests for absent/present thumbnails, `thumbnail_size`, present or
  absent `thumbnails_format`, explicit per-item formats, normalized output,
  invalid dimensions/range/format, and collision behavior. Add behavioral
  boundary tests proving `perimeter_feed_rate`, `wiping_volumes_matrix`, and
  `wiping_volumes_use_custom_matrix` are not accepted/created and canonical
  `flush_volumes_matrix` is unchanged. GREEN only the thumbnail module and its
  pre-resolution call. Then prove the real fixture remains canonically
  idempotent through the public byte API and browser project load still reaches
  the existing post-load slicing boundary. Freeze and independently approve
  both reviews.

- [ ] **Whole Task 19A, documentation, and release gates**

  Freeze the complete implementation manifest. A fresh independent reviewer
  must return whole-spec `APPROVE`, and a separate quality reviewer must return
  whole-quality `APPROVE`; any edit invalidates both. Only then update the two
  owned docs, freeze that docs diff, and require independent docs `APPROVE`.
  Run the complete frozen release matrix:

  ```powershell
  cargo +1.91.0 fmt --all -- --check
  cargo +1.91.0 nextest run -p ares-core typed_legacy
  cargo +1.91.0 nextest run -p ares-core -E 'test(/(typed_legacy|project_deserialize|object_options|region_options|project_import)/)'
  cargo +1.91.0 nextest run --workspace
  cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
  cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
  cargo +1.91.0 check -p ares-core
  cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
  cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
  cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser ci
  npm --prefix crates/ares-wasm/tests/browser test
  git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
  ```

  Also require source-proof review commands against fixed commit
  `8500fcdccaa10b5099ac20d252af3a7c560046f1` for the
  `perimeter_feed_rate` registration/load order and the absence of fixed
  wiping-volume registrations/writers; these are review evidence, not committed
  source-pinning tests. Require exact changed-file ownership, per-added-file
  no-index whitespace, every changed Rust file below 400 physical lines,
  unchanged fixture hashes, the forbidden-path scans above, fresh frozen-byte
  results, and index/workspace/commit-tree byte equality. Only after all
  approvals and fresh verification, commit
  `feat(config): port typed legacy conversion`, push, and require all five Tier
  1 jobs green for that exact pushed SHA before Task 19B.

Task 19B retains complete model-key classification, effective
`FullPrintConfig`, sizing, and FDM normalization. Task 19C retains exact
effective config-block serialization. Task 20E retains deletion/replacement of
the temporary dynamic `SliceOptions` legacy path. Task 19A does not pull any of
those responsibilities forward.

---

### Task 19B: Effective FullPrintConfig Resolution and FDM Normalization

**Upstream boundary:** Fixed-tag
`DynamicPrintConfig::{normalize_fdm,normalize_fdm_1,normalize_fdm_2,
set_num_extruders,set_num_filaments,get_parameter_size}`, `PrintApply.cpp`
active object/filament sizing and final region regeneration at
`:1620-1740`, `Model.hpp:161-201,917-918` and
`Model.cpp:622-652,2500-2508` optional in-memory material association, plus
`Format/bbs_3mf.cpp:216,1880-1910,2092-2095,2886-2939` optional
`Metadata/layer_config_ranges.xml` import/association. Fixed
`Format/3mf.cpp:1729-1734,1808-1813` ignores material property identifiers;
there is no fixed-source 3MF material-config document reader in this slice.
The retained model-config boundary is fixed to
`PrintConfig.cpp::PrintConfigDef`, `Config.cpp:573-685`, and
`Config.cpp:461-500` canonical lookup, lexical decode, and static projection.

**Files:**
- Create: `options/full_print_config.rs`
- Create: `options/project_normalize.rs`
- Create: `options/tests/project_normalize.rs`
- Create: `options/model_config_deserialize.rs` and focused
  `options/tests/model_config_deserialize.rs`
- Create: bounded in-memory `project/layer_config_ranges.rs` document module
  with focused tests
- Modify: project document retention/assembly only as needed to associate
  parsed layer ranges by object ID/range
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `FullPrintConfig::resolve(&Project, &ProjectSettings) -> Result<Self, SliceError>`.
- Produces `normalize_project_config(&mut FullPrintConfig) -> Result<(), SliceError>`.
- Completes the project-document ownership explicitly deferred by Task 16:
  optional layer-range configs are read from the bounded in-memory 3MF archive,
  retained as typed sparse region inputs, and associated before final
  per-region resolution. Optional typed material config is applied only when a
  source-supported model boundary supplies it; no 3MF material parser or
  synthetic archive material resource is invented. No native filesystem API
  enters `ares-core`.
- After Task 19A per-entry legacy rewriting, `model_config_deserialize.rs`
  consumes every Task 15 `retained_config` entry in XML order against a
  source-cited compile-time registry covering the complete fixed
  `PrintConfigDef` canonical key/type universe, not the fixture-only 653 rows.
  It directly validates each lexical concrete value, routes object/region keys
  to their typed sparse owners, retains only source-supported state needed by
  normalization, and rejects a still-unknown key with its exact name. It never
  creates a generic option value or dynamic map.

- [ ] **Step 1: Write RED active-sizing and normalization tests**

  Assert single-material `enable_prime_tower` normalizes from raw `1` to
  effective `0`; 8-/4-stride source vectors resolve to the two active values
  used by the reference; object/region overrides and active extruder/filament
  maps apply in fixed order; invalid cardinality names its key. Synthetic
  in-memory 3MF documents prove layer-range import, ID/range association, and
  Task 16 precedence end-to-end; the current fixture's absence of that resource
  is explicit. Material precedence remains a pure typed Task 16 test because
  fixed-source 3MF supplies no material config document.

  Independently freeze the complete fixed global model-config key/type ledger
  and prove canonical keys outside the fixture's 653 rows are accepted and
  lexically validated, while a still-unknown retained key fails with its exact
  name after Task 19A dispatch. Cover ordered duplicate/alias outcomes before
  any object or region projection.

- [ ] **Step 2: Implement FullPrintConfig resolution and exact normalization order**

  ```rust
  pub struct FullPrintConfig {
      pub printer: PrinterOptions,
      pub process: ProcessOptions,
      pub filament: FilamentOptions,
      pub object: Vec<ObjectOptions>,
      pub region: Vec<RegionOptions>,
      pub gcode: GCodeOptions,
      pub project: ProjectRuntimeOptions,
  }
  ```

  Follow the interleaved fixed-tag `Print::apply` order rather than normalizing
  after projection: resolve the typed defaults/base merge; run
  `normalize_fdm_1`; determine initial active sizing and run the first
  `normalize_fdm_2`; build only the preliminary object/region usage needed to
  discover the new used-filament set; run the second `normalize_fdm_2`; apply
  its changed keys to the full/default object/default region bases; discard
  preliminary projections; and only then resolve the final per-object
  `ObjectOptions` and per-volume/material/layer-range `RegionOptions`, followed
  by final G-code/export projection.

  A stage-order test forces the second pass through changed filament usage and
  proves preliminary regions are discarded and final region resolution occurs
  afterward. The fixed `normalize_fdm_2` write set contains only
  `enable_prime_tower` and `independent_support_layer_height`, with zero
  intersection with the 153 `RegionOptions`; freeze that fact instead of
  inventing a nonexistent second-pass region-field mutation. Tests still place
  a real changed value at every source-supported stage and prove the next
  owning stage observes it, including infill relationships and the
  single-material prime-tower change. Missing required values are reported only
  at this external configuration boundary.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core project_normalize
  git commit -m "feat(config): resolve effective FDM config"
  git push
  ```

---

### Task 19C: Exact Effective Config-Block Serialization

**Upstream boundary:** Fixed-tag `ConfigBase::opt_serialize`, `GCode.cpp::append_full_config`, and config-block delimiters.

**Files:**
- Create: `options/config_export.rs`
- Create: `options/tests/config_export.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `write_config_block(&FullPrintConfig, &mut Vec<u8>)` with exact upstream ordering/escaping.

- [ ] **Step 1: Write the config-block byte RED test**

  Extract only the bytes between `; CONFIG_BLOCK_START\n` and `; CONFIG_BLOCK_END\n` from the committed reference in the test. Assert the independently inspectable shape before byte comparison: 639 assignment lines, 637 unique keys, exactly two occurrences each of `wipe_tower_x` and `wipe_tower_y`, and final computed `first_layer_bed_temperature` / `first_layer_temperature` lines. The 15 nil filament retract/ironing override keys and metadata `from`, `name`, and `version` are absent. Cover the fixed nine-key banned set, nil omission, `extruder_colour` sourced from `filament_colour`, flush-matrix multiplier correction, and plate-indexed wipe-tower formatting in synthetic tests. Compare the complete block bytes without any normalization.

- [ ] **Step 2: Implement serialization from concrete fields**

  Each concrete field declaration supplies its fixed key, resolved effective source, and typed Orca serializer. Build the equivalent of `print.full_print_config()` as one compile-time-known canonical entry per real option; raw-scope structs and Object/Region/G-code projections are not blindly concatenated and therefore cannot duplicate a key. Iterate canonical keys in fixed-tag `DynamicConfig::keys()` order, skip banned/nil entries, apply the flush-matrix and `extruder_colour` rules, emit the extra three-decimal plate value immediately before each ordinary `wipe_tower_x/y` line, and append the two computed temperature lines after the canonical loop. Preserve blank versus empty-vector behavior and normalized active cardinality without converting any field to a generic value.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core config_export
  git commit -m "feat(config): serialize effective config block"
  git push
  ```

---

### Task 20A: Migrate Option and Profile Consumers

**Upstream boundary:** Typed `PrintConfig.*` reads represented under Ares `options` and `profiles`.

**Files:**
- Modify: production dynamic users under `crates/ares-core/src/options` and `profiles`
- Modify/delete: parsing/registry helpers made unreachable
- Modify: `scripts/dynamic_value_baseline.txt`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Option/profile consumers receive concrete resolved option groups. Only the existing, explicitly named STL compatibility parser may still inspect the baseline-covered map; it is not moved, copied, or used by the project path.

- [ ] **Step 1: Remove only this ownership set from the baseline and establish RED**

  Classify the existing `options`/`profiles` rows into (a) consumer logic that can now take concrete groups and (b) the minimum pre-existing parser rows still required to convert the temporary STL map into those groups. Remove only set (a), leaving the `SliceOptions` declaration, `values()`, deserializer, and required pre-existing parser rows in the baseline, then run:

  ```powershell
  cargo nextest run -p ares-core --test no_unapproved_dynamic_values
  ```

  Expected: nonzero exit, with the audit reporting exactly that removed consumer set and no finding owned by the compatibility parser, `print_apply`, retained STL planning, or G-code modules. Save both sorted sets in the task review notes so GREEN proves that no finding was hidden or moved into a new adapter. New dynamic fingerprints remain forbidden.

- [ ] **Step 2: Convert consumers and retain behavior tests**

  Change consumer interfaces and profile composition to concrete source/effective group structs. Existing parsers such as `parse_acceleration_options(&values)` may survive only as pre-existing, baseline-covered STL-boundary adapters that construct those structs; they may not leak a map/value into a migrated consumer. Tests may use `json!` as input text, but runtime behavior below the compatibility boundary receives typed structs.

- [ ] **Step 3: Run the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core options
  cargo nextest run -p ares-core profiles
  git commit -m "refactor(config): type option and profile consumers"
  git push
  ```

---

### Task 20B: Migrate PrintApply Consumers

**Upstream boundary:** `PrintApply.cpp::Print::apply` and retained Ares `print_apply` runtime behavior.

**Files:**
- Modify: production dynamic users under `crates/ares-core/src/print_apply.rs` and `print_apply/`
- Modify/delete: helpers made unreachable
- Modify: `scripts/dynamic_value_baseline.txt`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Print application consumes typed config/diffs and preserves existing behavior.

- [ ] **Step 1: Remove the PrintApply fingerprints from the baseline and establish RED**

  Remove exactly the rows rooted at `crates/ares-core/src/print_apply.rs` and `crates/ares-core/src/print_apply/`, run the syntax audit, and require a nonzero exit whose sorted findings equal that removed set. Findings outside this ownership set remain covered by the baseline and must not appear in the RED output.

- [ ] **Step 2: Replace dynamic diff/value access with explicit typed fields and rerun every retained PrintApply behavioral test**

  Construct config diffs as typed field-change enums/structs owned by `PrintApply`, then dispatch exhaustively over those fields. Do not serialize typed fields back to JSON to compare them. Run `cargo nextest run -p ares-core print_apply` first, followed by the syntax audit; GREEN requires both and requires that every removed fingerprint has disappeared from production syntax.

- [ ] **Step 3: Run the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core print_apply
  git commit -m "refactor(print): type PrintApply consumers"
  git push
  ```

---

### Task 20C: Migrate Existing STL Geometry and Planning Consumers

**Upstream boundary:** Typed config reads used by retained STL-only `planning`, `pipeline`, `segments`, `contours`, perimeter/fill/path/move/speed scaffolds.

**Files:**
- Modify: dynamic config users in those retained production modules
- Modify/delete: helpers made unreachable
- Modify: `scripts/dynamic_value_baseline.txt`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- The separate STL API remains behaviorally green through concrete resolved option groups below its temporary compatibility boundary; no project fallback is introduced.

- [ ] **Step 1: Remove this ownership set's fingerprints and establish RED**

  Select baseline rows for the retained STL-only `planning`, `pipeline`, `segments`, `contours`, `bridges`, `perimeters`, `infills`, `print_paths`, `extrusions`, `moves`, `speeds`, and `printable_height` modules, excluding test files. Delete exactly those rows and require the syntax audit to fail with the same sorted set and no G-code-owned finding.

- [ ] **Step 2: Convert each consumer with its existing focused behavior test before deleting its dynamic helper**

  Migrate one existing focused test group at a time to construct/pass the concrete resolved group expected by that consumer, replace the corresponding production `values()`/`Value` branch with exhaustive typed access, and re-run that group before continuing. The temporary map may be read only by the pre-existing boundary adapters retained for Task 20E. Finish by running the complete retained STL suite and syntax audit; the project path remains forbidden from calling these modules.

- [ ] **Step 3: Run the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core planning
  cargo nextest run -p ares-core pipeline
  git commit -m "refactor(slicing): type retained STL config consumers"
  git push
  ```

---

### Task 20D: Migrate Existing G-code Consumers

**Upstream boundary:** Typed `GCodeConfig` reads in retained Ares `gcode*`, `gcode_writer*`, extrusion, speed, and template scaffolds.

**Files:**
- Modify: remaining production dynamic users in G-code-related modules
- Modify/delete: helpers made unreachable
- Modify: `scripts/dynamic_value_baseline.txt`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Existing STL G-code remains behaviorally green; project code will use typed `FullPrintConfig` directly.

- [ ] **Step 1: Remove G-code fingerprints from the baseline and establish RED**

  Select the remaining baseline rows in existing `gcode*`, `gcode_writer*`, `gcode_header*`, template, fan/temperature, object-label, scan, wrapping, and statistics modules. Delete exactly those rows and require the syntax audit to fail with that same sorted set. It must not report any already-approved ownership set.

- [ ] **Step 2: Convert consumers in focused test groups (header/config, custom code, motion, fans/temperature, statistics)**

  For each named group, first retain or add an active behavior test that changes one concrete config field and observes bytes/state; then replace all consumer-side map lookups with concrete `GCodeOptions` or another concrete resolved group and run the group. Only the unchanged STL boundary adapter may still read the temporary map. The final GREEN for this task is the union of those focused suites plus the syntax audit, with no field converted back through `serde_json`.

- [ ] **Step 3: Run the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core gcode
  git commit -m "refactor(gcode): type existing config consumers"
  git push
  ```

---

### Task 20E: Close the Dynamic-Value Migration Baseline

**Upstream boundary:** Final typed-config audit across all production crates.

**Files:**
- Modify: `scripts/dynamic_value_baseline.txt`
- Modify/delete: `options.rs`, `options/legacy.rs`, the remaining pre-existing STL compatibility parsers, and only other production files still reported by the AST audit
- Modify: explicit-option STL API tests for strict concrete partial input
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Replaces the temporary map shell with the final concrete partial `SliceOptions` over the Task 5-18 typed builder fields.
- The migration baseline is empty and stays empty; no open-field exception is expected.

- [ ] **Step 1: Delete the compatibility-shell baseline entries and establish RED on every residual fingerprint**

  By dependency, Tasks 20A-20D have already migrated every caller away from `values()` and erased option payloads. Remove all remaining baseline rows, including the `SliceOptions` map/deserializer and retained boundary parsers, then run the audit. Expected: nonzero exit listing exactly those still-present shell/parser fingerprints; compilation is allowed to remain green at this RED point because no consumer depends on their public map API.

- [ ] **Step 2: Install final concrete SliceOptions, remove the shell, and prove the allowlist is empty**

  Rename/expose the concrete partial builder result as public `SliceOptions`, deserialize canonical and reviewed legacy inputs directly into concrete optional fields, reject unknown keys, and delete `values()`, the transparent map, dynamic parsers, and conflicting unknown-preservation tests. Preserve the explicit STL API's already-supported native bool/number forms through the concrete visitors. Assert no production file in core/CLI/WASM contains an unapproved JSON value/map/raw value, `from_value`, `json!`, generic `ConfigValue`, erased `Any`, or DOM/runtime-type dispatch.

- [ ] **Step 3: Run the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core --test no_unapproved_dynamic_values
  git commit -m "refactor(config): complete typed value migration"
  git push
  ```

---

### Task 21A: Orca-Compatible Coordinates and Polygon Domain Types

**Upstream boundary:** Fixed-tag `libslic3r.h` scaling constants; `Point.hpp`; `Polyline.*`; `Polygon.*`; `ExPolygon.*`; `Geometry.*` ordering helpers.

**Files:**
- Create: `crates/ares-core/src/geometry.rs`
- Create: `geometry/coord.rs`
- Create: `geometry/polygon.rs`
- Create: `geometry/expolygon.rs`
- Create: `geometry/tests/{coord,polygon}.rs`
- Modify: `crates/ares-core/src/lib.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `type Coord = i64`, `Point`, `Polyline`, `Polygon`, `ExPolygon`, and deterministic non-clipping geometry methods.
- Produces scale/unscale functions matching the fixed upstream cast semantics and the fixture's `SCALING_FACTOR=0.000001`.

- [ ] **Step 1: Write RED scale/orientation/domain vectors**

  Transcribe focused vectors from fixed-tag `tests/libslic3r/test_{geometry,polygon}.cpp` and add boundary cases around positive and negative fractional scaled coordinates. At v2.4.2, integer `scaled<Tout>` casts `value / SCALING_FACTOR` rather than calling `std::round`; tests therefore prove truncation toward zero. Cover signed area, clockwise/counter-clockwise orientation, deterministic lexicographic point/path ordering, containment, bounding boxes, length, and full-point equality.

- [ ] **Step 2: Define zero-cost coordinate/domain types**

  ```rust
  pub type Coord = i64;
  pub const SCALING_FACTOR: f64 = 0.000_001;

  #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
  pub struct Point { pub x: Coord, pub y: Coord }

  pub fn scale(value_mm: f64) -> Coord {
      (value_mm / SCALING_FACTOR) as Coord
  }

  pub fn unscale(value: Coord) -> f64 {
      value as f64 * SCALING_FACTOR
  }
  ```

  Validate finite and `Coord`-representable project coordinates before calling `scale`; internal geometry trusts that invariant.

- [ ] **Step 3: Implement non-clipping Polygon/ExPolygon behavior**

  Port area, orientation, reverse, containment inputs, bounding boxes, duplicate/collinear cleanup primitives, and deterministic comparisons needed by the next tasks. Clipping and offsetting are not part of this review unit.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core geometry_coord
  cargo nextest run -p ares-core geometry_polygon
  git commit -m "feat(geometry): port scaled polygon domain"
  git push
  ```

---

### Task 21B: Clipper 6 Boolean Sweep and PolyTree

**Upstream boundary:** Fixed bundled Clipper 6 scanbeam/AEL/intersection/output/PolyTree sources and the boolean portions of `ClipperUtils.*`.

**Files:**
- Create: `geometry/clipper.rs`
- Create: `geometry/clipper/engine.rs`
- Create: `geometry/clipper/scanbeam.rs`
- Create: `geometry/clipper/intersections.rs`
- Create: `geometry/clipper/output.rs`
- Create: `geometry/clipper/polytree.rs`
- Create: `geometry/clipper/tests/{boolean,polytree}.rs`
- Modify: `geometry.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces integer union, difference, intersection, and XOR with PolyTree output and fixed-tag ordering.

- [ ] **Step 1: Write RED boolean/PolyTree vectors**

  Transcribe exact full-path cases from `tests/libslic3r/test_clipper_utils.cpp`, including holes, touching edges/vertices, coincident edges, fill rules, reverse solution, preserve-collinear, and deterministic sibling ordering.

- [ ] **Step 2: Port scanbeam, active-edge, intersection, and output construction**

  Preserve fixed Clipper 6 integer arithmetic, tie comparisons, maxima/minima handling, winding counts, intersection order, output joins, and PolyTree parent/hole semantics. Do not bind C++ and do not substitute Clipper2 behavior. Split files again if any reaches 400 LOC.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core geometry_clipper_boolean
  git commit -m "feat(geometry): port Clipper boolean engine"
  git push
  ```

---

### Task 21C: Clipper Offset, Simplification, and `ClipperUtils` Wrappers

**Upstream boundary:** Fixed bundled Clipper 6 offset engine; `ClipperUtils.*` offset/offset2/safety/simplify/opening/closing and contour-hole conversion.

**Files:**
- Create: `geometry/clipper/offset.rs`
- Create: `geometry/clipper/ordering.rs`
- Create: `geometry/polygon_ops.rs`
- Create: `geometry/tests/{offset,polygon_ops}.rs`
- Modify: `geometry/clipper.rs`, `geometry.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Completes the deterministic polygon kernel required by mesh/surface/perimeter/fill tasks.

- [ ] **Step 1: Write RED offset/wrapper vectors**

  Transcribe `tests/libslic3r/test_clipper_offset.cpp` and wrapper cases from `test_clipper_utils.cpp`: miter/round/square joins, open butt/square/round ends, positive/negative offset, offset2, simplify, opening/closing, safety offset, and contour/hole reconstruction. Expected values are complete ordered paths.

- [ ] **Step 2: Port offset and wrapper semantics**

  Preserve miter limit 3, arc tolerance, shortest-edge decimation factor, safety-offset order, end types, orientation, and output ordering. Build wrappers over Task 21B rather than adding a second engine.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core geometry_clipper_offset
  cargo nextest run -p ares-core geometry_polygon_ops
  git commit -m "feat(geometry): port Clipper offset wrappers"
  git push
  ```

---

### Task 22: Layer Planning, Mesh-Plane Intersections, and Slice Chaining

**Upstream boundary:** `Slicing.hpp/cpp::SlicingParameters`; layer-profile generation; `TriangleMeshSlicer.*::{slice_facet_at_zs,make_loops,make_expolygons,slice_mesh_ex}`; `PrintObjectSlice.cpp::{slice_volume,slice_volumes_inner,PrintObject::slice}` only through raw object/volume slices. Region assignment begins in Task 23.

**Files:**
- Create: `crates/ares-core/src/layer.rs`
- Create: `crates/ares-core/src/mesh_slicer.rs`
- Create: `mesh_slicer/intersection.rs`
- Create: `mesh_slicer/chaining.rs`
- Create: `mesh_slicer/repair.rs`
- Create: `mesh_slicer/tests/{intersection,chaining,fixture_layers}.rs`
- Create: `crates/ares-core/src/print_object.rs`
- Create: `print_object/slice.rs`
- Modify: `project_slice.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `Layer { id, height, print_z, slice_z }`, `LayerSlice`, and `slice_project_mesh(&ProjectObject, &ObjectOptions) -> Result<Vec<LayerSlice>, SliceError>`.
- Project slicing reaches this path directly and never calls `planning.rs`, `segments.rs`, or `contours.rs`.

- [ ] **Step 1: Write RED degeneracy and fixture-layer tests**

  Cover facets crossing a plane, a vertex exactly on a plane, an edge in a plane, horizontal facets, duplicate/reversed segments, open-chain repair, multiple contours, and holes with fixed expected point sequences derived from `tests/fff_print/test_trianglemesh.cpp` and the cited slicer functions. For the fixture assert only independently known facts: 460 planned layers, first print height/print Z 0.2, final print Z 92.0, strictly increasing upstream slice Z values, no empty required layer, and deterministic repeated output.

- [ ] **Step 2: Port layer parameter construction**

  Construct slicing parameters only from typed `ObjectOptions`, `RegionOptions`, and project bounds: regular mode, first height 0.2, layer height 0.2, min/max layer constraints, object height, raft/support offsets where active. Reproduce fixed-tag epsilon and final-layer handling; do not derive 460 from the fixture filename or expected output.

- [ ] **Step 3: Port facet intersection and loop construction**

  Transform project vertices in `f64`, intersect triangles at all slice planes using the upstream edge/vertex ownership rules, scale only the resulting XY points, order segments with the same indexed chaining strategy, repair only the upstream-supported gaps, and build oriented `ExPolygon` slices through the Task 21C kernel.

- [ ] **Step 4: Replace the project-path incomplete stage with typed slices**

  Add an internal `ProjectSliceState` carrying the loaded project, full config, and object layer slices. It may still end with a later-stage `ProjectSlicingIncomplete` error, but diagnostics expose 460 real layers for focused tests and no approximate G-code is returned.

- [ ] **Step 5: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core mesh_slicer
  cargo nextest run -p ares-core fixture_layers
  git commit -m "feat(slicing): port project mesh layer slicing"
  git push
  ```

---

### Task 23: Slice Regions, Flow, and Pre-Perimeter Elephant-Foot Compensation

**Upstream boundary:** `PrintObjectSlice.cpp` reached slice/region construction and EFC calls at the fixed tag, including the path around lines 1252-1292 and 1343-1365; `Flow.*`; `Layer::make_slices`. The unused `LayerRegion::elephant_foot_compensation_step` definition is explicitly not the port boundary.

**Files:**
- Create: `crates/ares-core/src/flow.rs`
- Create: `print_object/regions.rs`
- Create: `print_object/elephant_foot.rs`
- Create: `print_object/tests/{flow,elephant_foot,fixture_regions}.rs`
- Modify: `print_object.rs`, `print_object/slice.rs`, `project_slice.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces per-layer/per-region raw slice geometry and upstream-compatible `Flow` values before any perimeter exists.
- Applies reached EFC to the raw first-layer region slices that Task 24A consumes; it does not classify top/bottom/internal fill surfaces.

- [ ] **Step 1: Write RED region/Flow/EFC vectors**

  Use explicit multi-volume region-assignment polygons, fixed-tag `Flow` formula substitutions, and complete expected contours transcribed from `test_elephant_foot_compensation.cpp`. Cover holes, density, compensation layers, clipping order, and zero compensation. The fixture test asserts typed selection of `elefant_foot_compensation=0.15`, the independently known first-layer boundary invariants, and repeated determinism only.

- [ ] **Step 2: Build raw layer regions and reached Flow values**

  Convert Task 22 layer slices into per-region slices in the exact fixed-tag ownership/order. Compute external-perimeter Flow from nozzle, first/regular layer height, line width, and flow ratio now because upstream EFC needs that Flow before perimeter generation. Each consumed option receives a focused change-observation test.

- [ ] **Step 3: Apply EFC at the reached slice-stage call site**

  Apply compensation to configured first layers before Task 24A, preserving contour/hole orientation and upstream clipping/offset order. Prove changing every active EFC option changes only the expected raw region layers. Do not call or port the dead `LayerRegion::elephant_foot_compensation_step` as the runtime path.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core print_object_flow
  cargo nextest run -p ares-core print_object_elephant_foot
  cargo nextest run -p ares-core fixture_regions
  git commit -m "feat(slicing): port pre-perimeter flow and EFC"
  git push
  ```

---

### Task 24A: Classic Perimeters and Gap Policy

**Upstream boundary:** `LayerRegion.cpp::make_perimeters`; `PerimeterGenerator.*::process_classic`; `Flow.*`; `ExtrusionEntity.*`.

**Files:**
- Split/replace: `crates/ares-core/src/perimeters.rs`
- Create: `perimeters/classic.rs`
- Create: `perimeters/classic/{loops,thin_walls,gaps}.rs`
- Create: `perimeters/tests/classic_fixture.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces upstream-compatible `PerimeterPath`/`PerimeterLoop` results with role/Flow metadata plus the `fill_surfaces` consumed by Task 25A; Task 26A wraps these paths in final extrusion-entity variants.
- The project path no longer calls rectangular/whole-layer perimeter or gap proxies.

- [ ] **Step 1: Write RED classic-wall and gap tests**

  Cover two wall loops, classic generator selection, inner/outer spacing, widths, wall order, thin-wall disabled, gap target nowhere, small perimeter classification, overhang flags, and the resulting fill-surface remainder on synthetic polygons with explicit expected sequences. Perimeter generation neither selects nor rotates to a seam; byte evidence waits for the writer/filter pipeline.

- [ ] **Step 2: Port flow and classic-loop generation**

  Consume Task 23 `Flow`, offset contours in the same sequence as `process_classic`, create loop roles and overhang splits, and produce the fill-surface remainder at the exact upstream stage. Do not apply seam placement or arc simplification here. Every option read gets a focused change-observation test and ledger entry.

- [ ] **Step 3: Port reached gap behavior**

  Respect `gap_fill_target=nowhere` without producing gap-fill entities and test the active/inactive gap policy independently of brim generation.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core classic_perimeters
  git commit -m "feat(slicing): port classic wall generation"
  git push
  ```

---

### Task 24B: Auto Brim Geometry and Ordering

**Upstream boundary:** `Brim.*::make_brim`, brim flow/ordering helpers, and `ElephantFootCompensation.*` interactions.

**Files:**
- Modify: `crates/ares-core/src/brims.rs`
- Create: `brims/tests/project_auto_brim.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces first-layer `BrimPath` results with Flow/role metadata and removes the project path's rectangular brim proxy; Task 26A owns final extrusion-entity wrapping.

- [ ] **Step 1: Write RED brim vectors**

  Cover first-layer auto brim width 5, object gap 0.1, flow, combine flag, ears thresholds, EFC outline interaction, multiple islands, holes, and deterministic loop order with explicit synthetic expected paths.

- [ ] **Step 2: Port fixture-reached auto-brim classification and loops**

  Use Task 21C polygon operations, Task 23 Flow, and Task 24A path conventions. Prove inactive support/prime-tower brims create no path because typed config disables them, not because of a fixture branch.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core project_auto_brim
  git commit -m "feat(slicing): port project auto brim"
  git push
  ```

---

### Task 25A: Post-Perimeter Surface Typing and Shell Preparation

**Upstream boundary:** `Surface.*`; `LayerRegion.cpp::{slices_to_fill_surfaces_clipped,process_external_surfaces,prepare_fill_surfaces}`; `PrintObject.cpp::{detect_surfaces_type,prepare_infill,process_external_surfaces}` reached after `LayerRegion::make_perimeters` has populated `fill_surfaces`.

**Files:**
- Replace: `crates/ares-core/src/surface.rs`
- Create: `print_object/surfaces.rs`
- Create: `print_object/tests/surfaces.rs`
- Modify: `print_object.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Consumes Task 24A `fill_surfaces` and produces upstream-compatible `Surface`, `SurfaceType`, and prepared per-layer/per-region fill surfaces.
- Does not re-run raw slicing, EFC, or perimeter generation.

- [ ] **Step 1: Write RED post-perimeter classification microtests**

  Feed explicit perimeter-produced fill-surface stacks, not raw object contours. Assert bottom, bottom-bridge, internal, internal-solid, internal-bridge, top, and void classification, including top five/bottom three shell enforcement, thickness rules, clipped external surfaces, area conservation, and orientation. Expected polygons come from fixed source formulas/hand calculation before implementation; the fixture test checks typed option selection and invariants only.

- [ ] **Step 2: Port reached surface detection and shell logic in upstream order**

  Start from Task 24A's fill-surface remainder, then run fixed-tag detection, external-surface processing, and shell preparation exactly as `prepare_infill` does. Add a focused behavioral test before each option read: shell layer counts/thickness, ensure-vertical-shell policy, bridge-no-support, narrow-solid detection, top/bottom density and pattern selection, and related active fixture fields.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core print_object_surfaces
  git commit -m "feat(slicing): port post-perimeter surface preparation"
  git push
  ```

---

### Task 25B: Cross-Hatch, Monotonic, and Monotonic-Line Fill

**Upstream boundary:** `Fill/FillBase.cpp`; `Fill/Fill.cpp`; `Fill/FillCrossHatch.*::_fill_surface_single`; `Fill/FillRectilinear.*` monotonic algorithms; fill connection/order helpers.

**Files:**
- Replace/split: `crates/ares-core/src/infills.rs` into `crates/ares-core/src/fill.rs`
- Create: `fill/cross_hatch.rs`
- Create: `fill/monotonic.rs`
- Create: `fill/rectilinear.rs`
- Create: `fill/connect.rs`
- Create: `fill/tests/{cross_hatch,monotonic,fixture_fill}.rs`
- Modify: `lib.rs`, `print_object.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces pattern-local `FillPath` results with role/Flow metadata for sparse, internal solid, top, and bottom surfaces; Task 26A wraps them as extrusion entities and Task 27B performs current-position-dependent chaining at emission.
- Project slicing no longer calls the old whole-layer/rectangular infill proxy.

- [ ] **Step 1: Write RED pattern tests**

  Cover sparse cross-hatch at the fixture's 15% and configured angle/rotation, bottom/internal solid monotonic, and top monotonic-line. Assert spacing, clipping, line connection, alternating direction, monotonic ordering, extrusion role, and explicit point sequences on synthetic polygons derived from fixed-tag fill tests/formulas. The fixture test proves pattern selection and nonempty role production without treating final G-code as a pre-order fill oracle.

- [ ] **Step 2: Port fill-base setup and direction calculation**

  Build fill objects from concrete `RegionOptions`, compute density/spacing/overlap/anchor/direction using fixed-tag formulas, group compatible surfaces, and clip lines with the Task 21C kernel. Add one behavioral case per consumed fill option before reading it in production.

- [ ] **Step 3: Port the three active algorithms and connection order**

  Implement cross-hatch, monotonic, monotonic-line, and their rectilinear base exactly for reached branches. Inactive support, ironing, fuzzy, prime-tower, and unrelated fill algorithms return no entity only because typed effective options disable them; do not add a fallback pattern.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core fill_cross_hatch
  cargo nextest run -p ares-core fill_monotonic
  cargo nextest run -p ares-core fixture_fill
  git commit -m "feat(slicing): port fixture fill patterns"
  git push
  ```

---

### Task 26A: Print Domain and Extrusion Entity Structures

**Upstream boundary:** `Print.hpp/cpp::{Print,PrintObject,PrintRegion,process}` and `ExtrusionEntity.*` ownership/data variants, excluding later simplification, runtime chaining, and seam placement.

**Files:**
- Replace project semantics in: `crates/ares-core/src/print.rs`
- Replace project semantics in: `crates/ares-core/src/extrusion_entity.rs`
- Create: `extrusion_entity/tests/domain.rs`
- Modify: `project_slice.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `Print`, `PrintObject`, `PrintRegion`, and role-tagged path/loop/multipath collections with concrete geometry/flow metadata.
- Project slicing ceases to orchestrate through `pipeline.rs`, `print_paths.rs`, and `moves.rs`; entities are not yet arc-simplified or dynamically chained.

- [ ] **Step 1: Write RED ownership/entity tests**

  Cover object/region/layer ownership, collection flattening, path/loop/multipath variants, reversible flags, role, width, height, mm3-per-mm, overhang degree, object labels, and lossless movement between collections. These tagged unions model geometry, never option values.

- [ ] **Step 2: Port print ownership and concrete entities**

  Construct the entity graph from Tasks 24A/24B/25B in the fixed processing order, preserving insertion order and metadata. Do not choose loop starts, reverse a path for current nozzle proximity, place a seam, or arc-fit in this task.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core extrusion_entity_domain
  git commit -m "feat(print): port print and extrusion entity domain"
  git push
  ```

---

### Task 26B: Pre-Export Arc Fitting and Entity Simplification

**Upstream boundary:** `Print.cpp:2567-2572::simplify_extrusion_path`; `PrintObject.cpp:916-951`; `LayerRegion.cpp:1071-1126`; `ArcFitter.*::{do_arc_fitting,do_arc_fitting_and_simplify}`; `Polyline::simplify_by_fitting_arc`; `ExtrusionEntity::simplify_by_fitting_arc`.

**Files:**
- Create: `extrusion_entity/arc_fitting.rs`
- Create: `extrusion_entity/tests/{arc_fitting,simplification}.rs`
- Modify: `extrusion_entity.rs`, `print.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Populates each reached wall/fill entity's fixed-tag simplification/fitting result before any export-time seam or current-position decision.
- Rejects same-start/end full circles as the fixed fitter does; no P/multi-turn writer behavior is invented.

- [ ] **Step 1: Write RED fitting/simplification vectors**

  Fix explicit collinear and circular point sets, tolerances, expected line/arc partitions, direction, centers, and rejection fallbacks before production code. Expected values use source-cited `ArcFitter`/`Circle` numeric substitutions recorded in the test; the upstream suite has no dedicated vector file. Include the same-start/end rejection and tolerance option change-observation cases.

- [ ] **Step 2: Port simplification at the upstream process stage**

  Walk perimeters and infills in the same fixed-tag `simplify_extrusion_path` order after all entities exist and before export/SeamPlacer initialization. Preserve original points alongside fitting results wherever upstream needs them for later seam split/reverse maintenance.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core extrusion_arc_fitting
  cargo nextest run -p ares-core extrusion_simplification
  git commit -m "feat(print): port pre-export arc simplification"
  git push
  ```

---

### Task 26C: Static Layer Grouping and Seam Candidate Initialization

**Upstream boundary:** `GCode/ToolOrdering.*`; `GCode/SeamPlacer.*` initialization/candidate cache only; `GCode.cpp::{collect_layers_to_print,sort_print_object_instances}` and static object/island/region/role grouping before emission.

**Files:**
- Create: `crates/ares-core/src/print_order.rs`
- Create: `print_order/{islands,entities,seam_candidates}.rs`
- Create: `print_order/tests/{entities,seam_candidates,fixture_order}.rs`
- Modify: `project_slice.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces a statically grouped `LayerEmissionPlan` and initialized seam candidate/cache data.
- Does not claim a fully ordered path stream: nearest chaining, reverse decisions, loop rotation, seam split, and seam-gap clipping require the evolving writer `last_pos` and belong to Task 27B.

- [ ] **Step 1: Write RED static grouping/candidate tests**

  Assert object/island/region/role grouping, insertion-order tie behavior, layer order, brim-before-object, perimeter/fill group sequence, object-label boundaries, and aligned seam candidate/cache construction with explicit synthetic entities. Do not test a final seam point or nearest-neighbor route without a writer position. Fixture assertions are structural/deterministic only.

- [ ] **Step 2: Port static grouping and SeamPlacer initialization**

  Initialize fixed-tag candidate data after Task 26B fitting, then build static layer groups. Preserve upstream insertion order rather than sorting for convenience. Record focused tests/ledger rows for options that affect static grouping or candidate generation; defer every current-position-dependent decision.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core print_order
  cargo nextest run -p ares-core fixture_order
  git commit -m "feat(print): port static emission grouping"
  git push
  ```

---

### Task 27A: G-code Writer Numeric Formatting and Command State

**Upstream boundary:** `GCodeWriter.*` command state/formatting and `GCode.cpp::process_layer` state transitions.

**Files:**
- Split/replace project semantics in: `crates/ares-core/src/gcode_writer.rs`
- Replace/create: `gcode_writer/formatting.rs`
- Replace: `gcode_writer/acceleration.rs`
- Create: `gcode_writer/state.rs`
- Create: `gcode_writer/tests/{formatting,state}.rs`
- Modify: `project_slice.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces the stateful relative-E Marlin command core and allocation-light numeric formatters.

- [ ] **Step 1: Write RED numeric/state transition tests**

  Transcribe only the fixed `test_gcodewriter.cpp` cases that actually exist (set-speed formatting and z-hop). Before production code, add source-cited hand-calculated cases for coordinate/E precision, omitted leading zeros, negative-zero suppression, unchanged-state omission, G90/M83, role/layer markers, acceleration/jerk/feed changes, tool selection, and raw-command/blank-line formatting; each comment records the exact fixed writer function, inputs, formatting rule, and expected bytes.

- [ ] **Step 2: Port writer formatting and command state**

  Implement upstream numeric formatting as dedicated allocation-light formatters rather than Rust default `Display`. Apply typed machine/filament/region options to writer state and emit only the same transitions as fixed-tag `GCodeWriter`.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core gcode_writer_formatting
  cargo nextest run -p ares-core gcode_writer_state
  git commit -m "feat(gcode): port writer formatting state"
  git push
  ```

---

### Task 27B: Dynamic Chaining, Seam, and Linear Motion

**Upstream boundary:** `ShortestPath.*`; `GCode.cpp::{extrude_loop,_extrude,travel_to,retract}` including the fixed-tag seam placement/gap path around lines 5766-5805 and dynamic infill chaining around 6149-6172; `GCode/SeamPlacer::place_seam`; fixed-tag `GCodeWriter` extrusion/travel/retraction/fan/temperature helpers.

**Files:**
- Replace: `gcode_writer/travel.rs`
- Replace: `gcode_writer/retraction.rs`
- Create: `gcode_writer/extrusion.rs`
- Create: `gcode_writer/layer_result.rs`
- Create: `gcode_writer/chaining.rs`
- Create: `gcode_writer/seam.rs`
- Create: `gcode_writer/fan.rs`
- Create: `gcode_writer/temperature.rs`
- Create: `gcode_writer/tests/{chaining,seam,extrusion,retraction,fan,linear_motion}.rs`
- Modify: `gcode_writer.rs`, `project_slice.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Consumes Task 26C static groups, makes nearest/reverse/seam decisions from the evolving writer `last_pos`, and returns linear executable `LayerResult` bytes plus statistics events; Task 26B fitted arcs use their line fallback until Task 27C emission.
- `LayerResult` carries `gcode`, `layer_id`, `spiral_vase_enable`, and `cooling_buffer_flush`; Task 30B may additionally create the fixed NOP flush sentinel, but no task flattens this boundary before filters complete.

- [ ] **Step 1: Write RED motion tests**

  Cover nearest chaining and tie breaks as `last_pos` changes, reverse-allowed paths, loop start rotation, aligned seam selection, seam split, active fixture `seam_gap=10%` clipping, and maintenance of Task 26B fitting metadata after split/reverse. Also cover extrusion calculation, volumetric cap, retract/unretract, wipe, spiral lift, travel Z restore, acceleration restoration, fan/temperature changes, object labels, and fixture retract length 0.4. Add one change-observation test per consumed option, including `seam_gap`.

- [ ] **Step 2: Port linear extrusion and motion state**

  Traverse each static group using fixed-tag current-position chaining at the moment of emission. Call `place_seam` with the current writer position, rotate/split the loop, apply seam-gap clipping, and preserve/update fitting results exactly before travel/extrusion. Preserve retract gates, restart extra, wipe distance/fraction, lift mode, role/fan timing, temperature transitions, and writer-state omission rules. Emit fitted curves through their original line fallback in this task so Task 27C has an explicit RED delta.

- [ ] **Step 3: Prove complete linear writer behavior and run the mandatory task gate**

  Reuse only the verified set-speed/z-hop cases from fixed `test_gcodewriter.cpp` where they intersect this stage. Fix every `_extrude`, chaining, seam, travel, retraction, fan, and temperature expected byte string by source-cited hand calculation before production code, recording inputs and formula substitutions in the test. The fixture has G2/G3 in every one of its 460 final layer blocks and active cooling after the writer, so no final fixture layer is a valid linear-only raw-writer oracle. Fixture assertions here are limited to typed option selection, successful grouped-stream consumption, and repeated-run determinism.

  ```powershell
  cargo nextest run -p ares-core gcode_writer_motion
  cargo nextest run -p ares-core gcode_writer_linear_motion
  git commit -m "feat(gcode): port linear project motion"
  git push
  ```

---

### Task 27C: G2/G3 Emission from Precomputed Fits

**Upstream boundary:** `GCode.cpp::_extrude` consumption of already-populated fitting results and `GCodeWriter::extrude_arc_to_xy`. Fitting itself belongs to Task 26B; `Geometry/ArcWelder.*` is only the later processor's input-arc reader.

**Files:**
- Create: `gcode_writer/arc.rs`
- Create: `gcode_writer/tests/{arc,arc_integration}.rs`
- Modify: `extrusion_entity.rs`, `gcode_writer.rs`, `gcode_writer/extrusion.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Completes raw writer output for Task 26B fitted curves after Task 27B seam/chaining transforms; final fixture bytes remain intentionally incomparable until downstream filters and processor are applied.

- [ ] **Step 1: Write RED arc fitting/emission vectors**

  Feed explicit precomputed line/clockwise/counter-clockwise fitting segments and assert exact X/Y/I/J/E/feed bytes plus line fallback. Fixed `extrude_arc_to_xy` emits no P value and no multi-turn/full-circle command; same-start/end fits were already rejected in Task 26B. Hand-calculate every expected byte string from the cited writer formula before production code.

- [ ] **Step 2: Emit preserved fitting results at the upstream writer stage**

  Consume the fitting results populated by Task 26B and maintained through Task 27B split/reverse operations, then emit through `extrude_arc_to_xy`. Never fit already formatted G-code and never synthesize P/multi-turn semantics. Preserve original-line fallback only where the earlier fitter rejected a candidate.

- [ ] **Step 3: Integrate fitting with entities and the writer**

  Feed the same explicit Task 26B fitted entities through Task 27B seam/chaining transforms and writer emission, then compare complete hand-derived byte strings. For the fixture assert only that configured fitting results reach emission, emitted G2/G3 fields satisfy the writer's typed invariants, and repeated runs are identical; do not copy arc counts, lines, or blocks out of the final post-processed reference as a raw-writer oracle.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core gcode_writer_arc
  cargo nextest run -p ares-core gcode_writer_arc_integration
  git commit -m "feat(gcode): port arc fitting emission"
  git push
  ```

---

### Task 28: Typed Placeholder Lexer, Parser, and Evaluator

**Upstream boundary:** `PlaceholderParser.*::{process,evaluate_boolean_expression,ContextData}` and the fixed-tag variable/function semantics reached by the fixture templates.

**Files:**
- Create: `crates/ares-core/src/placeholder_parser.rs`
- Create: `placeholder_parser/lexer.rs`
- Create: `placeholder_parser/ast.rs`
- Create: `placeholder_parser/context.rs`
- Create: `placeholder_parser/evaluate.rs`
- Create: `placeholder_parser/tests/{expressions,fixture_templates}.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `Template`, typed expression AST, exhaustive `PlaceholderKey`, and typed `PlaceholderContext` built from `FullPrintConfig`, layer state, tool state, and statistics.
- Expression scalar variants model the actual upstream expression-language union; they are not serde option storage and cannot choose an Option type.

- [ ] **Step 1: Write RED syntax and fixture-template tests**

  Cover `[name]`, `{expression}`, indexing, arithmetic, comparison, boolean operators, right-associative `condition ? then : else`, `if`/`elsif`/`else`/`endif`, `min`, `max`, `ceil`, string equality/escaping, typed vectors, whitespace preservation, comments, unknown variables, out-of-range indices, and division errors. Prove ternary evaluation is lazy by placing an error in each non-selected branch. Parse and render the fixture machine-start (including its four lines/eight `?` tokens), filament-change, before/after-layer, timelapse, filament start/end, and machine-end templates with deterministic contexts and fixed expected results.

- [ ] **Step 2: Define a typed AST and exhaustive variable registry**

  ```rust
  enum Expr {
      Bool(bool), Number(f64), Text(Box<str>),
      Variable(PlaceholderKey), Index(Box<Expr>, Box<Expr>),
      Unary(UnaryOp, Box<Expr>),
      Binary(BinaryOp, Box<Expr>, Box<Expr>),
      Conditional { condition: Box<Expr>, then_expr: Box<Expr>, else_expr: Box<Expr> },
      Call(Function, Vec<Expr>),
  }
  ```

  `PlaceholderKey` has one explicit variant per supported fixed-tag variable and resolves through typed context methods. Unknown names fail parsing/evaluation; there is no string-keyed erased context map.

- [ ] **Step 3: Port fixed-tag evaluation and rendering order**

  Preserve fixed-tag precedence, right associativity, lazy ternary/branch evaluation, numeric/string formatting, truth rules, conditional nesting, substitution order, line retention, and error diagnostics. Add ledger entries for config options exposed to templates, with tests showing changed rendered bytes.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core placeholder_parser
  cargo nextest run -p ares-core fixture_templates
  git commit -m "feat(gcode): port typed placeholder evaluation"
  git push
  ```

---

### Task 29: Exact Header, Config, Custom-G-code, Executable, and Footer Block Assembly

**Upstream boundary:** `GCode.cpp::{do_export,_do_export,placeholder_parser_process,append_full_config,change_layer}`; `GCodeWriter.*`; `ConfigBase::opt_serialize`; generated/header/footer helpers.

**Files:**
- Split/replace project semantics in: `crates/ares-core/src/gcode.rs`
- Replace project behavior in: `gcode_header.rs`
- Replace project behavior in: `gcode_config_header.rs`
- Replace project behavior in: `gcode_start_custom.rs`
- Replace project behavior in: `gcode_layer_custom.rs`
- Replace project behavior in: `gcode_finish.rs`
- Create: `gcode/project_document.rs`
- Create: `gcode/tests/{blocks,custom_gcode,fixture_document}.rs`
- Modify: `project_slice.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `ProjectGCodeDocument { prelude, layers: Vec<LayerResult>, epilogue }`: typed header/config/executable-prefix bytes, unflattened writer layers, and end/statistics/footer placeholders.
- Prelude and epilogue bypass the layer filters but, like filtered layer output, are later streamed through `GCodeProcessor::process_buffer`; no task reconstructs layer boundaries from flat bytes.
- Existing `gcode::format_gcode` remains the STL formatter only.

- [ ] **Step 1: Write RED block-boundary and custom-G-code tests**

  Assert exact `HEADER_BLOCK`, `CONFIG_BLOCK`, and `EXECUTABLE_BLOCK` marker placement, blank lines, newline style, model label 133, max Z formatting, filament vectors, start/layer/timelapse/end template insertion, role comments, object start/stop labels, and end commands across the typed prelude/layer/epilogue boundaries. Compare the reference config block exactly because no processor rewrites it. For executable assembly, inject short synthetic `LayerResult`/template sequences with hand-fixed expected placement and assert fixture structure/determinism only; cooling, fan movement, Adaptive PA selection, progress, and finalization have not run, so no final-reference executable prefix/suffix or layer block is compared here.

- [ ] **Step 2: Assemble project blocks in fixed-tag export order**

  Render custom G-code only from typed embedded templates and context. Put header/config/executable start and startup bytes in `prelude`, attach before/after-layer and timelapse bytes at their fixed positions inside the corresponding Task 27C `LayerResult` without losing its metadata, retain every result as a separate ordered element, and put finish/executable-end/statistics placeholders in `epilogue`. Keep time/stat fields as typed document slots to be finalized by Tasks 30E-30F; do not concatenate the layers yet and do not use text search to guess unrelated values.

- [ ] **Step 3: Emit the exact Ares generator line**

  Format only from `GenerationMetadata` and compatibility version constant:

  ```text
  ; generated by Ares 2.4.2 on YYYY-MM-DD at HH:MM:SS
  ```

  Tests cover zero padding, deterministic timestamp, and exactly one generator line. No other metadata difference is permitted.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core project_gcode_blocks
  cargo nextest run -p ares-core project_custom_gcode
  git commit -m "feat(gcode): assemble exact project gcode blocks"
  git push
  ```

---

### Task 30A: GCodeProcessor Line Parser and Command State

**Upstream boundary:** `GCode/GCodeProcessor.*::{apply_config,process_buffer,process_gcode_line}` and `Geometry/ArcWelder.*` only for reading/discretizing existing arcs.

**Files:**
- Create: `crates/ares-core/src/gcode_processor.rs`
- Create: `gcode_processor/parser.rs`
- Create: `gcode_processor/state.rs`
- Create: `gcode_processor/tests/{parser,state}.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces typed parsed commands/events and persistent processor machine/tool/position state across repeated `process_buffer` calls while preserving original line bytes/chunk order.

- [ ] **Step 1: Write RED parser/state vectors**

  The fixed `test_gcode_timing.cpp` contains filament-change regressions, not parser vectors. Before production code, fix source-cited hand-calculated input/expected cases for comments/tags, G0/G1/G2/G3 with R-or-IJ arcs, absolute/relative XYZ/E modes, omitted axes, feedrate, acceleration/jerk/limits, dwell, temperature/fan/tool/progress commands, malformed numeric tokens, CR/LF and chunk-split handling, and byte preservation. A raw P token is preserved but does not control arc turns; same-XY IJ input becomes the fixed processor's single-circle arc event for Task 30E to time. Every case records the cited parser branch and expected state/event.

- [ ] **Step 2: Port parser and state transitions**

  Parse bounded line slices without a generic map or second slicer path, retaining incomplete trailing lines across buffers. Apply typed config, preserve reserved layer/role/object events, and discretize input arcs only through the fixed `ArcWelder` R/IJ reader semantics; P remains uninterpreted original syntax.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core gcode_processor_parser
  git commit -m "feat(gcode): port processor command parser"
  git push
  ```

---

### Task 30B: SpiralVase and PressureEqualizer Filters

**Upstream boundary:** `GCode/SpiralVase.*`, `GCode/PressureEqualizer.*`, and their `GCode.cpp::process_layers` filter adapters.

**Files:**
- Create: `gcode_processor/filters.rs`
- Create: `gcode_processor/filters/spiral_vase.rs`
- Create: `gcode_processor/filters/pressure_equalizer.rs`
- Create: `gcode_processor/tests/{spiral_vase,pressure_equalizer}.rs`
- Modify: `gcode_processor.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces individually testable spiral-vase and pressure-equalizer streaming transforms.
- Consumes and returns `LayerResult`; PressureEqualizer owns its fixed one-layer latency and receives one typed NOP sentinel after the final real layer to flush its last output.

- [ ] **Step 1: Write RED fixed-filter vectors**

  The fixed upstream suite has no dedicated SpiralVase/PressureEqualizer test file. Before production code, commit short explicit input/expected buffers hand-calculated from the named fixed-tag `process_layer` interpolation, loop-clipping, slope/flow, one-layer buffering, and flush formulas; each test comment records the cited function and numeric substitution. Include disabled-option identity tests proving byte preservation. Never derive the expected buffer from Ares or the final fixture G-code.

- [ ] **Step 2: Port both independent filters**

  Each filter receives typed config and a `LayerResult`, owns only its upstream state, and emits the same byte stream/events. Implement and test the generator-side final NOP flush contract without serializing it as a real layer. Pipeline selection is not part of this task.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core processor_spiral_vase
  cargo nextest run -p ares-core processor_pressure_equalizer
  git commit -m "feat(gcode): port spiral and pressure filters"
  git push
  ```

---

### Task 30C: CoolingBuffer and FanMover Filters

**Upstream boundary:** `GCode/CoolingBuffer.*`, `GCode/FanMover.*`, and their fixed-tag `GCode.cpp` adapters.

**Files:**
- Create: `gcode_processor/filters/cooling.rs`
- Create: `gcode_processor/filters/fan_mover.rs`
- Create: `gcode_processor/tests/{cooling,fan_mover}.rs`
- Modify: `gcode_processor/filters.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces individually testable cooling and fan-mover streaming transforms.

- [ ] **Step 1: Write RED cooling/fan vectors**

  Cover layer-time slowdown, minimum speeds, role fan markers, fan kickstart/speedup, first-layer gates, overhang fan behavior, flush, and disabled identity with short input/expected buffers fixed before implementation from the cited `CoolingBuffer`/`FanMover` formulas. Each hand-calculated case records its numeric substitution because the fixed suite has no standalone filter oracle. Every consumed cooling/fan option gets its own change-observation case; final fixture output is not used to bless an isolated filter snapshot.

- [ ] **Step 2: Port both filters and exact buffering behavior**

  Preserve marker consumption, command placement, buffer flush, numeric formatting, and unchanged input bytes. Do not yet compose them with Task 30B.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core processor_cooling
  cargo nextest run -p ares-core processor_fan_mover
  git commit -m "feat(gcode): port cooling and fan filters"
  git push
  ```

---

### Task 30D: Adaptive Pressure Advance and Conditional Filter Pipeline

**Upstream boundary:** `GCode/AdaptivePAProcessor.*` and fixed-tag `GCode.cpp:3693-3752` pipeline composition.

**Files:**
- Create: `gcode_processor/filters/adaptive_pa.rs`
- Create: `gcode_processor/pipeline.rs`
- Create: `gcode_processor/tests/{adaptive_pa,pipeline_order}.rs`
- Modify: `gcode_processor.rs`, `gcode_processor/filters.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces the exact conditional `LayerResult` pipeline, not a single unconditional sequence. It accepts Task 29's unflattened layers and yields filtered layer byte chunks; it never receives or filters the document prelude/epilogue.

- [ ] **Step 1: Write RED adaptive-PA vectors and four branch-order tests**

  Assert the fixed upstream branches exactly:

  ```text
  spiral + pressure: spiral -> pressure -> cooling -> fan
  spiral only:       spiral -> cooling -> fan
  pressure only:     pressure -> cooling -> fan -> adaptive_pa
  neither:           cooling -> fan -> adaptive_pa
  ```

  Adaptive PA is absent from both spiral branches. Tests record filter calls as typed events. Adaptive-PA transformed-byte cases use explicit inputs and hand-calculated interpolation/prior-state results cited to the fixed processor functions; branch composition uses test-only deterministic spy filters, not snapshots from Ares or the final fixture.

- [ ] **Step 2: Port AdaptivePAProcessor and branch composition**

  Preserve calibration interpolation, prior-PA state, role handling, PressureEqualizer NOP flush semantics, and upstream placement after `FanMover` only in the two non-spiral branches. Compose existing filters by borrowing/moving `LayerResult` buffers as upstream does without whole-layer duplicate copies; return filtered chunks in layer order and leave prelude/epilogue untouched.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core processor_adaptive_pa
  cargo nextest run -p ares-core processor_pipeline_order
  git commit -m "feat(gcode): compose Orca processor filters"
  git push
  ```

---

### Task 30E: Motion Time Estimator and Progress Commands

**Upstream boundary:** `GCodeProcessor.*::{calculate_time,update_estimated_times_stats}` time-machine state, motion planners, synchronization, and progress insertion.

**Files:**
- Create: `gcode_processor/time_estimator.rs`
- Create: `gcode_processor/progress.rs`
- Create: `gcode_processor/tests/{time,progress}.rs`
- Modify: `gcode_processor.rs`, `gcode/project_document.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces normal/silent time modes, prepare/model/first-layer/total time, and exact progress commands.

- [ ] **Step 1: Write RED motion-time microvectors**

  Transcribe only the five filament-change regressions that actually exist in fixed `test_gcode_timing.cpp`. Before production code, add source-cited hand-calculated microvectors for trapezoid timing, axis limits, feed/acceleration/jerk changes, R/IJ arc length (including one same-XY `2π` circle), queue look-ahead/synchronization, dwell, tool/temperature commands, and progress rounding. Each case records inputs, the exact fixed estimator formula/order, numeric substitution, and expected duration/bytes; no Ares snapshot becomes an oracle.

- [ ] **Step 2: Port the estimator and progress insertion**

  Preserve machine block order, floating-point accumulation, mode enabling, prepare-time subtraction, formatting, and known insertion boundaries. Change speed/acceleration/limit options individually and observe time changes.

- [ ] **Step 3: Match fixture time fields and run the mandatory task gate**

  Assert model `1h 43m 49s`, total `1h 48m 58s`, and first layer `5m 8s` before proceeding.

  ```powershell
  cargo nextest run -p ares-core processor_time
  cargo nextest run -p ares-core processor_progress
  git commit -m "feat(gcode): port time estimation and progress"
  git push
  ```

---

### Task 30F: Filament Statistics and Final Document Rewriting

**Upstream boundary:** `GCodeProcessor.*::{finalize,update_estimated_times_stats}`, filament/stat accumulation, generated header/footer rewriting.

**Files:**
- Create: `gcode_processor/statistics.rs`
- Create: `gcode_processor/finalize.rs`
- Create: `gcode_processor/tests/{statistics,fixture_processor}.rs`
- Modify: `gcode_processor.rs`, `project_slice.rs`, `gcode/project_document.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Completes `GCodeProcessorResult`, final processed bytes, filament statistics, and typed header/footer slot replacement. Project output always passes through this processor.

- [ ] **Step 1: Write RED statistics/finalization tests**

  Cover extrusion length/volume/mass/cost by filament, density/diameter/cost changes, unused filament zeros, exact rounding, header field replacement, footer placement, and unchanged-body preservation.

- [ ] **Step 2: Port statistics and known-boundary finalization**

  Accumulate through parsed events and rewrite only typed document slots/known tags. Never search arbitrary output text to guess statistics. Orchestrate the fixed boundary exactly: call `process_buffer(prelude)`, send only `layers` through Task 30D (including its internal pressure flush) and call `process_buffer` for each filtered chunk, then call `process_buffer(epilogue)` and finalize. Compare complete header/footer ranges to the reference; only the generator helper may normalize. `fixture_processor` performs the first valid complete fixture-body comparison, validates both generator lines, normalizes only them, and on mismatch panics solely with the bounded first-difference helper—never `assert_eq!` on full arrays. Task 31A separately proves the same bytes through the public one-way core API.

- [ ] **Step 3: Match fixture statistics and run the mandatory task gate**

  Assert `11335.74 mm`, `27.27 cm3`, `34.35 g`, and cost `0.86` for filament 1, with zero values for filament 2.

  ```powershell
  cargo nextest run -p ares-core processor_statistics
  cargo nextest run -p ares-core fixture_processor
  git commit -m "feat(gcode): finalize processor statistics"
  git push
  ```

---

### Task 31A: Complete Core Project Orchestration and Semantic Parity

**Upstream boundary:** `Format/bbs_3mf::load_bbs_3mf`, `Print::process`, `Print::export_gcode`, and `GCode::do_export` at the fixed commit.

**Files:**
- Complete: `crates/ares-core/src/project_slice.rs`
- Modify: `crates/ares-core/src/lib.rs`
- Modify: `crates/ares-core/Cargo.toml` (add workspace `sha2` and `regex` as dev-dependencies)
- Create: `crates/ares-core/tests/ksr_fdmtest_v4_core.rs`
- Create: `crates/ares-core/tests/support/golden.rs`
- Modify: `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- Modify: `docs/roadmap.md`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Completes `ares_core::slice_project(project_bytes, GenerationMetadata) -> Result<Vec<u8>, SliceError>` without any adapter or fallback.

- [ ] **Step 1: Write RED orchestration/boundary tests**

  A core integration test reads the committed fixture only in test code,
  supplies deterministic metadata, checks no production project module
  references `run_slicing_pipeline`, and compares semantic output with bounded
  diagnostics. The test-only parser validates exact per-layer deposited
  segment multisets, feature/width/extrusion/acceleration/fan state, lifecycle,
  controls, templates, and statistics. It permits only the measured
  nondeterministic envelopes: feed within 10 mm/min and 1%, time within five
  seconds, and displayed filament length within 0.05 mm. A source guard rejects
  reference-G-code reads, fixture names/hashes, Orca invocation/FFI,
  catch/retry, and legacy project fallback in production.

- [ ] **Step 2: Complete the one-way project orchestration**

  ```rust
  pub async fn slice_project(
      project: impl AsRef<[u8]>,
      metadata: GenerationMetadata,
  ) -> Result<Vec<u8>, SliceError> {
      let project = load_project(project)?;
      let config = FullPrintConfig::resolve(&project, project.settings())?;
      let print = Print::process_project(&project, &config)?;
      let document = ProjectGCodeDocument::generate(&print, &config, metadata)?;
      GCodeProcessor::new(&config).process(document)
  }
  ```

  There is no catch/retry/fallback branch. Remove the intermediate
  `ProjectSlicingIncomplete` error after all callers are migrated.

- [ ] **Step 3: Satisfy active core semantic equality**

  Validate exactly one Orca line and one Ares line and substitute the common
  sentinel. Normalize only the known indeterminate object-ID decimal field.
  Compare the semantic contract above and emit the first bounded layer/field
  mismatch. Keep the reference SHA only as fixture-integrity evidence.

  The browser test exercises the same project route and validates its complete
  output contract without requiring one TBB island schedule.

- [ ] **Step 4: Run full GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core --test ksr_fdmtest_v4_core
  cargo build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser test
  cargo nextest run --workspace
  git commit -m "feat(parity): complete core project slicing"
  git push
  ```

---

### Task 31B: CLI and WASM Project Adapters

**Upstream boundary:** Fixed-tag CLI project export dispatch; adapter-only metadata ownership around the completed core API.

**Files:**
- Modify: `crates/ares-cli/src/main.rs`
- Modify: `crates/ares-cli/Cargo.toml` (adapter-only `jiff = 0.2.32` local-time dependency)
- Modify: `crates/ares-cli/tests/cli.rs`
- Modify: `crates/ares-cli/tests/ksr_fdmtest_v4.rs` (remove ignore)
- Modify: `crates/ares-wasm/src/lib.rs`
- Modify: `crates/ares-wasm/tests/browser/project-slice.spec.mjs` only if adapter-level assertions need tightening; normalized browser parity must remain unchanged
- Modify: `docs/roadmap.md`

**Interfaces:**
- CLI: `ares slice -o OUTPUT PROJECT.3mf`; `--options` is rejected for 3MF and still required for STL.
- WASM: preserve the byte-oriented `sliceProject` from Task 4 plus a Rust-testable deterministic metadata helper and the browser parity established in Task 31A.

- [ ] **Step 1: Write RED adapter-boundary tests**

  CLI tests assert 3MF succeeds without `--options`, 3MF plus `--options` fails clearly, STL without options fails clearly, and STL with typed options retains existing behavior. WASM Rust tests call project bytes with deterministic metadata and compare native core bytes; the existing browser test continues to exercise JavaScript-owned local time.

- [ ] **Step 2: Implement strict extension dispatch and adapter-owned local time**

  CLI obtains local calendar fields through `jiff` and constructs `GenerationMetadata`; WASM obtains the same fields from `js_sys::Date`. Core tests always supply deterministic fields and core never reads a clock. Dispatch contains no catch/retry and no silent option override.

- [ ] **Step 3: Remove the CLI golden ignore and satisfy semantic equality**

  The active CLI test performs the same generator validation, semantic
  comparison, and bounded diagnostic as core while also proving the
  no-external-options command contract.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-cli --test ksr_fdmtest_v4
  cargo nextest run -p ares-wasm
  cargo build -p ares-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
  npm --prefix crates/ares-wasm/tests/browser test
  git commit -m "feat(adapters): expose project slicing without options"
  git push
  ```

---

### Task 31C: Original OrcaSlicer v2.4.2 Provenance

**Upstream boundary:** Fixed commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, its `build_release_vs2022.bat`, CLI project export, and generator identity.

**Files:**
- Create: `crates/ares-cli/tests/orca_v242_provenance.rs`
- Modify: `docs/roadmap.md`

**Interfaces:**
- Produces test-only evidence that the supplied reference is reproduced by a binary built from the exact fixed source commit.

- [ ] **Step 1: Build the exact OrcaSlicer commit in a detached source worktree**

  On the current Windows environment, create a never-before-used detached worktree, prove its tracked tree is exactly clean at the fixed commit, prove both build directories and the installed binary are absent, then build through the fixed commit's own VS2022 script. Reusing any prior worktree, object directory, install tree, or binary is forbidden; a version string alone is not provenance:

  ```powershell
  $commit = '8500fcdccaa10b5099ac20d252af3a7c560046f1'
  $orcaRepo = (Resolve-Path OrcaSlicer).Path
  $orcaWorktree = Join-Path $env:TEMP ("OrcaSlicer-v2.4.2-8500fcdc-{0}" -f [guid]::NewGuid().ToString('N'))
  if (Test-Path -LiteralPath $orcaWorktree) { throw 'unique Orca worktree path already exists' }
  git -C $orcaRepo worktree add --detach $orcaWorktree $commit
  if ($LASTEXITCODE -ne 0) { throw 'failed to create pinned Orca worktree' }
  if ((git -C $orcaWorktree rev-parse HEAD).Trim() -ne $commit) { throw 'wrong Orca commit' }
  $expectedTree = (git -C $orcaRepo rev-parse "$commit^{tree}").Trim()
  $actualTree = (git -C $orcaWorktree rev-parse 'HEAD^{tree}').Trim()
  if ($actualTree -ne $expectedTree) { throw 'Orca source tree id differs from fixed commit' }
  git -C $orcaWorktree diff --quiet --
  if ($LASTEXITCODE -ne 0) { throw 'Orca tracked worktree is dirty before build' }
  git -C $orcaWorktree diff --cached --quiet --
  if ($LASTEXITCODE -ne 0) { throw 'Orca index is dirty before build' }
  $orcaBinary = Join-Path $orcaWorktree 'build\OrcaSlicer\orca-slicer.exe'
  foreach ($freshPath in @((Join-Path $orcaWorktree 'build'), (Join-Path $orcaWorktree 'deps\build'), $orcaBinary)) {
      if (Test-Path -LiteralPath $freshPath) { throw "refusing pre-existing build artifact: $freshPath" }
  }
  $buildLog = Join-Path $orcaWorktree 'build-v242-provenance.log'
  $buildStarted = [DateTimeOffset]::UtcNow
  Push-Location $orcaWorktree
  try {
      cmd /c build_release_vs2022.bat 2>&1 | Tee-Object -LiteralPath $buildLog
      $buildExit = $LASTEXITCODE
      if ($buildExit -ne 0) { throw 'OrcaSlicer v2.4.2 build failed' }
  } finally {
      Pop-Location
  }
  if (-not (Test-Path $orcaBinary)) { throw "missing $orcaBinary" }
  git -C $orcaWorktree diff --quiet --
  if ($LASTEXITCODE -ne 0) { throw 'Orca build modified tracked source' }
  git -C $orcaWorktree diff --cached --quiet --
  if ($LASTEXITCODE -ne 0) { throw 'Orca build modified the index' }
  if ((Get-Item -LiteralPath $orcaBinary).LastWriteTimeUtc -lt $buildStarted.UtcDateTime) {
      throw 'Orca binary predates this clean build'
  }
  $buildLogSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $buildLog).Hash.ToLowerInvariant()
  $binarySha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $orcaBinary).Hash.ToLowerInvariant()
  $env:ORCA_SLICER_242_SOURCE = $orcaWorktree
  $env:ORCA_SLICER_242 = $orcaBinary
  $env:ORCA_SLICER_242_SOURCE_TREE = $actualTree
  $env:ORCA_SLICER_242_BUILD_LOG = $buildLog
  $env:ORCA_SLICER_242_BUILD_LOG_SHA256 = $buildLogSha256
  $env:ORCA_SLICER_242_BUILD_STARTED = $buildStarted.ToString('O')
  $env:ORCA_SLICER_242_BINARY_SHA256 = $binarySha256
  ```

  Record the unique source path, commit/tree IDs, tracked-clean checks, exact build command, build-log SHA-256, build start time, installed binary path/time, and binary SHA-256 in verification notes. Linux/macOS reruns use the same fresh-worktree/empty-build/clean-tree rule with the fixed commit's `build_linux.sh`/`build_release_macos.sh`; the Windows run is the required final local provenance evidence for this workspace.

- [ ] **Step 2: Re-run pinned Orca as a test-only provenance check**

  `orca_v242_provenance.rs` is ignored in ordinary CI and loads all seven environment variables above into a concrete test-only `ProvenanceEvidence` struct. It canonicalizes the source/log/executable paths, requires the log and executable to be under the unique source worktree, requires the executable at `<source>/build/OrcaSlicer/orca-slicer.exe`, re-runs both tracked/index clean checks, verifies HEAD and `HEAD^{tree}`, verifies the non-empty build log and its recorded hash, verifies binary mtime is not older than the recorded build start, recomputes the binary SHA-256, and only then checks that the binary reports 2.4.2 and runs:

  ```powershell
  & $env:ORCA_SLICER_242 --slice 1 --outputdir $temporaryDirectory tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf
  ```

  It discovers the single generated `.gcode` inside the temporary directory, validates the Orca generator line, compares it with the committed reference under the same one-line normalization through the bounded-only inequality path (no full-buffer `assert_eq!`), and re-runs the tracked/index clean checks after export. The executable is invoked only by this ignored test, never by production or the Ares golden.

- [ ] **Step 3: Run full GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-cli --test ksr_fdmtest_v4
  cargo nextest run -p ares-cli --test orca_v242_provenance --run-ignored all
  cargo nextest run --workspace
  git commit -m "test(parity): verify pinned Orca provenance"
  git push
  ```

---

### Task 32: Whole-Branch Acceptance Review and Completion Audit

**Upstream boundary:** Every fixed-tag boundary named by the approved spec and Tasks 1A-31C.

**Files:**
- Modify only files required by whole-branch review findings
- Modify: `docs/roadmap.md`
- Modify: `docs/architecture/option-parity-v4.md`
- Modify: `docs/architecture/ard-0023-3mf-project-gcode-parity.md` only if implementation evidence clarifies, but does not weaken, the decision

**Interfaces:**
- Produces final independently reviewed evidence that the entire branch, not only the last task, satisfies the spec.

- [ ] **Step 1: Run fresh whole-branch independent reviews**

  Give a new Codex reviewer and OpenCode's default model the complete diff from baseline commit `a0eec942f` through HEAD, the approved spec/ARD/plan, the fixed upstream source access command, and fresh verification logs. Require each to inspect production for fixture/reference reads, fixture hash/name branches, Orca invocation/FFI, legacy fallback, untyped values, incomplete option consumers, and mismatched upstream semantics. Both must return literal `VERDICT: APPROVE`; findings restart both reviews after fixes.

- [ ] **Step 2: Run the complete acceptance matrix from a clean status**

  Run all commands in the Mandatory Gate plus active golden, ignored original-Orca provenance, fixture hash checks, normalized hash check, `rg` guards for forbidden production references, and `git status --short`. Confirm all pushed Tier-1 jobs are green for the reviewed commit.

- [ ] **Step 3: Close the roadmap with exact evidence**

  Record the final Ares/reference normalized hash, layer/stat totals, original-Orca provenance result, empty dynamic-value baseline, no allowlist entries, native/WASM CI run URLs, and both whole-branch approval verdicts. Mark only the approved parity program complete; do not claim unrelated Orca behavior.

- [ ] **Step 4: Commit and push any final reviewed documentation/fixes**

  If review required code fixes, repeat focused/full verification before this commit. Then:

  ```powershell
  git commit -m "docs(parity): record final Orca 2.4.2 verification"
  git push
  ```

## Dependency and Parallelism Summary

The strict dependency chain is:

```text
1A -> 1B -> 1C -> 1D -> 1E -> 2 -> 3 -> 4 -> 5
5 -> 6 -> 7 -> 8
5 -> 9 -> 10 -> 11
5 -> 12 -> 13
5 -> 14
8 + 11 + 13 + 14 -> 15 -> 16 -> 17 -> 18 -> 19A -> 19B -> 19C
19C -> 20A -> 20B -> 20C -> 20D -> 20E -> 21A -> 21B -> 21C -> 22 -> 23 -> 24A -> 24B -> 25A -> 25B -> 26A -> 26B -> 26C -> 27A -> 27B -> 27C -> 28 -> 29 -> 30A
30A -> 30B -> 30C -> 30D -> 30E -> 30F -> 31A -> 31B -> 31C -> 32
```

After Task 5, upstream inspection and test-vector preparation for the four raw scopes may run concurrently, but their implementation/review/commit tasks are serialized because they share `options/project_settings.rs`, the option-parity document, and the branch/CI gate. Tasks 30B and 30C are also serialized because both integrate through the filter module and documentation. No concurrent implementation task may edit a shared file; independent reviewers and read-only source audits remain parallelizable.

The four lines branching from Task 5 describe knowledge/preparation
dependencies, not authorization for parallel implementation: the executed
order is Tasks 6 through 14 in plan order. Tasks 15 through 17 and 21A through
21C are also intentionally serialized. This conservative order keeps every
reviewed commit green and avoids shared option/config documentation edits even
where isolated source-reading work could otherwise overlap.

## Plan Self-Review Checklist

- [x] Re-read every approved-spec section and map it to at least one numbered task.
- [x] Confirm every named exact Ares destination appears in the file map or a task.
- [x] Confirm all 15 package entries are either concretely parsed or explicitly opaque preview PNGs.
- [x] Confirm all archive/path/XML/JSON rejection classes have a named test.
- [x] Confirm the 653 fields, raw-scope counts, effective projections, type histogram, nullable fields, legacy aliases, normalization, and export are covered.
- [x] Confirm every consumed option requires an observed behavior test and ledger row.
- [x] Confirm project geometry never calls the legacy STL approximation modules.
- [x] Confirm source-level pinning removal retains behavioral runtime code.
- [x] Confirm final output normalization changes only the one validated generator line.
- [x] Confirm every task contains TDD/characterization, dual review, docs, full verification, Conventional Commit, push, and CI-green gates.
- [x] Confirm the final whole-branch review is separate from per-task review.

## Execution Handoff

The user selected the mandatory SDD plus Subagent-Driven workflow in the goal. After this plan itself receives independent Codex and OpenCode `VERDICT: APPROVE`, commit and push the approved plan, then apply both `sdd-workflow` and `superpowers:subagent-driven-development`, with a fresh TDD implementation subagent for Task 1A and the complete mandatory gate after every task. Neither workflow may be replaced by `superpowers:executing-plans`.
