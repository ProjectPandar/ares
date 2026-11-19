# Extra wall areas for non-organic tree support

## Native boundary

OrcaSlicer 2.5.0 `TreeSupport.cpp::draw_circles` compares a processed node's squared XY movement against the square of the scaled half support line width (`:2184`, `:2329–2330`). Convert both movement components to double before squaring, and take the threshold from the configured support line width as Native does, not the resolved extrusion flow. Use strict `>` and append the node's resulting area before routing it to base or interface areas.

After the base geometry is finalized, `:2567–2569` marks each base group independently: it needs an extra wall only if it does not need infill and overlaps the collected node areas. The existing `generate_toolpaths` Auto wall-count override (`:1816–1818`) consumes this group flag.

## Ares boundary

In `project_slice/support_material/tree/non_organic`, collect the mask in `drawing/areas.rs` and consume it in `drawing/groups.rs`. Replace the smoothing-only extra-wall heuristic and layer-wide boolean; neither models the Native overlap decision. Leave smoothing positions, movement, and radii, as well as the Auto wall-count consumer, unchanged.

Defer overhangs, contacts, drop and collision classification, height planning, shape construction and clipping, Hybrid infill and expansion, explicit wall counts, Organic and Normal support, raft, arc fitting, and timing.

## Verification

Derive a supported half-scale X1 Carbon overhang at runtime from `tests/parity/overhang.stl`. Use Tree Auto as the passing control; change only `support_style` to Slim or Hybrid for each witness. Compare the complete ordered Native and public Ares API/CLI output with only the generator line normalized, and require raw API/CLI equality. Run both Native modes and compare the same related support/tree inventory before and after the change.
