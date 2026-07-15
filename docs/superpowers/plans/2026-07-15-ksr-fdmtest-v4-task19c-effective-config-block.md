# Task 19C: Exact Effective Config-Block Serialization Implementation Plan

> **Execution contract:** Follow the approved SDD workflow and this checklist
> in order. Production implementation may begin only after this plan receives
> literal `VERDICT: APPROVE` from both the independent reviewer agent and the
> required default-model OpenCode reviewer. Use fresh bounded implementer and
> reviewer subagents for each slice. Do not commit between slices; commit and
> push only after whole-spec implementation approval and documentation review.

**Approved specification:**
`docs/superpowers/specs/2026-07-15-ksr-fdmtest-v4-task19c-effective-config-block.md`

**Approved specification SHA-256:**
`9E2C3195D8578969943E97A0FE0424C1F343C0B1F1684892C86B87A078748B8F`

**Pinned Orca baseline:**
`8500fcdccaa10b5099ac20d252af3a7c560046f1`

**Ares baseline SHA:**
`99fb0beba0a48603cb7875591cf77d02c26fb525`

## Goal and source boundary

Port the fixed Orca Bambu `CONFIG_BLOCK` writer from
`GCode.cpp:2637-2658,5591-5644`, `Config.cpp:48-120,543-548,1715-1721`,
the concrete `ConfigOption` serializers in `Config.hpp`, the bed selector in
`PrintConfig.hpp:489-509`, and the CLI Bambu classification in
`OrcaSlicer.cpp:6045-6060`.

The implementation must serialize the final Task 19B.3 effective project
configuration into the exact 49,004-byte committed KSR config block. Canonical
options come from `ProjectConfigViews::full`; the two computed temperatures
come from `ProjectConfigViews::runtime`; plate index remains explicit context.
The production project path executes this boundary but still returns
`ProjectSlicingIncomplete`.

No task in this plan may add geometry, executable G-code, public partial output,
CLI project dispatch, a second 650-field config type, JSON/dynamic production
fallback, or fixture/reference branching.

## Baseline and evidence discipline

Before Task 1:

```powershell
$spec = 'docs/superpowers/specs/2026-07-15-ksr-fdmtest-v4-task19c-effective-config-block.md'
(Get-FileHash $spec -Algorithm SHA256).Hash
git rev-parse HEAD
git status --short
```

Require the approved spec hash above, baseline SHA above, and only the approved
spec/plan artifacts as uncommitted tracked candidates. Record commands and
outputs in ignored `.superpowers/sdd/task19c-evidence.md`; do not stage evidence
files.

For every implementation slice:

1. Run and record the focused RED test before production changes for that slice.
2. Assign a bounded implementer subagent explicit file ownership.
3. Inspect the diff and run the focused GREEN command locally.
4. Assign a fresh independent reviewer to compare the slice against the
   approved spec and plan. Require literal `VERDICT: APPROVE`; fix and re-review
   until approved.
5. Update the plan checklist/evidence before starting a dependent slice.

For every slice review, enumerate every changed path owned by that slice. Give
the reviewer the ordinary `git diff -- <path>` for tracked paths and the full
`git diff --no-index -- /dev/null <path>` patch for each untracked path. The
reviewer must inspect all new file contents; an ordinary worktree `git diff`
alone is never sufficient because it omits untracked files.

Task 1 and Task 5 have disjoint files and may be implemented in parallel after
plan approval. Tasks 2-4 are dependency-ordered and must remain sequential.

## Planned file boundary

Exact private names may change when a smaller Rust expression is found, but the
ownership boundary must remain equivalent and every Rust file must stay below
400 physical lines.

**Create:**

- `crates/ares-core/src/options/config_export.rs`
- small siblings under `crates/ares-core/src/options/config_export/`, expected
  as `collector.rs`, `transform.rs`, `value.rs`, and `writer.rs`
- a crate-private semantic-tag/borrowed-wrapper sibling under
  `crates/ares-core/src/options/config_types/`
