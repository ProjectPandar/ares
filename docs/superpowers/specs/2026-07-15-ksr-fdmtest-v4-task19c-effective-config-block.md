# Task 19C: Exact Effective Config-Block Serialization

## Status and objective

This specification is a draft until its frozen bytes receive the independent
review approvals required below.

Task 19C is the next bounded slice of the approved
`ksr_fdmtest_v4` parity program. It ports OrcaSlicer's effective Bambu G-code
configuration block from the final typed project configuration produced by
Task 19B.3. It does not slice geometry, emit executable toolpaths, assemble a
complete G-code document, or make the public project API return partial output.

After a valid project has resolved and its configuration block has been
serialized successfully, `slice_project` must still return
`ProjectSlicingIncomplete`. The persistent goal remains byte-for-byte parity
with `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode` after normalizing only the
allowed generated-by product name and timestamp metadata.

### Pre-implementation review contract

The independent approvals that freeze this specification are design reviews
performed before a Task 19C implementation plan or implementation exists.
Reviewers must judge whether the behavior is source-faithful, complete within
its boundary, implementable through the named typed destinations, and covered
by acceptance criteria that reject a wrong implementation.

A `REVISE` verdict must identify a defect in this specification, such as an
incorrect fixed-source claim, missing serializer behavior, ambiguous source
ownership, unsafe API boundary, dishonest deferral, or acceptance criteria
that cannot distinguish the required bytes. Implementation conformance is a
later review gate.

## Corrections to the older aggregate plan

The older aggregate Task 19C text proposed
`write_config_block(&FullPrintConfig, ...)`. Ares has no separate 653-field
`FullPrintConfig`, and Task 19B.3 explicitly rejected creating one.

`ProjectSettings` is the canonical concrete typed representation of the 650
real project options plus three preset metadata fields. Task 19B.3 returns the
final configuration through `ProjectConfigViews`:

- `views.full` is the equivalent of `print.full_print_config()` and owns the
  canonical option lines;
- `views.runtime` is the equivalent of the effective runtime `PrintConfig` and
  owns the two computed first-layer temperature lines;
- `views.runtime_gcode` is a smaller consumer projection and is not an export
  source.

Task 19C must not build a second flat config struct, serialize
`ProjectSettings` as project JSON, or round-trip through JSON to obtain G-code
tokens. It reuses the existing compile-time field/key ownership in the four
flat typed option-group serializers while deliberately excluding
`ProjectSettings::metadata`.

The existing STL-only `gcode_config_header` and
`filament_config_export` paths are compatibility scaffolds, not the Task 19C
destination. They remain unchanged except that the already-correct Orca
string-vector escape primitive may be moved to a shared typed helper and its
existing behavior tests retained.

## Fixed upstream rewrite boundary

