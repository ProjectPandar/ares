# Arachne center walls follow the selected wall order

## Source boundary

OrcaSlicer v2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/Arachne/WallToolPaths.cpp:810-818,885-899`,
`WallToolPaths::getRegionOrder`, changes the ordering constraint for adjacent wall
insets. Odd center lines follow the selected inner-to-outer or outer-to-inner order
just like even enclosing wall loops. The old special case forced an odd center line
to follow its enclosing even wall regardless of the selected direction, and omitted
other odd adjacency constraints.

`PerimeterGenerator.cpp:2918-3000`, `PerimeterGenerator::process_arachne`, consumes
these constraints before greedily choosing available paths. This slice ports only
the constraint policy, not the greedy scheduler, diagonal-neighbor lookup or source
input geometry.

## Rust destination

`crates/ares-core/src/arachne/wall_toolpaths/region_order.rs::region_constraint`
retains the old odd-wall special case. The existing `get_region_order` and
`project_slice/perimeters/arachne/order.rs::order_walls` are the owning rewrite
boundary. Replace the old odd-specific return with the same selected-direction
rule used for even walls. Keep inset adjacency, distance/width arithmetic, spatial
lookup, deduplication and candidate traversal unchanged.

This is an ordering policy over existing generated wall lines. It introduces no
new Clipper2 operation or numeric geometry fix. If public generation reveals an
independent geometry mismatch, record the exact boundary instead of compensating
inside the ordering constraint.

Deferred: outline/Clipper2 migration, collinear tolerance/self-intersection changes,
new beading or top-surface behavior, precise seams, sandwich-order behavior and
any unrelated greedy-order change.

## Public witness and failure modes

The fixture is a constant-width rectangular prism, 1.4 by 10 by 10 mm, from the
existing minimal cube fixture with only the X vertex coordinates narrowed. It
uses Arachne, two wall insets, inner-wall/outer-wall sequence, zero shells/infill,
no first-layer one-wall override, brim, support or elephant-foot compensation,
and precise-outer-wall disabled. The narrow open center line is an inner wall
adjacent to an enclosing outer wall.

Failure modes are retaining the old even-before-odd constraint, losing all odd
constraints instead of applying selected direction, or accidentally changing
adjacency/distance geometry. The public witness observes the inner-to-outer case;
it does not claim coverage of every odd/odd adjacency or outer-to-inner control.

`arachne_inner_outer_prints_center_line_before_outer_loop` first qualifies stable
runtime official-beta Native replay and requires an Inner-wall role before the
Outer-wall role, including a single observable center-line move along `X110`.
It then strictly compares the first object-layer output against
`ares_core::slice_project_plate_validated`, including wall-entry travel, widths
and E bytes. Expected output comes only from runtime Native, not a committed
Native artifact or copied implementation. Both full streams and compared blocks
are retained externally. No complete-output or CLI parity claim follows from a
matching selected layer block.

No private test is added or executed. An existing private test encodes the old
special-case policy; its authorized deletion is part of the pending source port,
replaced by the public output witness rather than updated private structure
assertions. The test root remains byte-identical to main; new registration belongs to
the existing option-domain group. The approved bottom/top shell gates and their
public witnesses remain unchanged.

Per-slice gates are the single directed public RED/GREEN filter and fmt/workspace
all-target Clippy. No batch merge is authorized by this document; whole-output
migration and ledger linkage remain integration responsibilities.