- `crates/ares-core/src/options/tests/config_export.rs`
- focused siblings under `crates/ares-core/src/options/tests/config_export/`
- `crates/ares-core/src/project/tests/effective_config/config_export.rs`

**Modify narrowly:**

- `crates/ares-core/src/options.rs` and
  `crates/ares-core/src/options/config_types.rs` for module wiring only
- `crates/ares-core/src/options/config_types/opaque.rs`
- `crates/ares-core/src/options/config_types/point.rs`
- `crates/ares-core/src/options/config_types/scalar_nullable.rs`
- `crates/ares-core/src/options/printer_options/remaining/structured.rs`
- `crates/ares-core/src/options/printer_options/gcode_source/enums.rs`
- only the existing flat wire entries that need a nullable-vector view under
  `filament_options/wire/` and, if required by the audited inventory,
  `printer_options/wire.rs`
- `crates/ares-core/src/options/filament_config_export/serialization.rs` only
  if the existing exact string-vector helper is extracted for shared reuse
- `crates/ares-core/src/options/tests.rs`
- `crates/ares-core/src/options/tests/project_inventory.rs`
- `crates/ares-core/src/options/typed_legacy/thumbnails.rs`
- `crates/ares-core/src/options/tests/typed_legacy/thumbnails.rs`
- `crates/ares-core/src/project/tests/effective_config.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/Cargo.toml` only if `sha2` is required by the fixture test
- approved architecture/roadmap docs only after whole implementation approval

Do not modify `gcode.rs`, `gcode_config_header.rs`, `gcode_header.rs`, the CLI
adapter, dynamic-value baselines, the fixture 3MF/reference G-code, or the
pinned Orca checkout.

---

## Task 1: Typed config-token serializer and semantic tags

**Upstream boundary:** `Config.cpp:48-120`; concrete scalar/vector/string/
point/nullable serializers in `Config.hpp:764-2194`.

**Files:**

- Create the semantic-tag/borrowed-wrapper module under `config_types/`.
- Create `options/config_export/value.rs` and the minimum parent module wiring.
- Create `options/tests/config_export/value.rs` and test module wiring.
- Modify only the typed wrapper serializers listed in the planned boundary.
- Optionally extract the legacy string-vector helper without changing its
  behavior.

### Step 1.1: Write focused RED tests

Add tests for a crate-private `serialize_config_value` seam or equivalent that
exercise:

- bool, signed/unsigned int, float, negative zero, defaultfloat rounding, and
  scientific notation through existing typed serializers;
- percent and float-or-percent;
- exact renamed enum tokens, including a token with a space that must not be
  string-vector quoted;
- scalar string C-style escaping of CR, LF, backslash, and quote exactly once;
- string vectors: semicolon separation, sole empty quoting, multi-empty
  behavior, spaces/tabs, CR/LF, backslash, and quote;
- ordinary numeric/enum sequences joined with comma;
- point, point list, and point-group punctuation;
- a tagged nullable vector with empty, all-nil, mixed, and all-value contents;
- an empty non-nullable vector remaining present and distinct from empty
  nullable.

Add JSON golden assertions for every wrapper whose `Serialize` implementation
will change. Snapshot both scalar/sequence shape and representative values.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core config_export_value
```

Require RED because the config-value serializer and semantic tags do not yet
exist. Save the failing test names and reason.

### Step 1.2: Implement explicit typed tags and value serialization

Implement shared static semantic tags for exactly:

- ConfigOptionStrings;
- ConfigOptionPointsGroups;
- ConfigOptionNullableVector;
- ConfigOptionNil.

Do not use Rust `type_name`, field-key lookup, token-content inference, a
registry, JSON, or a fallback kind.

Give ConfigOptionStrings semantics to:

- `OrcaStrings`;
- `AmsCounts`;
- `RammingParameters`;
- `CsvTable`;
- `SpaceTuple`;
- `VariantStride`;
- `ExtruderVariantLists`.

Give `Point2dGroups` its own semantic tag. Keep `Point2dList` as an ordinary
comma sequence. Make `Nullable::Nil` produce the nil semantic event while
`Nullable::Value` delegates to its concrete typed value.

Provide a borrowed top-level nullable-vector wrapper that is JSON-transparent
and preserves nullable identity when the sequence is empty. Named nullable
wrappers may serialize through the same tag directly.

Implement the private config-value serde sink exhaustively for the events the
650 typed fields use. `serialize_str` is a scalar ConfigOptionString token and
applies scalar C-style escaping. `serialize_unit_variant` emits the exact enum
token. The ConfigOptionStrings branch collects raw elements and applies its own
semicolon/conditional-quote escape once. Ordinary sequences join with comma;
point groups join with `#`; tagged nullable sequences carry explicit nil state.

