# G4 other-layer sequence-count bounded coverage

Status: SPEC-REQUEST only; no Native admission, RED, implementation or classification promotion yet.
Base: merged G4 Manual-map `ee277389dfa0fe8664caa2bf490ddf0cdda3d362`.
This is a narrower next vertical under the approved six-key G4 source boundary,
not a new Ares tool-ordering pipeline.

## Selection and non-negotiable ownership

Choose only `other_layers_print_sequence_nums`. Its scalar count and coherent
plate vector already have Rust importer/consumer owners; a one-nozzle fixture
avoids dual-nozzle variant reconstruction, dataset resources, Type2 ramming
and sequential-object masks. Master preference needs observable automatic
grouping; self-index needs variant reconstruction; dataset needs resource/matrix
reconstruction; ramming needs active Type2 SEMM; by-object masks need the
sequential-object lifecycle (`GCode.cpp:3297-3316`). Those five alternatives
remain deferred, not silently credited through this simpler fixture.

No worker5 `--load-settings`, profile overlay/application-stack work, new CLI
flags or worker1 PCP/string serialization changes. No shared Native rebuild,
new Native oracle, speed/time/travel/restate changes, or by-object mask port.

## Source-cited rewrite boundary

Pinned upstream v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
paths below are under `OrcaSlicer/src`. Rust production destinations in the
table are under `crates/ares-core/src`; harness destinations are separately
listed under `crates/ares-cli/tests/orca_parity/option_coverage`.

| Upstream owner | Included behavior / Rust destination |
|---|---|
| `libslic3r/PrintConfig.cpp:1115-1123` | `other_layers_print_sequence` coInts default0 and count coInt default0. Count has no schema maximum proving an arbitrary domain. Typed owners: `options/project_runtime_options/print_source.rs` and registry; no schema redesign. |
| `libslic3r/ParameterUtils.cpp:6-29` (`get_other_layers_print_sequence`) | Count0 OR empty vector produces no ranges; otherwise divisibility and item size>2 are prerequisites. Split into equal-width rows, first two entries inclusive one-based layer bounds, remaining entries one-based filament IDs. Existing Rust: `project_slice/gcode_emit/tool_ordering/reorder.rs::custom_sequences`. |
| `libslic3r/GCode/ToolOrdering.cpp:1320-1362` (`reorder_extruders_for_minimum_flush_volume` / `get_custom_seq`) | Layer index+1 matches inclusive range; reverse traversal means last matching row wins. Settled first layer remains locked when `!reorder_first_layer`; non-BBL maps are grouped as physical0. Custom sequence is passed to the existing minimum-flush ordering algorithm, not an independent scheduler. Rust: `tool_ordering/reorder.rs` plus `reorder/groups.rs`. |
| `libslic3r/GCode/ToolOrdering.cpp:498-513` (`apply_first_layer_order`) | First-layer configured order compares **zero-based** tool IDs. Keep this dependency distinct from the one-based other-layer rows; Rust: `tool_ordering/first_layer.rs::apply_configured_order`. |
| `libslic3r/Format/bbs_3mf.cpp:344-346,4428-4435,7943-7957` | Plate metadata stores/reads first-layer sequence, other-layer vector and count separately. Rust: `project/plate_config.rs::apply_plate_config_overrides`; test binding only changes these requested owner fields. |
| `OrcaSlicer.cpp:5979` | Native applies plate config over print config. This is an upstream source citation, not authorization to change CLI settings/overlay behavior. Same bound ZIP goes to both slicers. |

`first_layer_sequence_choice` / `other_layers_sequence_choice` enum definitions
are `PrintConfig.cpp:1125-1147`. Retain Customize as coherent fixture metadata,
but do not pretend that enum text proves consumption: the cited runtime owner
reads the plate vector/count. GUI choice conversion is outside this slice.

Existing Ares imports and custom-sequence code are compatibility shells around
these upstream owners. Initial implementation authorization is test/harness
only. If live Native admission exposes an Ares consumer defect, preserve that
RED and request an exact source-boundary production extension before repair.
Do not rewrite `solve.rs`, create a parallel sequence scheduler or normalize
away a mismatch.

