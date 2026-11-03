# Arachne unsupported-wall loop deferral

Upstream: OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/PerimeterGenerator.cpp` and `src/libslic3r/GCode.cpp`.

## Upstream ownership and order

- `PerimeterGenerator.cpp:1391-1398::is_unsupported_loop`: only nonempty
  closed loop entities whose every path is `erOverhangPerimeter` qualify.
  An open multipath or mixed ordinary/overhang loop is not an unsupported
  loop merely because its first role is overhang.
- `:1405-1455::defer_unsupported_loops`: `unsupported_wall_last` gate;
  stable nonunsupported prefix; stable descending inset-index tail;
  original entity polyline operands; attachment waves from already anchored
  entities; unsigned nearest distance with strict
  `< 1.5 * max(ext_perimeter_flow.scaled_spacing(), perimeter_flow.scaled_spacing())`;
  only unreached tail loops acquire `print_after_infill=true`.
- `:563-566`, `:578-584::traverse_extrusions`: closed-loop construction and
  source `inset_idx` carrier, orientation and existing thin-hole collection
  reversal. These remain separate from the whole-collection deferral.
- `:3093-3111::process_arachne`: complete `traverse_extrusions`, reject an
  empty collection, apply the existing optional `reorient_perimeters`, then
  call `defer_unsupported_loops` at3108, then publish loops/has-loops. The
  call is per completed surface collection, not once globally per layer or
  during individual path construction. Later infill-boundary and
  extra-perimeter handling retain their existing order.
- `GCode.cpp:7742-7755,8039-8053,8889-8933`: ordinary-wall / infill /
  deferred-wall / infill-first dispatch and region selection consume the
  flag. `:8903-8929` owns optional printed-wipe context, deferred-pass
  preseed, entity emission and post-emission append. These consumers are
  outside this producer boundary.

## Rust destination

The generator-neutral implementation resides in private
`project_slice/perimeters/unsupported.rs`, shared by the Classic and Arachne
adapters. The former `classic/perimeter_append/unsupported.rs` location was a
compatibility shell around a shared upstream helper, not a distinct algorithm.
There is one implementation, without a legacy alias, re-export shim or
numerical variant. Its predicate, stable tail, wave boundaries, nearest query,
strict comparison, original vertices and final flag assignment remain shared.

`classic/perimeter_append.rs` retains the gate, actual Flow operands and call
order after the narrow-aware `reorder_walls`. Reorder, sandwich and thin-hole
behavior are unchanged.

`arachne/materialize.rs::materialize_record` calls the shared helper within the
nonempty `outcome.collection` block, immediately after optional
`reorient_perimeters` and before `perimeters.push(outcome.collection)`, matching
upstream3108. The gate is the resolved `region.unsupported_wall_last`. Touch
distance uses `input.ext_perimeter_flow.scaled_spacing(scale)` and
`input.perimeter_flow.scaled_spacing(scale)` in the same arithmetic order as
the Classic adapter. Path width, nozzle width and Arachne's separately
quantized wall-generation spacing are not substitutes for these operands.

`arachne/traverse.rs` retains neutral `print_after_infill=false` initialization
and populated `inset_idx`. Existing layer-region, island and seam-local clones
transport the flag without inferring it during construction or adding another
carrier. Arachne materialization remains a compatibility shell around
`process_arachne`, not an independently designed wall scheduler. Core remains
rendering-neutral and portable to browser WASM, Windows, macOS and Linux;
filesystem/UI/terminal behavior and public API or crate expansion are excluded.

## Included and deferred behavior

Include only the Arachne source producer call, its existing gate/Flow operands
and shared-helper placement. Closed all-overhang loops may reorder and defer;
ordinary paths/multipaths can anchor waves without becoming deferred loops.
Disabled configurations remain unchanged.

Defer Arachne wall generation, beading/Voronoi, wall-order selection,
topological/sandwich ordering, overhang clipping/width interpolation,
fuzzification and seam scoring/placement changes. Thin/narrow-hole predicates,
orientation, Classic producer behavior, reach heuristics, attachment kernels,
EPS/margin/tie policy, Flow repair and extra-perimeter/support/infill generation
are outside this boundary.

GCode region/pass ownership, printed-wipe construction/execution, ordinary
wipe/retraction/travel, arcs/formatting, cooling/time/M73, config/footer/preset
and calibration admission remain unchanged. New UI controls, legacy fallback
and global reversal are excluded. This producer does not establish parity for
all Arachne supported-wipe combinations or for unrelated geometry consumers.

## Failure inventory

Failure modes include treating open or mixed-role entities as unsupported
loops; losing stable prefix/tail ordering; changing attachment-wave boundaries;
substituting width for source spacing; changing strict distance comparison or
nearest operands; deferring attached walls or ordinary paths; losing the flag
through clones; and changing disabled behavior. Changes to closed-loop
orientation, Classic reorder/sandwich behavior, infill dispatch, printed-wipe
ownership or unrelated serialized movement are outside the included producer
behavior.
