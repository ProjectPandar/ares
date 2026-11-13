# Filtering state from a printer model selection

Choosing a system printer model in the plater printer combo leaves the sidebar unchanged
until an engine round trip resolves the model to a printer preset. A check that waits for
the filtering state to clear can pass during that window, before filtering has started.
Citations refer to OrcaSlicer 2.5.0 (`40ecc66f7`), `src/slic3r/GUI` unless noted.

## Upstream boundary

- `Plater::priv::on_select_preset` (`Plater.cpp:12933`) handles the combo's selection
  event.
- For a printer model entry it calls `PresetBundle::get_similar_printer_preset`
  (`Plater.cpp:13016-13024`). In the same handler it then selects that preset
  (`Tab::select_preset`, `Plater.cpp:13027-13031`), which updates the compatible
  presets.
- The selection is therefore in effect as soon as the event returns. No state exists in
  which the combo has accepted a model but the old presets still show as settled.

## Ares boundary

The web runs the same steps through engine calls in the slicer worker, which answer in
later tasks:

- `selectPrinter` (`web/app/src/state/presets.ts`) sets
  `compatible: { status: "filtering" }` before its first engine call, so a printer chosen
  by name shows filtering from its selection event.
- `selectPrinterModel` first awaits `PresetCatalog.similarPrinterPreset` and only then
  calls `selectPrinter`. Until the model resolves, the sidebar shows the previous
  compatible state and no filtering indicator.

## Included

- `selectPrinterModel` sets the filtering state before it resolves the model, so a model
  selection, like a name selection, shows filtering from its selection event until the
  compatible presets are set or an error replaces it.

## Deferred

- Name and process selections already set their state before the first engine call; they
  are unchanged.
- Rendering the sidebar from the selection event itself: the web still waits for engine
  answers, and the filtering state stands for the work the upstream handler does
  synchronously.

## Verification

`web/e2e/printerModelSelection.spec.ts` chooses a system printer model and reads the
sidebar one microtask after the click. An engine answer always comes in a later task, so
only state the selection event set can show there. The spec then waits for the selected
preset and the end of filtering.