Unsupported maps/nesting must produce an internal serialization error rather
than guess a representation. Do not add validation for impossible trusted
internal events beyond the serializer boundary.

### Step 1.3: Prove GREEN and wire stability

```powershell
cargo +1.91.0 nextest run -p ares-core config_export_value
cargo +1.91.0 nextest run -p ares-core config_types
cargo +1.91.0 nextest run -p ares-core filament_config_export
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets --all-features -- -D warnings
```

Require JSON golden equality and all retained legacy string-vector tests green.
Review the slice independently before Task 2.

---

## Task 2: Canonical 650-entry collector and nullable wire identity

**Upstream boundary:** `DynamicConfig::keys()` lexical iteration and
`ConfigOption::is_nil`; Ares' already-approved four canonical typed owners.

**Depends on:** Task 1 approved.

**Files:**

- Create `options/config_export/collector.rs`.
- Create `options/tests/config_export/inventory.rs` and `nullable.rs`.
- Modify the 31 audited nullable top-level wire entries only.
- Modify parent module wiring only as needed.

### Step 2.1: Write collector/inventory RED tests

From a typed `ProjectSettings`, test that the collector:

- invokes the four existing group map serializers and collects exactly 650
  entries before omission;
- has 650 unique compile-time keys;
- excludes `PresetMetadata::{from,name,version}` without adding
  `Serialize for ProjectSettings`;
- globally sorts by key independent of the four group boundaries;
- sees all 31 nullable fields as nullable even when a field is an empty vector;
- distinguishes the five fixture empty non-nullable fields from all-nil
  nullable fields;
- rejects a synthetic duplicate canonical key;
- encounters no unsupported serde event for any of the 650 fields;
- leaves the four groups' ordinary JSON wire bytes unchanged.

The RED output must identify missing collector/nullable identity behavior, not
an unrelated compile failure.

```powershell
cargo +1.91.0 nextest run -p ares-core config_export_inventory
cargo +1.91.0 nextest run -p ares-core config_export_nullable
```

### Step 2.2: Annotate all nullable vector owners

Audit the approved 31-field inventory against current declarations. There are
27 bare `Vec<Nullable<T>>` fields plus four fields using named nullable wrapper
types. At each bare field's existing flat group wire entry, serialize a borrowed
ConfigOptionNullableVector view. Give the named wrappers the same top-level
tag through a JSON-transparent manual serializer.

Do not change field storage types, constructors, runtime consumers, declaration
keys, defaults, or group counts merely to carry the export tag. Do not dispatch
on the option key inside the value serializer.

### Step 2.3: Implement the canonical collector

Invoke `Serialize` separately on `views.full.printer`, `.process`, `.filament`,
and `.project`, appending their map entries to one transient `Vec`. Do not
invoke `.metadata` and do not serialize a `ProjectSettings` object.

Each entry contains only:

- its serialized key;
- its final typed config token;
- explicit nullable/all-nil state needed for omission.

It must not retain a generic typed value, enable mutation, or recover a kind
from the key/token. Sort the completed vector lexically and validate uniqueness
at this internal construction boundary.

### Step 2.4: Prove GREEN and unchanged project JSON

