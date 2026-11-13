# Project filament refresh and variant layout

Port the retained-project filament path of OrcaSlicer
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, not a new slicing pipeline.
The Rust starting point is `1dc0f6d7a71d68e20debb99d607272d696e85460`.
This document defines the source boundary and tests before implementation.

## Owning source

The CLI orchestrates library-owned configuration operations:

| Source | Behavior | Rust boundary |
|---|---|---|
| `OrcaSlicer.cpp:2774–2820` | Compute `refresh_unlisted`; retain project keys except `filament_extruder_variant`; resolve unreplaced filament slots and retain slot order | `profiles/project_refresh/preset.rs::refresh` |
| `PresetBundle.cpp:403–482,724–762` | Exact manifest-name system lookup and resolved preset configuration; an optional lookup miss retains its project slot | Existing supplied-catalog/system-only resolver |
| `OrcaSlicer.cpp:3510–3577` | Establish old variant ranges; regenerate incompatible `filament_self_index`; normalize options to the old layout | `profiles/project_refresh/filament.rs` |
| `PrintConfig.cpp:11400–11428` | `normalize_filament_values_to_variants` and source index mapping | Existing raw-settings variant normalization |
| `OrcaSlicer.cpp:3577–3813` | Per-slot protected/unprotected merge, variant-string matching, range replacement, later range offsets | `profiles/project_refresh/filament/slot.rs` and its existing range helper |
| `Config.hpp:542–583` | Typed `ConfigOptionVector::set_with_restore_2`: source padding when enabled, saved-value restoration, typed insertion | Existing `replace_variants` helper, corrected only where a source comparison demonstrates a difference |
| `OrcaSlicer.cpp:3815–3852` | Reassemble device options and regenerate self indices from resulting per-filament variant counts | Existing filament refresh completion |
| `PrintConfig.cpp:11430–11444` | `set_filament_dev_options`: append supplied vectors in filament order, use definition defaults for absent keys, omit empty vectors | Existing device-option loop |

`Preset::load_external_config` (`Preset.cpp:1090–1119`) and the retained
machine/process calls (`OrcaSlicer.cpp:3164–3200`) remain the owners of missing-key
system inheritance. They are controls here, not authorization to redesign their
resolver or to change definition defaults.

## Non-negotiable boundaries

1. This is the CLI refresh policy: `{"from_project":true}` and optional
   `uptodate`. Its public settings result remains flat. Preserve
   `uptodate || any non-empty difference entry` exactly. Empty entries do not
   establish that all imported values equal their parents.
2. `load_and_select:"always"` retains the separate single-filament FFF contract
   in `project-preset-selection-always.md`, including its result shape and
   multi-filament rejection. This slice does not implement GUI multi-filament
   `Never`/`OnlyIfModified`, embedded-preset import or browser state changes.
3. Core consumes supplied catalog/project bytes. It performs no file access,
   resource discovery, installation or terminal behavior. The Native resolver's
   resource-backed entry is observed in tests; it is not recreated as core
   filesystem search. Optional misses preserve existing behavior; malformed
   found presets and unresolved parents are not converted into misses.
4. Variant strings define layout; protecting project values does not freeze
   `filament_extruder_variant`. Preserve vector type, order, length, nils and
   per-filament boundaries. Do not replace source restoration with a flat map
   merge or infer one variant per slot from the number of colors alone.
5. User-provided projects are local-only diagnostics. All committed cases are
   independently synthetic. Do not change their slots, insert G-code markers,
   remove waits, hard-code printer names or change comparators/tolerances.

## Existing compatibility shell and concrete questions

`project_refresh/filament.rs` already represents optional slot configs, builds
old groups, matches variant strings, applies slots and assembles device vectors.
This is a temporary approximation of the cited source operations, not an empty
module to replace wholesale. Reuse behavior already matching the source.

Source comparisons must first address these questions:

- The current shell pads short self-index lists and groups contiguous IDs.
  Native additionally regenerates indices when their range/cardinality is
  incompatible with the filament count. A two-slot project carrying `[1,1]`
  must exercise that distinction before changing the grouping code.
- The source removes variant metadata from the keep-project keyset. The shell's
  general keep predicate currently tests project-key presence. Determine whether
  resulting values or layout differ; do not manufacture a code change when the
  existing string-based restoration already produces the source result.
- Device options are concatenated source vectors, not a fixed one-value-per-slot
  array or a Boolean reduction. Check empty vectors, defaults and an unresolved
  slot. With refresh enabled, incomplete source coverage must not replace the
  existing global device values; without it, only absent project keys are filled.
- A missing machine option may already be supplied correctly by the retained
  machine path. Establish its effective public input on this starting tree;
  historical missing-key observations alone do not establish a new regression.

The synthetic stale-index reproduction uses two protected flow values
`[0.91,0.92]`, system values `[1.02,1.04]`, identical variant names and imported
self indices `[1,1]`. Native's range/cardinality predicate at
`OrcaSlicer.cpp:3534–3547` regenerates indices to `[1,2]` before saving old ranges.
The current Rust shell instead groups both old values into the first slot,
leaves an empty second range and replaces the protected second value with
`1.04`. The public settings comparison observes `[0.91,1.04]` rather than Native's
`[0.91,0.92]`. Valid `[1,2]` indices are an unchanged passing control. The intended
correction is the source's regeneration predicate before grouping, not a change
to protected-key handling or numeric comparisons.

