# Elegoo M6211 processor

## Source and destination

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`): `GCode/GCodeProcessor.cpp:2900,2911–2927,3130–3131,3443–3445` registers `M6211`/`m6211` and dispatches only for a case-insensitive `elegoo` prefix on the effective `printer_model`. Preset vendor, display name, nozzle diameter, and G-code flavor are not the dispatch key. `GCode/ElegooGCodeProcessorHelper.cpp:16–191` owns parameter validation, model-specific estimates, temperature, tool selection, and flush accounting. It interprets the command; it does not rewrite the emitted line or generate motion.

In Rust, `project_slice/gcode_emit/processor/elegoo.rs` ports parsing and the two estimates. `processor/estimate.rs` dispatches from the existing streaming line pass and feeds the existing `FilamentChange`, `UsedFilaments`, and move recorder. Reuse the existing streaming tool-change block and rolling `Timing::flush` for Native synchronization (`GCodeProcessor.cpp:7390–7511`). Do not create a second timer or delay scanner.

## Included and witnesses

| Effective model / branch | Native semantics | Native witness |
| --- | --- | --- |
| `Elegoo Centauri Carbon`, `Elegoo Centauri` (`:51–86,126–129`) | Clamp timing `L` to 10–1000 mm; `P` defaults to 5000 ms, clamp to 0–20000. Initial 18.2 s plus XY Manhattan travel to (256, 0) at F5000. For each 73 mm full segment use E30+35 at F500, E10+2 at F400, cooling and 2.8 s wipe. A partial segment extrudes its length at F500, retracts E2 at F400, then cools and wipes. | Centauri Carbon 0.4, `0.28mm Extra Draft @Elegoo CC 0.4 nozzle`, 10 mm cube; `G90\nM83\n` with/without `M6211 A1 L200 T0 Q210 R210 S210`. L200 includes full and partial segments. |
| `Elegoo Centauri Carbon 2`, `Elegoo Centauri 2` (`:36–49,89–124,130–132`) | Clamp timing L to 10–1000. K segment length defaults to 75 mm (10–300); M/N old/new feeds default to F300 (10–600); P defaults to 3000 ms (0–20000). 31 s initial, then up to E15 at M. At least one segment (`ceil(max(L−15,0)/K)`); each uses K with a max-10 mm tail at F400. Intermediate segments use N and E6 retract at F1800; final uses 0.8 N and E4 at F1800. Each adds P plus 5 s wipe. R defaults to S + 20 (S initializes to zero if absent), S to 250 for cooling; both clamp to 185–350 °C and add `(R−S)/1.36` only when R > S. | Stock Centauri Carbon 2 0.4 start on the 10 mm cube (`ECC2/...0.4 nozzle.json` already includes M6211). A reduced Carbon 2 start supplies K/M/N/P and R > S to distinguish explicit feeds and cooldown from defaults. |
| Any other case-insensitive `Elegoo` prefix (`:126–135,139–191`) | Zero calculated purge delay, but **still execute** temperature, filament selection, and material accounting. | Same Centauri Carbon 0.4 cube/minimized start with synthetic effective model `eLeGoO Processor Witness`; a repeated valid command on the same filament isolates volume accounting from the time/selection transition. |
| Non-Elegoo prefix; invalid command arguments (`GCodeProcessor.cpp:2924–2927`, helper `:139–152`) | Bypass non-Elegoo entirely. Require L > 0 and T >= 0; round T (nearest, halves away from zero) and require an in-range logical filament. | Synthetic non-Elegoo effective model on the same witness; minimized first-generation start with invalid L/T combinations as a no-op control. |

### Shared effects (`ElegooGCodeProcessorHelper.cpp:154–190`)

1. Resolve physical head through the filament map. S, if present, updates that recorder temperature slot before the existing filament-change vertex is recorded. A and Q are ignored by these branches (R only affects second-generation timing).
2. Store the calculated load time and set unload time to zero, then invoke ordinary `process_filament_change` on the rounded logical ID. These time settings persist; do not inject an unconditional delay. Same-filament selection returns without a time block, even though the shared temperature/flush work still executes.
3. Compute volume from the logical filament cross-section and **unclamped** L. Compare with configured nozzle volume of the physical head (zero if missing). When enough material is purged, charge the displaced nozzle volume to the old filament only if it exists and differs; charge the remainder to the selected filament and clear remaining volume for that head. For a smaller purge, charge all of it to the selected filament and subtract it from the head's existing remaining-volume state. Accumulate through the existing flush statistic, not the model-extrusion cache. Use the same f32 arithmetic boundaries as Native.

## Deferred

No further printer/model defaults, new G-code emission, extra interpretation of A/Q, or changes to non-M6211 temperature/filament commands, planner arithmetic, UI statistics schema or streaming architecture. Follow the shared processor's map invariants; undefined non-finite/out-of-range C++ casts are not a parity target. The named model variants share an implementation; do not enumerate nozzle sizes or vendor presets in the handler.

## Acceptance

Compare Native and the public `slice_project` seam on the same harness-exported cubes. Check ordinary progress, first-layer/total duration and filament usage as observable effects; preserve complete G-code apart from the usual generator-line normalization when no independent discrepancy exists. Require non-M6211 controls and a frozen processor/material inventory to have P→F=0 in both official and seeded Native modes. Preserve the literal M6211 line.
