# Print::apply's printable instances

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. Upstream prints only instances that are `ModelInstance::is_printable()`, and that includes "fully inside the print volume". Ares computes the print-volume state for the `-50` gate only, so it prints a plate's fully outside instance next to its inside ones. A slice that Native completes then fails the toolpath check. Found by the per-instance delete slice (`web-plater-delete-instance.md`, its cross-plate case); ported here first.

## Upstream boundary
- **`ModelInstance::is_printable()`** (`Model.hpp:1514`) is `object->printable && printable && print_volume_state == ModelInstancePVS_Inside`.
  - `print_volume_state` starts at `ModelInstancePVS_Inside` (`Model.hpp:1564`).
  - `ModelObject::printable` (`Model.hpp:402`) defaults to true, and the 3MF import never sets it; it sets only the instance flag (`bbs_3mf.cpp:4397, 4429, 4448`). In the CLI the object term is therefore true.
- **`Model::update_print_volume_state`** (`Model.cpp:839-852`), run by the CLI for the selected plate (`OrcaSlicer.cpp:6560-6562`), calls each object's `update_instances_print_volume_state` (`:2548-2562`). That stores every instance's `calc_print_volume_state` (`:3973-4032`) and counts the inside ones; a count of 0 is the `-50` rejection.
- **`print_objects_from_model_object`** (`PrintApply.cpp:193-230`), called by `Print::apply` (`:1958`), turns only `is_printable()` instances into print instances (`:201`). An object with none yields no print object.
- **`Print::update_shrinkage_compensation`** also skips an object with no printable instance (`:1298`), but only in its non-uniform-shrinkage scan.

## Ares destination
- **`ProjectInstance`** gains `print_volume_state`, `Inside` at construction, and `is_printable()`: printable and inside. The object term is the constant true above.
  - `domain.rs` (398 lines) first moves `ProjectInstance` into `domain/instance.rs`, in a move-only commit.
  - The state enum moves from `project_slice/state/print_volume.rs` to the instance.
- **`update_print_volume_state(&mut Project)`** replaces `count_inside_instances`.
  - It is the same loop over the existing `calc_print_volume_state`, now storing each state and returning the inside count, so the state is computed once.
  - The `-50` gate (`project_slice/state.rs`) keeps its count test.
- **The print-object grouping** (`project/effective_config/grouping.rs`, Ares's port of `print_objects_from_model_object`) filters on `is_printable()` instead of `printable()`, both for the transform groups and for each group's members.
  - An object with no printable instance already drops out (`candidates.rs`: empty transform groups).
- **Entry points without the CLI's state update** keep `Inside`, as upstream's default, so their prints are unchanged.

## Included and deferred
- **Included:** the stored print-volume state and the `is_printable()` print-instance filter for every slice that runs the gate.
- **Deferred:**
  - `:1298`. Ares's shrinkage compensation (`z_shrinkage_compensation`) has no non-uniform-filament scan: it turns compensation off when filaments differ. The skip belongs to that unported branch.
  - The partly-outside (sinking) instances' other consumers.

## RED
**Fixture:** `print_volume_printable.rs` under the empty-plate controls. The Ender-3 cube project (`tests/parity/orca_cli_ender3.3mf`) gets a second instance of its cube at x = 400, fully outside the 220 mm bed, on plate 1, with its own identify id.
- Native `--slice 1` prints the inside cube only.
- Ares's plate-1 slice must equal it (ordered-bytes comparison, generator line excepted). Today Ares includes the outside instance and fails.

**Breadth for GREEN:** a paired run of the plater and project families, base vs HEAD (0 PASS → FAIL), plus a full ares-cli/ares-core comparison noting every visible change.