No production change is justified until a source-owned difference has a runnable
public-API reproduction and a real Native result for the same synthetic inputs.
Passing controls remain controls rather than being relabeled as failures.

## Test-first plan

Use ordinary synthetic FFF projects with two distinct filament identities,
colors and values, plus a synthetic printer/process/system catalog. Keep every
case's complete archive, catalog, Native export, API result and producer receipts.

| Cases | Required comparison |
|---|---|
| Two slots; printer Boolean absent versus explicitly false; system value true | Missing-key inheritance versus retained authored value; no default change |
| All-empty differences, a non-empty process entry, a protected filament entry, explicit `uptodate` | Global refresh predicate and exact per-slot protection |
| Both slots resolved versus one optional miss | Imported missing slot retained; resolved slot still applied; no guessed aliases |
| Unequal old/new variant counts and reordered variant names; protected nullable and numeric vectors | Complete typed vector restoration, range shifts and nil preservation |
| Stale `[1,1]`, short/absent self indices, and incompatible index ranges | Native regeneration and resulting full self-index/variant vectors |
| Device vectors with multiple values, empty values, missing keys and partial source coverage | Exact append/default/presence semantics, not flattened slot assumptions |
| Explicit public overrides and loaded placement/model settings | Overrides last; identities, colors, model/plate state and unrelated values preserved |
| Separate Always request and actual CLI adapter controls | Existing single-filament Always shape/boundary and CLI protocol unchanged |

First observe actual Native CLI `--export-settings` after its original retained
filament merge. The synthetic resource index must reach the original
`find_preset_vendor`, `resolve_system_preset` and CLI merge; a test-side resolver
or a copied C++ merge is not an oracle. Confirm a two-slot smoke before adding
regression tests. If the sealed launcher cannot route a synthetic resource root,
obtain an explicitly authorized observational entry before proceeding; do not
substitute unrelated stock configurations or user-derived input.

Synthetic machine entries must carry `printer_model`, `printer_variant` and a
matching vendor machine-model/variant record: `PresetBundle.cpp:7867–7898`
rejects the entire vendor load otherwise. A manifest-name match is not proof of
successful resolution. Reject resolver-load errors in the Native premises.
With a valid synthetic vendor, the two-slot missing `enable_pre_heating` control
inherits system `1` with either empty or nonempty difference metadata; an
explicit authored `0` stays `0` with empty metadata. These are controls for the
existing machine path, not a reason to change its implementation.

The source device-vector comparison also covers JSON deserialization before
assembly. `Config.cpp:887–918,1008–1060` serializes a JSON array for
`set_deserialize`; `ConfigOptionFloatsTempl::deserialize`
(`Config.hpp:933–940`) maps the empty string to the one-element vector `{0}`.
An empty float JSON array therefore contributes `0`, not no elements, for
all `coFloats` and `coFloatsNullable` keys through this loader. This is not a
device-key-specific rule or a change to `set_filament_dev_options`' internal
empty-vector skip behavior. The existing shared Rust wire decoder already
implements it; raw normalization in `options/json_deserialize.rs` bypasses
that decoder for empty non-nullable float arrays and maps all bare empty
vector strings to empty lists. Before changing that shared owner, inventory
the affected raw entry points and existing parity inputs. Do not implement a
device-key whitelist around the temporary raw-config shell.

Native observations may have typed envelopes while the public API has flat
values. Decode the observation at that boundary while retaining complete vector
length/order/value/type checks. A public core request using CLI policy is not an
actual CLI subprocess; keep those checks and their artifacts distinct.

Record unchanged-starting-tree runtime failures before the corresponding code
change. A setup/build failure is not such a failure. Keep base and candidate
Cargo output directories distinct; pin Native to CPU60 with a receipt before
ELF execution, and keep compilation/test work on CPUs60–63 with four jobs/threads.
Use locked/offline Cargo commands and `cargo nextest run` for directed cases.

After a demonstrated literal correction, compare both official and seeded modes,
existing refresh/miss/replacement controls, strict workspace/all-target Clippy,
formatting and ordinary WASM builds. Record inherited failures separately rather
than changing unrelated assertions. Do not run the entire parity inventory for
this slice.

## Included and deferred

Included: completion of the cited retained-project multi-filament refresh,
variant-layout/self-index and device-option behavior, with missing-key controls
and preservation of public project data. Keep changes within the existing core
boundary and split files only if their size requires it.

Deferred: explicit replacement features beyond necessary unchanged controls,
new discovery/resolver policy, GUI multi-filament selection, published projects,
SLA, persistence, adapters/UI, geometry, G-code generation, scheduling and timing
algorithms. An effective-input correction is not proof that generated commands,
end-code timing or total cost are equal; any residual difference retains its
separate source owner.