```powershell
cargo +1.91.0 nextest run -p ares-core config_export_inventory
cargo +1.91.0 nextest run -p ares-core config_export_nullable
cargo +1.91.0 nextest run -p ares-core project_deserialize
cargo +1.91.0 nextest run -p ares-core printer_remaining
cargo +1.91.0 nextest run -p ares-core filament_remaining
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets --all-features -- -D warnings
```

Independently review exact 650/31 counts, JSON stability, and absence of
key/type guessing before Task 3.

---

## Task 3: Fixed transforms, Bambu block writer, and byte-exact fixture

**Upstream boundary:** `GCode.cpp:2637-2658,5591-5644`,
`Config.hpp:624-627`, `PrintConfig.hpp:489-509`, and
`OrcaSlicer.cpp:6045-6060`.

**Depends on:** Tasks 1-2 approved.

**Files:**

- Create `options/config_export/transform.rs` and `writer.rs`.
- Create `options/tests/config_export/special.rs`, `fixture.rs`, and
  `bambu.rs`.
- Modify `options/typed_legacy/thumbnails.rs` and its focused test only for the
  approved no-space multi-item canonical form prerequisite.
- Add `sha2` as an `ares-core` dev dependency only if the test cannot reuse an
  existing crate-local SHA helper.

### Step 3.1: Write special-case and fixture RED tests

Synthetic tests must freeze:

- per-head flush-matrix segment multiplication and `f64::round`;
- source `views.full` immutability;
- one-filament size mismatch passthrough;
- multi-filament size mismatch exact `InvalidInput` message;
- zero-head deliberate safe error and atomic output buffer behavior;
- a preseeded caller buffer remaining byte-identical after a late writer error
  such as Default Plate or an empty required runtime temperature vector;
- fixed nine-key banned filtering through synthetic serialized entries;
- `print_compatible_printers` remaining distinct and exportable;
- `extruder_colour` using typed `filament_colour` without source mutation;
- non-zero wipe x/y selection;
- out-of-range wipe index falling back to the first item;
- the three-decimal special coordinate line followed by the ordinary vector
  line for each key;
- all six concrete bed type mappings and runtime element-zero nozzle/bed
  values;
- Default Plate and empty required runtime temperature vectors failing without
  hardcoded fallback;
- exact, case-sensitive `Bambu Lab` printer-model prefix classification.
- the typed thumbnail composite canonicalizing arbitrary multiple definitions
  with `,` and no space while retaining parsing, validation, formats, order,
  duplicates, and JSON behavior.

The fixture test must extract the committed reference block by byte markers,
then independently assert:

- 49,004 bytes;
- SHA-256
  `b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`;
- LF-only markers and trailing blank line;
- 639 assignment lines and 637 unique keys;
- two x and two y lines in exact order;
- 15 all-nil omissions, three metadata omissions, and five retained empty
  assignments;
- the final two temperature lines.

Resolve the committed 3MF through Task 19B.3 and compare the writer's complete
block bytes without normalization. Also assert full-vs-runtime sentinel
ownership for the six retract/wipe lines named in the spec and the two runtime
tail lines.

```powershell
cargo +1.91.0 nextest run -p ares-core config_export_special
cargo +1.91.0 nextest run -p ares-core config_export_fixture
cargo +1.91.0 nextest run -p ares-core options::tests::typed_legacy::thumbnails
```

Require RED because the complete transforms/writer do not exist.

For the discovered thumbnail prerequisite, first change the focused expected
multi-item canonical form from `", "` to `","` and require that assertion to
fail against the existing composite before changing production code. This RED
is independent of the missing writer RED and must be recorded separately.

### Step 3.2: Implement typed transforms before serialization

Clone `views.full`, then transform only the clone's typed `FlatMatrix` using
typed `flush_multiplier` and typed `filament_colour` cardinality. Do not parse
or replace serialized text. Preserve the exact single-filament exception and
multi-filament message. Handle zero heads as the spec's deliberate safe
translation of fixed-source UB.

