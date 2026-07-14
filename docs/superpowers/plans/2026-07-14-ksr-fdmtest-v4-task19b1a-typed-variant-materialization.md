# Task 19B.1A Typed Variant Materialization Implementation Plan

> **For agentic workers:** REQUIRED WORKFLOW: apply `sdd-workflow` and
> Subagent-Driven Development. Every implementation slice is owned by a fresh
> subagent that follows TDD. Review agents are read-only and independent from
> implementers. Do not begin production code until this plan has both an
> independent Codex `VERDICT: APPROVE` and an OpenCode `VERDICT: APPROVE` for
> the exact same file hash.

**Goal:** Port the fixed OrcaSlicer 2.4.2 active printer, process, and filament
variant materialization into a pure typed Ares transform, one source-cited
family at a time, without wiring the transform into project slicing yet.

**Architecture:** Add a `pub(crate)` typed transform below `options` that
clones an unmaterialized `ProjectSettings`, installs the supplied
`filament_map`, evaluates the exact four upstream families in their observable
sequential order, and returns the materialized clone. The implementation uses
only concrete typed vectors and a monomorphized slice-selection helper. It does
not call, extend, or remove the existing dynamic STL compatibility scaffold.

**Approved spec:**
`docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task19b1a-typed-variant-materialization.md`,
SHA-256
`b96b79b54c5cfdf231ddd508647de51851f3c4e34dd06ce86c993b129fc464ee`.
The exact hash received fresh independent Codex and OpenCode approval.

**Tech stack:** Rust 1.91.0, edition 2024, existing typed option wrappers,
`cargo-nextest`, and the existing native/WASM/browser release harness. No new
dependency or workspace crate is authorized.

## Fixed rewrite boundary

The only upstream baseline is OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Source checks use
`git -C OrcaSlicer show/grep` at that commit, never the mutable checkout.

- `src/libslic3r/PrintConfig.cpp:8344-8473`: exact 2 / 37 / 24 / 15 family
  ledgers.
- `PrintConfig.cpp:588-606`: typed canonical variant spelling.
- `PrintConfig.cpp:8981-9054`: active-count guard, complete/generated ID maps,
  and first exact match.
- `PrintConfig.cpp:9634-10023`: physical/process and logical-filament
  materialization.
- `PrintApply.cpp:1164-1173`: variant-1, variant-2, process, then filament.
- `Print.cpp:3166-3175`: restore the pre-filament source before rematerializing
  a changed map.
- `Config.hpp:624-630`: non-empty vector `get_at` repeats its first item.

This plan supersedes only the variant-materialization portion of the older
monolithic parent Task 19B. It does not adopt that task's preset/UI sizing
calls or preliminary-region lifecycle.

## Scope and non-negotiable constraints

- Production interface:

  ```rust
  pub(crate) fn materialize_project_variants(
      source: &ProjectSettings,
      filament_map: &OrcaInts,
  ) -> Result<ProjectSettings, SliceError>;
  ```

- The input may already contain `normalize_fdm_1` and cold first
  `normalize_fdm_2` writes. This task never runs normalization or reloads the
  3MF. Task 19B.3 owns orchestration and is the first production caller.
- Clone once, write `project.gcode.filament_map` in the clone, and never mutate
  `source`.
- Evaluate against the current clone in this exact order:

  1. printer variant 1 from raw printer selectors;
  2. printer variant 2 after variant 1 has shortened those selectors;
  3. process from its separate raw selectors;
  4. filament from its separate raw selectors and the supplied map.

- For the committed fixture, variant-1/process base indices are `[0, 2]`,
  variant-2 re-resolves `[0, 1]` and selects stride positions
  `[0, 1, 2, 3]`, and filament raw indices are `[0, 4]`.
- All family writes are explicit concrete field writes. Existing registry key
  arrays are test/review ledgers only, not runtime dispatch.
- Production must not use `SliceOptions`, `serde_json::Value`, `Map`,
  `RawValue`, a runtime key/value map, `ExtruderIndexIdMapLookup`, any
  `update_values_to_printer_extruders*` helper, or type erasure.
