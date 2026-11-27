# Admit skirt settings on belt printers without generating skirts

## OrcaSlicer boundary

The reference is OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
Paths below are relative to `src/libslic3r` unless noted otherwise.

- `Print.cpp:833–850`, `Print::has_infinite_skirt` and `Print::has_skirt`,
  return false in belt mode. `Print::_make_skirt` (`4030–4036`) returns
  before building loops, outlines or groups. `Print::process` (`3875–3907`)
  keeps belt brim independent of the ordinary skirt/brim groups, and
  `GCode.cpp:5816–5842,6004–6013` requires populated skirt data to emit it.
  Thus a valid `skirt_loops`, `skirt_height`, and `skirt_type` (`combined`
  or `perobject`) do not produce a skirt in a by-layer belt print.
- `Print::validate` (`2144–2152`) explicitly rejects non-disabled
  `draft_shield` on belt printers, independently of skirt loop count or
  height. It does not reject the ordinary skirt settings for that mode.
  The valid configuration values remain those from `PrintConfig.cpp:576–586,
  6744–6789`; this does not exempt malformed values from general validation.
  `src/OrcaSlicer.cpp:7007–7042` reports the draft-shield failure through
  the generic print-validation family, before export.
- `GCode::append_full_config` (`GCode.cpp:8272–8274,8370–8380`) serializes
  the resolved configuration. The compatible G-code export writes it at
  `4912–4928`. The absence of skirt geometry does not zero or delete the
  configured skirt values.

## Rust destination

In `crates/ares-core/src/project_slice/belt.rs::admit`, replace the
blanket skirt rejection with the narrower `Print::validate` draft-shield
rejection. Ordinary skirt options do not take part in belt admission.
Report an enabled draft shield as `SliceError::Validate` with
`PrintValidationKind::Generic` and the diagnostic `Draft shield is not
compatible with belt printer mode.` Use the resolved plate's validation
context and the existing CLI translation in
`crates/ares-cli/src/failure/mapping.rs`; do not introduce a new error family.

The existing `gcode_emit/skirt/generate.rs::SkirtPlan::generate` belt
short-circuit already represents the upstream generation boundary. Keep it
unchanged, including for active belt brim. Do not clear resolved settings
to suppress emission: `options/config_export/writer.rs::write_config_block`
should continue to serialize them through the normal export path. There is
no new belt skirt geometry, planner, or config normalization in this slice.

## Public behavior and verification

Use synthetic variants of the existing belt cube in
`crates/ares-cli/tests/orca_parity/belt_print.rs`. Compare the emitted
G-code for each accepted project to that same project's Native result
using the existing generator-only comparator, including its config block.
Check that the positive cases have no skirt extrusion and that the active
belt brim case still prints its brim. For the rejection cases, check the
public diagnostic, the generic validation result (`return_code = -51`
on Linux), and the absence of G-code. Run both Native reference variants.

The paired Native probe produced the following results. The equality
column compares each project's output to its Native control, disregarding
only the generator timestamp and config payload for this diagnostic
comparison. The public Ares-to-Native byte comparison keeps the payload.

| Input change | Output outside config | Changed config entries |
| --- | --- | --- |
| Three combined loops, height four | Equal to no-skirt control | `skirt_loops`, `skirt_height` |
| Three per-object loops, height four | Equal to no-skirt control | `skirt_loops`, `skirt_height`, `skirt_type` |
| Three combined loops, height zero | Equal to no-skirt control | `skirt_loops` |
| Three per-object loops, height four, active outer belt brim | Equal to same brim without skirt | `skirt_loops`, `skirt_height`, `skirt_type` |
| Draft shield enabled with zero or three loops | Rejected before export | Not applicable |

## Deferred behavior

The supported topology remains a by-layer project with one printable
instance and one material. Multi-object sequential clearance is distinct:
`Print::validate` calls `sequential_print_clearance_valid` for multiple
by-object copies (`Print.cpp:2238–2254`), and that function can read
`object_skirt_offset` even on belt printers (`895–931,5802–5822`).
That check, arrangement, and non-belt skirt behavior are unchanged.

Spiral vase stays outside this admission. Upstream rejects it with an
active belt brim (`Print.cpp:2153–2162`) but otherwise can use it. Its
layer filter still reads the configured `skirt_height` even when belt
skirt geometry is absent (`GCode.cpp:6655–6671`). Do not normalize that
setting or remove the spiral-vase restriction as part of this change.