Collect/sort the transformed clone. Apply the fixed banned set, nil omission,
x/y duplicate logic, and `extruder_colour` substitution during writing. Obtain
wipe coordinate values directly from the typed `OrcaFloats`; use requested or
first-element fallback and exact fixed precision three.

### Step 3.3: Implement runtime tail and complete atomic writer

Select the runtime bed vector exhaustively from `ProjectBedType`; reject
Default Plate. Take element zero from the selected bed vector and
`nozzle_temperature_initial_layer`. Write exact LF bytes:

1. start marker;
2. sorted canonical lines;
3. bed temperature;
4. nozzle temperature;
5. end marker and blank line.

Build into a private scratch buffer and append to the caller's buffer only
after all operations succeed.

Implement the Bambu predicate from typed `printer_model.starts_with("Bambu Lab")`
with no unavailable name/system fallback.

### Step 3.4: Drive the exact fixture to GREEN

```powershell
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets --all-features -- -D warnings
```

Do not weaken, normalize, trim, regenerate, or update the committed expected
bytes. Independently review the writer and exact fixture evidence before Task 4.

---

## Task 4: Production project caller and incomplete boundary

**Upstream boundary:** final effective configuration after `Print::apply`,
CLI Bambu classification, and `PrintBase` default plate index zero.

**Depends on:** Task 3 approved.

**Files:**

- Modify `project_slice.rs` narrowly.
- Create `project/tests/effective_config/config_export.rs` and register it.
- Modify WASM tests only if an assertion needs to prove the same existing
  incomplete result; do not change the public WASM API.

### Step 4.1: Write caller RED tests

Test the production caller order:

1. malformed archive errors remain before resolution/export;
2. materialization/cardinality errors remain before export;
3. a Bambu project with a real flush-matrix export error returns that error
   before `ProjectSlicingIncomplete`;
4. a valid Bambu fixture executes export and still returns
   `ProjectSlicingIncomplete`;
5. a non-Bambu valid project skips the Bambu block writer and still returns
   `ProjectSlicingIncomplete`;
6. current caller plate context is the fixed source default zero, while direct
   writer tests already prove non-zero behavior.

Prefer an existing test-only trace or a deliberately failing export input over
adding production observability. Do not expose serialized scratch bytes.

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_export
```

### Step 4.2: Wire the production call

Retain the resolved value instead of discarding it. After final resolution:

- evaluate the typed Bambu predicate from `resolved.views.full`;
- when true, call the writer with explicit plate index `0` into a local
  scratch `Vec<u8>` and propagate a real error;
- keep the scratch buffer inside the incomplete pipeline for later assembly;
- preserve the current document references and `GenerationMetadata` ownership;
- return `ProjectSlicingIncomplete` after successful bounded work.

Do not add a public selector, result type, debug accessor, fixture branch, or
fallback to the STL slicer.

### Step 4.3: Prove GREEN through native and browser boundaries

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_export
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run -p ares-wasm
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets --all-features -- -D warnings
```

Run the existing wasm-bindgen/browser project test in the final release gate.
Independently review caller order and non-observability before whole-spec
review.

---

## Task 5: Remove obsolete executable source pinning

**May run in parallel with:** Task 1, because its only production-adjacent file
is the existing behavioral inventory test.

**Files:**

- Modify only `crates/ares-core/src/options/tests/project_inventory.rs`.
- Do not rewrite `tests/ksr_fdmtest_v4/options-v242.json` in this task.

### Step 5.1: Freeze retained behavioral coverage

Run the existing inventory tests before editing and record the passing test
names and assertions for:

- 653 total rows and sorted unique keys;
- scope/owner/type/projection counts;
- defaults and wire shapes;
- legacy key/conversion presence;
- config-export rules;
- fixture key/shape agreement.

```powershell
cargo +1.91.0 nextest run -p ares-core project_inventory
```

Then run the focused absence audit below before editing. It must be RED because
the obsolete executable source-pinning fields and assertions still exist:

