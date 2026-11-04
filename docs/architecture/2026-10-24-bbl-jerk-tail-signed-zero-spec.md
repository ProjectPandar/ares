# SPEC: preserve signed zero in shared writer ostream formatting

SPEC and RED; no GREEN or FULL in this stage. Any new GREEN residual requires
STOP and a report, not broader fixes or comparison masks.

## Source owner and architectural constraints

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:58-59::apply_print_config` copies raw
Z/E jerk values into caches. `set_jerk_xy`, lines 298-299, streams them with
precision two. C++ ostream retains negative zero, confirmed independently by
fresh Native output from the accepted saved `-0` input.

Rust destination: `gcode_emit/motion/format.rs::ostream_with_precision`, its
existing `value == 0.0` branch. It currently always returns `"0"`. After RED,
change ONLY this branch to select `"-0"` when the value's sign bit is
negative and `"0"` otherwise. Preserve the single implementation, default-six
wrapper, all consumers, nonzero rounding/exponent/trailing-zero logic and axes.
No precision-specific patch, second formatter, cache or configuration changes.

This is a new source rewrite slice explicitly deferred by the earlier precision
parameterization. The formatter remains a compatibility shell around upstream
ostream numeric serialization, not an independent G-code number design.
The shared change also affects default-six consumers that receive negative
zero; positive zero and every nonzero result must remain unchanged. The public
RED proves precision-two BBL Z, not a new default-six witness.

Included: the shared zero branch only. Deferred: fixed-PA's independent numeric
formatter, zero-E command admission, axis/extrusion quantizers, near-zero
thresholds, config serialization, cache predicates, origin/travel and time
estimation. Do not edit travel/origin, tokenizer/fan/zero-E, brim,
value.rs or other consumers to compensate for a residual.

## Saved-project witness

All cases derive from the existing GUI-generated
`tests/parity/bbl-layer-object-labels/one-object.3mf`, retain stock marlin flavor
and BBL identity, disable the tower, and set `default_jerk=9` and
`travel_acceleration=6000`. Only the target's `machine_max_jerk_z` becomes `-0`;
the positive-zero control uses `0`, and the other control retains stock Z=3.
Geometry, templates, PA, temperatures, fans and other process settings remain
unchanged. No cross-family archive, physical tuning or GUI decimal claim.

| Case | SHA256 |
|---|---|
| Stock control | 6cb522350420360807f04e3e920a71c71286a92583f2d513a8def2eec5b2c975 |
| Negative-zero Z target | c9eb20cb0b3390c71180e76854c80c9f81357e24e40abfba652f6582584f4891 |
| Positive-zero Z control | aa4f1d49d031216cc1fc3c325bc9562f0c3527e7499ac687336e1a6ce8421ce7 |

First normalized target difference: byte 67330. Native emits
`M205 X9 Y9 Z-0 E2.5`, Ares emits `M205 X9 Y9 Z0 E2.5`. The entire target diff
contains 299 such Z-sign differences. Line counts and every other output byte,
including M73, raw headers, configuration and footers, match. Both controls
match complete streams. This is byte fidelity; zero's numeric motion semantics
are unchanged.

All six fresh Native discovery slices replay identically; all Native/CLI exits
are zero. Discovery is not public API, seeded, RED or FULL qualification.

## RED seam and failure modes, before implementation

Freeze the three discovery archives. Add direct calls to existing
`native_stream_case` for fresh Native replay and full Native/API/CLI comparison.
Callbacks read the saved Z value and inspect actual Native body M205 Z words.
For zero, compare classification and sign bit separately; numeric equality
alone cannot distinguish signed zeros. For stock Z, compare the saved value.
No Ares formatter calls, hardcoded Native coordinates/counts or output goldens.

Run official and seeded modes, keeping timing, headers, configuration and
footers with only established generator-line normalization. Require raw
API==CLI. Callbacks are observations, not filters; commit no Native outputs.
Expected RED: target fails at the Z sign, stock and positive-zero controls pass,
all replay and callback observations qualify. RED precedes production changes.

Failure modes: erasing negative zero, making positive zero negative, confusing
numeric equality with sign equality, changing nonzero/default-six formatting,
patching only the BBL caller, or compensating in config/axis/zero-E code.

## Shared-consumer GREEN regression contract

Inventory at the base tip, searching production references to `format::ostream`:

| Consumer | Existing directed regression coverage, both modes |
|---|---|
| motion/jerk.rs | jerk_dedup_tolerance, machine_jerk_rounding, repetier_jerk_precision, klipper_separate_jerk_precision, klipper_separate_jerk_xy_clamp, bbl_jerk_tail_precision, bbl_separate_jerk_tail_gate |
| motion/acceleration.rs | merged_jerk_tolerance, merged_jerk_precision (square corner), accel_to_decel_precision, velocity_limit_comments, machine_acceleration_cache_rounding, repetier_travel_cache_gate, klipper_velocity_limit |
| expression.rs numeric text | existing placeholder_signed_zero, placeholder_digits_rounding, placeholder_rounding_results, placeholder_minmax_results, placeholder_int_result and placeholder_brace_output |
| motion/writer_state.rs coordinate text | existing bbl_layer_object_labels and print_order_as_obj_list |

Fixed PA uses its independent formatter, but GREEN additionally
runs existing pa_signed_zero (negative/positive zero and disabled),
bbl_pressure_advance_precedence and pressure-advance boundary cases in both
modes. Run the existing core formatter neighbors as well; add no post-code
unit tests. This inventory extends beyond the initially listed motion fields:
expression and coordinate consumers also share the changed zero branch.
Their files remain untouched, including origin/placeholder logic.

All regression runs are bounded directed checks, not FULL or a non-BBL stock
preset sweep. Record actual test counts and seam-specific qualifications;
legacy compare_case tests do not imply native_stream_case replay/raw equality.
If a filter matches nothing or a new residual appears, STOP and report instead
of claiming coverage or expanding production scope.

After RED: zero-branch-only GREEN, frozen RED inputs/tests. Run fmt, workspace all-target clippy,
three cases in both modes and every listed existing caller/PA regression in
both modes. Preserve registration separators, root registries and the 400-line
Rust file limit. Do not fix any new residual in this slice.
