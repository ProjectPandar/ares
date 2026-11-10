# Organic interface projection counts the contact layer

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `generate_interface_layers` (`Support/SupportCommon.cpp:194-402`) is shared by the Grid,
  Snug and Organic support generators. Before projecting top and bottom contacts into the
  intermediate layers, it derives the projection counts (`:215-225`):
  - a separately printed top contact consumes one requested top interface layer, so the
    top count is `num_top_interface_layers - 1` when `has_top_contacts`;
  - a bottom contact consumes one requested bottom interface layer, except for Organic
    supports (`organic_tree`), whose bottom contacts are projection seeds;
  - the base-interface counts are clamped to these counts, and the interface-only counts are
    the difference.
- The projection slab (`:305-348`) uses these derived counts. The top slab of an
  intermediate layer ends at `intermediate_layers[idx + num_top_interface_layers - 1]`.
- Organic supports reach this function from `TreeSupport3D.cpp:3641-3642`, after
  `organic_draw_branches` has filled `top_contacts` and the precomputed
  `top_interface_layers`.

## Rust destination

- `crates/ares-core/src/project_slice/support_material/interface_counts.rs`
  (`InterfaceCounts::projected`): the derivation of `:215-225`, taking whether the support
  is Organic. The raw counts stay the support-parameter representation for the other
  consumers.
- `crates/ares-core/src/project_slice/support_material/interface_grid.rs`: the Grid and Snug
  path already derives these counts inline. It now calls `projected` instead, with the same
  result.
- `crates/ares-core/src/project_slice/support_material/tree/organic/interfaces.rs`
  (`generate`): the Organic copy of `generate_interface_layers`. Today it projects with the
  raw requested counts. It now projects with `projected(true)`. The `has_interfaces()` gate
  (`:208`) keeps using the raw counts, as upstream does.

## Included

- The contact-layer deduction and base-count clamp of `:215-225` in the Organic projection.

## Deferred

- `SupportParameters.hpp:54-68`, the number of top and bottom base-interface layers when the
  interface and base filaments differ, is not changed here.
- Contact generation, branch drawing, layer installation and toolpaths are unchanged.

## Premises

1. `skirt_per_object_flush_into_support::skirt_per_object_flush_into_support_matches_native`
   fails in both modes. The case uses Organic tree support (`tree(auto)` with the default
   style resolves to Organic, `SupportParameters.hpp:215-218`),
   `support_interface_top_layers = 2` and `support_top_z_distance = 0.26`.
2. With `flush_into_support` off, the Native and Ares outputs differ at the same layers, so
   the flush pass is not the cause.
3. A debugger capture of Native at the entry of `generate_interface_layers` and a temporary
   Ares capture at the same point agree: the top contact at Z3.8, the precomputed interface
   at Z3.6, and all 19 intermediate layers have the same Z, polygon count and bounding box.
4. After the step, Native has interface at Z3.6 and Z3.8 only, and the base at Z3.4 is
   untouched. Ares also projects the Z3.8 contact into Z3.4: with the raw count 2, its slab
   for Z3.4 ends at Z3.6 and admits a contact whose bottom is Z3.6.
5. Before writing this spec, I applied the derivation as a temporary experiment, since
   reverted. The sliced output then matched Native apart from the time estimate lines.

## Observable behavior

Organic tree support prints as many top interface layers as requested, counting the contact
layer, as Native does.

## Failure modes

- Deducting the bottom contact for Organic: upstream keeps the requested bottom count there.
- Gating on the derived counts: with one requested top layer and no bottom layers, the
  derived counts are zero, but upstream still runs the step and merges the precomputed
  Organic interfaces.
- Base counts not clamped: with a single requested layer, the interface-only count must not
  underflow.

## Test plan

The witness already exists as a whole-output parity test, is named by behavior, and fails at
the base in both modes:

| Test | Before the port | After the port |
|---|---|---|
| `skirt_per_object_flush_into_support::skirt_per_object_flush_into_support_matches_native` | F | P |

This slice adds no new test commit. The support, tree, raft and skirt tests must show no PASS
turning into FAIL in either mode.