```powershell
$pinning = rg -n "upstream_definition|upstream_consumers|SourceCitation|\.citation|\.line|\.symbol" `
  crates/ares-core/src/options/tests/project_inventory.rs
if ($LASTEXITCODE -gt 1) { throw "source-pinning audit failed to run" }
if ($pinning) {
    $pinning
    throw "obsolete executable source pinning remains"
}
```

Save the failing matches as the Task 5 RED evidence.

### Step 5.2: Remove only source-level pinning code

Delete:

- `InventoryRow::upstream_definition`;
- `InventoryRow::upstream_consumers`;
- `LegacyInput::citation`;
- `SourceCitation`;
- path/line/symbol/consumer-presence assertions.

Keep and rerun every behavioral assertion listed above. Serde may ignore the
extra JSON evidence fields. Do not delete `registry_lookup_*`, behavioral
truth tables, legacy conversion behavior, or source citations in reviewed docs.

```powershell
cargo +1.91.0 nextest run -p ares-core project_inventory
$pinning = rg -n "upstream_definition|upstream_consumers|SourceCitation|\.citation|\.line|\.symbol" `
  crates/ares-core/src/options/tests/project_inventory.rs
if ($LASTEXITCODE -gt 1) { throw "source-pinning audit failed to run" }
if ($pinning) {
    $pinning
    throw "obsolete executable source pinning remains"
}
```

The test and the same absence audit that was RED must now pass. Review this
cleanup independently before merging it into the whole implementation manifest.

---

## Task 6: Whole-spec static audit and fresh verification

**Depends on:** Tasks 1-5 approved.

### Step 6.1: Freeze the implementation manifest

Create ignored evidence files containing:

- every intended changed path, one relative path per line;
- the SHA-256 of every manifest path;
- the exact subset of manifest paths that are Task 19C production code;
- one complete patch containing tracked modifications and every untracked new
  file as a `/dev/null` no-index diff.

Build and validate that complete patch without intent-to-add staging:

```powershell
$manifestFile = '.superpowers/sdd/task19c-manifest-paths.txt'
$productionFile = '.superpowers/sdd/task19c-production-paths.txt'
$hashFile = '.superpowers/sdd/task19c-manifest-sha256.txt'
$patchFile = '.superpowers/sdd/task19c-complete.patch'
$manifestPaths = Get-Content $manifestFile | Where-Object { $_ }
$actualPaths = @(
    git diff --name-only --diff-filter=ACMR
    git ls-files --others --exclude-standard
) | Sort-Object -Unique
$manifestPaths = $manifestPaths | Sort-Object -Unique
$drift = Compare-Object $manifestPaths $actualPaths
if ($drift) { $drift; throw "Task 19C manifest does not match the worktree" }

$manifestPaths | ForEach-Object {
    "{0}  {1}" -f (Get-FileHash $_ -Algorithm SHA256).Hash, $_
} | Set-Content -Encoding utf8 $hashFile

$patchChunks = foreach ($path in $manifestPaths) {
    git ls-files --error-unmatch -- $path *> $null
    if ($LASTEXITCODE -eq 0) {
        $chunk = git diff --binary -- $path
        if ($LASTEXITCODE -ne 0) { throw "git diff failed for $path" }
    } else {
        $chunk = git diff --no-index --binary -- /dev/null $path 2>$null
        if ($LASTEXITCODE -notin @(0, 1)) { throw "no-index diff failed for $path" }
    }
    $chunk
}
$patchChunks | Set-Content -Encoding utf8 $patchFile

git status --short
git diff --check
Get-Content $patchFile | Select-Object -First 40
Get-Content $productionFile
```

For every untracked manifest path also run the executable whitespace check:

```powershell
$untrackedPaths = git ls-files --others --exclude-standard
foreach ($path in $untrackedPaths) {
    $checkOutput = git diff --no-index --check -- /dev/null $path 2>$null
    if ($LASTEXITCODE -notin @(0, 1)) { throw "no-index check failed for $path" }
    if ($checkOutput) { $checkOutput; throw "whitespace error in $path" }
}
```

