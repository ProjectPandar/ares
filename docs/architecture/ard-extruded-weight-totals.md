# ARD — extruded weight/volume template totals

Non-negotiable decisions (spec: `extruded-weight-totals-spec.md`).

1. **Literal per-extruder port.** The template totals compute exactly as
   `GCode.cpp:1675-1690`: volume from the writer's cumulative filament ×
   the typed per-extruder `filament_diameter` cross-section, weight ×=
   the typed per-extruder `filament_density × 0.001`. No defaults, no
   hard-coded 1.75/1.24 anywhere in the template path.
2. **Single-boundary GREEN.** Only `EmitState::extrusion_totals()` changes;
   the `layer_gcode.rs` consumers keep their existing volume semantics.
   Queued files (`motion/scan.rs`, `format.rs`, brim/*, jerk,
   seam_placement/alignment, config value.rs, fill_entities/grid.rs,
   config_export/writer.rs) are untouched.
3. **Deferred stays deferred.** Vector `extruded_weight`/`extruded_volume`
   placeholders, `interpolate_table`, the statistics footer, and the M201
   consumer are out of scope; the M73 transpositions in the density witness
   are downstream effects that must disappear with the value fix — if they
   remain after GREEN, that is a STOP.
4. **Native-first RED.** Runtime-generated Native via
   `native_stream_case::check`; no Native artifacts committed; no silent
   pass; complete comparator never weakened; seeded mode confirmed in the
   formal RED runs.
5. **E2E-only verification.** The three public witnesses (stock control +
   two single-key targets) are the sole verification; no unit tests.