The baseline is OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/Print.cpp:2618-2638` enters G-code export.
- `src/libslic3r/GCode.cpp:2030-2095` establishes the C numeric locale and
  printer classification before export.
- `GCode.cpp:2461-2534` prepares the runtime G-code configuration.
- `GCode.cpp:2637-2658` writes the Bambu config-block markers, full-config
  body, two computed temperature lines, end marker, and final blank line.
- `GCode.cpp:5591-5644` implements `GCode::append_full_config` from a local
  clone of `print.full_print_config()`.
- `src/libslic3r/Config.cpp:543-548` delegates `ConfigBase::opt_serialize` to
  the concrete `ConfigOption` serializer.
- `Config.cpp:1715-1721` and `Config.hpp:2924-2925,2963` establish lexical key
  order through the dynamic config's ordered map.
- `Config.hpp:281-284,832-837,1010-1015,1873-1878` define nullable `is_nil`:
  a nullable vector is nil only when every entry is nil; the empty nullable
  vector is therefore nil. Nullable enum vectors at `Config.hpp:2100-2139`
  inherit that implementation from the nullable integer-vector base.
- `Config.hpp:624-627` defines vector `get_at`: a non-empty vector returns the
  requested element when present and otherwise falls back to its first item.
- `PrintBase.hpp:517-518,558` makes plate index external print context with a
  zero default.
- `PrintConfig.hpp:489-509` maps each concrete bed type to its first-layer bed
  temperature vector.
- `Config.cpp:48-120` and `Config.hpp:764-920,954-1157,1165-1417,
  1452-1528,1605-1698,1809-2194` define scalar, vector, nullable, string,
  percent, point, bool, and enum serialization.
- `src/OrcaSlicer.cpp:6045-6060` is the fixed CLI boundary analogous to Ares'
  byte API: non-empty `printer_model` values whose first nine characters are
  `Bambu Lab` set the Bambu printer flag. The unavailable printer-name fallback
  is not recreated.

The upstream config block has no generated timestamp metadata. The allowed
generated-by product/timestamp difference belongs to `GCode.cpp:2574-2576`
and later document assembly, not this task. Estimated print times are not
timestamp metadata and do not become allowed differences.

## Rust destination and interface

The implementation adds a small crate-private typed export boundary under
`crates/ares-core/src/options/config_export.rs` with siblings split by
collector, value serialization, fixed transforms, and writer responsibility as
needed to keep every Rust file below 400 physical lines.

The crate-private interface is equivalent to:

```rust
pub(crate) fn is_bambu_project(settings: &ProjectSettings) -> bool;

pub(crate) fn write_config_block(
    views: &ProjectConfigViews,
    plate_index: usize,
    output: &mut Vec<u8>,
) -> Result<(), SliceError>;
```

Equivalent private nesting and names are allowed, but the interface contract
is fixed:

1. The caller, not the serializer, supplies a zero-based plate index.
2. The complete writer includes `CONFIG_BLOCK_START`, all assignments, the two
   computed temperatures, `CONFIG_BLOCK_END`, and its following blank line.
3. The canonical body reads only `views.full`.
4. The two temperature lines read only `views.runtime`.
5. The implementation creates no public API and exposes no partial project
   output.
6. On error, it must not append a partial block to the caller's output buffer.

`is_bambu_project` performs the exact available CLI test:
`settings.printer.remaining.printer_model.0.starts_with("Bambu Lab")`.
It is case-sensitive and does not inspect fixture names, hashes, reference
G-code, printer settings IDs, or unavailable preset objects.

`project_slice.rs` calls the final Task 19B.3 resolver, applies the Bambu test,
and, for Bambu projects, writes the block to an internal scratch buffer with
the source default plate index `0`. The scratch buffer remains owned by the
incomplete production pipeline. A successful valid project still returns
`ProjectSlicingIncomplete`; a real config-export error is returned first.
Non-Bambu projects skip this Bambu-only writer and retain the same incomplete
boundary.

The writer accepts non-zero indices in focused tests. A future plate-selection
API may pass a selected index, but adding or guessing that public API is outside
Task 19C.

## Canonical typed entry projection

### Ownership and inventory

Collect exactly these four existing flat typed groups from `views.full`:

- `PrinterOptions`: 132 options;
- `ProcessOptions`: 352 options;
- `FilamentOptions`: 122 options;
- `ProjectRuntimeOptions`: 44 options.

The resulting 650 entries must have 650 unique keys before nil omission. Do not
collect `PresetMetadata::{from,name,version}`. Do not implement
`Serialize for ProjectSettings` merely to flatten these owners.

The existing group `serde::Serialize` implementations are the single
compile-time inventory of key literals and concrete field paths. Task 19C
reuses those four map serializers through a crate-private custom serializer and
then globally sorts a transient `Vec` of serialized entries by key. It must not
create a runtime option registry, `serde_json::Value`, JSON object, dynamic
config map, reference-derived entry table, or a second 650-field access list.

The transient entry representation may erase a value only after its concrete
typed serializer has produced the final config token and nullable state. It
must not support mutation, lookup-based type recovery, or reconstruction of
typed options.

### Explicit semantic tags

Serde sequence shape alone cannot distinguish every Orca option kind. The
implementation therefore uses internal `serialize_newtype_struct` semantic
tags that `serde_json` treats transparently but the config collector consumes
explicitly. Tag names are shared constants; the implementation must not use
`type_name`, key-based kind lookup, token inspection, or fallback guessing.

Required tagged categories are:

1. **ConfigOptionStrings**: `OrcaStrings`, `AmsCounts`,
   `RammingParameters`, `CsvTable`, `SpaceTuple`, `VariantStride`, and
   `ExtruderVariantLists`. These are semicolon-separated string vectors.
2. **ConfigOptionPointsGroups**: `Point2dGroups`. Groups use `#`, points inside
   a group use comma, and coordinates use `x`.
