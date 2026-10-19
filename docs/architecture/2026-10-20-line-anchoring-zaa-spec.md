# Line bridge-over-infill anchoring: regional ZAA direction

Status: SPEC-REVIEW-REQUEST; discovery only, no public RED or production edit.
Base: approved/merged main `27888e2d41429621fe4966018f75d686f9e3e8e7`.

## Owning upstream boundary

This independently addresses the Line anchoring context deferred in
`2026-10-15-zaa-line-direction-spec.md` and explicitly recorded by
`b89e8add9` in `2026-10-20-honeycomb-anchoring-zaa-spec.md`.
Ordinary emitted Line is already ported. The accepted Honeycomb anchoring
change is not extended or modified by this slice.

Upstream OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `Fill.cpp:1394-1504`,
  `Layer::generate_sparse_infill_polylines_for_anchoring` owns the filler
  lifecycle; `:1440-1447` creates the configured filler and assigns the
  actual printing region's `zaa_enabled && zaa_dont_alternate_fill_direction`.
- `FillBase.cpp:54` maps `ipLine` to `FillLine`;
  `FillLine.hpp:12-25` inherits Fill's surface/direction lifecycle.
- `FillBase.cpp:105-118,275-323` contracts the surface, honors bridge/fixed
  precedence, suppresses layer alternation when the bit is true, then adds
  pi/2. `FillBase.hpp:211` supplies Line's **two-phase** odd-layer pi/2 term.
- `FillLine.cpp:10-119` consumes that direction for rotation into/out of the
  oscillating-column frame. Its clipping/extension/chaining/connection
  algorithms are not being re-ported.
- `PrintObject.cpp:2751,2848-2936,3242-3251` consumes projected sparse anchors
  to choose internal bridge geometry; this establishes downstream ownership,
  not authorization to change bridge-angle selection.

Rust destination: only the ordinary `ProcessInfillPattern::Line` parameter
assembly in `project_slice/prepare_infill/bridge_over_infill/
sparse_anchoring/arms.rs:207-235`. Its `LineFillParams` literal still sets
`dont_alternate_fill_direction: false`. `fill/line.rs:82-115` already
implements contraction and consumes the bit; emitted `fill_entities/line.rs`
already forwards its regional `SurfaceCall` and is not a missing kernel port.
This dispatch remains a temporary adapter around the upstream Layer/FillLine
lifecycle, not an Ares-owned pipeline. Reuse the existing resolved printing
region (`print_object_config` / `all_regions[fill.region_id]`), not a new
configuration/context API or an arbitrary first region.

## Independent Native witness

Input provenance: the approved authored Honeycomb anchoring mesh/profile
bundle (`tests/parity/honeycomb-anchoring-zaa/top6-enabled-flag1.3mf`).
Keep all input ZIP members and metadata; change sparse pattern to `line`
and multiline to1. Vary only the regional ZAA bits and top-shell count after
that. Direction45, sparse20%, layer0.2 and the 10mm model stay fixed. These
are external discovery inputs, not yet committed public fixtures.

Both canonical official and seeded modes, CPU12, execute two fresh Native
references per input. All repeats and normalized cross-mode Native bytes
agree. All runs admit Line/multiline1 and effective flags in the full footer.
The unchanged generator-only whole-file comparison gives:

| top | enabled | flag | input SHA256 | outcome, both modes |
|---|---|---|---|---|
| 6 | 0 | 0 | `7a742a8c0b1819cbc16f574dfb186617b73dfa0e2b6f116c0080ffd8a4089a81` | PASS |
| 6 | 1 | 0 | `aea1bfca56737f4b58e2b2321bb90d067b814c9b406960b15ca6475376591472` | PASS |
| 6 | 1 | 1 | `ce4695fd7095f07426733d26400c6d93ae8cab0fee0b04fe5fd5bed98d5f310f` | DIVERGENT |
| 6 | 0 | 1 | `f16bbca3785434c6e41e390e27ff3f4b87217413f7bfa694aa69fa4f40c7e3b8` | PASS |
| 5 | 1 | 1 | `fbe15bc57cac6a87b53e2afe4a447420bf991e1f24b87c465eff7e95e7493397` | PASS |
| 7 | 1 | 1 | `2211372b011e9f8ff10e8548d52fd56c57144e97479968212c9f7b34461b6756` | PASS |

The 50-layer model with top6 generates an internal bridge at physical45,
projecting anchors from object index43 (odd). Top5/top7 project indices44/42
(even), masking the missing odd-layer term. These are Line parity controls,
not Honeycomb's modulo-three phase controls. No mesh or pattern search was
needed. Total discovery:24 fresh Native references and12 Ares CLI runs.

## First inconsistent intermediate and downstream observations

Read-only GDB on the target, using the actual seeded reference executable
and its existing matching DWARF, plus the existing main27888e2d4 Ares CLI:

