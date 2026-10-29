# Rectilinear multiline: public Native-backed RED

Base `5a9687ac020962f254b6262f2184f3345025c70f`; approved
[SPEC a56766fa4](fill-rectilinear-multiline-dispatch-spec.md).
RED `4d366f3b637a2ea092998c914983e9da6dacd970` is test-only; separately
approved control setup correction `98084d50c0670440a394043878f710fda3aa9067`
is also test-only. No production change or anchor_projection edit exists.

## Corrected controls and preserved assertions

Initial root `/tmp/worker6-rectilinear-red.svpAos` retains both original
mode logs, exit100 and Native outputs. Both modes had five true DIVERGENT,
five PASS and two setup failures. Neither setup failure counts as parity RED:
M2 sparse Monotonic was rejected at export with -18; S2 `stars` exported as
`crosshatch`, caught by the unchanged requested/effective value assertion.

The coordinator approved correcting these inputs in a separate test-only
commit. M2 now uses density100%, sparse Rectilinear, count2, and Monotonic
top/bottom/internal-solid roles; `FillRectilinear.cpp:3401-3408` owns that
by-lines subclass. S2 uses `tri-hexagon`, the `ipStars` canonical enum in
`PrintConfig.cpp:238/3030`. All effective requested settings are checked.
Both controls now pass full Native/API/CLI ordered byte and line-count checks.
This corrects the SPEC's conceptual M2/S2 labels; it does not introduce a new
production behavior or change any accepted Rectilinear RED input.

All code from `fn compare` through the byte comparator assertions and sparse
observer is exactly equal before/after the control fix. The five admitted RED
fixtures have identical ZIP member names/bytes before/after that fix, and their
Ares raw outputs are byte-identical. The correction neither deletes an assertion
nor substitutes a selected-line comparison for complete G-code.

## Fresh official and seeded runs

Corrected root `/tmp/worker6-rectilinear-red-fixed.7UC7US`:

| Mode | Exit | Summary |
| --- | ---: | --- |
| official | 100 | 12 run, 7 PASS, 5 FAIL; 21.088s |
| seeded | 100 | 12 run, 7 PASS, 5 FAIL; 29.165s |

The 1217 unselected tests are nextest filter skips, not skipped controls.
All 12 Native requests were admitted. Public validated API and CLI also admit
every case; the API layer Z/height sequence equals Native for each capture.
There are no timeout or missing-oracle failures. Each leg uses a fresh Native
cache/artifact root and its own target. Compiled dependencies were copied from
worker6's completed targets; CLI/test fingerprints were rebuilt. The directed
driver and descendants run under taskset CPU11, jobs1/threads2; actual Native
executable sampling recorded 55 entries, all affinity 11, zero violations.

`red-review-audit.json` binds both modes' complete comparisons, line counts,
input/native hashes, per-member input invariance, Ares raw invariance and
assertion-body equality. The root contains `native-affinity.json`; each leg has
`log`, `exit`, `cache`, `target` and `artifacts`. Each capture retains its input,
complete Native/API/CLI files, CLI channels and `comparison.json` outside git.

## Real sparse first differences

Both modes report exactly these ordered sparse differences, all layer 4.
The line numbers are Native/Ares respectively; E is actual positive extrusion.

| Case | Lines | Native | Ares |
| --- | --- | --- | --- |
| R2 | 1416/1416 | `G1 X141.02 Y133 E.01235` | `G1 X141.02 Y132.712 E.02302` |
| R3 | 1415/1416 | `G1 X139.369 Y133.02 E.05401` | `G1 X141.02 Y132.712 E.02302` |
| R10 | 1422/1416 | `G1 X141.02 Y119.951 E.05401` | `G1 X141.02 Y132.712 E.02302` |
| R2A0 | 1424/1416 | `G1 X123.773 Y133.02 E.05401` | `G1 X139.853 Y86.98 E1.53329` |
| RH2 | 1700/1703 | `G1 X141.02 Y133 E.01235` | `G1 X141.02 Y132.712 E.02302` |

R2's complete stream first differs at line18 (`M73 P0 R38` / `M73 P0 R37`),
but the independent sparse observations prove a path difference as well.
The API/CLI line counts differ from Native for all five; complete comparisons
are retained as FAIL, not weakened. Their count1 Native/API/CLI baselines pass,
and actual Native sparse commands differ from those baselines.

R1/RH1/RF100/M2/Z2/G2/S2 all pass full API/CLI ordered bytes and line counts
in both modes. The source boundary remains `FillRectilinear.cpp:3387-3399`
dispatch into `:2996-3045` for one multiline sweep, invoked by both emission
and `Fill.cpp:1477-1498` anchoring. Existing Ares Rectilinear consumers still
use by-lines parameters with no multiline count. GREEN has not begun.