3. **ConfigOptionNullableVector**: all 31 nullable-vector fields. The four
   named nullable wrapper fields receive a transparent manual serializer. The
   27 bare `Vec<Nullable<T>>` fields use a crate-private borrowed wrapper at
   their existing top-level wire entries. This tag is required even for an
   empty vector.
4. **ConfigOptionNil**: `Nullable::Nil` marks a nil element without inferring
   nil from the literal text. `Nullable::Value` delegates to its concrete value.

`Point2dList`, enum vectors, numeric vectors, bool vectors, percentages, and
`FlatMatrix` remain ordinary comma-separated sequences. Scalar
`serialize_str` events are ConfigOptionString/scalar tokens and apply Orca's
C-style escaping; unit enum variants emit their exact renamed token without
string-vector quoting. Special string-vector collection receives raw elements
and applies its own quoting/escaping exactly once.

All changed wrapper serializers must preserve their existing `serde_json`
wire bytes. Focused JSON golden tests plus the existing full typed wire suite
must prove that the semantic tags are invisible to project JSON serialization.
This task makes no promise about unrelated non-JSON serializers.

### Exact value serialization

The collector covers every current option kind without an unsupported or
catch-all branch:

| Kind | Count | Config token |
| --- | ---: | --- |
| Bool | 105 | `1` or `0` |
| Bools | 22 | comma-separated, nullable entries may be `nil` |
| Enum | 44 | exact case-sensitive renamed token |
| Enums | 9 | comma-separated, nullable entries may be `nil` |
| Float | 160 | C-locale defaultfloat, six significant digits |
| FloatOrPercent | 36 | float token or float token plus `%` |
| Floats | 90 | comma-separated, nullable entries may be `nil` |
| Int | 41 | decimal integer |
| Ints | 45 | comma-separated, nullable entries may be `nil` |
| Percent | 25 | numeric token plus `%` |
| Percents | 5 | comma-separated, nullable entries may be `nil` |
| Point | 4 | `x,y` |
| Points | 6 | comma-separated `x`-joined coordinate pairs |
| PointsGroups | 1 | `#` groups, comma points, `x` coordinates |
| String | 30 | C-style escape with no outer quotes |
| Strings | 27 | semicolon elements with conditional quotes/escape |

The existing typed scalar serializers and `format_number` remain authoritative
for bool, int, float, percent, float-or-percent, and enum tokens. Do not use
JSON number text, `Debug`, fixed-decimal formatting, locale-sensitive parsing,
or parse a serialized token back into a number.

Scalar strings escape carriage return, newline, backslash, and double quote,
without adding outer quotes. String-vector elements are separated by `;`.
An element is quoted when it contains a space, tab, backslash, quote, carriage
return, or newline; a sole empty element is also quoted. Empty elements in a
multi-element vector are not independently quoted. Empty non-nullable vectors
remain present as `; key = \n`.

An empty nullable vector and a nullable vector whose every value is nil are
omitted. A mixed nullable vector remains present and preserves every literal
`nil` position. The implementation must not omit an empty non-nullable vector.

### Thumbnail canonicalization prerequisite

