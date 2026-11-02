# The first-layer-scan calibration region in the arrange (A7)

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. On a Bambu Lab printer that scans its first layer, the arrange keeps parts off the front strip where the printer draws its extrusion and hand-eye calibration, while "Avoid extrusion calibration region" is on. The plater session refuses that case today ("arranging around the first-layer scan region is not supported yet", `needs_calibration_strip` in `plater/arrange_job.rs`). Its test is a stand-in: it reads `printer_model`'s "Bambu Lab" prefix and the raw `scan_first_layer`. This slice ports the region and lifts the refusal. It extends the A3 decisions in `web-plater-move-rotate-arrange.md`.

## Upstream boundary
- **`ArrangeJob::process`** (`ArrangeJob.cpp:732-734`): when `is_bbl`, `params.avoid_extrusion_cali_region` and the full config's `scan_first_layer` all hold, it calls `partplate_list.preprocess_nonprefered_areas(m_unselected, MAX_NUM_PLATES)`. This runs before `update_arrange_params` and the inflation updates (`:736-739`), in both arrange scopes.
- **`is_bbl`** is `PresetBundle::is_bbl_vendor()` (`PresetBundle.hpp:374`): `get_current_vendor_type() == VendorType::Marlin_BBL`. A2b3 ported that vendor type for the session (`set_installed_vendors`).
- **`PartPlateList::preprocess_nonprefered_areas`** (`PartPlate.cpp:7799-7829`, defaults `num_plates = 1`, `inflation = 0` at `PartPlate.hpp:1049`).
  - The region is the box (18, 0)–(240, 15) mm, "new extrusion & hand-eye calibration region".
  - It becomes `scaled(BoundingBoxf).polygon()` on each of the `num_plates` beds. Each item has translation 0 and rotation 0, `is_virt_object`, `is_extrusion_cali_object`, height 1, the name "NonpreferedRegion0" and inflation 0.
- **The kernel side**, already in Ares: an `is_extrusion_cali_object` item keeps inflation 0 when the unselected items are inflated (`Arrange.cpp:231`, `arrangement::params`).
- **The CLI** adds the region only in its assemble path (`OrcaSlicer.cpp:5485-5486`), never in the global arrange (`:5966` only sets the parameter). So Native cannot witness it.

## Ares destination
- **`plater/plate_list/arrange.rs`:** `nonprefered_areas(num_plates)`, the items above, beside `exclude_areas`.
- **`plater/arrange_job.rs`:**
  - The `is_bbl && avoid && scan_first_layer` test uses `vendor_type == VendorType::MarlinBbl` and the full config's `scan_first_layer`. It appends the regions to the unselected items on `MAX_NUM_PLATES` beds, before the parameter and inflation updates.
  - `needs_calibration_strip` and its refusal are removed.
- **No new public surface.** The arrange popup already shows the option for Bambu Lab printers, and the web already hands the session its installed vendors.

## Included and deferred
- **Included:** the region, its gate, both scopes, and the refusal lifted.
- **Deferred:** per-object skirts, which remain the arrange's last refusal.

## Oracle and RED
- **GUI-only.** The CLI global arrange never adds the region, and upstream has no unit test of it. The cases are session properties derived from the region's box and the arrange's centring, each with a control that tells the region's presence apart.
- **Fixture:** `generated-printer-ids/input.3mf` (a Bambu Lab X2D; shared bed X 20.5..256, Y 0..256) with `scan_first_layer` "1" and its 10 mm cube scaled to 230 mm deep at Y 128. Centred, the cube spans Y 13..243, so it reaches into the region's Y 0..15 inside the region's X 18..240. BBL is installed through `set_installed_vendors`, as A2b3's cases install it.
- **Cases** in `plater_arrange/calibration_region.rs`:
  1. With "avoid" on, "arrange all" keeps the cube's front edge at Y ≥ 15 on plate 1. The controls arrange it centred (front edge Y < 15) with "avoid" off, or with `scan_first_layer` "0".
  2. With no vendor installed, the vendor type is Unknown, so there is no region: the same centred position as the control.
  3. The plate's own arrange with "avoid" on: front edge at Y ≥ 15.
- **Web E2E** in `plateArrangeVendors.spec.ts`: on the case-1 project with BBL installed, "Arrange all" with the popup's "Avoid extrusion calibration region" checked leaves the cube's front edge at Y ≥ 15.
- **Expected before GREEN:**
  - Cases 1-3 are refused ("around the first-layer scan region"). Case 2 is refused because the stand-in reads `printer_model`.
  - The E2E shows "Arrange failed".

## GREEN
- **`PlateList::nonprefered_areas(num_plates)`** builds the box with `scaled<coord_t>`'s division and truncation and `BoundingBox::polygon`'s vertex order: min, (max x, min y), max, (min x, max y). It uses `preprocess_nonprefered_areas`'s default inflation of 0.
- **The gate.** `arrange()` appends the regions on `MAX_NUM_PLATES` beds after the prepared unselected items and before `update_arrange_params`, as `process` does. It reads `vendor_type == VendorType::MarlinBbl`, the arrange settings' "avoid", and the full config's `scan_first_layer`.
- **Refusal removed.** `needs_calibration_strip`, its raw `scan_first_layer` read and its `printer_model` stand-in are removed. So is `unsupported`, which no other refusal used.
