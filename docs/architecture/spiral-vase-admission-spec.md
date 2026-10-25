# Spiral vase multi-copy by-layer admission — source-cited spec

Status: SPEC approved (coordinator ACK 2026-10-25); freezes the boundary for
the tests-only RED, then a bounded GREEN on the W1 branch chain
(base tip `a62041e78`).

## Witness (discovery on main `28539c7fb`, BBL A1 two-object scene)

`a1-spiral` (`a1-bylayer.3mf` + `spiral_mode = 1`, two objects, by layer,
input sha `0bebe385b85e9aba0d317b66c154d735d8064c4b66d3e0081a3c77209a494050`):

- Native (official and seeded, fresh binaries): exit **205**, stdout
  `run found error, return -51, exit...`, stderr
  `Please select "By object" print sequence to print multiple objects in spiral vase mode.`,
  output dir carries only `result.json` (`return_code: -51`, the generic
  validate `error_string`, `layer_height`/`wall_loops`/`sparse_infill_density`).
- Ares (`a62041e78` fresh build, sha `b50d4b9657c23aa5b612d16e89acbd1b2d6d7a6d418e92771aa88342cd43658f`):
  exit **0** with a complete spiral G-code (`; spiral_mode = 1`,
  `; print_sequence = by layer`).

Stable across both Native modes; admission class, not an unknown-key echo.

## Upstream boundary (pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`)

- `Print.cpp:1334-1341` (`Print::validate`): with `spiral_mode` on,
  `total_copies_count = Σ object->instances().size()`; if it exceeds 1 and
  `print_sequence != PrintSequence::ByObject`, return the diagnostic above
  with `opt_key = "spiral_mode"`.
- The error rides the existing validate channel: CLI exit
  `CLI_VALIDATE_ERROR` (-51, `Utils.hpp:62` → shell 205) and the
  `result.json` validate fields (`OrcaSlicer.cpp:5981-5987`).

## Ares destination boundary

- `crates/ares-core/src/print/validate.rs::validate_print` — add exactly the
  copies check: sum instance counts over the project objects; when
  `spiral_mode` is set, `total_copies_count > 1` and the typed print
  sequence is not by-object, return the existing `SliceError::Validate`
  with the exact upstream message. Typed reads only; no new exit plumbing
  (the `Validate` → -51 → 205 CLI mapping already exists and is exercised by
  the `validate` fixture family).
- `print/validate.rs` stays under 400 LOC.

Not the queued `motion/scan.rs`, `format.rs`, brim/*, jerk,
seam_placement/alignment, `config_export/value.rs` (W6), or
`fill_entities/grid.rs`; not the W1-touched `config_export/writer.rs`,
`gcode_emit/motion/state.rs`, `gcode_emit/layer_gcode.rs`; no W3 geometry or
W5 flush/wipe surface.

## Included

- The `Print.cpp:1334-1341` copies check with the exact diagnostic and the
  existing -51/205 exit path.
- RED guards proving no over-rejection: one copy by layer, and two separated
  copies by object, both accepted by Native and Ares at runtime.

## Deferred

- `Print.cpp:1342-1353` (spiral vase multi-material regions check) — not
  witnessed; separate slice if a witness appears.
- **BBL spiral emission byte parity** (new findings from the guard probes,
  out of this slice's scope): single-object A1 spiral first diff byte 81880
  (Native `G1 Z0.600` layer step vs Ares continuous Z); single-object X1C
  byte 72286 (`G1 Z0.400`); two-object by-object-apart even diverges on
  `; total layer number` (83 vs 82). Accepted-scene guards therefore assert
  acceptance only; the witnesses' own assertions stay full channel and
  `result.json` equality.

## RED plan (tests-only, on `a62041e78`)

Fixtures under `tests/parity/spiral-vase-admission/` (see the README there;
frozen SHAs in the review request):

| Case | Base edit | Expectation |
|---|---|---|
| `two-object-by-layer` (witness) | `spiral_mode=1` | Native -51/205 + diagnostic twice-deterministic; Ares currently exit 0 → RED fails at the exit assertion |
| `single-object-by-layer` (guard) | drop the overhang object, `spiral_mode=1` | Native exit 0 + `plate_1.gcode`; Ares exit 0 + G-code |
| `two-object-by-object-apart` (guard) | `spiral_mode=1`, `print_sequence="by object"`, cube item moved apart | Native exit 0 + `plate_1.gcode`; Ares exit 0 + G-code |

Runtime Native in both official and seeded modes (`ARES_REQUIRE_ORCA=1`);
no Native artifacts committed; complete comparator untouched.

## Ways it could fail (checked by the cases)

- Counting objects instead of instances (a future multi-instance scene);
  counting disabled/unprintable instances; a 0/1-based sequence mix-up;
  rejecting by-object or single-copy scenes (guards); emitting the diagnostic
  on the wrong channel or with a wrong exit code; producing a G-code file
  alongside the rejection.

## Gates

| Stage | Gates |
|---|---|
| RED | the three cases above, official + seeded: witness fails only at the Ares admission assertion; guards pass |
| GREEN | witness passes both modes byte-exact on channels + `result.json`; guards still accepted; `cargo fmt`; strict clippy; nextest `ares-cli` directed; no FULL without COORD |