Exit code `1` with no check output is only the expected new-file difference.
Reject unrelated changes. Preserve any user changes encountered; do not revert
them or include them in the manifest. Every slice and whole-spec reviewer
receives the complete patch, path manifest, production subset, and hash
manifest.

### Step 6.2: Run hardcoding/dynamic/pinning/LOC audits

Run at minimum:

```powershell
$productionPaths = Get-Content .superpowers/sdd/task19c-production-paths.txt |
  Where-Object { $_ -and (Test-Path $_) }

# No production fixture/reference coupling.
rg -n "(?i)ksr_fdmtest_v4|options-v242|b33c9790|49004|639|637" $productionPaths

# No JSON/dynamic registry fallback in any Task 19C production file.
rg -n "serde_json::Value|serde_json::to_|serde_json::from_|option_definition|registry::|SliceOptions|BTreeMap|HashMap|type_name" `
  $productionPaths

# No executable source pinning remains in the cleaned inventory test.
rg -n "upstream_definition|upstream_consumers|SourceCitation|\.citation|\.line|\.symbol" `
  crates/ares-core/src/options/tests/project_inventory.rs

# No allow/expect suppression in any Task 19C production file.
rg -n '#\s*\[\s*(allow|expect)\s*\(' $productionPaths

# All changed Rust files stay below 400 physical lines.
$rustPaths = git diff --name-only --diff-filter=ACMR -- '*.rs'
$rustPaths += git ls-files --others --exclude-standard -- '*.rs'
$rustPaths | Sort-Object -Unique | Where-Object { Test-Path $_ } | ForEach-Object {
    $lines = (Get-Content $_).Count
    if ($lines -ge 400) { throw "$_ has $lines lines" }
}
```

The production subset must include every changed non-test Rust source path, not
only `config_export` and `project_slice.rs`. The first, second, source-pinning,
and suppression searches must return no forbidden result. Test-only fixture
constants are permitted only inside the approved config-export fixture test
files. Review every match manually; do not hide findings by renaming or moving
dynamic code.

### Step 6.3: Run fresh native/WASM/browser release verification

Use the repository's configured bundled dependencies when required. Run:

```powershell
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --release --target wasm32-unknown-unknown
cargo +1.91.0 nextest run -p ares-cli
cargo +1.91.0 install --locked wasm-bindgen-cli --version 0.2.121
wasm-bindgen --version
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm `
  --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser audit --audit-level=low
