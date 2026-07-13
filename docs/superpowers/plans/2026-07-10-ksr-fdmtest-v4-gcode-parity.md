# KSR FDM Test V4 G-code Parity Implementation Plan

> **For agentic workers:** REQUIRED WORKFLOW SKILLS: use both `sdd-workflow` and `superpowers:subagent-driven-development` for every implementation task; every fresh implementation subagent also follows `superpowers:test-driven-development`. `superpowers:executing-plans` is not a substitute for either required workflow. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Ares slice the committed `ksr_fdmtest_v4.project.3mf` entirely from its embedded model and options into G-code that differs from the committed OrcaSlicer 2.4.2 reference only in the validated generator name/timestamp line.

**Architecture:** Add a byte-oriented project path beside the existing explicit-option STL API. The project path is a source-cited Rust rewrite of the fixed OrcaSlicer 2.4.2 `libslic3r` import, configuration, slicing, G-code, and post-processing boundaries; it owns typed 3MF/config data and never retries through the existing approximate STL pipeline. Work proceeds through independently useful, testable increments, with typed intermediate results used as the oracle boundary before final byte parity.

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
- Normalizing the one validated generator line to `; generated by <SLICER> 2.4.2 on <TIMESTAMP>` yields reference SHA-256 `c61202df3fa26ffcb3064f2dbc02e06a89f95565b8325b31029ec4ed6cedcdc4`.
- Intermediate RED tests use expected vectors fixed before implementation from pinned upstream tests (`tests/libslic3r/test_{clipper_offset,clipper_utils,geometry,polygon,elephant_foot_compensation,placeholder_parser}.cpp` and `tests/fff_print/test_{trianglemesh,fill,gcode,gcode_timing,gcodewriter}.cpp`) or from a small hand-calculated cited formula. They never bless an Ares-generated snapshot. Fixture-level pre-G-code assertions are limited to facts independently available from the 3MF/reference contract (mesh, transforms, typed config, layer count/Z, exact config block); internal contour/surface/pre-seam snapshots are not reconstructed from final G-code. The committed G-code is post-filter/post-processor output and is never used as a byte oracle for raw writer layers or the pre-processor executable document; fixture-body byte comparison begins only after Task 30F has applied every reached filter/finalizer, with public-API equality in Task 31A.
- No active test is weakened to make an increment green. The CLI golden remains explicitly ignored until Task 31B; progress evidence follows the stage-specific boundary/error/internal-diff/core-browser/CLI contract below and never claims bytes before they exist.
- Every comparison of complete/normalized G-code uses an explicit `if expected != actual { panic!(bounded_difference) }` path. `assert_eq!`/`assert_ne!` and other macros that Debug-print either multi-megabyte byte array are forbidden in Task 1B, 30F, 31A, 31B, and 31C.

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

**Upstream boundary:** Fixed-tag filament raw scope intersected with `PrintConfig.hpp::GCodeConfig`; filament temperature, cooling, retraction, material, and custom-G-code definitions in `PrintConfig.cpp`.

**Files:**
- Create: `options/filament_options.rs`
- Create: `options/filament_options/gcode_source.rs`
- Create: `options/tests/filament_gcode_source.rs`
- Modify: `options/project_settings.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `FilamentGCodeSourceOptions` with 53 concrete resolved vector fields plus its private typed builder.
- Starts `FilamentOptions` raw ownership.

- [ ] **Step 1: Add RED tests for 53 keys and per-filament cardinality**

  Cover two active filament values, original 8-entry source vectors, temperature vectors, cooling and fan fields, material density/diameter/cost, retraction, multi-line start/end G-code, enum labels, `nil`, empty strings, and wrong vector lengths.

- [ ] **Step 2: Implement typed vector fields**

  Deserialize each upstream option directly into `Vec<T>` or a dedicated typed structure. Preserve source cardinality in raw state; do not shrink to active values during deserialization. Record the stride/variant policy only as typed metadata for Task 19B normalization.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core filament_gcode_source
  git commit -m "feat(config): type filament gcode options"
  git push
  ```

---

### Task 13: Remaining Filament Raw Options and Nullable Overrides (69 Fields)

**Upstream boundary:** Fixed-tag filament scope intersected with `PrintConfig` (48), `PrintRegionConfig` (4), and unowned keys (17); `filament_extruder_override_keys` and `add_nullable` loop for 16 additional nullable override fields.

