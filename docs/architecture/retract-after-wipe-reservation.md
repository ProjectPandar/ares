# Retract-after-wipe reservation

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/PrintConfig.hpp:1793` and `PrintConfig.cpp:6126–6134`:
  `retract_after_wipe` is a percentage vector, default zero.
- `Extruder.cpp:202–213`: multiply percentages by `0.01`, clamp to `[0,1]`,
  and cap the after fraction at `1 - retract_before_wipe()`.
- `GCode.cpp:538–602`, `Wipe::calculateWipeRetractionLengths`: reserve
  before and after amounts before calculating remaining/during retraction.
- `GCode.cpp:10691–10722`, `GCode::retract`: emit before, then wipe;
  the existing subsequent ordinary retract supplies the after remainder.
- `PrintConfig.cpp:9007–9090`: nullable filament retraction overrides and
  printer retraction vectors use the existing configuration projection rules.

## Rust destination

The typed option path is `options/gcode_fields.rs`,
`options/project_runtime_options/{gcode_source.rs,wire.rs,gcode_source/wire.rs}`,
`options/filament_options/{retract_overrides.rs,retract_overrides/wire.rs,wire/middle.rs}`,
`options/project_config_views/retract.rs`, `options/project_variants/{printer,filament}.rs`
with the relevant `options/registry/key_lists.rs` transport categories.
Registration and source defaults already exist; do not duplicate them.

The consumer boundary is `project_slice/gcode_emit/motion/options.rs`,
`options/build/fields/{wipe,assemble}.rs` and
`travel/wipe_sequence.rs::retract_and_wipe_impl`.
The before-only option and budget shapes are temporary incomplete shells
around the cited upstream retraction concept, not an independent pipeline.

## Arithmetic and lifecycle invariants

Carry the effective per-filament after percentage through the existing typed
configuration/nullable override path. Resolve the fraction with Native's
multiplication, clamp and upper cap. Compute full length times before and
after fractions, then subtract their sum from the full length. Preserve the
source EPSILON returns, during speed budget/cap and before increment order.
Do not substitute algebraically equivalent reassociations.

Do not add an explicit after-only writer operation. Preserve existing writer
override dispatch, restart-extra assignment, post-wipe top-up, Hilbert
suppression, wipe eligibility, firmware behavior and reset-E ordering.

## Included and deferred behavior

Include typed percentage transport, nullable filament override transport,
variant projection and the literal after-reservation arithmetic only.
Defer wipe geometry/clipping, seam/precision, path ordering, lift policy,
filament grouping, tower generation/planning/ramming and final purge.
The standalone non-project G-code adapter is outside this slice.
No replacement retraction pipeline, local shadow configuration reader or
legacy fallback is introduced. Core logic remains platform-neutral for
browser WASM, Windows, macOS and Linux.

## Test premise

Before production changes, a public byte-input Native/API/CLI test must fail
with a positive Native premise: eligible real wipe motion, nonzero effective
after fraction, and a phase split different from a zero-after input.
A nonzero footer alone is not activation; before equal to one caps after to zero.
Then require ordered whole-output equality, API/CLI raw equality and bounded
zero-after/inactive controls in both official and seeded modes. Keep metadata,
configuration, progress commands and ordering in the comparison, using only
the existing generator normalization. No numeric positions are premises.
