# Combining sparse wipe-tower layers

Native reference: `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Included

- Enable combination only with `wipe_tower_sparse_layers_combination`, without
  sparse-layer skipping, smooth timelapse, or wrapping detection
  (`WipeTower.cpp:77–80`). Those modes otherwise retain their behavior.
- Before Type1 and Type2 tower planning (`WipeTower.cpp:4934–4935`,
  `WipeTower2.cpp:2479–2481`), fold consecutive layers with no tool changes,
  excluding the first tower layer. Carry the run only if the next sparse layer
  can accept it under the loaded tool's nozzle-specific maximum height. Add
  pending height to current height, then to next height, in `f32` with the
  upstream epsilon comparison. Do not fold across a tool change
  (`WipeTower.cpp:89–116`, `WipeTower.hpp:745–774`).
- In Ares `gcode_emit/wipe_tower`, retain the original layer indices/print Z;
  pass the increased height to tower generation. Keep folded results marked
  but exclude them from deposited filament totals (`WipeTower.cpp:4772–4779,
  5133–5135`, `WipeTower2.cpp:2146–2147,2581–2583`). The integration cursor
  must treat folded layers as empty for both entry hooks and dispatch
  (`GCode.cpp:2099,2118,2148`). The receiving layer still prints at its own Z,
  and exposes the combined height in G-code (`GCode.cpp:1079–1091`).

## Public verification

Derive both projects at runtime from the stock Type2 `empty-klipper` and Type1
X2D archives. Compare Native, API, and CLI with the setting off and on. For
Type2 compare whole bytes except the generator line. For X2D compare the
ordered tower-bearing model Z values and the first folded/receiving layer's
tower commands; preserve the strict stream mismatch as diagnostic evidence.

## Deferred

No changes to unrelated tower outline/depth or timing differences, the
no-sparse compaction planner, timelapse/wrapping behavior, or preset defaults.