**Files:**
- Create: `options/filament_options/print_source.rs`
- Create: `options/filament_options/region_source.rs`
- Create: `options/filament_options/runtime.rs`
- Create: `options/filament_options/retract_overrides.rs`
- Create: `options/tests/filament_remaining.rs`
- Modify: `options/filament_options.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Completes `FilamentOptions` at exactly 122 raw keys while representing all 31 nullable fixture fields concretely.

- [ ] **Step 1: Add RED completeness and nullable tests**

  Assert 53 + 48 + 4 + 17 = 122 unique raw keys. Separately assert the 15 statically nullable fixture fields and 16 dynamically registered filament retract override fields have `Nullable<T>` element types. Cover 8-to-active stride metadata, adaptive pressure model lines, ramming data, volumetric coefficient tuples, and `nil` at individual vector positions.

- [ ] **Step 2: Implement all remaining fields and explicit retract overrides**

  Each of the 16 dynamically registered upstream names becomes an explicit Rust field and direct match arm; no prefix/suffix reflection chooses a type at runtime. Structured strings that downstream code parses receive dedicated concrete newtypes with serde and validation now.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core filament_options
  git commit -m "feat(config): complete typed filament project options"
  git push
  ```

---

### Task 14: Typed Project/Runtime Residual Options (47 Fields)

**Upstream boundary:** The fixed-tag fixture keys outside the three `Preset.cpp` raw scope lists; their individual `PrintConfig`/`GCodeConfig` owners and project metadata semantics.

**Files:**
- Create: `options/project_runtime_options.rs`
- Create: `options/preset_metadata.rs`
- Create: `options/tests/project_runtime_options.rs`
- Modify: `options/project_settings.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `ProjectRuntimeOptions` for 44 real options and `PresetMetadata { from, name, version }` for the three metadata strings.
- Completes raw ownership of all 653 keys when combined with Printer/Process/Filament.

- [ ] **Step 1: Add RED residual and global-union tests**

  Assert exactly 47 residual keys, including exactly three metadata fields, and prove the four raw scopes are pairwise disjoint with a 653-key union identical to the committed fixture. Assert all 650 real options map to the exact type histogram and all 653 values can be serialized without loss of string/array shape.

- [ ] **Step 2: Implement the 47 concrete fields**

  Metadata is retained for project provenance but excluded from the G-code config block exactly as upstream does. Every other field is placed under its source-cited semantic child within `ProjectRuntimeOptions`; no catch-all remainder exists.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core project_runtime_options
  git commit -m "feat(config): type project runtime options"
  git push
  ```

---

### Task 15: Effective Object Options (126-Field Projection)

**Upstream boundary:** `PrintObject.cpp::object_config_from_model_object`; `PrintApply.cpp::Print::apply`; `ModelObject` metadata overrides; `PrintObjectConfig` inheritance.

**Files:**
- Create: `options/object_options.rs`
- Create: `options/tests/object_options.rs`
- Modify: `project/model_settings.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces non-raw `ObjectOptions`, built from the 126 process object-source fields plus typed per-object overrides.
- Produces `ObjectOptions::resolve(process, object_settings) -> Result<Self, SliceError>`.

- [ ] **Step 1: Write RED override/projection tests**

  Use synthetic typed settings to prove missing object overrides inherit process values, explicit overrides replace only their field, invalid object-scope keys fail at model-settings parsing, and the fixture object resolves layer/shell/support/seam values used downstream. Each field remains concrete during override application.

- [ ] **Step 2: Implement typed projection and override dispatch**

  Model-setting metadata keys that name real options deserialize directly through an object-scope key visitor into the same concrete types; arbitrary metadata such as `name` and `matrix` remain named typed fields. Build `ObjectOptions` without serializing back through JSON and without a string-to-value intermediate.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core object_options
  git commit -m "feat(config): resolve effective object options"
  git push
  ```

---

### Task 16: Effective Region Options (153-Field Projection)

**Upstream boundary:** `PrintObject.cpp::region_config_from_model_volume`; `PrintApply.cpp`; `PrintRegionConfig` inheritance; four filament ironing overrides.

**Files:**
- Create: `options/region_options.rs`
- Create: `options/tests/region_options.rs`
- Modify: `project/model_settings.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `RegionOptions::resolve(process, filament, part_settings, active_filament) -> Result<Self, SliceError>` with 153 concrete effective fields.

- [ ] **Step 1: Write RED merge-precedence tests**

  Prove the 149 process region-source values plus four filament ironing overrides form exactly 153 fields; part settings override process values; selected filament overrides apply only where upstream permits; `nil` means inherit for nullable overrides; and a second filament selection changes the effective typed value without changing raw storage.

- [ ] **Step 2: Implement direct typed projection**

  Match upstream precedence in `region_config_from_model_volume` and `Print::apply`. Use explicit field assignments or generated concrete-field macros; do not loop over erased option values. Record a behavioral ledger row for each field whose merge affects the fixture.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core region_options
  git commit -m "feat(config): resolve effective region options"
  git push
  ```

---