- Retain the old dynamic modules unchanged for the STL compatibility path;
  Task 20E owns their eventual removal.
- The new core module has no filesystem, process, clock, terminal, FFI, or
  native-only behavior and remains WASM-compatible.
- Production cannot mention the fixture path/name/hash, reference G-code, or
  branch on fixture values.
- Every changed Rust file stays below 400 physical lines. Split before adding
  a line to a file already at that limit.
- No source-level Orca pinning test is added. Fixed-source inspection is review
  evidence; behavior tests use typed sentinels and the committed 3MF contract.
- `slice_project` and the browser binding continue to return exactly
  `ProjectSlicingIncomplete` for the real fixture after this increment.

## Typed selector locations

The implementation must read the current typed owners directly:

| Meaning | Typed location |
| --- | --- |
| Physical count | `project.print.nozzle_diameter` |
| Guard/generated groups | `printer.remaining.extruder_variant_list` |
| Physical type | `printer.gcode.extruder_type` |
| Physical nozzle-volume type | `project.gcode.nozzle_volume_type` |
| Printer selectors | `printer.gcode.printer_extruder_id` / `printer.gcode.printer_extruder_variant` |
| Process selectors | `process.region.print_extruder_id` / `process.region.print_extruder_variant` |
| Logical filament IDs | `project.preset.filament_self_index` |
| Logical filament variants | `filament.gcode.filament_extruder_variant` |
| Installed map | `project.gcode.filament_map` |

All exact family members and counts remain the approved spec's authoritative
inventory. `NullableInts`, `VariantStride`, and `SpaceTuple` are concrete
Vec-backed wrappers and receive the same checked selection as the ordinary
typed vectors.

## Planned file manifest

Production:

- Modify `crates/ares-core/src/options.rs` only to register
  `#[cfg_attr(not(test), allow(dead_code))] mod project_variants`. The narrow
  temporary allowance is required because Task 19B.3, not this task, owns the
  first production caller; it must be removed when that caller lands.
- Create `crates/ares-core/src/options/project_variants.rs`: public-to-crate
  transform, clone/no-op ordering, and sequential family orchestration.
- Create `crates/ares-core/src/options/project_variants/index.rs`: guard token
  scan, generated-map token scan, typed variant spelling, and physical/logical
  index resolution.
- Create `crates/ares-core/src/options/project_variants/select.rs`: one checked
  monomorphized `select_stride<T: Clone>` over concrete slices.
- Create `crates/ares-core/src/options/project_variants/printer.rs`: explicit
  24-field variant-1, 15-field variant-2, and 2-field process writes.
- Create `crates/ares-core/src/options/project_variants/filament.rs`: explicit
  37-field logical-filament writes.

Tests:

- Modify `crates/ares-core/src/options/tests.rs` only to add
  `mod project_variants;`.
- Create `crates/ares-core/src/options/tests/project_variants.rs`; Slice 1
  registers only `support` and `index`, and later slices edit it to register
  each child only when that child file is created.
- Create `crates/ares-core/src/options/tests/project_variants/support.rs`.
- Create `crates/ares-core/src/options/tests/project_variants/index.rs`.
- Create
  `crates/ares-core/src/options/tests/project_variants/printer_process.rs`.
- Create `crates/ares-core/src/options/tests/project_variants/filament.rs`.
- Create `crates/ares-core/src/options/tests/project_variants/fixture.rs`.

Documentation is updated only after whole-implementation approval:

- Include the approved spec and this plan unchanged in the final manifest. Any
  edit to either file invalidates its frozen hash and requires the applicable
  spec/plan approval gates to run again.
- `docs/architecture/option-parity-v4.md`.
- `docs/roadmap.md`.
- Ignored `.superpowers/sdd/progress.md` evidence ledger.

No other path is authorized without first revising and reapproving this plan.

## Test support contract

`support.rs` is test-only and may use `serde_json::Value`. It will:

- load the committed fixture with `load_project(include_bytes!(...))` and clone
  `project.settings()`;
- serialize the five existing typed owners separately into one test-only flat
  map, without adding aggregate `Serialize` to `ProjectSettings`;
