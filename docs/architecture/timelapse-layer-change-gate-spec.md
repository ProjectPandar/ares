# Timelapse hook exclusivity

Status: Accepted slice specification.

## Decision and upstream boundary

Upstream pin: OrcaSlicer v2.5.0-beta `345efec64`.

At this pin a layer renders `time_lapse_gcode` through one of two mutually
exclusive paths, and `need_insert_timelapse_gcode_for_traditional` picks it.

| Upstream | Behavior |
| --- | --- |
| `GCode.cpp:6689-6692` | The gate: no tower printing a smooth timelapse, a BBL printer or a nonempty template, and an I3 printer outside spiral mode or more than one nozzle. |
| `GCode.cpp:6710-6715` | The layer-change render runs only when the gate is false, on a non-BBL printer with a nonempty template. |
| `GCode.cpp:7386-7392` | The BBL layer-start insert, also only when the gate is false. |
| `GCode.cpp:7484-7502`, `:7516-7530`, `:7743-7748`, `:8044-8049`, `:8180` | The traditional inserts, each only when the gate is true and at most once per layer through `has_insert_timelapse_gcode`. |
| `GCode.hpp:125,154`, `WipeTower.cpp:1993` | `enable_timelapse_print()` is `timelapse_type == tlSmooth`. |
| `GCode.cpp:4718-4719` | `m_wipe_tower` exists only when the print has a tower. |

Ares destination, both in `crates/ares-core/src/project_slice/gcode_emit/layers/`:

- `boundary.rs` `timelapse_at_layer_change` gains the missing `!gate` term and reads
  the value `timelapse.rs` already stores in `state.traditional_timelapse`.
- `timelapse.rs` `gate` gains the missing tower term, so that stored value is the
  upstream boolean. `state.wipe_tower` is the `m_wipe_tower` counterpart.

## Why both terms

The layer-change term alone is not enough. On a smooth timelapse project with a
tower, upstream's gate is false, while Ares's value ignores the tower and is true.
Adding only the `boundary.rs` term would remove the correct layer-change render on
such a project and keep the incorrect traditional one. The tower term also corrects
the other consumers of the same value: the traditional inserts, and the BBL
layer-start insert at `boundary.rs`, which reads the value as upstream's `:7386` does.

No other gate logic changes. Farthest-point timelapse, the two-nozzle photo-head
selection and the insertion sites themselves stay as they are.

## Deferred behavior

- Belt printers, which never have a tower (`Print.cpp:5189`).
- `m_spiral_vase` is read through `spiral_mode`, as the existing gate does.
- Any difference a test reaches only after this gate is fixed. The earlier legs of
  `timelapse_templates_render_current_logical_filament_vectors` stopped those from
  being seen; they are localized and reported, not folded into this slice.

## Verification

- **Test premises.** `timelapse_context_observation.rs` asserts two hooks per layer,
  one in each phase. That was true before the gate existed and is stale. The test
  derives its expectation from the gate instead: Native's config block supplies
  `printer_structure`, `spiral_mode`, `nozzle_diameter`, `timelapse_type` and
  `enable_prime_tower`, the gate selects the phase, and each layer has exactly one
  hook. No hook count or Native output is stored.
- **Traditional path.** That test, with its implicit, handoff and explicit legs, must
  match Native's complete output. All of them run for the first time with the fix.
- **Smooth tower path.** A new test sets a template on the smooth timelapse tower
  project `tests/parity/validation/timelapse-unprintable-gcode.3mf` at runtime, with
  an on-bed tower position so that it does not depend on the pre-slice fit. Native
  renders at layer change. Ares must match its complete output. No derived project
  is committed.
- **Existing tests with the same first difference.** These reach the extra
  layer-change render today and must be examined with the fix:
  `timelapse_type_conditional_template_and_empty_controls` and
  `infill_first_ironing_preserves_traditional_timelapse`. Each is reported with its
  result, passing or not.
- **Regression.** The `P->F` delta against the base must be empty in official and
  seeded modes.
- **Record.** `option-timelapse-type-bounded-coverage.md` notes two hooks per layer
  as observed. It is corrected to the single render at this pin.
