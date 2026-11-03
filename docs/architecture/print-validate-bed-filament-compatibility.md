# Print::validate bed/filament compatibility

Status: spec only, awaiting approval; no Rust tests or product implementation. The Default Plate accepted deviation was explicitly decided before this spec.

## Upstream boundary and Rust destination

Upstream: OrcaSlicer v2.4.2, `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `src/libslic3r/Print.cpp:1751-1781`, immediately after the G92/absolute-E block already covered by O1. `Print.cpp:1264` obtains `Print::extruders()`; `Print.hpp:942` defaults its custom-G-code argument to false. `Print.cpp:487-547` combines object/support/wipe-tower usage and sorts/deduplicates it. `PrintConfig.hpp:466-486` owns current-bed → ordinary-temperature-key selection; `Config.hpp:624-628` owns vector `get_at` front fallback. `PrintConfig.cpp:1080-1100` declares the six supported current bed values and Cool Plate default.

Destination: a private module under `crates/ares-core/src/print/validate/`, called after O1 succeeds. Reuse the existing project BBL identity and shared logical used-filament owner; do not create a second usage collector or use `filament_map`. Thread the actual selected plate's native index/display-number boundary. Apply the same predicate to the existing flat public validator and forward only the needed typed bed/temperature/identity/usage fields through its STL adapter. Existing flat/STL validation remains a temporary compatibility shell, not a new pipeline. Preserve the existing unsupported-sentinel error below; do not change loading validation or export.

## Non-negotiable source semantics

- Activate only for `is_BBL_printer() || support_multi_bed_types`. When both are false, return OK from this guard before any bed-type/key lookup, including Default Plate. Preserve O1 and preceding height/width/bridge error priority.
- Carry the actual selected one-based plate ID from both callers: `validate_loaded_project` passes its `select_plate(plate)` selection and `prepare_loaded_project_slice_with_validation` passes `select_plate(plate_id)`. These correspond to native `get_plate_index()+1`; do not infer identity from the first plate in a container.
- Select the **ordinary** bed-temperature vector from the current applied bed type: Supertack→`supertack_plate_temp`, Cool→`cool_plate_temp`, Textured Cool→`textured_cool_plate_temp`, Engineering→`eng_plate_temp`, High Temp→`hot_plate_temp`, Textured PEI→`textured_plate_temp`.
- Check sorted used logical filament indices with `get_at(extruder_id)`; an out-of-range index reads front. Neither physical mapping, all-filament scanning, first-layer vectors, highest-temperature formula, nor generated G-code temperature fallback owns this check.
- Only exact zero rejects. Report the first failing logical filament as index+1. Bed name is the canonical enum key, not the UI label (`Cool Plate`, not `Smooth Cool Plate`).
- Preserve `Plate {native_plate_index+1}: {bed_key} does not support filament {logical_index+1}` and the **one newline inside** `StringObjectException.string`. Native CLI adds another newline. Rust's current string error API must not silently drop the source newline.
- The source redundantly nests the same extruder loop twice. This does not change first-error order; implementation must not introduce a different traversal/order.

## Included and deferred behavior

Included proposed slice: the six defined bed types, BBL/non-BBL multi-bed activation, shared object/support/wipe-tower logical usage, direct-vector/front semantics, selected plate and exact message, public validation/validated composition. Pure processing remains free of implicit validation.

Deferred: adjacent motion warning block beginning `Print.cpp:1783`, GUI localization/structured `StringObjectException` display plumbing and native CLI exit-code transport. Do not change temperature export, heating algorithms, presets, templates, loading validation or generic DTOs. Any old assertion migration requires separate approval. No legacy fallback or input repair.

## Public native reachability, before Rust RED

Existing unchanged `tests/parity/validation/e5max-scv/project.3mf`, SHA256 `c503c92c8f524e16b4223d6349b572f333634bac418b7874236e64ceb536eaff`, already contains Klipper, valid uppercase reset, `support_multi_bed_types=1`, Cool Plate and ordinary temperature 45. Native original input exits 0 with G-code; changing only CLI `--cool-plate-temp 0` exits **195**, stderr `Plate 1: Cool Plate does not support filament 1\n\n`, no G-code. Current public Ares CLI, same original bytes and override `{"cool_plate_temp":["0"]}`, exits 0 with G-code: genuine publicly reachable missing guard, not an earlier loading/O1 failure. Only initial-layer temperature zero still gives native G-code. Unmodified BBL X2D project `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` with only `--hot-plate-temp 0` gives native195 `Plate 1: High Temp Plate does not support filament 1\n\n`, proving the BBL arm with its original multi-bed flag false. Raw evidence retained under `${TMP_ROOT}/ares-print-bed-native`; no identities/templates/profiles/geometry repaired.

After approval: capture both public project APIs' genuine RED; prewrite controls for both guard arms/off, ordinary-vs-initial, all six selected vectors, used-vs-unused logical indices, support/wipe usage, front fallback and no map, first failing filament, actual plate numbering, O1/bridge priority and unchanged successful export. Then implement only this source slice and run current-base full gates/independent reference-byte controls.

## Accepted deviation: unsupported Default Plate sentinel

`ProjectBedType` also represents `Default Plate`, but upstream `get_bed_temp_key` returns an empty key for it. Controlled native CLI on the same unmodified E5max input with `--curr-bed-type 'Default Plate' --cool-plate-temp 0` exits **139 (SIGSEGV)**, not a defined validation diagnostic. This attempt is retained separately and is **not** a parity RED or valid native parity control. Do not port undefined behavior or invent a bed/temperature fallback.

The coordinator explicitly accepted the same category of deviation as Cubicon: Ares must return an error, not G-code or panic, without opening a prerequisite slice. Actual current-base `028c758d3` public CLI measurement with the same original input and `{"curr_bed_type":"Default Plate","cool_plate_temp":["0"]}` exits **1**, has no G-code and reports `InvalidInput("curr_bed_type Default Plate has no first-layer temperature")`. This is the existing exact diagnostic from `options/config_export/writer.rs:258-262`, not a proposed new message. Preserve it at public boundaries; do not rewrite the export owner, create duplicate loading rules, silently return OK, or substitute the differently worded generic bed-temperature error.

After approval, prewrite a sentinel control asserting both public project APIs' **exact** `SliceError::InvalidInput` text `curr_bed_type Default Plate has no first-layer temperature`, no G-code and no panic. The bounded flat/STL guard must safely use the same existing diagnostic for the unsupported current-bed path **only inside the active BBL/multi-bed block**. With both flags false, skip Default Plate like upstream; any pre-existing export rejection is outside this guard and its owner is unchanged. This control documents accepted deviation, not native parity. Current raw Ares evidence is `${TMP_ROOT}/ares-print-bed-native/ares-default-028`; native139 remains `${TMP_ROOT}/ares-print-bed-native/default-plate`.
