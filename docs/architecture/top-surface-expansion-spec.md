# Top surface expansion in surface type detection

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/PrintObject.cpp:2112-2172`, inside `PrintObject::detect_surfaces_type`
  (`:1941`): after the small-crack handling and the `diff_ex_by_piece` of the top surfaces
  against the bottom ones (`:2105-2107`), each layer region grows its top surfaces:
  - the gate (`:2121-2122`): `top_surface_expansion > 0`, `top_shell_layers > 0`,
    `top_surface_density > 0` and at least one top surface;
  - the wall band (`:2127-2132`): `wall_loops`, capped at one when `only_one_wall_top` is
    set, gives the external perimeter flow's scaled width plus the perimeter flow's scaled
    spacing for each further loop;
  - per island of the region's slices (`:2139-2165`): the infill region is the island
    shrunk by the wall band; the island's top is the layer's top clipped to that infill
    region; an island whose top does not survive an opening by the top solid infill
    flow's scaled width is skipped; the top grows by `top_surface_expansion`
    (`offset_ex`, miter, limit 2); `inward` keeps only the growth inside the top's filled
    outline and `outward` only the growth outside it; the result is clipped to the infill
    region shrunk by `top_surface_expansion_margin`;
  - the new top (`:2169-2171`) is the union of the original top and all grown parts,
    minus the bottom surfaces, typed `stTop`.
- `src/libslic3r/PrintConfig.cpp:2586-2625` and `PrintConfig.hpp:91-96,1562-1564`: the
  three options and the `TopSurfaceExpansionDirection` enum (`inward_and_outward`,
  `inward`, `outward`).

## Rust destination

- `crates/ares-core/src/project_slice/prepare_infill/surface_type_detection/stage.rs`,
  `classify_with_fill`: after `cracks::resolve`, the port of `:2105-2107`, and before the
  internal surfaces are taken from the region's slices (`:2190-2204`). It acts on the same
  slices the top and bottom comparison uses, the port of `layerm_slices_surfaces`.
- A new child module `surface_type_detection/top_expansion.rs` holds the growth.
- The region options gain `top_surface_expansion`, `top_surface_expansion_margin` and
  `top_surface_expansion_direction`. The registry already defines and parses the three keys,
  but no slicing code reads them.
- The flows: the external perimeter width is already passed in; the perimeter spacing comes
  from the record's perimeter flow, and the top solid infill width from
  `perimeters::flow::fill::resolve_fill_flow` with `FillFlowRole::Top`.

## Premises

Measured on the chain tip `be71e599d` in the official mode, and in the seeded mode where
stated. The base is the single-nozzle base of `ironing-single-nozzle-legs-spec.md`: the KSR
model of `x1c-wall-generator-arachne.3mf` with the MK4 configuration of
`rear-seam-visibility/aligned-control.3mf` (Arachne, three walls, `only_one_wall_top` off,
four top shell layers, 100% top density). Its many top surfaces with raised features are what
the expansion acts on.

1. Ares ignores the option. Native's output with `top_surface_expansion = 2` differs from
   its output with `0` by 31474 lines (both modes). Ares's two outputs are identical apart
   from the configuration comment.
2. Each branch has its own Native result. `inward`, `outward`, both directions, and both
   directions with a 1 mm margin give four pairwise different Native outputs. They differ
   from Ares by 12287, 21740, 31474 and 31119 lines.
3. The gate holds today. With expansion 2 and either `top_surface_density = 0%` or
   `top_shell_layers = 0`, Native and Ares are byte-equal.
4. The base has one known difference: at expansion 0, Native and Ares differ only by one
   `M73` line placed one G1 apart (both modes). That is the `M73` gap deferred in
   `ironing-single-nozzle-legs-spec.md`.
5. Smaller single-nozzle bases are not enough. On the MK4 `rear-seam-visibility` and
   `grid-surface-collections` bases, expansion 2 leaves the Native output unchanged, because
   no raised feature sits on their top surfaces.

## Observable behavior

With the expansion active, top solid infill covers the area left under features that rise
from a top surface, and joins top surfaces separated by such features. Without it, that area
stays internal (sparse) infill. Where the infill pattern changes, the following layers'
moves, extrusion values, time estimates and `M73` lines change as well.

## Failure modes

- Growing across the gap from one island into another: each island is grown and clipped
  alone, so a neighbour without a top surface stays partly filled with internal infill.
- Seeding from the raw island instead of its infill region: a section whose exposed top is
  only its walls would be flooded inward.
- A wall band that ignores the `only_one_wall_top` cap, or uses width instead of spacing for
  the inner loops.
- Claiming bottom surfaces: the new top is taken minus the bottom surfaces.
- Applying the expansion before the crack handling, or after the internal surfaces are cut:
  the internal infill would overlap the grown top.
- Treating the margin as a growth limit instead of a clearance from the infill region.

## Test plan

`crates/ares-cli/tests/orca_parity/option_coverage/top_surface_expansion.rs` builds the
single-nozzle base at runtime with the helpers of `ironing_pattern`, sets the expansion keys
in the configuration, and compares Native with the validated API and the CLI, whole output,
API equal to CLI, without masking. Runtime premises, never stored Native values:

- the derived configuration has Arachne walls, `only_one_wall_top` off, and none of the three
  expansion keys before they are set;
- the Native configuration block echoes the expansion value that was set.

Legs, in both modes:

| Test | Settings | Before the port | After the port |
|---|---|---|---|
| `expansion_grows_top_surfaces_inward_and_outward_like_native` | 2 mm | F | P |
| `expansion_grows_top_surfaces_inward_like_native` | 2 mm, `inward` | F | P |
| `expansion_grows_top_surfaces_outward_like_native` | 2 mm, `outward` | F | P |
| `expansion_keeps_the_wall_margin_like_native` | 2 mm, margin 1 mm | F | P |
| `expansion_needs_top_surface_density_like_native` | 2 mm, 0% top density | P | P |
| `expansion_needs_top_shell_layers_like_native` | 2 mm, 0 top shell layers | P | P |

Effect on the full suite: six new identifiers. The test commit adds four FAIL and two PASS
per mode, and the port is expected to turn the four FAILs into PASS. If a leg is left with
only an `M73` placement line after the port, it stays an honest FAIL on that line and is
reported, as for `cubic` and `tpmsfk`.

## Deferred

- `only_one_wall_top` together with the expansion: `top_fill_replaces_inner_walls`
  (`src/libslic3r/PerimeterGenerator.cpp:637-640`), its Classic use in
  `process_classic` (`:1724-1736` and the wall reduction after it) and its Arachne use in
  `process_arachne` (`:2842-2855`). Both sites are reached only with `only_one_wall_top`
  (`:1724`, and `:2751` through `inner_loop_number`). Their premise does not hold yet: on
  this base with `only_one_wall_top` on and expansion 0, Native and Ares already differ by
  4491 lines. The Ares `only_one_wall_top` paths cite older upstream lines
  (`perimeters/arachne/walls.rs`, `:2113-2166`; v2.5.0-beta has that logic at
  `:2681-2760`), so that feature needs its own port first.
- Classic walls on this base: at expansion 0, Native and Ares differ by 80 lines; not
  localized.
- `PrintObject.cpp:1666-1667` and `:1749-1757`: invalidation steps after an option change;
  the CLI slices once.
- The `M73` placement gap of the base.
