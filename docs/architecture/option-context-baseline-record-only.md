# ARD/spec: retained context and baseline Option-domain gaps (record-only)

## Contract and source boundary

Read base: integrated `57d63ba5233875cd78a8822839d1532f10bfc2cc`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This phase classifies **16 historical context gaps and 12 baseline/non-injected
gaps**. It is a source-cited inventory/architecture record, not implementation
of an Ares-owned pipeline and not permission to repair production or fixtures.

Owners: `libslic3r/PrintConfig.cpp::PrintConfigDef` definitions and
`validate(const FullPrintConfig&, bool)`; `Print.cpp::Print` build-volume and
extruder-area consumers; `GCode.cpp` output validation; and the configuration
structures consumed by existing mapping, sequence, and external-processing
boundaries. No `libvgcode` behavior belongs to this slice.

Rust destination **in this phase**: this document and a retained classification
artifact. Existing `ares-cli/tests/orca_parity/option_coverage/{domains.rs,
domains/probes.rs,widths.rs,sweep.rs}` are only the inventory/execution
compatibility shell around those upstream concepts. Their omission wording is
not upstream semantic authority. No Rust implementation, sample injection,
Native run, fixture change, validation relaxation, production rewrite or
comparator change is included. Any future executable slice needs a separately
approved source-owned plan and independent RED first.

Non-negotiable: preserve historical statuses, rejected cases, unsafe/unknown
behavior and provenance limitations. Do not convert an unbounded side into
±FLT_MAX/i32 extrema; do not synthesize vectors/IDs/polygons to force admission;
do not promote any of these28 rows to PASS in this record-only phase. Requested,
exported, applicable, compared and declared-scope-complete are separate facts.
Full bytes, M73/footer and producer/input receipts remain authoritative.

## Retained evidence and accounting

Historical matrix: `${TMP_ROOT}/ares-matrix-main-5cde90c23-single-attempt/option-domains.json`,
SHA256 `f68c9d5be063060ba4a78c590bb1d7eb8e14c638012bfb5f955677c297804c71`.
The source hash was rechecked during this read-only classification.
Classification: `${TMP_ROOT}/ares-option-context-baseline-28/classification.json`,
including the original rows and current metadata plans, not new executions.

The historical context group has **41 cases / 40 comparisons / 1 rejection**:
13 authored-sample rows (25 comparisons), two width rows (14 comparisons), and
timelapse (two cases, one comparison, one rejection). The baseline group has
**12 cases / 12 comparisons**, but each plan contains `value:null`: a default
comparison does not establish that the named setting was injected or exercised.
All28 historical rows are INCOMPLETE. These counts are retained historical
observations, not fresh acceptance results. The separately integrated196-key
numerical scan does not certify these28 rows.

## Context classification:16 keys

### C1 — source-proven validation thresholds (10)

`PrintConfig.cpp::validate`, approximately10340–10443, rejects nonpositive
layer/initial-layer heights, bridge/internal-bridge flow, three extruder
clearances, nozzle height and filament flow ratio; filament diameter must be
at least1. Layer-height quantization is also checked. Source min0 is therefore
not universally a sliceable endpoint. Neighboring nozzle, layer, object and
region constraints must not be conflated with raw definition bounds.

| Key | Definition in PrintConfig.cpp | Retained authored samples |
| --- | --- | --- |
| bridge_flow |1327|0.5,2.0|
| internal_bridge_flow |1354|0.5,2.0|
| extruder_clearance_height_to_lid |2225|0.5,250|
| extruder_clearance_height_to_rod |2215|0.5,250|
| extruder_clearance_radius |2234|0.5,250|
| nozzle_height |2242|1,400|
| filament_diameter |2606|[1.0],[3.0]|
| filament_flow_ratio |2315|[0.5],[1.5]|
| layer_height |771|0.05,0.36|
| initial_layer_print_height |3384|0.05,0.36|

Record class: `CONTEXT_VALIDATION_THRESHOLD`. Definition and validator evidence
prove threshold conflicts; the historical row does **not** itself prove that
every omitted endpoint was run and rejected. Preserve that distinction. No
positive-probe success certifies the omitted/unsupported configuration space.

### C2 — model-relative build-volume constraint (1)

`printable_height` (`PrintConfig.cpp:779`), retained50/400. `Print.cpp:1375–1391`
checks the last slicing plane and shrinkage against build height. This is a
model/configuration relationship, not merely an independent numeric range.
Record class: `CONTEXT_BUILD_VOLUME`; no model resizing or input repair here.

### C3 — generic omission reason unsupported by the owning definition (1)

`adaptive_pressure_advance_bridges` (`PrintConfig.cpp:2401–2408`) declares
max2, **no min**, default0, and explicitly says0 disables bridge PA. Retained
samples[0.5]/[2.0] already include the declared maximum. The blanket authored
probe wording “schema bounds rejected by upstream validation” is not evidence
for this option and must not be asserted as a source-proven rejection. No new
endpoint or invented lower bound is required by the definition alone. Active PA,
bridge paths and applicability would need separate consumer evidence.

Record class: `CONTEXT_GENERIC_REASON_UNSUPPORTED_BY_SOURCE`. This is a
reporting/admission question for later review, not a production defect or an
automatic PASS promotion. No new activation/slicing coverage is claimed.

### C4 — grouped geometry/cardinality (1)

`extruder_printable_area` (`PrintConfig.cpp:692–697`) is `coPointsGroups`, with
an empty default, not a numeric interval. `Print.cpp:3256–3275` consumes polygon
groups per extruder. Retained one bed-sized group establishes one specific
single-extruder shape only. The generic “bounds rejected” label is not a numeric
proof. Record class: `CONTEXT_GROUPED_GEOMETRY`; defer other group counts,
parallel heads, object placement, invalid polygons and multi-extruder contexts.

