# Per-extruder minimum skirt length budget SPEC

Status: **COORDINATOR-APPROVED SPEC; RED only, no GREEN authorization**.
Base: `0266b34949a6f9503002a3420e1c526cb745462b`.
Branch: `worker2/skirt-minlength-multitool`.

## Upstream rewrite boundary

Checkout `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/Print.cpp:2710-2767`,
`Print::_make_skirt::append_skirt_loops_for_hull`:

- Obtain actual `this->extruders()` and each extruder's
  `Extruder(...).e_per_mm(mm3_per_mm)` (:2712-2719).
- Start a fresh per-extruder accumulated length vector and current extruder0
  for each hull; remaining loop counter starts at `skirt_loops` (:2725-2729).
- Add each generated loop's length times the current extruder coefficient.
  When below `min_skirt_length`, extend the remaining counter only at `i==1`;
  when satisfied, advance to the next actual extruder if present (:2754-2767).
- Preserve that exact remaining-counter/advance order. Do not substitute a
  total tool-count-times-minimum threshold or always finish every tool budget
  regardless of the source loop termination condition.

This ports the upstream loop budget, not an independently designed skirt pipeline.
`GCode.cpp:4260-4278` still emits all skirt loops on the first layer extruder;
per-tool emission splitting is explicitly not implemented upstream and is NOT
part of this slice. Existing scheduling, paths and extrusion consumers stay fixed.

## Exact Rust destination

`crates/ares-core/src/project_slice/gcode_emit/skirt/loop_generation.rs`:
`LoopParameters` / `make_loops` currently carry a single `e_per_path_mm` and
scalar accumulated filament. Replace that temporary one-budget shell with
upstream's per-used-extruder accumulation and switching semantics.

`skirt/generate.rs` currently prepares only the first filament diameter/flow
coefficient. The bounded preparation must supply coefficients for actual print
extruders in source order, NOT configured filament-slot count or unused slots.
Minimal existing tool-carrier wiring, if needed, requires reporting before edits.
No new independent ordering or used-tool inference algorithm is authorized.

Included: only minimum-length generation budget preparation, per-extruder
accumulation and switching/last-iteration extension on the existing hull loop.
Witness envelope: combined skirts, ByLayer, two PLA model parts, one or two
actual logical tools, four prescribed skirt loops, skirt height2, no brim,
support, raft, tower or draft shield. Core stays browser-WASM/Windows/macOS/Linux
compatible with no filesystem/UI/terminal behavior.

Deferred: ALL other skirt behavior, per-object grouping, loop offset/spacing,
skirt emission/scheduling, angle, draft shield, repeated instances, brim,
tower geometry, flush/purge/filament-change sequencing, fan/processor/time,
infill/rotation, painted, Arachne/travel/avoid-crossing, ironing/compensation.
No fallback, new guard, feature flag, ignore/allow/expect or result constants.

## GUI input provenance and frozen matrix

Source: tracked `tests/parity/auto-brim-object-adhesion/no_brim-pla.3mf`, SHA256
`35de53e42d3622a52b8f76f8b628116f5907617d9f8cb1bb349a461323270f38`.
Keep meshes, placements, speeds, temperatures and material/flow settings fixed.
Uniform setup: skirt_loops4, skirt_height2, combined skirt, shield disabled,
combine_brims0. Vary only minimum extrusion length0/10; orthogonal controls
assign the second part to filament1 instead of2. Both filaments are PLA.
Unused configured slots stay identical across cases; only actual tool usage varies.
GUI field references: `PrintConfig.cpp:5751`, `GUI/Tab.cpp:2705`.

| Role | Archive | SHA256 |
|---|---|---|
| single-tool zero control | tools1-minimum0.3mf | `5d40f7b3764ae88c6ee5dd730b4f46841203eed5ebe62934d5464bc76175dbe8` |
| single-tool positive control | tools1-minimum10.3mf | `f621a44b830b18c21139adb91926560dc938e14b7a5410330414669605eaeb3b` |
| two-tool zero control | tools2-minimum0.3mf | `db9233ceef4f11ec285030264edd2f9231d504a0ec2c5d2e9c37af8f813e61d8` |
| target | tools2-minimum10.3mf | `cd9460b1c11e7dff39c5f1f4ffb60e9558489efd1edf7e3abb90ea3ff2a7caba` |

