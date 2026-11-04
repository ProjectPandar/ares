# Ironing geometry option activation audit

Outcome: six activated domains pass both references; no production bug found.

## Source boundary

OrcaSlicer 2.4.2 `libslic3r/PrintConfig.cpp:3493–3528` defines filament
ironing flow, spacing and inset overrides;4332–4370 defines process inset,
angle and fixed-angle controls. `Fill/Fill.cpp::Layer::make_ironing`
(around1593–1611) selects filament overrides and computes ironing geometry.
The corresponding Rust boundary already exists in
`project_slice/fill_entities/ironing.rs` and region option resolution.
This is an upstream-boundary audit, not a new Ares pipeline feature.

The frozen16ac907b sweep has423 PASS and225 INCOMPLETE across648 selected
domains,1655 successful full comparisons and one timelapse rejection. There
are no DIVERGENT artifacts to repair. Its activation table enables ironing
for process flow/spacing/speed/pattern but not for the six controls below.
Their previous PASS therefore does not prove active geometry coverage.

## Test-first plan and constraints

1. Inject `ironing_type=top` for
   `filament_ironing_flow`, `filament_ironing_spacing`,
   `filament_ironing_inset`, `ironing_inset`, `ironing_angle` and
   `ironing_angle_fixed`. Keep existing cases, value verification and strict
   full-output comparison unchanged. Do not change production code.
2. Replay both native references serially, retain saved presets,
   saved3MF and full outputs in new directories. Check native ironing
   extrusion in suitable nonzero/nonempty controls; empty geometry at a
   limiting value must not be mistaken for a disabled feature.
3. Only after a real DIVERGENT exists, isolate it and specify the exact
   source-owned geometry behavior before any implementation. If all outputs
   pass, record that result, not a fabricated fix. Report native rejection
   separately from output mismatch or incomplete schema coverage.

Exclude fan lifecycle, cooling, motion speed emission and all
`gcode_emit` changes. The separate ironing-fan slice is untouched. No comparator relaxation,
private assertion removal, implicit skip or unsupported guard is allowed.
Any subsequent implementation requires ordinary full quality gates.

## Result

Official and seeded runs each pass6/6 domains,17/17 full comparisons, with
zero rejected cases. All17 native outputs record `ironing_type=top`.
Each reference has11 cases with positive-E XY ironing extrusion. Six limiting
cases have none: minimum filament flow/spacing and maximum/seeded process
or filament inset. These remain full-output comparisons, not skipped tests.
There is no DIVERGENT to repair in this group.