## Finite context and canonical requests

One Ender-3 0.4mm nozzle, relative extrusion, SEMM, two distinguishable PLA
logical filaments (colours differ, temperatures211/233), Manual physical map
`[1,1]`, coherent Standard selector, baseline identity `[1,2]` and four-cell
supplied flush matrix. Identity/variant classification is not promoted here.
No support, raft, skirt or prime tower; cube10,
layer0.2mm, nonzero sparse fill40%. Canonical role IDs assign outer/inner wall1,
sparse/internal-solid/top/bottom filament2; remove conflicting legacy role
aliases rather than crediting a silently overwritten request. Both filaments
must actually extrude on the inspected layers. No Type2 ramming dependency.
Keep real logical changes; no fake tool IDs or mask markers.

Bind first-layer sequence `[0,1]` at its owner, preserving zero-based semantics.
Count/vector canonical cases are paired, not independent numeric probes:

| Count | One-based other-layer vector | Required local witness |
|---|---|---|
|0|`[2,5,2,1]`|Nonempty-vector inactive-count control; no custom range selection.|
|1|`[2,5,2,1]`|Layers2..5 inclusive print logical filament2 then1.|
|2|`[2,5,1,2,4,6,2,1]`|Layers2..3 order1 then2; layers4..6 order2 then1, with second row winning overlap4..5.|

Separate dependency controls, not extra canonical count-domain values:
- count1 + empty vector: inactive through the other early-return condition;
- count2 + reversed row precedence `[4,6,2,1,2,5,1,2]`: layers4..5 now order1 then2,
  while layer6 remains order2 then1. This must distinguish last-row-wins from
  first-row-wins, mere retention or unordered filament membership.

Use layers1,2,3,4,5,6,7 and later layers as boundary observations. Counts must
not override the settled first layer. Outside ranges, custom selection is
inactive and the source minimum-flush algorithm still owns the order. Do NOT
assert a fixed outside `[1,2]` or byte equality between different requests:
previous final filament can affect subsequent automatic ordering. Both streams
must agree on their actual outside-layer order/state and full output.

Count0 is a necessary explicit inactive control, not an activated-value
witness by itself. No negative count, count>=3, nondivisible vector, item<=2,
empty/reversed range, missing/duplicate/out-of-range filament, arbitrary layer
shape, by-object scheduling or unbounded integer certification.

## Input, application and consumer proof

Reuse existing OrcaRunner export/slice/cache and shared Native required/optional
handling. New key-local files under
`crates/ares-cli/tests/orca_parity/option_coverage/`:
`tool_sequence_context.rs`, `tool_sequences.rs`, `tool_sequence_observation.rs`
and a dedicated finite sequence-domain module if needed. Keep each below400LOC.
Existing `domains.rs`, `sweep.rs`, `option_coverage.rs` get minimal wiring only;
preserve APA bridges, bridge/filament-flow, G2 and G4 Manual-map hooks verbatim.
No extraction of one-off generic context frameworks.

Preserve requested options and original exported ZIP/config before binding.
Bind count, vector and first-layer order at both project and target plate;
retain every nonowned ZIP member byte-for-byte. Required proofs distinguish
requested/exported/project/plate/Ares effective settings, not just footer text.
A conflicting-project/plate control may be used only to verify the already-cited
plate precedence; it is not a new CLI overlay experiment. Export/application
failures stay harness failures, not production RED or PASS.

Observe Native/Ares commands by actual layer boundaries and positive extrusion,
maintaining logical tool state across boundaries even if no redundant T command
appears. Track actual wall versus infill FEATURE/role use. Ordered extrusion
runs must prove the expected ranges, inclusive endpoints and overlap priority;
comment-only markers, declared palette size or a sorted tool set do not count.
Config-block exclusion is observer-only interval exclusion as in the merged
G4 observer; the whole-output comparator sees everything except its existing
generated-by-banner normalization.

Save requested/exported/bound bytes, typed/effective/application proofs,
per-layer ordered-extrusion records, fresh cache/command receipts, complete
streams and SHA sets. Do not commit Native G-code goldens or synthesize Native
sequence answers from Rust's implementation.

