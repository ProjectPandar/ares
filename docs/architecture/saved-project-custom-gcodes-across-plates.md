# Custom layer G-code on saved-project plates

Reference: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
This specification covers direct CLI slicing of saved FFF 3MF projects. It separates the
`Model::plates_custom_gcodes` replacement from the `ToolOrdering` behavior that uses its events.
Neither requires a new event format, print pipeline, or multi-plate traversal.

## Replace the map once, not once per plate

- `src/libslic3r/PrintConfig.cpp:13185–13189` declares `--load-custom-gcodes` as one string
  argument. `src/OrcaSlicer.cpp:1660–1662` reads the path; `:2049–2081` reads it once if nonempty,
  after model import and before CLI preset resolution. It decodes **one** `CustomGCode::Info`,
  not a map or array of plates. `src/libslic3r/CustomGCode.cpp:14–49` reads the optional `mode`
  and the `gcodes` items in input order. Reuse the existing decoder and its missing-file
  (`CLI_FILE_NOTFOUND`, POSIX 253) and read/JSON-error (`CLI_CONFIG_FILE_ERROR`, POSIX 251)
  paths, including plate index `0` and the existing report location. The empty path is a no-op;
  a successfully read document with an empty `gcodes` array is not.
- `OrcaSlicer.cpp:2064–2073` creates a one-entry map: key `0` for `--slice 0`, key `N - 1`
  for positive `--slice N`. `:4104–4115` assigns that map to `m_models[0].plates_custom_gcodes`,
  replacing the **entire** archived map loaded by `src/libslic3r/Format/bbs_3mf.cpp:3437–3465`.
  With no JSON, keep all archived events. With JSON and `--slice 0`, only plate 1 sees the
  JSON events; plates 2 onward still slice and see no events, including any events saved in the
  archive. An empty JSON event list clears archived events for all plates. Do not merge the maps
  or apply the JSON to the first plate that happens to publish successfully.
- `OrcaSlicer.cpp:6522–6531` sets the existing zero-based `model.curr_plate_index` for each
  selected plate; `:6986` calls `Print::apply`. `src/libslic3r/Model.hpp:1742–1750` looks up
  the current plate and returns empty info if no entry exists. `src/libslic3r/PrintApply.cpp:1716–1727`
  transfers that plate's info to the applied print. The map key is the original model plate index,
  not the index in a filtered list of requested outputs.

### Rust seam

The subcommand path `ares-cli/src/slice_plates.rs::run` and
`ares-core/src/project_slice/cli_plates.rs::slice_cli_project_plates` already own saved-project
selection, all-plates preflight and ordered publication. The earlier direct
`ares-cli/src/direct_project.rs::run` handles a positive selection with
`custom_gcodes.rs::load` and `SlicerContext::with_custom_gcodes`. It has no all-plates saved-project
route. Compose these existing interfaces before changing event semantics. The direct adapter
reads and decodes once for the request, supplying a single keyed `CustomGCodeInfo` through the
existing context handoff; it does not start a new slice request for each plate.

In `slice_cli_project_plates`, consume that handoff once on the loaded `Project`, after any
Embedded or profile/file loading and before `Project::plate_view` or `state.rs::apply_plate`.
Reuse `project/plate_custom_gcodes.rs::Project::replace_plate_custom_gcodes` from the direct
adapter's core path. `project/domain/plate_selection.rs::Project::plate_view` retains the event
map and the original `curr_plate_index`; its views must not rekey the map to zero. Pass the
existing views to `apply_plate`, leaving preflight, filenames, publish order, tower fitting,
status and reports with the existing owners. Reuse `CustomGCodeInfo` and `CustomGCodeItem`; no XML
roundtrip or archive rewrite. The positive-only direct dispatch is a compatibility shell around
the shared saved-project traversal, not a second implementation to extend plate by plate.

## Multi-filament tool changes are tool ordering, not inline G-code

- `src/libslic3r/GCode/ToolOrdering.cpp:584–600` activates per-layer object tool switches in
  the by-layer whole-print constructor only when the effective `filament_diameter` vector has
  more than one slot, `Print::object_extruders()` has exactly one distinct assignment, and the
  current plate's stored event mode is `MultiAsSingle`. Use the effective object's assignments,
  not the total used-tool set (which may already include event tools and support). Do not substitute
  physical nozzle count for filament-slot count. A two-slot print whose objects already use both
  slots does not acquire an object tool switch from this event path.