The committed project stores the current `thumbnails` value as
`48x48/PNG,300x300/PNG`, and the committed Orca config block retains that exact
scalar token. Pinned Orca declares the option with the same no-space default in
`PrintConfig.cpp:7122-7127`; `ConfigOptionString::serialize` in
`Config.hpp:1087-1110` escapes but otherwise preserves the stored scalar, and
the generic loop in `GCode.cpp:5631-5639` has no thumbnail key special-case.

Ares' earlier typed legacy composite currently rebuilds every present
multi-item thumbnail value with `", "`. Pinned Orca's JSON-load composite also
uses that intermediate spelling in `PrintConfig.cpp:8290-8316`, but the fixed
fixture's final effective option and generated block prove that the value
reaching this writer is the no-space spelling. Ares does not reproduce the
same preset refresh/load ordering, so its typed option boundary must
canonicalize multiple definitions with `","` to produce the same final
effective value.

This correction belongs in `options/typed_legacy/thumbnails.rs`, with focused
typed-legacy expectations updated for arbitrary multi-item definitions. It is
not a `thumbnails` branch in the config writer, is not fixture-specific, and
must preserve parsing, validation, format completion, case normalization,
ordering, duplicates, and JSON wire behavior.

## Fixed config transforms and output order

### Flush matrix

Clone `views.full` for export and transform only the clone's typed
`flush_volumes_matrix` before collection:

1. `heads = flush_multiplier.len()`.
2. `filaments = filament_colour.len()`.
3. When `filaments * filaments * heads == matrix.len()`, divide the matrix into
   one equal contiguous segment per head, multiply each segment by its matching
   multiplier, and apply `f64::round` to each product.
4. When the size does not match and `filaments == 1`, leave the matrix unchanged.
5. When the size does not match and `filaments != 1`, return
   `SliceError::InvalidInput("Flush volumes matrix do not match to the correct size!".to_owned())`.
6. A zero head count cannot be divided into segments and returns the same
   external-input error. Fixed Orca divides by zero before its size guard in
   this invalid state; returning the existing mismatch error is the deliberate
   safe Rust translation of that upstream undefined behavior, not a claim that
   Orca returns cleanly.
7. Never mutate `views.full` or parse/replace an already serialized matrix.

The KSR transform is:

```text
multiplier: 0.3,1
source:     0,280,280,0,0,280,280,0
output:     0,84,84,0,0,280,280,0
```

### Canonical loop

Globally sort the 650 collected entries lexically by key. Reject duplicate
canonical keys in focused tests; the four current typed owners must be unique.

For each sorted entry:

1. Skip the fixed nine-key upstream banned set:
   `compatible_printers`, `compatible_prints`, `print_host`,
   `print_host_webui`, `printhost_apikey`, `printhost_cafile`,
   `printhost_user`, `printhost_password`, and `printhost_port`.
   The current typed schema contains none of them, but the source rule remains
   active and is covered through a synthetic entry-level test.
2. Skip a nullable value only when its tagged vector is empty or all nil.
3. For `wipe_tower_x` and `wipe_tower_y`, first write the selected typed value
   with exactly three digits after the decimal point. When the supplied index
   is out of range, use the vector's first item. Then continue to its ordinary
   canonical vector line; the two `if` statements are deliberately not an
   exclusive chain.
4. For the `extruder_colour` key, retain that key but write the already-typed
   serialized value of `filament_colour`.
5. Write every other canonical entry as `; {key} = {token}\n`.

No special branch may use fixture identity, reference bytes, expected option
values, or the KSR file name. Only the four fixed upstream key behaviors above
may dispatch on a key.

### Runtime temperature tail and delimiters

After the canonical loop, select the first-layer bed vector from
`views.runtime` according to `curr_bed_type`:

- Supertack -> `supertack_plate_temp_initial_layer`;
- Cool -> `cool_plate_temp_initial_layer`;
- Textured Cool -> `textured_cool_plate_temp_initial_layer`;
- Engineering -> `eng_plate_temp_initial_layer`;
- High Temp -> `hot_plate_temp_initial_layer`;
- Textured PEI -> `textured_plate_temp_initial_layer`.

