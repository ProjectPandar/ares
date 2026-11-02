# The shared bed of several nozzles in the arrange (A6)

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. On a printer with several nozzle areas (`extruder_printable_area`: H2D, X2D and similar), the arrange packs into the area every nozzle reaches. The plater session refuses such a printer today ("arranging on the shared area of several nozzles is not supported yet", `plater/arrange_job/params.rs`), so no two-nozzle printer can arrange. This slice ports the shared bed and lifts that refusal. It extends the A3 arrange decisions in `web-plater-move-rotate-arrange.md`.

## Upstream boundary
- **`get_shared_poly`** (`PrintConfig.cpp:13820-13835`): the first nozzle area, then intersected with each next area. Each intersection keeps its first polygon.
- **`get_bed_shape(const DynamicPrintConfig&, bool use_share = true)`** (`:13837-13864`, default from `PrintConfig.hpp:2542`):
  - with nozzle areas, the shared polygon;
  - without them, the printable area turned counter-clockwise.
- **Its arrange readers.** Both pass the `DynamicPrintConfig`, so both read the shared bed:
  - `get_shrink_bedpts` (`Arrange.cpp:339-351`), the bed the kernel packs into;
  - `update_selected_items_inflation`'s bed box (`:166`).
  - The GUI job calls them (`ArrangeJob.cpp:741`), and so does the CLI global arrange (`OrcaSlicer.cpp:5514, 5989`).
  - `ArrangeJob.cpp:668` reads the bed only under `SAVE_ARRANGE_POLY`, a debug dump, and is not ported.

## Ares destination
- **`plater/arrange_job/params.rs` `bed_shape`:** one nozzle area stays as it is, as today. Several areas give `get_shared_poly` through the crate's Clipper2 intersection. `ArrangeConfig::bed_shape` already feeds both readers.
- **Disjoint areas.** Upstream indexes the first polygon of an empty intersection, which is undefined behaviour. The port returns `InvalidInput("the nozzle areas share no printable area")`, since the areas come from the loaded project.
- **No new public surface.** The web arranges through the existing calls.
- **Prerequisite, move-only:** `crates/ares-cli/tests/orca_parity/plater_arrange.rs` (395 LOC) moves its eight top-level "arrange all" cases and their two constants into `plater_arrange/arrange_all.rs`, unchanged, before `plater_arrange/shared_bed.rs` is registered.

## Included and deferred
- **Included:** the shared bed for both arrange scopes, and the refusal lifted.
- **Deferred:**
  - the plate's per-nozzle "left only / right only" regions (`PartPlate.cpp:8976-8994`), which are scene drawing;
  - per-object skirts and the first-layer-scan region, which remain the arrange's last refusals.

## Oracle and RED
- **Native `--arrange`**, where the GUI and the CLI agree: both read `get_bed_shape` with the shared default.
  - **Probe** (`generated-printer-ids/input.3mf`, a Bambu Lab X2D with nozzle areas X 0..256 and X 20.5..256, its object moved to (60, 60)). Native arranges it to X 91.55. With the second area removed, Native arranges it to X 77.2.
  - The probe's evidence is kept outside the repo.
- **Cases** in `plater_arrange/shared_bed.rs`, compared at runtime (`assert_arranged_like_native`):
  1. The probe.
  2. The same X2D with two more instances of the object, so the pile packs inside the shared bed.
  3. The plate's own arrange (`--slice 1`) on the probe's input.
  4. **Session-only:** the second area moved off the first, `"300x0,556x0,556x256,300x256"`. The arrange returns the `InvalidInput` above.
- **Web E2E** in `arrange.spec.ts`: "Arrange all" on the probe's input, compared with the runtime Native oracle.
- **Expected before GREEN:** cases 1-3 and the E2E are refused with the "shared area of several nozzles" message. Case 4 gets that same message instead of the disjoint-areas one.