- create synthetic sources by overriding serialized typed defaults and
  deserializing the concrete `ProjectSettings`;
- provide a keyed `SliceError::InvalidInput` assertion;
- compare family cardinalities through the existing 2 / 37 / 24 / 15 registry
  ledgers; and
- never read the reference G-code.

Typed equality is the value oracle for special wrappers. Flattened JSON is
used only for exhaustive ledger cardinality and changed-key comparisons.

## Subagent execution protocol

Implementation slices are serialized because they share the orchestrator and
typed test support. Parallel agents may perform read-only source/type audits or
review frozen diffs, but two implementers never edit the shared workspace at
the same time.

Before each slice, the primary agent writes an ignored SDD brief containing the
approved spec/plan hashes, allowed paths, exact tests, source citations, and
forbidden APIs. A fresh implementation subagent must:

1. inspect the current shared tree and state assumptions;
2. add the named failing test before production behavior;
3. run the exact focused RED command and record the failure;
4. implement only that slice;
5. run focused GREEN, rustfmt, and `git diff --check`;
6. report changed paths, commands, results, and remaining expected gaps; and
7. make no commit, push, documentation change, or unrelated cleanup.

Before each slice diff check/review, the primary agent registers that slice's
task-owned new paths with `git add --intent-to-add -- <exact new paths>`. This
makes untracked file contents visible to `git diff` without staging their
content. The primary verifies the index contains intent entries only and never
uses a broad `git add`.

After each GREEN, a different read-only agent reviews that slice against the
brief, approved spec/plan, and fixed source. Any `REVISE` finding goes to a
fresh fixer subagent and the slice is re-reviewed. Slice approval is not whole
spec approval and does not authorize documentation or release.
Every mandatory slice reviewer returns a literal `VERDICT: APPROVE` or
`VERDICT: REVISE`; the prompt records the reviewer role separately.

## Slice 1: typed guard and index resolution

**Owned paths:** module registrations, `project_variants.rs`, `index.rs`,
`select.rs`, shared test module/support, and `index.rs` tests.

- [ ] Register an empty `options::project_variants` module first, then add the
  index tests while `materialize_project_variants` is still absent.
- [ ] Capture a genuine compile RED whose new cause is the unresolved
  `options::project_variants::materialize_project_variants`; do not add a
  test-local stub.
- [ ] Keep resolver helpers private to the production module and expose only a
  narrow `#[cfg(test)] pub(crate)` re-export for exact base-index assertions
  from the sibling test module. Remove or expand no other visibility.
- [ ] Add the minimal production module and implement:
  - physical count from `nozzle_diameter`, rejecting an empty vector by key;
  - guard scan over exactly `0..physical_count`, short non-empty group
    `get_at` repetition, compressed comma runs, no trim, edge-empty
    preservation, and trailing-group exclusion;
  - no-op return after only map replacement, without selector/payload/map
    validation;
  - canonical enum spelling for Direct Drive/Bowden and Standard/High Flow;
  - short non-empty type/nozzle control repetition and keyed empty errors;
  - complete ID map when `ids.len() >= variants.len()`, otherwise the generated
    map over all stored groups with compressed comma runs, trim, and empty skip;
  - first exact `(ID, variant)` match; and
  - one-based `filament_map` range validation only on the active branch.
- [ ] Cover these named behaviors:
  `complete_id_map_uses_first_exact_pair_and_allows_trailing_ids`,
  `generated_id_map_compresses_trims_and_skips_empty_tokens`,
  `guard_uses_only_physical_groups_repeats_first_and_ignores_trailing`,
  `guard_preserves_edge_empty_and_whitespace_tokens`,
  `one_extruder_multiple_variants_activates_materialization`,
  `one_extruder_one_variant_replaces_only_map_without_validation`,
  `short_nonempty_typed_controls_repeat_first`,
  `empty_boundary_vectors_name_their_orca_keys`, and
  `active_branch_invalid_selector_missing_match_and_map_name_keys`.
- [ ] Record the compile RED and focused GREEN:

  ```powershell
  cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::project_variants::index::/)'
  cargo +1.91.0 fmt --all -- --check
  git diff --check
  ```

