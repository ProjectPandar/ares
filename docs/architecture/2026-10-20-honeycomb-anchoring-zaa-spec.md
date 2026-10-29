# Honeycomb bridge-over-infill anchoring: regional ZAA direction

Status: SPEC-REVIEW-REQUEST; discovery only, no RED or production changes.
Base: approved main `1d0508fd1d4b29169dcddc57f7c6014acf3ddbc5`.

## Source boundary, not a new slicing pipeline

This is the anchoring context explicitly deferred by
`2026-10-15-zaa-honeycomb-direction-spec.md`; ordinary emitted Honeycomb
already carries the regional direction bit and is not being re-ported.
The previous half-spacing contraction fix remains intact.

Owning upstream sequence in OrcaSlicer `8500fcd`:

- `Fill.cpp:1394-1504`,
  `Layer::generate_sparse_infill_polylines_for_anchoring`: create the filler
  for the actual printing region and layer.
- `Fill.cpp:1440-1447`: assign
  `rcfg.zaa_enabled && rcfg.zaa_dont_alternate_fill_direction` to
  `f->dont_alternate_fill_direction` before calling `fill_surface`.
- `FillBase.cpp:105-118,275-323`: preserve base surface contraction,
  bridge-angle/fixed-angle precedence, and skip `_layer_angle` only when
  that conjunction is true (`:311-313`).
- `FillHoneycomb.hpp:53`: `_layer_angle(idx) = float(pi/3) * (idx % 3)`;
  `FillHoneycomb.cpp:9-80` consumes the supplied direction.
- `PrintObject.cpp:2751,2848-2936,3242-3251`: use the generated sparse
  anchor lines to choose the internal bridge direction. This consumer
  establishes the downstream effect; its algorithm is not part of the fix.

Rust destination: the Honeycomb parameter assembly in
`project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs`.
It currently supplies literal `false`, unlike the regional conjunction
already available to the neighboring linear branch. The adapter is a
compatibility shell for the cited upstream filler lifecycle, not an
Ares-owned anchoring design. Reuse that existing regional context; do not
invent a new context/configuration API. If the current 400-line module
requires a mechanical split, only its Honeycomb assembly may move to a
private `sparse_anchoring/honeycomb.rs` leaf, preserving other arms.

## Native-first discovery and first differing intermediate

Use the approved authored Honeycomb multiline target mesh/profile bundle:
multiline2, angle45, density20%, layer height0.2. The discovery modifies
only the two regional ZAA bits and, for layer-phase activity, top shell
count. Input-only ZIP members and metadata are preserved except the
explicit process settings; no Native output is a committed fixture.

The original top-shell count7 generates an internal bridge at object index43
and projects anchoring index42, whose `%3 == 0` hides the missing bit.
A source-bounded three-phase check used top-shell counts6/7/8, not another
mesh or pattern. Count6 exercises anchoring index43 (`%3 == 1`); count8
exercises index41 (`%3 == 2`). Count6 is the positive witness.

For top-shell count6, official and seeded each accepted four cases, each
with two fresh Native runs. Repeats and normalized cross-mode Native bytes
agree for every identical input SHA. Unchanged generator-only whole-file
comparison admits three controls and rejects only enabled-ZAA/flag1:

| enabled | flag | input SHA256 | outcome, each mode |
|---|---|---|---|
| 0 | 0 | `40cbfd9d0e5b26679a1b9ea161e2d064dc68234157d54e39cba1ce00120110f4` | PASS |
| 1 | 0 | `c1a68d0452738de551592b6bebdce8db631379320c3d25ea6f74c4c4a54f66f5` | PASS |
| 1 | 1 | `078df2b3f6ffa92167b8c044408b02beaf5534a7bcebdbe6bebc606b074a9812` | DIVERGENT |
| 0 | 1 | `662a314cc90f2a83c3a79de9e4c742b7bc29cd6eea30975bded1e1037079fbd7` | PASS |

Top-shell counts7 and8 are also exact official/seeded controls for enabled/flag1;
two fresh Native repeats and normalized cross-mode bytes agree. Their input
SHAs are respectively
`6604daa38d5c7dd73f39311c7094e988e0b87317968ceab9d817650497d669df`
and `a838438b5bd0b7c5895ef5b167f77bee0cce1519a7babe3425bc6ad587e12753`.

