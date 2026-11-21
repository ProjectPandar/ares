# CLI mixed-temperature validation

This slice ports OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) at the `libslic3r::Print::validate` boundary. `PrintConfig.cpp:13252–13256` defines the CLI-only Boolean, false by default. `Config.cpp:1783–1906` accepts bare `--allow-mix-temp` and an explicit value such as `--allow-mix-temp=1`. `OrcaSlicer.cpp:1617–1619,7008–7009` reads it and inverts it for `Print::set_check_multi_filaments_compatibility` (`Print.hpp:1360–1361`). It is not a project setting or a G-code option.

## Applied print validation

`Print.cpp:1987–1988,2226–2237` checks only a print with one nozzle and more than one used filament. The by-layer check uses plate extruders (object, support and the applicable wipe tower selector, excluding layer-slider changes by default; `Print.cpp:689–804`, `Print.hpp:1125`). The by-object check considers each object's extruders and explicit valid support selectors independently (`Print.cpp:1846–1905`). Idle configured slots do not participate. For two or more filaments in the group, replace each zero range bound with the matching material default (`MaterialType.cpp:8–118`), then reject any lower bound greater than or equal to its upper bound. Each filament's regular nozzle temperature must lie inside the other's inclusive range (`Print.cpp:1743–1794`). There is no fixed temperature-difference threshold, and first-layer target temperatures do not enter this check.

The flag skips only **rejection for mixed incompatible nozzle temperatures**. Native demotes that one condition to a validation warning; the invalid-range error is still fatal (`Print.cpp:1885–1945`). Every other validation, including collision and bed-size checks, still applies. Native's CLI logs the demoted advisory with the print validation warnings; it does not turn it into a structured slicing warning or modify G-code.
In Native's `--pipe` mode, `g_cli_callback_mgr` also sends the advisory via `SlicingStatus{4, warning_text}` (`OrcaSlicer.cpp:7065–7074`); Ares has no pipe callback, so this behavior is deferred with `--pipe`.

`Print.cpp:2230` assigns `STRING_EXCEPT_FILAMENTS_DIFFERENT_TEMP`, mapped by `OrcaSlicer.cpp:7010–7039` to `CLI_FILAMENTS_DIFFERENT_TEMP` (`Utils.hpp:83`, `-62`, observed as process status `194` on Linux). Stderr carries the detailed diagnostic from `Print.cpp:1842–1844`; the normal CLI's `result.json` uses the generic `OrcaSlicer.cpp:232` text, the selected one-based plate and the applied config snapshot (`:537–575,6926–6931`).

## Rust destination and scope

The applied-project check belongs in `ares-core::print::validate`. Reuse the resolved `PrintConfig` views and object usage projection as the bounded compatibility shell for upstream `Print`. Carry the CLI switch through the existing validated project requests, outside the persisted settings. Map the distinct validation failure in `ares-cli::failure`. Today the applied-project path has no cross-filament temperature rejection; the separate flat `SliceOptions` STL range check returns `InvalidInput` and is not the applied-project print check. No change to the STL option parser, arrangement temperature classification, GUI preference, or Native CLI log plumbing.

## CLI cases

Slice two synthetic cubes on one plate, each assigned its own filament on a single nozzle. The targets 220°C and 250°C fall outside each other's `[200, 240]` and `[240, 270]` ranges. Without the flag, compare process status, stderr and the Linux result report with Native; verify no G-code. With the flag, require success and compare the whole G-code with only the generator banner normalized. A compatible pair must produce identical output with and without the flag. An invalid range remains fatal with the flag. Use identical arguments except for the existing slice/output command wrapper.