## Slice 2: printer variant 1 and process

**Owned paths:** `printer.rs`, `project_variants.rs`, `index.rs`,
`crates/ares-core/src/options/tests/project_variants.rs`,
`crates/ares-core/src/options/tests/project_variants/index.rs`,
`printer_process.rs`, and test support only as required.

- [ ] Add compiling behavior REDs with unique source sentinels for base indices
  `[0, 2]`.
- [ ] Register `mod printer_process;` in the parent test module in the same
  change that creates `printer_process.rs`.
- [ ] After recording the behavior RED, refactor the Slice 1 discard-only
  validation seam into narrow `pub(super)` typed APIs for activation plus
  printer, process, and filament base-index resolution. Keep token splitting,
  generated-ID construction, and exact-pair matching private in `index.rs`;
  retain only the existing `#[cfg(test)] pub(crate)` external inspection seam.
  This is the single production resolver used by Slice 2, the Slice 3 printer
  re-resolution, and Slice 4 filament selection; do not duplicate it in the
  orchestrator or family modules.
- [ ] During Slices 2 and 3, keep invoking the filament base-index resolver as
  validation-only after process handling. This preserves every approved Slice
  1 no-op and keyed active-branch error until Slice 4 consumes those indices
  for the 37-field materialization.
- [ ] Once strict variant-1 payload writes exist, adapt the Slice 1
  generated-ID-map resolver test to use only its existing test-only inspection
  seam. Its intentionally short `printer_extruder_id` forces generated-ID
  resolution but is now invalid as a complete-family payload; remove only the
  obsolete full-materializer success assertion and preserve the exact resolved
  base-index assertion and every other Slice 1 behavior.
- [ ] Implement explicit checked stride-one selection for all 24 printer
  variant-1 members, including the printer selector fields themselves.
- [ ] Implement the separate process re-resolution and explicit selection of
  `print_extruder_id` and `print_extruder_variant`.
- [ ] Prove output cardinalities 2 / 2 through the fixed ledgers and concrete
  typed values, including `NullableInts` and enum vectors.
- [ ] Prove selected payload out-of-range errors name the printer/process key,
  rather than reproducing C++ first-value or transient-index-zero recovery.
- [ ] Run focused RED/GREEN and slice review:

  ```powershell
  cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::project_variants::printer_process::/)'
  cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::project_variants::/)'
  cargo +1.91.0 fmt --all -- --check
  git diff --check
  ```

## Slice 3: sequential printer variant 2

**Owned paths:** `printer.rs`, `project_variants.rs`,
`printer_process.rs`, and test support only as required.

- [ ] Add a compiling RED with raw stride-two sentinel values
  `[10, 11, 20, 21, 30, 31]` expecting `[10, 11, 20, 21]`.
- [ ] Resolve printer indices again from the post-variant-1 clone. Never cache
  or reuse variant-1 base indices.
- [ ] Explicitly materialize all 15 machine-limit fields at stride two.
- [ ] Freeze all 15 outputs at cardinality four and cover selected payload
  out-of-range by exact key.
- [ ] Extend only the shared synthetic `active_source()` test setup so all 15
  machine-limit payloads contain four valid stride-two positions. Two physical
  extruders require those four positions once Slice 3 strict writes exist;
  never add a production fallback or skip for the earlier two-entry defaults.
- [ ] Keep the process call after variant 2.
- [ ] Run focused RED/GREEN and slice review with the same
  `printer_process` filter plus the cumulative `project_variants` filter,
  rustfmt, and diff check from Slice 2.

## Slice 4: filament, rematerialization, and real fixture

**Owned paths:** `filament.rs`, `project_variants.rs`,
`crates/ares-core/src/options/tests/project_variants.rs`, `filament.rs` tests,
`fixture.rs`, and test support only as required.

- [ ] Before filament production behavior, add both filament and real-fixture
  tests. The fixture test must be RED because the logical filament family is
  still unmaterialized.
- [ ] Register `mod filament;` and `mod fixture;` in the parent test module in
  the same change that creates those child files.