### Task 17: Effective G-code Options (149-Field Projection)

**Upstream boundary:** `PrintConfig.hpp::GCodeConfig`; fixed-tag cross-scope ownership: Printer 62 + Process 17 + Filament 53 + Residual 17.

**Files:**
- Create: `options/gcode_options.rs`
- Create: `options/tests/gcode_options.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `GCodeOptions::resolve(printer, process, filament, project, active_filaments) -> Result<Self, SliceError>` with 149 concrete effective fields.

- [ ] **Step 1: Write RED cross-scope and template-string tests**

  Assert 62 + 17 + 53 + 17 = 149 unique effective fields. Verify machine/filament/layer/timelapse/end templates preserve exact newlines and escaping; active filament vector selection is deterministic; firmware/bed/temperature enums remain typed; invalid vector cardinality fails with the key name.

- [ ] **Step 2: Implement effective projection without raw lookup**

  Construct `GCodeOptions` only from typed source fields. Template strings are copied as strings; expression parsing belongs to Task 28. No runtime code accepts a key string to discover a value type.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core gcode_options
  git commit -m "feat(config): resolve effective gcode options"
  git push
  ```

---

### Task 18: Strict Top-Level ProjectSettings and a Bounded STL Compatibility Shell

**Upstream boundary:** `ConfigBase::set_deserialize*`, `DynamicConfig`, `PrintConfigDef` lookup and unknown-key behavior.

**Files:**
- Create: `options/project_deserialize.rs`
- Create: `options/tests/project_deserialize.rs`
- Modify: `options/project_settings.rs`, `options.rs`, `project/load.rs`, `lib.rs`
- Retain temporarily: the existing dynamic `SliceOptions` representation and its baseline-covered parser only for the explicit-option STL API

**Interfaces:**
- Produces concrete `ProjectSettings { printer, process, filament, project, metadata }`.
- Produces the concrete builder/patch field definitions that Task 20E will expose as the final partial `SliceOptions`; `ProjectSettings` resolves those builders to non-optional group fields now.
- Unknown project keys fail with the exact key after canonical dispatch in this task; Task 19A inserts reviewed legacy dispatch before that same branch. The old public `SliceOptions` map/`values()` shell remains baseline-covered and unchanged until all of its callers migrate in Tasks 20A-20D; project loading and project slicing may never call it.

- [ ] **Step 1: Write RED full-fixture/unknown/round-trip tests**

  Parse the committed `project_settings.config` directly into `ProjectSettings`; assert 653 dispatched fields, group counts, no remainder, representative typed values, and semantic round-trip through each field's declared scalar/array wire shape. Exact project-input lexical preservation is not required because Ares does not rewrite the 3MF; exact effective G-code config serialization is independently RED-tested in Task 19C. Assert unknown project keys, invalid types, duplicates, and invalid vector lexical forms fail with bounded key-specific diagnostics. Add a source/behavior guard proving `project/`, `project_slice.rs`, and `FullPrintConfig` do not name or call the temporary STL `SliceOptions::values()` shell.

- [ ] **Step 2: Implement the custom serde map visitor**

  ```rust
  while let Some(key) = map.next_key::<String>()? {
      let consumed = settings.printer.deserialize_known_field(&key, &mut map)?
          || settings.process.deserialize_known_field(&key, &mut map)?
          || settings.filament.deserialize_known_field(&key, &mut map)?
          || settings.project.deserialize_known_field(&key, &mut map)?
          || settings.metadata.deserialize_known_field(&key, &mut map)?;
      if !consumed {
          return Err(serde::de::Error::unknown_field(&key, ProjectSettings::FIELDS));
      }
  }
  ```

  The actual implementation tracks duplicates before assignment and leaves a typed dispatch point immediately before the unknown branch for Task 19A's reviewed legacy inputs; no untyped value is parked there. Serialization emits the canonical JSON wire shape for project persistence; config-block serialization is a separate Task 19C path.

- [ ] **Step 3: Wire strict project loading while preserving intermediate workspace compilation**

  `load_project` parses its embedded settings into `ProjectSettings` and stores that typed value in `Project`. Existing STL `slice(input, SliceOptions)` continues to compile and behave through the explicitly temporary, baseline-covered map shell; do not delete `values()`, its deserializer, or its existing unknown-preservation tests in this task because current `pipeline`, `profiles`, `print_apply`, and G-code callers still use them. The shell is not a project fallback and is deleted only in Task 20E after those callers are typed.

