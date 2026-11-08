# Type2 first-wipe-line planning guard

`WipeTower2::set_toolchange` credits the unused tail of a ramming row only
when such a row exists and the wipe continues on it. Port the upstream
predicate, rather than assigning a partial-row credit unconditionally.

## Source and destination

Upstream: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`GCode/WipeTower2.cpp:2288–2294`; predicate dependencies are
`WipeTower2.hpp:323–327`, with gap-wall selection at cpp1029–1033.
The resulting credit flows through the existing first-volume subtraction
and wiping-depth calculation at cpp2295–2310.

Rust destination: `project_slice/gcode_emit/wipe_tower/plan.rs` only.
`Plan::new` retains the source SEMM flag and the existing
`generator::use_gap_wall` result. `Plan::set_toolchange` combines them
with the existing enable flag and old filament's multitool-ramming flag:

```text
do_ramming = (semm && enable_filament_ramming) || old.multitool_ramming
boundary_wipe_start = do_ramming && !semm && use_gap_wall
first_wipe_line = if do_ramming && !boundary_wipe_start {
    -(width * (length / width - int(length / width)) - width)
} else {
    0.f
}
```

Preserve the current Rust formula's f32 operations, casts and order
verbatim inside the branch. Do not replace the source predicates with
a positive-depth test or tune arithmetic to particular output values.
Both initial allocation and rib-square replanning use this producer.
Existing `Plan` data remains a temporary compatibility shell around the
cited upstream concept, not an independently designed planning pipeline.

## Included and deferred behavior

Included: source predicate transport and conditional partial-row credit.
Keep first-volume conversion, volume subtraction, wiping-depth arithmetic,
spacing, returned fields and callers unchanged. No public API changes.

Defer ramming-depth reservation, profile/time-factor transport, active-ramming
expansion, last-wipe savings, final purge, tower lift and grouping. Do not
change generation, entry scrub, unload, writer arithmetic, geometry,
configuration producers, fixtures or comparison rules.

Use the existing public byte-output SEMM cases through the core API and CLI.
Keep ordered whole-file comparison, including M73, metadata and configuration;
only the existing generator/timestamp normalization applies. Enabled-ramming
siblings remain controls. A different case's earlier divergence is not assigned
a common producer without establishing its own causal connection.

This is allocation-free, platform-neutral core logic shared by browser WASM,
Windows, macOS and Linux, without filesystem or terminal dependencies.
