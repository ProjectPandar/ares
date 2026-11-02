# Plate locking in the plater session (A5)

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. A locked plate keeps its instances where they are when the plater arranges: "arrange all" skips the plate and numbers its beds over the unlocked plates, and the plate's own arrange icon does nothing. The plater session has no lock today:
- "arrange all" moves every instance, a locked plate's included;
- it refuses a project in which a plate has its own print sequence, because upstream handles that case by locking the plate for the arrange (`prepare::check_plate_sequences`);
- `prepare_imex_zones` takes every plate, not every unlocked plate.

This slice ports the lock and lifts that refusal. It extends the A3 arrange decisions in `web-plater-move-rotate-arrange.md`, which list plate locking as deferred.

## Upstream boundary
- **The lock.**
  - `PartPlate::m_locked`, `is_locked` and `lock` (`PartPlate.hpp:133, 614-615`).
  - `PartPlateList::lock_plate` (`PartPlate.cpp:7160-7176`), which the plate's lock icon toggles (`Plater.cpp:23414-23419`, action 4; tooltips "Lock current plate" / "Unlock current plate", `PartPlate.cpp:2629-2645`).
- **Loading.** The 3MF plate attribute `locked` (`bbs_3mf.cpp:374, 4637-4640`, read with `std::boolalpha`, so only "true" locks) reaches the plate through `PartPlateList::load_from_3mf_structure` (`PartPlate.cpp:8559`).
- **"Arrange all"** (`ArrangeJob::prepare_all`, `ArrangeJob.cpp:342-414`):
  - It locks each unlocked plate whose print sequence is not the global one (`:346-354`) and unlocks those plates after `finalize` (`:953-955`).
  - `PartPlateList::preprocess_arrange_polygon` (`PartPlate.cpp:7617-7660`) puts a locked plate's instance into the locked items: its bed is the plate index and its translation is plate-local.
  - `prepare_wipe_tower` (`ArrangeJob.cpp:504-526`) skips locked plates and numbers the towers' beds over the unlocked ones.
  - `prepare_imex_zones` (`:612-615`) takes the unlocked plates.
  - In `finalize` (`:819-965`):
    - `postprocess_bed_index_for_selected` (`PartPlate.cpp:7833-7879`) steps each arranged bed past the locked plates, creating plates as needed;
    - `postprocess_bed_index_for_unselected` (`:7882-7910`) does the same for the unselected items;
    - the locked items go back through `postprocess_arrange_polygon(ap, false)`.
- **The plate's arrange icon** (`prepare_partplate`, `ArrangeJob.cpp:548-552`): on a locked plate it shows "This plate is locked.\nCannot auto-arrange on this plate." and arranges nothing.
- **The CLI's global arrange** keeps a loaded lock (`OrcaSlicer.cpp:4701`), skips a locked plate's instances (`:5701-5716`) and post-processes them as the GUI does (`:6043-6054`). It does not lock a plate for its print sequence, and its tower loop does not skip locked plates (`:5784-5809`, `bed_idx = bedid`).

## Ares destination
- **`plater/plate_list`:**
  - `Plate` gains `locked`, loaded from the plate metadata and false for an added plate;
  - `lock_plate`;
  - the locked branch of the prepare step;
  - the bed post-processing over locked plates.
- **`plater/arrange_job`:**
  - `prepare_all` locks plates whose print sequence differs and `finalize` unlocks them. This replaces `check_plate_sequences` and its refusal.
  - The tower beds and `prepare_imex_zones` take the unlocked plates.
  - On a locked plate, the plate's own arrange returns `InvalidInput` with upstream's message and moves nothing. The web shows it the way it shows other arrange errors. Upstream shows the message as a warning notification; the session API has no notification channel, and nothing moves in either case.
- **Public:**
  - `PlaterPlate::locked`;
  - `PlaterSession::lock_plate(plate, locked)`, which returns `InvalidInput` for a missing plate, as upstream returns -1;
  - wasm: `lockPlate(index, locked)`, and the scene's plates carry `locked`.
- **Web:** the plate row in the object list gets a lock toggle beside the arrange icon, with upstream's tooltips.

## Included and deferred
- **Included:**
  - the lock, its loading, and the toggle;
  - both arrange scopes over locked plates;
  - the temporary print-sequence lock, which lifts the refusal;
  - the tower and IMEX beds over unlocked plates.
- **Deferred:**
  - saving the lock to the 3MF (`bbs_3mf.cpp:8445`), with the rest of project save;
  - the undo snapshot (`take_snapshot("lock partplate")`);
  - the CLI assemble path's locks (`OrcaSlicer.cpp:5391, 5562, 5614`), which are CLI-only;
  - the "All the selected objects are on a locked plate" notification of "arrange all" (`ArrangeJob.cpp:394-403`). That arrange still succeeds, moving nothing, so there is no error to return.

## Oracle and RED
- **Native `--arrange`** where the CLI and the GUI agree: plates locked in the 3MF, with no tower.
  - **Probe** (`custom-gcode-overview/two-plates.3mf`, plate 1's cube moved to (40, 40)). With plate 1 `locked="true"`, Native keeps that cube at (40, 40) and arranges plate 2's cube onto plate 2, at x 374. Unlocked, Native packs both cubes onto plate 1.
  - **Case 1** is that probe, compared at runtime (`assert_arranged_like_native`).
- **GUI-only parts: session cases** in `plater_arrange/plate_lock.rs`:
  2. **A plate with its own print sequence.** Plate 1 has `print_sequence` "by object" in its metadata; the global one is "by layer". "Arrange all" places the instances like Native does for case 1's input, where plate 1 is locked. Afterwards, plate 1 is unlocked.
  3. **The tower beds.** Two plates with two filaments: plate 1 locked, plate 2's tower stored in the middle of plate 2. Arranged next to plate 2's tower item, no arranged instance overlaps that tower's box. If the arrange used plate 1's tower on bed 0 instead, the pile would be centred over plate 2's tower.
  4. **The plate's own arrange on a locked plate** returns upstream's message, and every instance keeps its transform.
  5. **`lock_plate`** sets `PlaterPlate::locked`. A loaded `locked="true"` plate reports `locked`.
- **Web E2E** `plateLock.spec.ts`: on case 1's input with plate 1 unlocked, click plate 1's lock, run "Arrange all", and compare with the runtime Native oracle for the locked input. Then plate 1's arrange icon shows the message.
- **Expected before GREEN:**
  - Case 1 packs both cubes onto plate 1.
  - Case 2 is refused ("plates with their own print sequence").
  - Case 3: a pile overlaps plate 2's tower.
  - Cases 4 and 5 fail on the `todo!()` skeleton of `lock_plate` / `PlaterPlate::locked`.
