# Type2 post-toolchange gap entry

## Source boundary

Upstream revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

| Upstream file under `src/libslic3r/` | Ownership |
| --- | --- |
| `GCode.cpp:1696-1734` | Existing pre-toolchange approach conditions. |
| `GCode.cpp:1760-1794` | `WipeTowerIntegration::append_tcr2` post-change entry when no approach occurred, the result is not priming, and the gap wall is enabled. |
| `GCode/GCodeProcessor.cpp:4310-4407` | `get_last_position_from_gcode` recovers emitted motion endpoints from complete G0/G1/G2/G3 lines with partial-axis updates. |
| `GCode.cpp:11496-11503` | Origin and active-extruder-offset conversion through `gcode_to_point`. |
| `GCode/WipeTower2.cpp:1395-1411,1723-1737,1827-1848` | Custom-change substitution and the later mandatory generator positioning move. |

## Rust implementation owner

Commit `ce4cb5879` supplies this integration slice through
`project_slice/gcode_emit/wipe_tower/emit.rs::append_entry_travel`, the shared
`wipe_tower/last_position.rs` scanner, and `motion/tower.rs::entry_travel`.
The custom-dock control exercises that implementation. It does not introduce a
second scanner or emitter.

The existing Rust tower integration is a temporary compatibility shell around
the cited upstream owner. Preserve its pre-change approach, incoming-tool
installation, transforms, wall-opening route, Z restoration, deretraction and
mandatory generator move. No invented fallback point is allowed when neither
emitted motion nor a last defined position supplies a route origin.

Type1 generation, temperature parking and waits, ramming, automatic finishing
selection, mixed-filament selection, depth/rib/width producers, carousel and
geometry algorithms are outside this slice.

## Public verification

Keep complete Native/API/CLI ordered-byte comparisons with generator-identity
normalization only. The zero-receiving-minimum control exercises the ordinary
post-change entry. The custom-toolchange right-side-motion control distinguishes
an emitted dock endpoint from the earlier object position. Retain the
non-finishing receiving-minimum case as a neighboring allocation control.
These tests do not replace complete-output equality with waypoint or volume
assertions. Runtime receipts belong outside this architecture record.
