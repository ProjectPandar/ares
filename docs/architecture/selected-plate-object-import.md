# The selected-plate import keeps whole objects

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. For `--slice N` the 3MF importer keeps the objects the plate lists, each with all its instances. Which instances print is decided later by geometry: `Print::apply`'s `is_printable()`, ported in `print-apply-printable-instances.md`. Ares also keeps only the instances whose exact `[object, instance, identify]` triple the plate lists. It therefore disagrees with Native whenever a plate's pairs do not name their instances, as after a middle instance delete (`web-plater-delete-instance.md`).

## Upstream boundary
- **`_BBS_3MF_Importer::_load_model_from_file`, selection** (`bbs_3mf.cpp:2201-2222`).
  - With `plate_id` set it takes that plate's data (`:2203-2208`).
  - For every model object it looks the object id up in the plate's `obj_inst_map` and skips the object if it is absent ("not in current plate, skip", `:2215-2221`).
  - It never looks at instances: a kept object keeps every instance.
- **`obj_inst_map`** is keyed by object id: `emplace(object_id, (instance_id, identify_id))` per `<model_instance>` (`:5002`), and pairs with a missing id are skipped (`:4993-5000`).
- **The identify ids** (`loaded_id`) are applied per plate entry only where the pair's object and instance exist. An out-of-range instance index is skipped with "invalid instance id" (`:2466-2500`); it never removes an instance.

## Ares destination
- **`Project::select_plate`** (`project/domain/plate_selection.rs`) for an explicit plate keeps every instance of the objects `selected_import_objects` already chose by object id (`import_scope.rs`). The per-instance identity filter goes from that path.
- **The plate-less `None` view**, Ares's "first plate" default rather than the importer's `--slice N`, keeps its filter.
- **The per-instance decision** is `is_printable()` after the CLI's `update_print_volume_state`: an instance of a kept object outside the plate's volume does not print.
- **The empty-model read error** stays: a plate that lists no object still yields no objects.
- **Identify ids** keep coming from the loaded plate entries, as now.

## RED
1. **Derived fixture** (`print_volume_printable.rs`): `combined-brim.3mf` with object 2's second pair (instance 1, identify 15) moved from plate 1 to a new plate 2. The instance still sits on plate 1's area.
   - Native `--slice 1` keeps object 2 whole and prints all three cubes.
   - Ares keeps only the listed pair and prints two.
2. **Delete-instance case 4, plate 2.** It is retargeted to "slices like Native" in its own test commit on the delete-instance branch. My earlier expectation, the model-read error, assumed per-instance selection; Native keeps object 2 and prints its cube.

## Breadth
A paired FULL run, base vs HEAD: no PASS → FAIL, and FAIL → PASS only where Native already keeps the whole object.

## GREEN
- **`Project::select_plate`, explicit plate:** the objects `selected_import_objects` keeps by object id, each with every instance.
- **The plate-less view** keeps its listed-instance filter, now applied only on that path.
- **The decision is geometric:** `update_print_volume_state` stores each instance's state, and the grouping prints only `is_printable()` ones.
- **Result:** the derived fixture slices plate 1 like Native, all three cubes. The outside-copy case still passes.
