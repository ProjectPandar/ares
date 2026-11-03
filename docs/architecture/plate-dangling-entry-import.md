# Loading a plate entry that names no instance

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. After a middle instance delete, the GUI's 3MF writes the plate's now-dangling pair as a `<model_instance>` with the stale instance index and `identify_id` 0 (`bbs_3mf.cpp:8580-8600`; `web-plater-delete-instance.md`). Native loads that project. Ares rejects it twice:
- `identify_id` 0 fails as "invalid or repeated plate instance identity" (`project/load/metadata.rs`);
- an entry naming an instance absent from the build fails as "plate metadata references an object or instance absent from the build" (`project/load/assemble.rs`).

## Upstream boundary
- **Parsing.** `_handle_end_config_plater_instance` (`bbs_3mf.cpp:4986-5005`) takes every `<model_instance>` whose object and instance ids were read: `emplace(object_id, (instance_id, identify_id))` (`:5002`), with `identify_id` from `atoi` and never checked (`:4744`). Only pairs missing an id (`-1`) are skipped (`:4993-5000`).
- **Selection.** For `--slice N` the plate's map keeps whole objects (`:2215-2221`; `selected-plate-object-import.md`).
- **Identify ids.** The `loaded_id` loop over each plate's entries (`:2466-2500`) skips an entry whose object is unknown (`:2470-2489`) or whose instance index is past the object's instances ("invalid instance id", `:2491-2495`). An entry neither fails the load nor removes anything. An in-range entry sets `loaded_id` to its identify id (`:2497-2498`), and 0 there means none.

## Ares destination (`project/load/metadata.rs`, `project/load/assemble.rs`)
- **An entry with `identify_id` 0** is accepted. It stays in the plate's list, which keeps the object for the selected-plate import, but gives no instance a label: `loaded_id` 0 is "none".
- **An entry naming an instance absent from the build** is accepted the same way: listed on the plate, labelling nothing.
- **Unchanged:**
  - a malformed or missing id, which Ares rejects as unparsable before upstream's `-1` skip applies;
  - a repeated pair, which the GUI writer cannot produce: plate sets hold each pair once.

## Included and deferred
- **Included:** loading the GUI writer's dangling entry.
- **Deferred:** repeated pairs (upstream keeps a plate's first pair per object, `:5002`, and a later plate's identify id overwrites, `:2466-2500`), and `-1` (missing-id) entries.

## RED
`plater_delete_instance_import.rs` under the plater session tests: the export of a session after a middle delete, loaded as a project and sliced through `slice_project_plate`, against Native slicing the same export.
1. `combined-brim.3mf` with object 2's first instance deleted: plate 1.
2. The cross-plate delete: plate 1 (the moved instance left out by the print-volume filter) and plate 2 (listed only by the dangling entry).

Today both fail at Ares's load.
