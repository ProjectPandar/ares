# ARD: scarf clipping must use the remaining flat-loop role

Status: approved boundary fafec2987; pre-window implementation separately
authorized on e81afae89. Historical diagnosis used main5317e1c09 and
branch w5-scarf-empty-flat-role-spec. This slice is independent of the
random/stagger math although those approved chains precede it in the
preparation stack. No FULL or integration RR is authorized.

## Owning upstream boundary

This is a Rust rewrite correction for OrcaSlicer's existing loop-role
semantics, not a new Ares scarf or slicing algorithm:

- `OrcaSlicer/src/libslic3r/ExtrusionEntity.hpp:485`,
  `ExtrusionLoop::role()`: returns `erNone` when `paths` is empty;
  otherwise returns the first remaining path's role.
- `ExtrusionEntity.cpp:471-505`, `ExtrusionLoopSloped` construction:
  transfers the consumed head to `starts` / `ends`; inherited `paths`
  contains only the remaining flat portion. It may legitimately be empty.
- `ExtrusionEntity.cpp:550-554`, `ExtrusionLoopSloped::clip_front`:
  starts with `clip_dist = distance` and replaces it with 40% of slope
  length ONLY if the resulting loop's `role()` is `erPerimeter`.
- `GCode.cpp:5940-5949`, `GCode::extrude_loop`: whole-loop scarf uses
  `loop_length` as `slope_min_length`; a configured minimum longer than
  the loop is also capped to that same loop length.