- `src/libslic3r/CustomGCode.cpp:101–113` takes only `ToolChange` items, retaining their input
  order: despite the comment describing increasing Z, the function does not sort. IDs are
  one-based, with `0` meaning no override and an ID beyond the available slots falling back to
  slot 1. `ToolOrdering.cpp:781–786,825–837` consumes switches for each object's layer when
  `event.print_z < layer.print_z + EPSILON`, where the last consumed switch persists until another
  switch. This threshold is not the midpoint rule for inline events. `:118–175,842–891` uses the
  same override for required object tools and for the region-role tool lookup used by extrusion.
- `ToolOrdering.cpp:3263–3325` assigns inline events in a separate by-layer pass. It skips
  `ToolChange` items in its iterator (`:3293–3296`); do not reinterpret one as literal `T` text
  or as a color-change command. Keep the existing mode checks, midpoint placement and Custom,
  PausePrint, Template and ColorChange behavior. `src/libslic3r/Print.cpp:773–795` also includes
  valid event tools in print tool usage when requested; that set is separate from the object-only
  count above. By-object printing does not run the per-layer switch path
  (`src/libslic3r/GCode.cpp:3646–3655`).

### Rust seam

In `ares-core/src/project_slice/gcode_emit/tool_ordering.rs::collect_layers`, derive the
current plate's switch sequence for `OrderingScope::All` in by-layer printing and advance it
along each object's layer sequence. Reuse the object-tool predicate behind
`tool_ordering/custom_gcodes.rs::object_extruder_count`. Carry the resolved layer override with
`LayerTools`. Both `tool_ordering/collect.rs::object` and the object path assignment in
`layers/tool_buckets.rs::build` (through `tool_buckets/object.rs::split`) must consult that same
decision for normal object extrusion, preserving the separate precedence of wiping assignments.
The ordinary layer driver and toolchange writer then emit the scheduled transition and extrusion
on the correct tool. Do not merely insert a `T1`, change object or
preset assignments for the entire print, or treat an explicit support tool as an object role.

Continue using `project/effective_config/phases.rs` for post-region usage of event tools and
`tool_ordering/custom_gcodes.rs::ToolOrdering::assign_custom_gcodes` for inline events. No parser
change is needed for the observed `MultiAsSingle` gap; the same model event type can come from
an archive. The switch has the same meaning for either source.

## Implementation order and observable checks

1. Compose the direct saved-project dispatch with the existing all-plates traversal, then
   carry the JSON map replacement across that seam. Reuse `cli_slice_plates.rs` and the
   synthetic `tests/parity/custom-gcode-overview/two-plates.3mf` as traversal controls. Add a
   two-plate case with distinct archived Custom markers. Without the flag, both archived
   markers print. With `--slice 0 --load-custom-gcodes FILE`, plate 1 prints the JSON marker,
   plate 2 prints no marker, and neither output contains an archived marker. A valid
   empty event list produces both outputs without any saved marker. With `--slice 2`, the JSON
   marker prints on plate 2 only, retaining the native filename and original plate slot.
   Retain the existing `cli_load_custom_gcodes.rs` positive, missing-file and malformed-file cases.
2. Port `MultiAsSingle` tool collection and routing separately. Use two configured slots with
   both objects assigned to one slot, by layer, and a valid mid-print `ToolChange` to slot 2.
   Compare against the otherwise identical no-event print and verify the tool switch and
   subsequent object extrusion use the new slot, not just that the command appears. Include
   the companion fixture in which the objects use both slots: a Custom event still prints,
   but the tested `ToolChange` leaves the stream unchanged from its no-event control. Add a
   composed all-plates case proving a valid switch affects only plate 1 even when plate 2
   also meets the `MultiAsSingle` conditions.
3. During implementation use live Native and replay in both official and seeded modes. Compare
   exit, stdout/stderr, output inventory, complete `result.json` and each complete ordered
   G-code stream with only the existing generator/volatile metadata normalization. Marker and
   tool assertions supplement whole-output comparisons. Derive fixtures from synthetic inputs;
   do not commit Native outputs. The positive plate and no-flag controls must keep working.

Deferred: new import forms, compound invalid-argument precedence beyond the existing adapter,
non-project or assembly input extensions, export/inspection action combinations,
`--skip-modified-gcodes`, unrelated multi-filament event modes and hardware-specific changes not
required by this source slice. No new Web/WASM action interface.