Use element zero for the computed bed temperature and element zero of
`nozzle_temperature_initial_layer` for the computed nozzle temperature. A
Bambu runtime configuration with `Default Plate` or an empty required
temperature vector is invalid external input, not a fallback to a hardcoded
temperature.

The complete byte order is exactly:

```text
; CONFIG_BLOCK_START\n
<sorted canonical lines, including x/y duplicate lines>
; first_layer_bed_temperature = <integer>\n
; first_layer_temperature = <integer>\n
; CONFIG_BLOCK_END\n
\n
```

All line endings are LF. `GenerationMetadata` is not read by this writer.

## Committed fixture acceptance contract

The test reads only the committed fixture 3MF and committed reference G-code.
Production code must not read either test path or reference file.

Before comparing bytes, the test independently asserts that the extracted
reference block:

- includes both markers and the blank line after `CONFIG_BLOCK_END`;
- is 49,004 bytes and LF-only;
- has SHA-256
  `b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`;
- contains 639 assignment lines and 637 unique assignment keys;
- contains exactly two `wipe_tower_x` and two `wipe_tower_y` lines;
- ends its assignments with `first_layer_bed_temperature = 55` and
  `first_layer_temperature = 220`;
- contains the two KSR x lines `165.000` then `165` and the two y lines
  `220.096` then `220.096`;
- omits `from`, `name`, and `version`;
- omits the 15 fixture all-nil nullable fields:
  `filament_deretraction_speed`, `filament_ironing_flow`,
  `filament_ironing_inset`, `filament_ironing_spacing`,
  `filament_ironing_speed`, `filament_long_retractions_when_cut`,
  `filament_retract_before_wipe`, `filament_retract_lift_above`,
  `filament_retract_lift_below`, `filament_retract_lift_enforce`,
  `filament_retract_restart_extra`,
  `filament_retract_when_changing_layer`,
  `filament_retraction_minimum_travel`, `filament_retraction_speed`, and
  `filament_z_hop`;
- retains empty assignments for `bed_exclude_area`, `head_wrap_detect_zone`,
  `parallel_printheads_bed_exclude_areas`, `post_process`, and
  `wrapping_exclude_area`.

Then resolve the fixture through the production Task 19B.3 boundary, write the
block with the production Task 19C writer, and compare the complete bytes with
no normalization.

The fixture test also freezes six full-vs-runtime sentinel lines that must come
from `views.full`: `deretraction_speed`, `retraction_distances_when_cut`,
`retraction_length`, `retraction_speed`, `wipe_distance`, and `z_hop_types`.
It separately proves that the two temperature tail values follow runtime
fields.

## Focused behavioral acceptance

Focused tests must cover at least:

1. All four groups collect to 650 unique canonical entries with no unsupported
   serde event and without metadata.
2. Existing JSON serialization bytes are unchanged for every newly tagged
   wrapper and for a complete project settings fixture.
3. Bool, int, float, percent, float-or-percent, enum, ordinary vectors,
   scalar C-style strings, string-vector quoting, points, point lists, and point
   groups serialize exactly, including negative zero and scientific notation.
4. Empty non-nullable, empty nullable, all-nil nullable, and mixed nullable
   vectors have distinct required behavior.
5. Flush scaling handles multiple heads, per-segment rounding, source
   immutability, the single-filament mismatch exception, the multi-filament
   mismatch error, and zero heads.
6. `extruder_colour` uses typed `filament_colour` without mutating either
   source field.
7. Non-zero wipe coordinate selection, out-of-range first-element fallback,
   exact three-decimal formatting, ordinary vector fallthrough, and duplicate
   line order are all frozen.
8. Each of the six concrete bed types selects its typed first-layer vector;
   nozzle and bed use their first values; Default Plate and empty required
   vectors fail without hardcoded fallback.
9. Bambu model-prefix classification is exact and case-sensitive; a non-Bambu
   project does not enter the Bambu writer.
10. The production project caller reaches config serialization after final
    resolution, returns a real serialization error before incomplete, and
    still returns `ProjectSlicingIncomplete` for a valid fixture.

