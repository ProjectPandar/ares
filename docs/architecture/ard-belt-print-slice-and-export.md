# ARD: Slice and export a single belt-print instance

Source: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Paths in source citations are relative to `src/libslic3r`.

## Decision

Belt mode is a distinct print frame, not a different header count. Keep mesh
intersection, toolpath construction and output scheduling in the existing core
pipeline. Replace the flat-frame assumptions at the three Orca ownership points
below. Do not transform finished Cartesian G-code as a post-process: that cannot
recover the mesh sections or the layer schedule. Never emit a partly transformed
print when an enabled belt behavior is unsupported.

Distinguish the centered **slice frame** used for mesh intersections, the
placement-adjusted **print Z** used to order and mark layers, and the remapped
**machine XYZ** written to motion commands. These coordinates are not
interchangeable. In particular, a `;Z:` comment is not machine Z.

## Source ownership and Rust destinations

1. **Object frame and layer plan.** `BeltTransform.cpp:15–47,89–193` owns the
   rotation, projected bounding-box height and floor shear. `PrintObject.cpp:4468–4512`
   feeds those values to `SlicingParameters::create_from_config`;
   `Slicing.cpp:820+` generates object layer pairs. `PrintObjectSlice.cpp:188–206,345–350,899–921,1352–1366`
   and `BeltSliceStrategy.cpp:12–81` compose centered and per-volume transforms,
   scan model-part vertices and floor positions for the exact lift, then slice
   in the rotated frame. `PrintObjectSlice.cpp:962–1056` applies the clamped
   lift compensation and center/instance correction to print Z and floor shift,
   not to the intersection planes. Use `project_slice/{planning,bounds,parameters,layers,raw_intersections}.rs`
   and their per-object plan; reuse the layer generator and triangle intersector.
   Admit only one instance; general per-instance separation and invalidation
   (`PrintApply.cpp:193–218,1951–2059`) are outside this change.
2. **Printable layer schedule.** `GCode.cpp:2299–2507,3500–3556` owns ignoring
   empty lead-in, filtering entries without extrusion, grouping coincident
   print Zs with the upstream tolerance and counting emitted layer changes.
   Filter at the export schedule, not by discarding all empty planes before
   slice topology work. Wire the existing `project_slice/gcode_emit/layers/schedule.rs`
   and its consumers (`layers.rs`, `tool_ordering.rs`, `orchestration_header.rs`)
   to that schedule. The total-layer placeholder in `GCode/GCodeProcessor.cpp:1329–1334`
   reflects marker counting (`4691–4694`); do not adjust it independently.
3. **Writer, origin and XYZ.** `Print.cpp:3998–4003` selects `BeltGCode`.
   `BeltGCode.cpp:13–47`, `GCode.cpp:3455–3474,4340,7672–7900`,
   `GCode/BeltBackTransform.cpp:8–32`, `GCode/MachineFrameTransform.cpp:11–88`,
   and `GCode/BeltKinematics.cpp:10–27` own origin projection, inversion, axis
   remap and the machine shear/scale. Follow the upstream operation order and
   preserve the writer's logical position; apply kinematics at the existing
   motion-write seam in `project_slice/gcode_emit`, including all-axis lines
   when required, upstream lift behavior and the no-arcs contract
   (`GCodeWriter.cpp:55–98,1391+`). Country-independent numeric formatting and
   existing slot/offset handling stay in the shared writer. Emit tilt metadata
   at the existing header seam; never remap user-authored raw custom commands.

These changes stack: the first two alone do not authorize releasing transformed
parts through a Cartesian writer. The seams can be separate commits/branches,
with one integrated binary for public parity and no separately shipped partial
implementation. Core continues to be platform-neutral.

## Admission and explicit rejection

At resolved configuration/print-object admission shared by CLI, core and WASM
(`project_slice/state.rs::apply_plate` and `project_slice/capabilities`),
`belt_printer=false` follows the existing path unchanged. With `belt_printer=true`,
require all of the following on the selected plate after applying presets and
object overrides:

- One contributing model object with exactly one printable instance, one
  model-part volume, and no modifier, negative or support volumes. An instance
  may translate along the bed. Require by-layer sequence and one logical
  material slot, with all resolved extrusion using that slot. A physical
  nozzle count on its own is not a material count.
- `brim_type=no_brim` on the resolved object, zero skirt loops, disabled draft
  shield, no support (`enable_support=false`) and zero raft layers. Reject
  `enable_belt_purge_tower=true` and a `belt_purge_tower_object` even on a
  one-material plate. An enabled ordinary prime tower setting that Orca
  suppresses in belt mode stays suppressed (`Print.cpp:836–854`); it is not a
  belt purge request.
- A finite nonzero X or Y slicing rotation and a usable, nonsingular writer
  transform. Accept the signed supported remap axes as a permutation of XYZ;
  no missing or repeated target axes. No decoupled tilt, calibration, vase
  mode, or spiral/contoured toolpaths in this envelope. Unsupported modes must
  not pass merely because the flat writer already has a branch for them.
- No active unimplemented belt-height bands (contract below). Unknown or
  deferred branches fail closed; a configured `slow_down_layers=0` is not a
  band opt-out.

Report `SliceError::UnsupportedProjectFeature("belt_printer: <feature>")`
with a stable reason for each unsupported capability (for example,
`brim`, `support`, `belt_purge`, `instances`, `materials`,
`first_layer_bands`). Do not invent a Native error code for this Rust support
limit. The public CLI and browser adapters must return the error without
exposing G-code or moves for the rejected plate. Configuration and metadata
read/write do not themselves request a slice and are not restricted.

## First-layer bands and whole-file compatibility

`GCode.cpp:9226–9254,9257–9267,9314–9344,11615–11640` unconditionally
enters the belt first-layer classification for an ordinary tilted belt with
nonzero floor shear. Width-adjusted height above the floor affects flow and
speed even with `slow_down_layers=0`; upstream clamps the ramp to at least one
step, and a wall crossing a band can be split into additional XYZ/E commands.
`GCode.cpp:3483–3497,6874–6881` also uses the plane for travel and layer
behavior. `brim_type=no_brim` does not disable these paths.

First-layer bands are outside this change. Require an inactive floor plane
according to Orca's height query: `abs(belt_floor_shear_factor) < EPSILON`.
Reject otherwise as `belt_printer: first_layer_bands`. The X/Y rotation can
still be nonzero without an active floor, for example at the right-angle edge
of `BeltTransform.cpp:94–108`. This exercises the transform, schedule, and
writer but does not stand in for an ordinary 45° belt. The no-brim stock
variants are rejection cases until the ordinary belt-height classification,
path splitting and affected writer/cooling behavior are ported. Brim generation,
support, belt purge, arrange/calibration and processor/preview are also deferred.

## Verification

Use public CLI slicing for positive parity: a synthetic one-material cube with
no active floor plane in belt mode, its non-belt control, and a translated
instance of the same belt project. Run both official and seeded Orca with
identical CLI arguments and compare the entire output using the existing
generator normalization. Inspect marker count, every `;Z` and first XY
extrusion as diagnosis only. Do not commit Native G-code. The six stock
no-brim variants must return the named band error rather than G-code, not be
counted as whole-file matches. All six auto-brim variants must also reject;
belt apron bands are separate from first-layer speed bands.

Test explicit rejection via public interfaces for each unsupported capability,
including a tilted no-brim project when the bands are absent from the port.
Verify no output for rejections, preserve non-belt output, compare across
parallelism settings, and check the workspace and wasm32 build.
