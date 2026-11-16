# A preset switch during a slice invalidates that slice

Upstream baseline: OrcaSlicer v2.5.0 (40ecc66f7).

## Upstream source

The plater has one current model and print, and a slice result belongs to that print.

- `Plater::priv::on_select_preset` (`slic3r/GUI/Plater.cpp:12933`) applies the new selection with
  `q->on_config_change(preset_bundle->full_config())` (`Plater.cpp:13079`).
- `Plater::on_config_change` (`Plater.cpp:21575`) calls `schedule_background_process`
  (`Plater.cpp:21742`), and that leads to `update_background_process` (`Plater.cpp:11402`).
- `update_background_process` applies the model and the composed config to the print
  (`background_process.apply`, `Plater.cpp:11438-11442`). `Print::apply` stops a running slice
  through the cancel callback (`libslic3r/PrintApply.cpp:1702`, and the other
  `call_cancel_callback()` calls in `Print::apply`; the callback is
  `BackgroundSlicingProcess::stop_internal`, `BackgroundSlicingProcess.cpp:597`).
- When the apply invalidates the print, the plate's slice result is marked invalid and the
  G-code preview is reset (`Plater.cpp:11453-11466`).

A preset change therefore never leaves a result of the previous configuration on show, whether the
slice had finished or was still running.

## Ares destination

- `web/app/src/state/stlSession.ts`: a preset selection opens a new STL plater session, frees the
  previous one, and installs the new one with `invalidatedSlices`.
- `web/app/src/state/actions.ts` `slicePlates`: a slice whose STL session was replaced while it ran
  is discarded. The plate is left unsliced, as after `update_slice_result_valid_state(false)`, and
  the result of the replaced session is never shown.

Today `invalidatedSlices` leaves a running slice alone. The replaced session's result then arrives
and is shown, while the session that drew its toolpaths has already been freed. The preview shows
geometry that no longer has toolpaths, and the moves slider and view requests fail with
"No toolpaths: slice a project first".

A printer switch opens the session twice: once for the printer, then once more for the compatible
filaments it selects. A Slice pressed right after the switch can therefore start on the first of
the two sessions.

## Included

- A slice whose STL session is replaced while it runs is discarded, and the plate is left unsliced.
- The next Slice runs on the current session, and its toolpaths stay available.

## Deferred

- Stopping the worker's slice early. Upstream cancels the running print; the web worker finishes
  the slice and the app drops the result.
- Sidebar setting edits and other config changes during a slice, which do not replace the session.
- Opening the session once per printer switch instead of twice. A printer switch selects the
  compatible filaments in a second step. Upstream does both in the same `on_select_preset`: `Tab::select_preset`
  (`Plater.cpp:13029`, `Tab.cpp:7488`) calls `PresetBundle::update_compatible` (`Tab.cpp:7712-7714`)
  before the single `on_config_change`.

## Verification

`web/e2e/presetSwitchSlice.spec.ts`. The page holds the slice result before the app sees it. A process
preset switch then opens one new session, which frees the sliced one; any later opening is held too.
The result is delivered next, and the held openings go on only after the check, so the order is
fixed by events and not by timing. The plate must be left unsliced. The next Slice must then complete
on the new session, and the moves slider must show its toolpaths with no page error.