- [ ] **Step 4: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core project_deserialize
  cargo nextest run -p ares-core project_import
  git commit -m "feat(config): deserialize strict typed project settings"
  git push
  ```

---

### Task 19A: Legacy Key/Value and Composite Conversion

**Upstream boundary:** Fixed-tag `PrintConfigDef::{handle_legacy,handle_legacy_composite}`.

**Files:**
- Create: `options/tests/legacy.rs`
- Modify: `options/project_deserialize.rs`, `options/project_settings.rs`, `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Completes canonical/legacy typed dispatch before the strict unknown-key branch.
- Produces typed composite conversions without parking values in a dynamic container.

- [ ] **Step 1: Write RED cases for every fixed-tag legacy input targeting the 650 implemented options**

  The committed inventory records legacy aliases. Simple renames use serde aliases; complex names route directly into the canonical field's concrete deserializer. Cover obsolete ignored keys, enum spelling conversions, scalar conversions, collision/duplicate behavior, and unknown-after-legacy diagnostics.

- [ ] **Step 2: Port composite conversion and fixture idempotence**

  Apply `handle_legacy_composite` only after the complete typed document is loaded. Assert canonical fixture keys remain idempotent except defined composite normalization, including canonical `thumbnails` spacing.

- [ ] **Step 3: Run focused GREEN and the mandatory task gate**

  ```powershell
  cargo nextest run -p ares-core legacy_config
  git commit -m "feat(config): port legacy option conversion"
  git push
  ```

---

### Task 19B: Effective FullPrintConfig Resolution and FDM Normalization

**Upstream boundary:** Fixed-tag `DynamicPrintConfig::{normalize_fdm,normalize_fdm_1,normalize_fdm_2,set_num_extruders,set_num_filaments,get_parameter_size}` and `PrintApply.cpp` active object/filament sizing.

**Files:**
- Create: `options/full_print_config.rs`
- Create: `options/project_normalize.rs`
- Create: `options/tests/project_normalize.rs`
- Modify: `docs/architecture/option-parity-v4.md`

**Interfaces:**
- Produces `FullPrintConfig::resolve(&Project, &ProjectSettings) -> Result<Self, SliceError>`.
- Produces `normalize_project_config(&mut FullPrintConfig) -> Result<(), SliceError>`.

- [ ] **Step 1: Write RED active-sizing and normalization tests**

  Assert single-material `enable_prime_tower` normalizes from raw `1` to effective `0`; 8-/4-stride source vectors resolve to the two active values used by the reference; object/region overrides and active extruder/filament maps apply in fixed order; invalid cardinality names its key.

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

  Follow the interleaved fixed-tag `Print::apply` order rather than normalizing after projection: resolve typed defaults/base merge; run `normalize_fdm_1`; determine active extruder/filament sizing and run the first `normalize_fdm_2`; derive per-object `ObjectOptions` and per-volume/filament `RegionOptions` from that normalized state; run the second `normalize_fdm_2` pass after those merges; then derive the final G-code/export projection. Tests place a changed value in every stage and prove the following stage observes it, including infill relationships and the single-material prime-tower change. Missing required values are reported only at this external configuration boundary.

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

### Task 31A: Complete Core Project Orchestration and Core Byte Parity

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

  A core integration test reads the committed fixture only in test code, supplies deterministic metadata, checks no production project module references `run_slicing_pipeline`, and compares normalized output with bounded diagnostics. Its test-only helper owns the same complete-line regex contract and SHA-256 calculation as the CLI helper, backed by the explicit core dev-dependencies; production code cannot import it. A source guard rejects reference-G-code reads, fixture names/hashes, Orca invocation/FFI, catch/retry, and legacy project fallback in production.

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

  There is no catch/retry/fallback branch. Remove the intermediate `ProjectSlicingIncomplete` error after all callers are migrated.

- [ ] **Step 3: Satisfy active core normalized equality**

  Validate exactly one Orca line and one Ares line, substitute the common sentinel, compare every remaining byte, and assert normalized SHA-256 `c61202df3fa26ffcb3064f2dbc02e06a89f95565b8325b31029ec4ed6cedcdc4`. Use an explicit inequality branch that panics only with the first normalized byte/line and three context lines; never use `assert_eq!` on the full buffers. Also assert 460 layers and final statistics.

  Change the existing browser test from the temporary typed-error assertion
  to the same strict normalized comparison. It must fetch both committed
  files in Chromium, call `sliceProject` with the 3MF `Uint8Array`, validate
  exactly one Ares and one Orca generator line, compare every remaining byte,
  and assert the same normalized hash. All normalization and hashing execute
  in the browser page; Node may orchestrate Playwright but may not call core or
  substitute output bytes.

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

- [ ] **Step 3: Remove the CLI golden ignore and satisfy exact normalized equality**

  The active CLI test performs the same single-line validation and normalized SHA assertion as core, while also proving the no-external-options command contract. Its explicit inequality branch emits only the bounded normalized diagnostic and never Debug-prints either byte array.

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