Read-only debugger evidence on the positive input establishes the first
inconsistent intermediate at Honeycomb entry for layer43:

- Native anchoring: `dont_alternate=1`, direction `2.3561944961547852`.
- Ares anchoring: `dont_alternate_fill_direction=false` despite both
  effective regional flags being true.
- Both receive the same domain `-4019688..4019693`, spacing
  `0.40707963705062866`, density0.2 and multiline2.
- Their later actual-fill calls both carry true and agree on domain
  `-3612599..3612604`. All 1316 observed Sparse motion rows match.

The first differing motion is the entry travel immediately before Internal
Bridge at physical layer45, source line3750: Native XY `(113.656,114.201)`
versus Ares `(114.201,113.656)`. The next bridge extrusion also differs;
bridge activity is nonempty on both sides. This is not ordinary Sparse
emission, a surface-contraction regression, or an E-rounding discrepancy.

Whole-file comparison must retain its earlier difference at byte54136,
line2366: an M73 placement change. `GCodeProcessor.cpp:1232-1258` derives
progress from cached elapsed time divided by whole-file machine time, so
changed later travel/bridge paths can affect earlier progress placement.
The no-M73 diagnostic stream is used only to locate motion ownership,
never as an acceptance comparator. No progress, cooling or time-estimator
changes are authorized; a remaining delta after the bounded port means STOP.

## RED, GREEN and non-negotiable boundaries

After SPEC approval, freeze input-only top6 target and both conjunction
controls (enabled/flag0, disabled/flag1); retain the confirmed top7/top8
enabled/flag1 phase controls. Public API and CLI must
agree in raw bytes. Require effective flags, positive Internal Bridge,
Native fresh-repeat stability, unchanged Sparse motion, and strict complete
Native equality for every case. Expected RED is one target FAIL with all
controls PASS per mode; no inverted or weakened assertions.

GREEN only carries the printing region's conjunction into ordinary
Honeycomb anchoring. Preserve the half-spacing offset, direction arithmetic,
layer/thickness indices, bridge/fixed-angle precedence, lattice, clipping,
connection, ordering and all other pattern behavior. Do not add allow/expect,
skip paths, UnsupportedProjectFeature guards, fallback, or Native outputs.
Flow, E arithmetic/rounding, bridge-angle detection, M73/time processing,
object/support ironing, raft/tree and 3D/LateralHoneycomb are deferred.

Explicit deferred sibling: `sparse_anchoring/arms.rs:216` supplies literal
`false` to `LineFillParams::dont_alternate_fill_direction`. `FillLine`
inherits the same base `fill_surface` direction lifecycle
(`FillLine.hpp:12`, `FillBase.cpp:105-118,311-313`), while Rust
`fill/line.rs:102-115` consumes that bit. This is the same static
regional-context propagation discrepancy, not yet a separate Native-visible
witness. Do not change or fold Line anchoring into this Honeycomb chain.

Mandatory gates: fmt, workspace all-target clippy with `-D warnings`, wasm32,
all new and existing Honeycomb/ZAA controls in both modes, then canonical
exact-hash FULL in both modes with fresh artifacts/cache and zero FAIL.
Compilation/probes CPU12; FULL `ARES_PIN_OFFSET=20`,
`ARES_PARITY_THREADS=4` (CPU20-23). Submit RR; coordinator alone reviews
and merges. No production starts before separate SPEC and RED approvals.

## External evidence

Under `/home/indexyz/ares-evidence/`:
`w3-honey-anchoring-zaa-top6/` and `...-top6-seeded/` contain input SHA,
commands, binary identities, Native repeats, strict results and footer
admission. Target `role-observation.json` and `first-motion-context.json`
record localization without changing the strict verdict.
`w3-honey-anchoring-zaa-discovery1/`, `...-top7-seeded/`,
`...-top8/` and `...-top8-seeded/` are the phase controls.
`w3-honey-anchoring-zaa-localize/{seeded-native-gdb,ares-gdb}.log` record the
entry parameters. The seeded debugger uses the actual reference executable
with existing matching DWARF; no diagnostic build or gcroot was created.
The old auxiliary diagnostic path had been garbage-collected and was not
rebuilt or used as an oracle. Only the canonical live runners supply parity.