External discovery/generator/manifest:
`/home/indexyz/ares-coord/evidence/worker2-skirt-minlength-multitool-discovery-20261022T071537/`.
Input-only 3MFs are frozen without Native output or probes. All four must be
referenced and SHA-asserted. No geometry/position or parameter tuning for parity.

## Native-first evidence and accepted M73-leading failure

Both official and seeded Native accept all four inputs, and their complete
streams match after only generator-line normalization. Actual target uses T0/T1.
Three controls match Ares complete output in both modes. Target actual positive
Skirt XY extrusion count is Native212 versus Ares136 across two skirt layers;
first layer106/68. Summed positive skirt E is44.03276/28.77536. The Native target
changes skirt geometry from the two-tool zero control, while Ares retains it.
These are external diagnostic observations, not test result constants.

Strict first difference is normalized byte723: M73P0R133 versusR132. Coordinator
explicitly accepted this geometry-backed M73-leading failure provided all
controls pass complete bytes. No M73/time/geometry/metadata mask is permitted.
No processor/time correction belongs in this source boundary.

## SPEC → RED review gate

1. Commit this docs-only SPEC before tests.
2. Freeze the four input3MFs and use unchanged shared
   `runner::native_stream_case::check` (which owns NativeCase), without a new
   harness. It requires Native double replay then public API and actual CLI
   complete ordered generator-only comparisons plus raw API/CLI equality.
   Require actual T0/T1 via the existing flag for two-tool cases; no extra
   hard-coded Native geometry result assertions or expected-error assertions.
3. CPU13, test-threads1, canonical directed runs in both modes. Expect target
   strict FAIL and all three controls PASS; retain API/CLI status/comparisons,
   exact hashes and independent Native/API/CLI skirt geometry evidence externally.
4. Report RED RR with exact commit and STOP. No GREEN now; public tests,
   input SHA and helper/signature freeze before future authorized production work.
   Any strict residual later requires STOP rather than broadening this boundary.

No FULL before an allocated W2 window. Coordinator owns review and merge.

## Print::extruders candidate refinement and deferred witness

The budget set is `Print::extruders(false)` (`Print.cpp:453-547`,
`Print.hpp:942`), not ToolOrdering's materialized tool set. Reuse the existing
Rust `tool_ordering::collect::print_extruders`: independently deduplicate object
and support sets, append all object tools to support when either support selector
is zero, concatenate the sets without deduplicating across them, admit an explicit
tower tool when the concatenated size exceeds one, then finally sort/deduplicate.
The tower predicate ports `Print.cpp:3290-3301,3385-3388`. This same collector also
supplies nonsoluble-support candidates; no second skirt-specific collector exists.

Coordinator requested a GUI witness on production tip `6002cb666`: object uses
only filament1, default support selectors0, prime tower enabled, tower filament2,
minimum skirt length10. The single-nozzle input was rejected by the existing Ares
selector-validation boundary, which is deferred and must not be changed here.
The system Prusa XL 5T 0.4 nozzle preset with its complete inheritance chain was
then tried: five nozzles/two filaments produced Native SIGSEGV in both modes;
the final five-nozzle/five-filament attempt instead exited239 with Native
`run found error, return -17, exit...` in both modes, before Ares comparison.
Final input SHA256:
`5b18a86c8299a61dbab1657df9d67e3b88053e6fa0f20edb25e6a0688aa0d816`.
Generation, preset provenance, input and both command artifacts are retained in
external evidence `worker2-skirt-xl5-witness-20261022T153323`; earlier evidence is
`worker2-skirt-support-default-red-20261022T151030` and
`worker2-skirt-multinozzle-witness-20261022T152613`.

These runs are not a budget RED and establish neither output equality nor a
budget difference. Further toolchanger witness diagnosis is deferred. Coordinator
explicitly authorized accepting this refinement as a literal source port with
the frozen four-case directed regressions and fresh dual-mode FULL, without a
new RED witness. Temporary witness tests/inputs are removed from the branch;
existing tests, fixtures and the shared helper remain frozen. No Native crash,
selector-validation or toolchanger behavior is repaired by this slice.