- [ ] Resolve one raw index per logical filament ID using the mapped physical
  extruder type/nozzle-volume type, raw `filament_self_index`, and raw
  `filament_extruder_variant`.
- [ ] Explicitly materialize all 37 fields, including `VariantStride`,
  `SpaceTuple`, nullable vectors, and enum vectors, to logical-filament
  cardinality.
- [ ] Use a synthetic map `[1, 2]` whose second logical filament resolves raw
  index 6, preventing a `len / count` shortcut.
- [ ] Prove deterministic rematerialization from the same raw source and prove
  a changed map from that same raw source changes only `filament_map` plus the
  filament family. Never use a previous materialized output as input.
- [ ] Flatten source and materialized typed owners and prove every key outside
  the exact 2 / 37 / 24 / 15 family union and `filament_map` is unchanged;
  run this invariant against both synthetic sentinels and the real fixture.
- [ ] Prove already-normalized retract payloads are selected as supplied and a
  selected filament payload out-of-range error names its key.
- [ ] For the real fixture assert, at minimum:
  - source equality before/after the call;
  - printer/process IDs `[1, 2]` and variants
    `[Direct Drive Standard, Bowden Standard]`;
  - `retraction_length = [0.8, 2]`;
  - `machine_max_acceleration_e = [30000, 5000, 30000, 5000]`;
  - `machine_max_speed_e = [30, 30, 30, 30]`;
  - two Direct Drive Standard filament variants;
  - `filament_max_volumetric_speed = [21, 21]`;
  - every ledger cardinality 2 / 37 / 24 / 15; and
  - raw `filament_self_index` still has eight entries.
- [ ] Prove the public real-project boundary still returns
  `ProjectSlicingIncomplete`.
- [ ] Run focused RED/GREEN:

  ```powershell
  cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::project_variants::filament::/)'
  cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::project_variants::fixture::/)'
  cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::project_variants::/)'
  cargo +1.91.0 fmt --all -- --check
  git diff --check
  ```

## Whole implementation approval gate

After every slice is locally approved, verify every task-owned new file already
has its intent-to-add entry and the complete diff is visible. Freeze the
implementation path list and SHA-256 manifest in the ignored SDD ledger. Do
not stage content or update architecture/roadmap yet.

Run three independent, read-only whole-diff reviews against the frozen hash:

1. a fresh spec-compliance agent must return literal `VERDICT: APPROVE` and
   identify its role as `ROLE: SPEC COMPLIANCE` on a separate line;
2. a different fresh code-quality agent must return literal
   `VERDICT: APPROVE` and identify its role as `ROLE: CODE QUALITY` on a
   separate line; and
3. a fresh OpenCode run must return `VERDICT: APPROVE`.

Every reviewer checks the entire diff, approved spec/plan, fixed source,
sequential mutation, all family fields, typed-only isolation, TDD evidence,
tests, LOC, and absence of hardcoding. Any edit invalidates all three verdicts.
Assign remediation to a fresh implementation subagent, rerun relevant RED/GREEN
evidence, refreeze, and restart all three reviews until all approve the same
hash.

Only after these approvals may documentation be updated.

## Documentation gate

- [ ] Update `docs/architecture/option-parity-v4.md` with the fixed source
  boundary, typed selector ownership, sequential variant-2 re-resolution,
  strict boundary divergences, and explicit 19B.1B/19B.2/19B.3 deferrals.
- [ ] Update `docs/roadmap.md` to mark only Task 19B.1A complete and retain
  complete KSR G-code parity as open.
- [ ] Update the ignored progress ledger with slice RED/GREEN evidence and the
  frozen approvals.
- [ ] Preserve the approved spec and plan byte-for-byte with their approved
  hashes/provenance; only architecture, roadmap, and ignored progress receive
  post-implementation documentation edits.
- [ ] Obtain an independent read-only docs review returning literal
  `VERDICT: APPROVE` with `ROLE: DOCUMENTATION` on a separate line. Any docs
  edit after the verdict requires a fresh docs review.

## Full local release matrix

Run every command fresh on the final frozen manifest:

```powershell
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::project_variants::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_variants|project_deserialize|project_import|project_inventory|gcode_options|printer_gcode_source|printer_machine_envelope|printer_remaining|filament_gcode_source|filament_remaining|process_region_source|process_remaining|project_runtime_options|region_options|registry_lookup_variant_option_sets)/)'
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npx --prefix crates/ares-wasm/tests/browser playwright install chromium
npm --prefix crates/ares-wasm/tests/browser test
git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
```

The browser test must still observe `ProjectSlicingIncomplete` through the
generated JavaScript binding. The existing ignored full CLI golden remains
ignored; this task must not claim G-code bytes.

Run nonbehavior audits:

```powershell
if ((Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf -Algorithm SHA256).Hash.ToLowerInvariant() -ne '698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9') { throw '3MF fixture changed' }
if ((Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode -Algorithm SHA256).Hash.ToLowerInvariant() -ne '10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3') { throw 'G-code fixture changed' }

$variantPaths = @('crates/ares-core/src/options/project_variants.rs', 'crates/ares-core/src/options/project_variants')
$hits = rg -n 'serde_json::(Value|Map)|RawValue|BTreeMap<String|dyn Any|update_values_to_printer_extruders|get_index_for_extruder|ExtruderIndexIdMapLookup|ksr_fdmtest_v4|698f40f13c9075b8|10aec9a156849f59|["''][^"'']*\.gcode["'']|include_(bytes|str)!|std::fs|std::process|extern "C"|run_slicing_pipeline' @variantPaths
if ($LASTEXITCODE -gt 1) { throw 'rg failed' }
if ($hits) { $hits; throw 'forbidden project-variant production reference' }

$changedRust = git diff --name-only HEAD -- '*.rs'
$oversized = foreach ($path in $changedRust) {
    $lines = (Get-Content -LiteralPath $path).Count
    if ($lines -ge 400) { "$path`t$lines" }
}
if ($oversized) { $oversized; throw 'changed Rust file is not below 400 LOC' }
```

Verify working-tree, index intent, frozen manifest, and later commit-tree paths
are identical. Only the approved manifest may be staged.

## Commit, push, and exact-SHA Tier 1

Before committing, read and follow the `conventional-commits` skill. Then:

1. stage only the approved frozen manifest;
2. confirm `git diff --cached --check` and staged-path equality;
3. commit as `feat(config): materialize active variant options`;
4. push `codex/ksr-fdmtest-v4-parity`;
5. verify local HEAD equals the remote branch SHA and the worktree/index are
   clean; and
6. wait for that exact SHA's `tier1.yml` run until all five required jobs are
   green: `format`, `ubuntu-latest`, `wasm`, `macos-latest`, and
   `windows-latest`.

Do not begin dependent Task 19B.1B or 19B.2 while exact-SHA Tier 1 is pending
or failed. Record commit SHA, run ID, and job conclusions in the ignored SDD
ledger.

## Explicit deferrals

- Task 19B.1B: export/runtime split and nullable filament retract overlay.
- Task 19B.2: complete model-key classification and bounded optional layer
  range import/association.
- Task 19B.3: typed normalization, cold-start/source-ordered
  `FullPrintConfig` orchestration, and the first production call to this
  transform.
- Task 19C: exact config-block serialization.
- Tasks 20A-20E: remaining dynamic consumers and legacy scaffold removal.
- Later tasks: geometry, slicing, G-code generation, metadata, and final byte
  parity.

## Plan self-review checklist

- [x] Exact approved spec hash and fixed Orca commit are named.
- [x] Every 2 / 37 / 24 / 15 family is mapped to a concrete implementation
  slice and exhaustive cardinality test.
- [x] Variant 2 re-resolves from the post-variant-1 clone.
- [x] Guard, generated-map, no-op, and cardinality divergences are testable.
- [x] Only tests use dynamic JSON flattening.
- [x] The first RED is an unresolved production API; later REDs are behavior
  failures.
- [x] Implementation, review, documentation, release, push, and exact-SHA CI
  gates are separate.
- [x] No plan step wires project slicing early or claims G-code parity.
