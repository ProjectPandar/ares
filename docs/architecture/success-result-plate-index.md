# Success result.json plate_index

Rewrite slice of the `record_exit_reson` success call at OrcaSlicer v2.5.0-beta `345efec64`. It closes the `plate_index` field of the success result document, which Ares currently fills with the one-based working plate number.

## Upstream boundary
- **Success reporting:** `record_exit_reson(outfile_dir, 0, plate_to_slice, cli_errors[0], sliced_info)` (`OrcaSlicer.cpp:8132` and `:8135`, duplicate-count branch; the same form at `:6361`/`:6385`).
- **Field:** `record_exit_reson` copies its `plate_id` argument into the document at `OrcaSlicer.cpp:552` (`j["plate_index"] = plate_id`, signature `:535`).
- **`plate_to_slice`** is the raw `--slice` argument of the command line: `0` means every plate, otherwise the one-based plate to slice. The plate loop skips `plate_to_slice != 0 && plate_to_slice != index + 1` (`:6528-6531`).
- **Adjacent field, different source:** each `sliced_plates` entry takes `id` from `sliced_plate_info.plate_id`, which the loop fills with `index + 1` (`:6532`, written at `:563`). That one stays one-based.

So `plate_index` is the requested argument (`--slice 0` → 0, `--slice 2` → 2), while `sliced_plates[].id` is the one-based plate number.

## Ares destination
`crates/ares-cli/src/result_json.rs`. `Success` gains the requested plate index and `ResultJson` writes it into `plate_index`; `sliced_plates[0].id` keeps the one-based plate. Callers (`main.rs` two sites, `project_presets.rs` one site) pass the request: the `--plate` value when given, `0` when the flag is absent.

## Included and deferred
- **Included:** the success `plate_index` source.
- **Deferred:** failure documents, whose `plate_index` keeps its own values (`0` or `index + 1`, `OrcaSlicer.cpp:6572` region), and `sliced_plates[].id`, already one-based.
