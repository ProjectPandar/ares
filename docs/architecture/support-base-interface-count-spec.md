# Support base-interface layer count with a separate interface filament

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `SupportParameters::SupportParameters` (`Support/SupportParameters.hpp:38-69`) derives
  `num_top_base_interface_layers` and `num_bottom_base_interface_layers`.
  - With a zero gap and a soluble interface over a non-soluble base (`:38-44`), the count is
    `min(n / 2, 2)` (`:57-58`, `:64-65`).
  - Otherwise (`:61`, `:68`), the count is 1 when the interface filament differs from the
    support filament (`different_support_interface_filament`, `:54-55`) and more than one
    interface layer is requested; otherwise it is 0. The gap does not take part. A
    `support_filament` of 0 with a nonzero `support_interface_filament` counts as different.

## Rust destination

- `crates/ares-core/src/project_slice/support_material/interface_counts.rs`
  (`InterfaceCounts::new`, the `base_count` closure). The current scaffold, kept from an
  earlier baseline, returns 1 when the filaments differ and the gap is zero. It now returns
  1 when the filaments differ and the requested count is greater than 1.

The consumers of `top_base` and `bottom_base` are unchanged and receive the corrected values:
- Grid and Snug interface projection: `interface_grid.rs`.
- Organic interface placement and projection: `tree/organic/interface_placer.rs` and
  `tree/organic/interfaces.rs`.
- Hybrid tree roof grouping: `tree/non_organic/drawing.rs`.

## Included

- The non-soluble branch of `:61` and `:68` for top and bottom base-interface counts.

## Deferred

- The soluble branch (`:57-58`, `:64-65`) already matches and is unchanged.
- The projection count derivation (`SupportCommon.cpp:215-225`) is unchanged.

## Premises

1. Two scratch slices of the per-object skirt fixture, with `support_filament = 1`,
   `support_interface_filament = 2`, `support_top_z_distance = 0.26` and two top interface
   layers, use default tree and normal support. Both differ from Native. Across the whole output, Native
   emits `;TYPE:Support interface` once and Ares twice.
2. With the count rule of `:61`/`:68` applied as a temporary experiment, since reverted,
   both scratch slices match Native apart from the time estimate lines.
3. The two witnesses below fail in both modes at the base, and pass with the same
   temporary experiment.

## Observable behavior

When the interface filament differs from the support filament and more than one interface
layer is requested, the layer next to the support base is printed as a base interface,
whatever the gap.

## Failure modes

- Keeping the zero-gap condition: nonzero gaps keep printing every layer as an interface.
- Dropping the `> 1` condition: a single requested interface layer would turn into a base
  interface and leave no layer on the interface filament.
- Treating `support_filament = 0` as the same filament: upstream counts it as different.

## Test plan

Two whole-output parity tests in `crates/ares-cli/tests/orca_parity/runner/tree_support.rs`
use the existing Creality Ender-3 overhang case, with `support_filament = 0`,
`support_interface_filament = 1`, a top gap of 0.15 and three interface layers.

| Test | Before the port | After the port |
|---|---|---|
| `default_tree_interface_filament_keeps_one_base_interface_layer` | F | P |
| `normal_snug_interface_filament_keeps_one_base_interface_layer` | F | P |

The hybrid tree consumer has no separate witness. The support, tree, raft, skirt and organic
tests must show no PASS turning into FAIL in either mode.
