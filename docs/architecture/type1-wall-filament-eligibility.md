# Type1 layer wall filament eligibility

OrcaSlicer 2.5.0 (`40ecc66f7`), `src/libslic3r/GCode/WipeTower.cpp:4973–5001`, starts the Type1 layer wall search with the first outgoing filament. Only if it passes `is_valid_last_layer` (`:5528–5533`) may it win by matching the tower's preferred filament or become the first same-category candidate. The predicate rejects an outgoing filament when the layer is its physical extruder's last planned change and its Z exceeds that extruder's printable height.

In `wipe_tower/type1/wall_filament.rs`, use the existing `Plan::is_valid_last_layer` from `Plan::layer_wall_filament`, with the layer index supplied by `Generator::generate_layer`. Keep exact incoming matches, the first same-category candidate, and the first incoming fallback in their original priority. Empty change lists, global wall counts and ties are unchanged. The existing finish code consumes the selected filament before or after its change (`:5032–5050`); no finish logic needs to change.

Do not extend eligibility to incoming candidates or add support/solubility checks. Per-layer nozzle mapping, last-change state recording, ramming planning/emission, finish-block eligibility, and Type2 are outside this change.

Derive a one-layer project at runtime from the two-AMS-slot X1 Carbon tower fixture by shortening both build items to 0.2 mm. The stock zero-height value makes slot 0 ineligible on its final change, so the standalone finish wall follows the change to slot 1. With a height of 256 mm the wall instead precedes the change. Check both against repeat-stable Native output through the public project API and CLI, comparing the complete streams apart from the generator line and requiring byte-identical API and CLI output.