npx --prefix crates/ares-wasm/tests/browser playwright install chromium
npm --prefix crates/ares-wasm/tests/browser test
```

Require the version check to print `wasm-bindgen 0.2.121`, matching the workspace
binding version and Tier1 workflow. `npm audit --audit-level=low` must report
zero vulnerabilities. The CLI command must leave the ignored complete KSR
golden test ignored. Record exact run IDs, counts, skips, and commands in
ignored evidence.

### Step 6.4: Required whole implementation reviews

Dispatch two fresh independent reviews before docs:

1. **Spec compliance:** compare the approved spec, approved plan, frozen
   path/hash manifests, complete tracked-plus-untracked patch, and fresh
   verification evidence. Require literal
   `VERDICT: APPROVE`.
2. **Code quality:** inspect correctness, maintainability, portability, atomic
   error behavior, no dynamic/reference coupling, and tests. Require literal
   `VERDICT: APPROVE`.

Also run the required default-model OpenCode whole-spec implementation review.
If any reviewer returns `REVISE` or omits the literal approve verdict, fix,
rerun affected verification, rebuild the manifest, and rerun all required
whole-spec reviewers.

Do not update tracked architecture/roadmap docs before whole implementation
approval.

---

## Task 7: Documentation review after implementation approval

**Files:**

- Modify `docs/architecture/option-parity-v4.md`.
- Modify `docs/roadmap.md`.

Document only approved shipped behavior:

- `ProjectConfigViews::full` canonical ownership and runtime-only tail values;
- the typed serde semantic-tag collector and unchanged JSON wire;
- nullable empty/all-nil versus empty non-nullable behavior;
- flush/color/wipe/temperature/Bambu/plate source rules;
- exact KSR block length/hash/count evidence;
- production caller still ending at `ProjectSlicingIncomplete`;
- obsolete executable source-pinning cleanup;
- explicit remaining geometry/G-code/metadata/adapter deferrals.

Do not claim full G-code parity or Task 19C release before the exact-SHA Tier 1
run is green.

Dispatch a fresh documentation reviewer against the approved implementation
and require literal:

```text
ROLE: DOCUMENTATION
VERDICT: APPROVE
```

Revise and re-review until approved. Add the approved docs to the final
manifest and recompute all hashes.

### Step 7.1: Run the fresh post-documentation release gate

After documentation approval and final manifest/hash/complete-patch rebuild,
rerun the complete native, WASM, CLI, dependency, and browser command set. This
is the fresh release evidence used for staging; Task 6's pre-approval run does
not substitute for it.

```powershell
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --release --target wasm32-unknown-unknown
cargo +1.91.0 nextest run -p ares-cli
cargo +1.91.0 install --locked wasm-bindgen-cli --version 0.2.121
wasm-bindgen --version
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm `
  --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser audit --audit-level=low
npx --prefix crates/ares-wasm/tests/browser playwright install chromium
npm --prefix crates/ares-wasm/tests/browser test
```

Require the same exact outcomes as Step 6.3, including `wasm-bindgen 0.2.121`,
zero audit vulnerabilities, and the complete KSR golden test remaining ignored.
Record fresh counts and outputs after the documentation approval timestamp. If
any implementation fix is required, invalidate the whole implementation and
documentation approvals and rerun Tasks 6.4-7.1 before staging.

---

## Task 8: Commit, push, and exact-SHA Tier 1 release

Apply the Conventional Commits skill only after Tasks 6-7 are approved and all
fresh checks are green.

### Step 8.1: Stage only the final reviewed manifest

```powershell
git status --short
git diff --check
git add -- <exact reviewed manifest paths>
git diff --cached --name-status
git diff --cached --check
```

Confirm no ignored evidence, generated wasm/npm output, fixture/reference
change, Orca checkout change, or unrelated user file is staged.

### Step 8.2: Commit and push

```powershell
git commit -m "feat(config): serialize effective config block"
git push origin codex/ksr-fdmtest-v4-parity
```

Use a normal non-force push. If the remote advances, fetch and rebase the
reviewed commit without dropping user changes, rerun relevant verification,
and push normally.

### Step 8.3: Verify remote identity and exact-SHA Tier 1

```powershell
$local = git rev-parse HEAD
$tracking = git rev-parse origin/codex/ksr-fdmtest-v4-parity
$direct = git ls-remote origin refs/heads/codex/ksr-fdmtest-v4-parity
git status --short
```

Require local, tracking, and direct remote SHAs equal and the worktree clean.
Wait for the workflow whose `headSha` is exactly `$local`; require all five
jobs green:

- `format`;
- `ubuntu-latest`;
- `wasm`;
- `macos-latest`;
- `windows-latest`.

Only then record Task 19C as released in ignored progress evidence. The
persistent goal stays active for later typed consumer migration, geometry,
toolpaths, full document assembly, metadata/post-processing, adapter dispatch,
and complete normalized KSR G-code parity.

## Plan exit criteria

This plan is complete only when:

- the approved writer produces the committed 49,004-byte config block exactly;
- all five implementation slices have independent approval;
- whole spec compliance, quality, and OpenCode reviews all literally approve;
- documentation literally approves;
- fresh native/WASM/browser verification is green;
- the reviewed commit is pushed normally;
- local/tracking/direct remote SHAs match;
- the exact pushed SHA's five Tier 1 jobs are green;
- the persistent full-G-code goal remains explicitly open.