## Admission, RED and bounded GREEN contract

After exact spec approval, first run an actual live Native-backed vertical RED
on the existing INCOMPLETE plan: retain all canonical/control comparisons and
consumer assertions before the terminal bounded-classification expectation.
If Native application, execution, observation or complete comparison fails
first, preserve artifacts and report that stage, not a made-up metadata RED.
No consumer repair or classification promotion is pre-authorized by this spec.

Only after all three canonical requests AND both controls have correct actual
consumption and exact whole outputs may the single count plan gain source-cited
bounded metadata. Generate exactly count0/1/2 with the coupled vectors above;
selected-key sweep must invoke the identical admitted preparation/observer and
execute all three, not a baseline fixture or short-circuit. Metadata tests must
assert literal values, coherent paired vectors, complete bounded metadata and
all deferred behaviors. All other plans, including four merged Manual maps and
four still-unexecuted G4 keys, remain unchanged.

Required directed seeded/official fresh cold + repeat/cache-hit runs report
actual Summary/exit/case counts and preserve every attempt. Native start needs
load1<30,2threads, no browser overlap and coordinator queue permission; held
startup is not RED. Default-env workspace (unset ARES_*), strict all-targets /
all-features Clippy, fmt and WASM core/vgcode/adapter must pass. If production
later changes, fresh new-tree gates and complete fresh base/candidate seeded /
official pairs are mandatory. Even for harness-only promotion, use the common
acceptance controls and exact-name failure comparison, raw common-input SHA
sets and distinct/repeated capture inventory. Report ND to the owner; no
implicit waiver, tolerance or comparator change. Review binds exact final hash;
latest allowed rebase then non-force FF push only after APPROVED.

## Temperature fixture revision — approval requested

The approved fixture above fixes 211/233. Do not silently substitute it.
On production base `626ed1738`, standalone Native export succeeded
(`/tmp/worker2-g4-sequence-626-export-validation.HMwcXl`, exit0), but the single
fresh E2E (`/tmp/worker2-g4-sequence-626-fresh-e2e.k8K9V3`, run
`1239a398-31c7-4b75-94a7-4db0b2241bcd`) stopped at Native Slice -62/exit194:
both actual PLA recommended ranges are `[190,230]`, and 233 is outside the
other filament's range. This is admitted upstream validation, not Ares RED.

`libslic3r/Print.cpp:1052-1106::check_multi_filaments_compatibility` resolves
ranges and tests both temperatures against the other filament's inclusive
bounds (`1092-1102`). The by-layer caller gathers actual used filament fields
at `1217-1240`; `Print::validate` invokes it for one nozzle/multiple filaments
and sets `STRING_EXCEPT_FILAMENTS_DIFFERENT_TEMP` at `1273-1277`.
`OrcaSlicer.cpp:6077-6078` maps this to `CLI_FILAMENTS_DIFFERENT_TEMP` (-62).
These lines also match the seeded replay source used for the receipt.

**Requested sole spec change:** fixture nozzle and initial-layer temperatures
211/**223**, rather than 211/233. Retain the same two PLA presets, their actual
recommended `[190,230]` ranges, distinct colors, one Standard nozzle, Manual
map, role allocation, first-layer order, all count/vector requests and controls.
Both 211 and 223 lie within both ranges; temperature distinction is retained.
No production changes, preference override, range widening, guard bypass,
comparator normalization or new consumer scope is permitted.

Pending owner approval: no temperature code change or further Native launch.
After approval, audit both actual preset/project temperature arrays and mutual
bounds offline, then perform one standalone export **and slice** admission
validation on the bound count0 input before one fresh E2E on the current
production tree. Preserve the -62 receipt and every subsequent raw artifact;
export alone does not certify slice validation or sequence consumption.

## Exit / deferral

This closes only count0/1/2 in the stated coherent by-layer, one-nozzle context.
All general counts/layouts, by-object masks, variant identity, master preference,
resource-backed dataset reconstruction, active ramming and unsupported Native
inputs remain deferred. An existing stock baseline or config retention is not
coverage of this source-owned count/range consumer.