## Obsolete source-pinning test cleanup

The user-requested cleanup is part of this task because the remaining pinning
is in the same 653-field project inventory used by Task 19C.

In `crates/ares-core/src/options/tests/project_inventory.rs`, remove:

- `InventoryRow::upstream_definition` and `upstream_consumers`;
- `LegacyInput::citation`;
- the `SourceCitation` test-only struct;
- assertions that only check source path, line number, symbol, or consumer
  citation presence.

Retain behavioral inventory coverage: counts, ownership, option kinds,
defaults, projections, wire shapes, legacy keys/conversions, config-export
rules, and fixture key/shape agreement. The extra citation members in the
committed JSON evidence may remain ignored by serde; production and tests must
not consume or assert them. Do not delete behavioral `registry_lookup_*` tests
or upstream-derived truth tables merely because their names mention upstream.

No new source checkout, source-line, symbol-name, or pinned-source file test is
allowed. Fixed source citations remain in this reviewed design document, not
as executable tests.

## Explicit deferrals

Task 19C does not implement:

- geometry slicing, layer generation, toolpaths, G-code templates, cooling,
  post-processing, statistics, thumbnails, or complete document assembly;
- the generated-by Ares/version/timestamp line;
- non-Bambu thumbnail/config/footer branches;
- a public plate-selection API or multi-plate document selection;
- CLI project dispatch or enabling the ignored full golden test;
- deletion or migration of the legacy STL `SliceOptions`,
  `gcode_config_header`, or dynamic config consumers;
- remaining Tasks 20A-20E consumer migration;
- any reference-G-code access from production code;
- a compatibility fallback when typed project data is absent.

## Verification and review gates

Implementation may begin only after this specification and its implementation
plan each receive literal `VERDICT: APPROVE` from both the independent reviewer
agent and the required OpenCode reviewer.

Implementation follows Subagent-Driven Development. Each plan slice starts
with a focused RED test, is implemented by a bounded subagent, and receives an
independent spec/quality review before the next dependent slice. The complete
implementation then receives fresh independent agent and OpenCode spec-
compliance approvals before documentation is updated.

Fresh release verification must include:

```powershell
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 build -p ares-wasm --release --target wasm32-unknown-unknown
```

The existing CLI, WASM Rust, wasm-bindgen, npm audit, and browser project tests
must also be rerun by the final approved plan. Every changed Rust file must
remain below 400 physical lines. Static audits must prove:

- no new `serde_json::Value`, JSON round-trip, runtime registry lookup, or
  dynamic-map path under the Task 19C production modules;
- no production access to `tests/ksr_fdmtest_v4`, reference G-code, fixture
  names, hashes, or expected values;
- no source-pinning test fields/assertions described above;
- no `allow(dead_code)` or temporary lint suppression added for Task 19C;
- no expansion of the legacy STL config-header pipeline.

## Documentation, commit, push, and release

Only after the full implementation receives all required literal
`VERDICT: APPROVE` decisions:

1. Update `docs/architecture/option-parity-v4.md` with the shipped typed
   config-export boundary, full/runtime ownership, semantic tags, exact fixture
   evidence, and remaining deferrals.
2. Update `docs/roadmap.md` with Task 19C completion evidence and the next
   source-cited parity slice.
3. Independently review the documentation until literal
   `VERDICT: APPROVE`.
4. Run the fresh release verification above.
5. Stage only the reviewed Task 19C manifest and create a Conventional Commit,
   expected as `feat(config): serialize effective config block`.
6. Push the current branch normally, verify local/tracking/direct-remote SHA
   equality and a clean worktree, then require the exact pushed SHA's Tier 1
   `format`, Linux, WASM, macOS, and Windows jobs all green.

Task 19C release still leaves geometry, executable G-code generation, document
assembly, adapters, metadata/post-processing, and final normalized
`ksr_fdmtest_v4` golden parity open. Do not mark the persistent goal complete
at this milestone.