- Native anchoring at layer43: bit1, direction `2.3561944961547852`.
- Ares anchoring entry: bitfalse, spacing `0.40707963705062866`, density0.2,
  angle `0.785398185`, thickness1, no fixed/bridge override; computed
  direction `3.9269909858703613`. Its backtrace identifies the anchoring arm.
- Both contracted anchor domains agree at `-4019688..4019693`. Subsequent
  emitted-fill domains agree at `-3612599..3612604`; both later directions
  are `2.3561944961547852`. Ares's later backtrace identifies emitted Line.
- All391 ordinary Sparse motion rows are identical. Both sides have52
  positive Internal Bridge extrusion rows at physical45.

Thus the first differing filler parameter/frame is the anchoring bit and
extra odd-layer angle, not ordinary Sparse emission, surface contraction,
spacing, or E-only rounding. The first XY motion divergence occurs in
physical45's bridge-entry/wipe sequence: Native source2824
`G1 X113.764 Y114.201 F9000`, Ares source2828
`G1 X113.775 Y114.739 E-1.2`. First bridge extrusion also differs:
Native `G1 X114.003 Y113.863 E.02016`,
Ares `G1 X106.111 Y114.003 E.01835`.

Strict comparison retains the earlier **byte3099, line143** difference:
Native `M73 P10 R7`, Ares `G1 X113.987 Y106.983 E.04259`;
normalized lengths97368/97472. `GCodeProcessor.cpp:1232-1258` derives progress
from elapsed/whole-file time, so later changed geometry can affect early M73.
The first differing G0/G1 row is also before bridge entry within physical45:
Native `G1 F1796` versus Ares `G1 F1894` (source2811/2812).
`CoolingBuffer.cpp:317-333,638-693,907-939` parses the whole layer, derives
slowdown from its total duration, then rewrites feedrates. Both are downstream
observations, not authority to alter progress/cooling or omit their bytes.
Motion-only observation is localization only, never an acceptance comparator.

Some optimized inline debugger fields are unavailable; the non-inlined
entry supplies the full anchoring params, and post-branch breaks supply the
two directions/backtraces. No debug build, debugger state writes, gcroot or
production instrumentation was used. Canonical fresh references alone supply
acceptance; debugger outputs are not oracles.

## Separate RED then bounded GREEN

Only after this SPEC is approved, freeze five input-only public cases:
top6 enabled1/flag1 target; top6 enabled1/flag0 and enabled0/flag1 conjunction
controls; top5/top7 enabled1/flag1 even-phase controls. Retain the diagnostic
00 case externally. No committed Native outputs or probes.

The public E2E must require effective loaded options and Native footer,
positive Internal Bridge on physical `51 - top`, unchanged ordinary Sparse
motion, two fresh Native references, stable repeats and raw public API/CLI
equality. Reuse the existing strict whole-file comparator **including M73 and
feedrates**. Discovery currently covers CLI, not new public API acceptance.
Expected RED: only top6 target FAIL, four controls PASS per mode. Freeze that
RED independently, run both modes on its exact clean hash, then request
approval before production. No inverted assertions or optional silent skips.

GREEN only carries the anchoring printing region's conjunction into the
ordinary Line `LineFillParams` literal. The current arms378LOC has room for
this small regional lookup; no module refactor is required. Preserve every
other value, half-spacing contraction, odd-layer arithmetic, thickness
indexing, bridge/fixed precedence, reference, clipping, oscillation,
connection/order, density/spacing and all other pattern arms. Use existing
region lookup; no new helper/framework, allow/expect, Unsupported guard,
fallback or config/capability admission.

Deferred: emitted Line and shared kernel changes, modifier/multi-object
scope expansion, bridge-angle algorithm, extrusion/flow/E arithmetic,
M73/time/cooling, ironing/support/raft/tree, other patterns and independent
pipeline design. Any remaining strict difference after the bounded port
means STOP and report, not broaden the fix.

GREEN gates on the exact final hash: fmt, workspace all-target clippy with
`-D warnings`, wasm32, new and existing Line/ZAA/anchoring controls in both
modes, then canonical official and seeded FULL with fresh caches/artifacts,
zero FAIL/skipped, `ARES_PIN_OFFSET=20 ARES_PARITY_THREADS=4` (CPU20-23).
Compilation/probes CPU12. Coordinator alone reviews/merges; SPEC and RED
need separate approval before subsequent steps.

## Evidence

External `/home/indexyz/ares-evidence/w3-line-anchoring-zaa-` prefixes:
`top6/`, `top6-seeded/`, `top5/`, `top5-seeded/`, `top7/`, `top7-seeded/`
contain frozen discovery input SHAs, commands, binary identities, Native
repeats and complete-byte results. Target `role-observation.json` localizes
motion without changing the strict verdict. `localize/` holds Native/Ares
read-only debugger logs. `spec-review/matrix-audit.json` and the external
`w3-line-anchoring-zaa-audit.py` recheck repeat/cross-mode results and the
source ZIP/settings boundary. No RED test or production change is included.
