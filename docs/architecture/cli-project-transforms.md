# CLI arrange, scale and rotate for 3MF projects

Source pin: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Option | Native declaration (`src/libslic3r/PrintConfig.cpp`) | Effect on a loaded project |
| --- | --- | --- |
| `--arrange N` | `:12933–12938`, signed integer, default `0` | `0` disables, `1` forces, other integers leave the load-time decision in place. A loaded slicer project normally disables automatic arrangement (`OrcaSlicer.cpp:1768`). |
| `--rotate DEG` | `:12984–12988`, float, default `0` | Z rotation in degrees, on **volumes**, not instances. |
| `--scale FACTOR` | `:13030–13034`, float, default `1` | Uniform volume scaling, with the CLI ratio narrowed to `float` before validation and application. |

An omitted transform is not an operation. Native records transform keys on first occurrence and uses the last supplied scalar value (`Config.cpp::DynamicConfig::read_cli`, `OrcaSlicer.cpp:8203–8227,4909`). In that key order the rotate and scale operations modify the loaded model (`:5067–5073,5142–5153`). `--arrange` changes a decision at this stage; arrangement **runs after** the model transforms and the existing orientation phase (`:5261–5288,5619–6072`), regardless of where `--arrange` appears in argv. These flags work with the existing single-output Ares `slice -o FILE [--plate N]` entry. Without `--plate`, the arrangement scope is the whole project and this entry still exports one plate. An explicit plate selects the Native current-plate arrangement branch. This branch is independent of the separate multi-plate export entry.

## Core ownership

`Model.cpp:1950–1964,3054–3057,3087–3101` and `Geometry.cpp:471–535` own the volume transform updates. Scale changes a volume's scaling factors, not its offset. Rotate adds the Z-axis rotation through the existing transformation decomposition, then recenters the combined transformed volume bounds at the object origin (`Model.cpp:1751–1757,1883–1894`). Preserve instance transforms; CLI Z rotation does not invoke the GUI's drop-to-bed action. Reuse the existing core volume transformation and recentering code without changing the orientation behavior of its current callers.

Prepare the transformed project after the existing load/preset/config validation and before slicing selects the final plate view. Materialize explicit selected-import membership before volume edits, as the Native importer does. For arrangement, reuse the existing Ares `PlaterSession`/`arrangement` port. Supply CLI inputs at the arrange-job boundary rather than changing GUI defaults or implementing a second packing algorithm. The CLI defaults to no arrangement rotations and no extrusion calibration keep-out (`OrcaSlicer.cpp:1546,1597–1626`), and obtains bed geometry, print sequence, clearance, alignment and configured exclusions from the loaded settings (`:5681–5724,5964–5992`). Apply the resulting arrangement to the model consumed by the existing print pipeline. The CLI adapter parses flags, supplies resources, and writes the existing G-code and report; it does not edit model geometry.

## Failure boundaries and verification

- A scale ratio `<= 0` is `CLI_INVALID_PARAMS` (`-2`, POSIX 254) with `plate_index=0`, `Invalid parameters to the slicer.`, and no G-code (`OrcaSlicer.cpp:5142–5153`, `Utils.hpp:44`).
- If a forced selected-plate arrangement cannot keep a printable item on that plate, Native returns `CLI_OBJECT_ARRANGE_FAILED` (`-21`, POSIX 235), plate index 0 (`OrcaSlicer.cpp:6021–6034`). For global arrangement, leave outside/empty admission to the existing pre-slice volume check (`:6562–6573`); when arrangement ran without rotations, the Native `-50` report explains that fact.
- Do not treat `CLI_COPY_OBJECTS_ERROR` (`-9`, POSIX 247) as a general arrange failure: it is the exception handler of the separate `copy` transform (`:5002–5025`). The `copy` option registration is commented out in the pin (`PrintConfig.cpp:12950–12956`), so `--copy` is not an admitted alias or an arrangement control.

Use the committed Ender-3 project and duplicate Native runs on each lane. Freeze the no-transform control, forced/disabled/auto arrange, scale, rotate, a combination, and attributable failure cases before changing production. Compare ordered whole G-code with the existing generator-only normalization, Linux `result.json` with the existing wall-clock normalization, and status/output presence. Keep Native artifacts outside the repository.

Deferred: STL combinations until the STL project-pipeline milestone, non-project STL placement and `ensure_on_bed`, auxiliary transform flags, generalized Native argv parser emulation, and the `copy` command. No new Web or WASM invocation.