- `PrintConfig.cpp:5604-5609`, `seam_slope_entire_loop`: the GUI's
  advanced Quality option **Scarf around entire wall**.

Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/motion/scarf.rs::build`,
front-clipping role selection at 140-154. The existing `starts`, `flats`
and `ends` collections represent the upstream sloped-loop collections.
The current use of the ORIGINAL `first.role` is an inaccurate compatibility
shell for `ExtrusionLoop::role()`; replace that role lookup, not the
surrounding geometry or arithmetic.

## Native-first, normally exportable setting pair

Actual official OrcaSlicer 2.4.2 CLI exports, subsequently sliced afresh:

- Two walls, Aligned seam, staggered inner seams OFF, wipe OFF.
- Scarf Contour, scarf inner walls ON, minimum length 20 mm, steps 10,
  start height 0, seam gap 10%.
- Control: `seam_slope_entire_loop = "0"`.
- Positive: `seam_slope_entire_loop = "1"`.

The two exported ZIPs have identical payload names and differ ONLY in
`Metadata/project_settings.config`; that JSON differs ONLY in
`seam_slope_entire_loop`. No malformed settings or post-export ZIP edits.

| Case | Exported project SHA-256 |
|---|---|
| control | `f5180678b43d3f1895b02aa0b8c4ed096d1792c41fa0ce9217791d6ef0f7f454` |
| positive | `a99f4752e06f36102488649a944233cacb0cb2541ec42e03fa7ed431a8ac98ee` |
| no-inner control | `0a8f2e4ffac21a912c663cfd3498f166e533cfe86fad7f58f0a4ea9af18156e5` |
| zero-gap control | `5bb0a0e28d6b3ea5102b833406b9c508c25a168e959984465c6f4c2ab191c59c` |
| minimum-length-100 positive | `0d9d223e7276a81043f2e8640e7199bb40a1a6a49f015d159c8842633fb5f141` |

Each extra case changes the stated setting relative to the matching
pair: no-inner and zero-gap retain whole-loop ON; minimum-length-100
retains whole-loop OFF but sets the ordinary minimum length to 100 mm.
The latter exhausts this loop without requiring the whole-loop switch.

## Reproduced behavior and counter-controls

CPU affinity 7-8 throughout. Both official Native repeats and seeded
Native produce the same complete stream after the existing single
Orca generator identity/date-line removal. No motion, E, timing,
configuration or other output is excluded from acceptance comparison.

| Case | Official | Seeded |
|---|---|---|
| entire-loop OFF control | complete PASS | complete PASS |
| entire-loop ON positive | complete DIFF | complete DIFF |
| ON, inner scarf OFF | complete PASS | complete PASS |
| ON, seam gap 0% | complete PASS | complete PASS |
| OFF, minimum length 100 mm | complete DIFF | complete DIFF |

Both positives have 20,805 unequal aligned output lines; this includes
shifts and timing consequences, NOT 20,805 independent bugs. The first
physical movement difference is raw line 379:

    Native: G1 X119.325 Y100.755 Z.8
    Ares:   G1 X108.135 Y119.325 Z.8

Filtering to movements was used ONLY to locate this diagnostic witness;
the actual acceptance comparison remains the complete byte stream.
This changes actual XY motion and scarf coverage, not merely formatting.

The observations use approved staggered GREEN `6c3038b32` and its isolated,
uncommitted random-arithmetic probe. Their complete outputs are identical
for ALL five cases: aligned seam and stagger OFF remove those dependencies.
The owning `scarf.rs` file is also byte-identical to main `5317e1c09`
(SHA-256 `2fa6c40188fedb2adbb8f1f82a6a1bd3accbeb44b66c447292e4733cb883fefb`).
This is NOT a claim of exact-main integration or FULL evidence; those
must be obtained later at the assigned merge slot.

## Read-only GDB: first unequal intermediate

For the first eligible inner wall, both implementations have exactly the
same complete pre-crop integer vertex array: 17 vertices for the positive.
Both have one start-slope path, whose role is Perimeter. No debugger
function calls, register writes or inferior memory writes were used.

| Intermediate | Native control | Ares control | Native positive | Ares positive |
|---|---:|---:|---:|---:|
| remaining flat path count | 1 | 1 | 0 | 0 |
| pre-crop slope length, mm | 20 | 20 | 74.600004 | 74.600004 |
| effective crop distance, scaled | 8000000 | 8000000 | 80000 | 29840001.6 |
| effective crop distance, mm | 8 | 8 | 0.08 | 29.8400016 |

The first observed unequal intermediate is the crop distance AFTER role
selection, with equal pre-crop geometry. The source explains the decision:
Native's inherited `paths` is empty, hence `role()` is `erNone`, so its
requested `2 * seam_gap = 80000` is unchanged. Ares instead checks the
original Perimeter role and selects `0.4 * 74600004 = 29840001.6`.
The passing control has a flat path and correctly takes 40% in both.

This falsifies earlier-placement and pre-crop-length hypotheses for this
witness. No normalization, rounding, flow or formatting change is justified.

Evidence root: `/tmp/w5-next-entire-min1/`:

- `discovery-summary.json`: bounded settings, complete input/ZIP hashes,
  dual-mode outcomes, first physical difference and GDB snapshots.
- `verified.json`: full difference lists and fresh Native launch/artifact
  directories; each case retains `exported.3mf`, Native and Ares streams.
- `{native,ares}[-control]-clip-snapshot.{gdb,py,json,log}`:
  actual read-only positive/control captures.
- `/tmp/w5-next-entire-verify.py`: repeatable fresh Native verification;
  preserves previous artifacts by creating a new directory per invocation.

    taskset -c 7,8 /nix/store/60m4rxhg2fldqaak400c0lry96ijrzqn-python3-3.13.13/bin/python3 /tmp/w5-next-entire-verify.py

## Included rewrite / non-negotiable scope

1. In `scarf.rs::build`, select the clipping role from remaining `flats`:
   first flat path's role when present; upstream `erNone` semantics when
   empty. Only the remaining-flat Perimeter role selects 40% cropping.
2. Do not special-case the whole-loop option: exhausted flat paths are
   also reachable through the ordinary minimum-length setting.
3. Preserve ALL clipping numbers and geometry: scaling, 0.4 multiplier,
   seam-gap factors, comparisons, path construction/splitting/detailing,
   start/end clipping and slope interpolation remain unchanged.
4. Empty flats is a normal upstream state, not an error or a fallback.
   No new role type, helper abstraction, validation or panic is needed.

Deferred: random/stagger math; scarf eligibility/conditional behavior;
other loop-role or geometry changes; wipe-before-outer and the separate
wipe-retraction E witnesses from discovery; formatting and time estimation.

## RED → GREEN plan after separate authorization

- RED freezes the actual five CLI-exported projects above, whole-file
  and per-ZIP-entry manifests, and effective settings. Use the merged
  `runner/native_api_cli.rs::NativeCase`, now merged on main; never copy
  helpers or hand-write another Native/API/CLI runner. Native availability
  is the only environment condition, like main's NativeCase users; no
  borrowed PROFILE_ENTRY gate or extra gate that skips required cases.
- Fresh official and seeded Native: both positives FAIL through the
  public validated API and bare CLI; all three controls PASS. Retain all
  supplied and produced streams and comparator results as artifacts.
- GREEN contains ONLY the remaining-flat role predicate correction.
  Require all five COMPLETE Native/API/CLI cases to PASS in both modes.
  Re-read the target crop-distance intermediate: positive must use
  80000 scaled units and control must remain at 8000000. Stop and report
  any residual before expanding scope.
- Run rustfmt, clippy and Tier-1 WASM checks after implementation.
  FULL only at the assigned W5 merge slot, on the then-current rebased
  exact hash, never as a waiting/precheck activity.
