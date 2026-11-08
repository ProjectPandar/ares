# Type2 gap-wall entry scrub

## Source boundary

This slice rewrites `WipeTower2::toolchange_Wipe` in OrcaSlicer
`345efec64c7b95a05ce5cf96df0dde66a158835c`,
`GCode/WipeTower2.cpp:1924–1953`, using the existing core Type2 writer.
The row-lattice and old-tool box-completion contracts in
[type2-no-ram-purge-row-lattice.md](type2-no-ram-purge-row-lattice.md)
remain unchanged.

Rust destination: `project_slice/gcode_emit/wipe_tower/wipe.rs`.
`prime.rs` and `tool_change.rs` transport only the source `priming` flag
(`WipeTower2.hpp:425–431`). The previous post-row config-gated spiral
is a temporary compatibility shell replaced by this upstream pre-row
entry sequence, not retained as a fallback.

## Guard and sequence

Run only on row zero with a gap wall, outside interface and priming,
heading left-to-right and within `2.5f * line_width` of the left edge.
Capture traversed X before the branch, as in the source; completion
accounting still measures the whole purge row.

The ironing length starts at `3.f`, bounded by the remaining box width
with the same comparison and nonnegative clamp. Extrude that length at
the current wipe speed; retract using the current filament's length and
speed multiplied by `60.f`. Travel back by `1.5f * ironing_length` at
`600.f`, then forward by `0.5f * ironing_length` at `240.f`.
Capture `iron_end` from that resulting position plus ironing length.
Run the existing dry spiral at the current entry position with current
filament area, perimeter-width step, and `flat_iron_speed = 10.f * 60.f`
(cpp49). Travel to `iron_end` at wipe speed and reverse the retraction.
Only then emit the existing full-row endpoint.

Use the existing writer operations without changing their float order,
rounding, feedrate suppression, extrusion accounting or spiral generation.
`WipeTowerWriter2::retract` is `load(-e,f)` (cpp571–572).
A zero spiral area suppresses only spiral motion, not the surrounding
entry sequence. No independent flat-ironing config gate is introduced.

## Excluded behavior and API boundary

No planning, first-wipe-line credit, ramming-band expansion, tower placement,
filament grouping, final purge, tower lift, or geometry changes are included.
The first-wipe-line producer remains a separate source slice. Existing core
structures remain compatibility shells around the cited upstream functions.

The public byte-oriented API and CLI observe complete ordered Native output
through the existing SEMM suite. Their premises, inputs and strict comparator
remain unchanged, including M73, metadata and configuration. The five active
ramming siblings are controls; an unrelated earlier first-layer difference
is not attributed to this branch without causal proof.

This remains platform-neutral core logic shared by browser WASM, Windows,
macOS and Linux, with no filesystem, terminal or native runtime dependency.
