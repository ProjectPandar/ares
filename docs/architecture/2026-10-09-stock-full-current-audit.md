# Fresh full stock matrix and non-PASS ownership

Actual clean execution: `829977606fee7eef40809b69922b8db13bd425cc`, docs-only
successor of main2becb00a5; exit100. Required seeded controls, fresh
cache/artifact root; no retries or input/preset repairs.

| Raw status | Current | Historical baseline | Delta |
|---|---:|---:|---:|
| PASS |929|823|+106|
| DIVERGENT |0|100|-100|
| ORCA_ERROR |53|66|-13|
| VENDOR_INCOMPLETE |10|10|0|
| ARES_ERROR |0|2|-2|
| INPUT_REJECTED |9|not separately specified|not inferred|

Current1001 rows reconcile929+53+10+9. No remaining first-difference cluster
exists: zero DIVERGENT means zero groups/cases. Exit100 is the incomplete-stock
inventory result, not a whole-matrix PASS. Raw Native rejection/error statuses
are not promoted to valid-stream PASS by a matching Core rejection.

## Native errors:53 rows

Twenty loaded-config Export238(-18) plus eight minimum-width Slice205(-51)
are the #7 cohort already independently checked with public APIs. Their old
28-case execution bindings remain in the #7 conclusion; this stock
sweep stops on Native errors and does not freshly compare Ares on these rows.

Remaining25:
- Five custom-template Slice156(-100): FLSun QQ-S Pro0.4; Ginger G1
  1.2/3.0/5.0/8.0. GCode.cpp:3636-3645 failed-template aggregation;
  PlaceholderParser.cpp:931 missing variable,999 invalid variable name;
  OrcaSlicer.cpp:6286-6291 exception stderr and CLI_SLICING_ERROR.
- Four Kobra3 0.2/0.4/0.6/0.8 Slice206(-50): Model/BuildVolume eligibility;
  OrcaSlicer.cpp:137 exact error text,5710-5724 pre-apply eligible-object check.
- Four Prusa MK3S0.25/0.4/0.6/0.8 Slice237(-19):
  OrcaSlicer.cpp:131 exact text,1682-1690 embedded postprocess load guard.
- Nine SIGSEGV, empty captured stdout/stderr: MyToolChanger0.2/0.4/0.8;
  Creator5 0.4; Creator5Pro0.4; XL5T0.25/0.6; U1 0.4; ZR Ultra S0.4.
- Three SIGABRT: Cubicon xCeler-I/Mini/Plus0.4, exact
  `ConfigOptionVector::set_at(): Assigning from an empty vector` exception.

Signals are Native crashes, not portable API rejection contracts. Do not
simulate signals, add guards, repair singleton selectors or claim fresh PCs
from older UB stack evidence. All12 remain non-PASS; upstream Config.hpp:438-446
and the historical unsafe-selector audit are diagnostic context only.
The remaining13 defined errors need fresh per-case Core/CLI code and complete
channel comparison, not a conclusion inferred from family labels or old reports.

## INPUT_REJECTED9: actual Native validation, not a failed construction

Fresh o1-rejection result receipts include unchanged exported3MF hashes,
Native code205 and exact Native/Ares validation text equality. Seven relative-E
rows: Kobra Max0.4, Kobra Plus0.4, Vyper0.4, Raise3D Pro3 Plus0.4
Dual/Left/Right, Wanhao D12-3000.4. Two absolute-E rows: Dremel3D40/3D45 0.4.
Print.cpp:1732-1738 and1740-1745 own the exact G92 reset checks. Their valid
stock configurations reach Native Print::validate and reject; the harness did
not fail to construct an input. These receipts prove validation payload parity,
not complete CLI result.json/stdout/stderr parity or a raw-PASS promotion.

## VENDOR_INCOMPLETE10: selection/catalog failures, not Native rejection

K1 SE0.8 references missing default process
`0.40mm Standard @Creality K1 SE 0.8 nozzle` (1). No compatible process is
selected for RatRig V-Core4 300/400/500 0.8 (3), or iQ TiQ2/TiQ8
0.25/0.6/0.8 (6). No Native invocation was admitted for these rows.
Preset.cpp:809-850 owns upstream compatibility predicates; current harness
select_printer/pick_preset and VendorProfiles compatibility filtering produce
these diagnostics. A missing result is not yet proof that the upstream vendor
catalog itself has no compatible preset: independently audit definitions,
inheritance and conditions before proposing a selector fix. Never silently
substitute or repair a preset. These remain incomplete, not validator errors.

## Next bounded slice and provenance

All13 defined residual errors are selected for a source-only spec;
reuse #97 custom_gcode_reference/CLI tests and the existing postprocess/eligibility
runner/cache. All result.json comparisons use shared
native_wall_clock::result_comparison_bytes, ignoring only prepare_time/export_time;
stdout/stderr and every other result byte remain exact.

No failed attempt discarded. This report remains bound to
829/2bec, not execution after TiQ2. Subsequent13-case E2Es must execute on their
actual latest commit. No production change follows from this report.