### C5 — context-bound width samples and stale range wording (2)

`skin_infill_line_width` (`PrintConfig.cpp:4064`) and
`skeleton_infill_line_width` (`:4074`) are float-or-percent, min0, no declared
max, ratio over nozzle diameter. Current shell locks them INCOMPLETE and uses
seven samples each in0.4mm-nozzle /0.2mm-layer /lockedzag40% context
(`widths.rs::generate`). Other nozzle sizes and object/region contexts are
unverified; this is not a global finite interval.

The shell's non-executed metadata still describes implicit FLT_MAX as a raw
schema endpoint and global range completion. Under the approved numerical
contract this is **stale reporting**, not a declared endpoint obligation.
`execute:false` probes remain recorded predictions, never Native rejection or
comparison evidence. Record class:
`CONTEXT_WIDTH_DEPENDENCIES_AND_STALE_RANGE_LABEL`. No width/context generator
or production change is authorized by this document.

### C6 — observed Native toolpath rejection (1)

`timelapse_type` (`PrintConfig.cpp:5879–5894`) declares0/1, traditional/smooth.
The historical1 case was rejected at Slice/Process. Immutable retained status
is exit154, stdout `run found error, return -102, exit...`, empty stderr,
`snapshot_incomplete:false`:
`.../artifacts/orca-runner-iFCRbv/command-8OZAn6/{status.json,stdout.bin}`.
`libslic3r/Utils.hpp:74` maps-102 to `CLI_GCODE_PATH_IN_UNPRINTABLE_AREA`;
`OrcaSlicer.cpp:6220–6227` emits that exit for G-code validity failure.

`Print.cpp:3317–3319,3385–3387` connects smooth timelapse and tower behavior;
`Print.cpp:1286–1288` has a separate by-object restriction. These identify
adjacent dependencies, **not** proof of the precise offending path in the
retained case. No crash, comparator divergence or universal invalid-enum claim
is supported. Record class: `CONTEXT_NATIVE_TOOLPATH_REJECTION`; detailed path
attribution remains unresolved. Do not move/repair the tower or retry here.

## Baseline/non-injected classification:12 keys

Every current plan below remains one baseline case with no requested value.
Implicit bounds and baseline byte equality are insufficient for option-domain
completion. References are PrintConfig.cpp unless otherwise indicated.

| Keys | Owning definition/source | Read-only class / deferred behavior |
| --- | --- | --- |
| filament_map; physical_extruder_map |2489;2495; get_extruder_index:86–91 and mapping/configuration logic around9808–9846|Mapping/cardinality, logical→physical indices; no arbitrary ID injection|
| printer_extruder_id; print_extruder_id; filament_self_index |5385;5405;5432; definitions mark nocli|Mapping/identifier lifecycle; API/profile storage is not proof of CLI applicability|
| master_extruder_id |5399, default1|Mapping/identifier lifecycle; no guessed topology|
| nozzle_flush_dataset |2602–2604, nullable coInts|Nullable dataset identity; nil/default/data-shape behavior unresolved, no padded vectors|
| other_layers_print_sequence_nums |1121; adjacent other_layers_print_sequence:1115|Sequence-count relationship; do not vary count independently of sequence data|
| filament_ramming_parameters |2833–2838, packed coStrings; GCode/WipeTower.cpp:1590–1602|Packed-string schema; inspected parser is inside #if0, so do not claim active ramming consumption from it|
| post_process |5068–5079; GCode/PostProcessor.cpp::run_post_process_scripts:238–245,304 onward|External-process adapter boundary; empty list is not script execution coverage; no scripts/processes launched here|
| print_compatible_printers |1832–1834, nocli coStrings|Compatibility metadata; no consumer/compatibility decision coverage inferred|
| wrapping_exclude_area |4120–4125; GCode.cpp:2166–2172|Exclusion geometry; feeds G-code validity check; do not infer all rectangles valid or invent a repaired polygon|

Counts: mapping/identifiers6; nullable dataset1; sequence count1; packed
string/inactive inspected consumer1; external processing1; compatibility
metadata1; exclusion geometry1. Existing probe comments say a rectangle for
wrapping_exclude_area was rejected, but without a linked admitted command that
is only an inherited comment, not a new verified runtime result. Similarly,
post-process “post_process” must not be confused with GCodeProcessor's separate
internal footer/time post-processing routine.

External script execution remains outside platform-neutral ares-core and
browser WASM. Record-only classification neither ports it into core nor adds
an adapter fallback. No legacy fallback or UB emulation is included.

## Deliverables, failure modes and exit criteria

Deliver this source-cited spec plus the hash-bound28-row classification.
Do not reinterpret historical208 NUMERICAL_RANGE_GAP labels as28 remaining
numeric endpoint gaps: the12 baseline rows were explicitly excluded before the
196-key numerical scan, and context semantics differ from absent bounds.

Reject a record that: conflates unbounded with missing extrema; generalizes
blanket omission text into a proved rejection; confuses default/no-injection
with application coverage; credits non-executed width probes; claims active
behavior from #if0 or nocli metadata; treats timelapse-102 as a numerical
mismatch; imports new Native results without admission/receipts; or changes
production/context/comparators in this phase.

Exit: all28 unique keys retained with original status/counts, source owner,
current plan and explicit known/unknown/deferred behavior; partition16/12 and
all subcategory totals checked; matrix SHA preserved; no executable changes or
new runs; no PASS promotions. A later implementation or runtime investigation
requires independent exact-hash approval and a separate bounded plan.
