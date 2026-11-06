# Validation message text (2.5 wording)

Rewrite slice of the `Print::validate` line-width messages at OrcaSlicer v2.5.0-beta `345efec64`. It also brings the test expectations for those messages and for the `DynamicPrintConfig::validate` range message up to the same version. The runtime Native binary prints the 2.5 text. Ares's `Print::validate` port and the test literals still carry an older wording.

## Problem
At main `8d2af62db`, 37 tests fail in both modes for this reason alone:

| Tests | Count |
|---|---|
| `runner::stock_validation::cases` | 22 |
| `print_validation::minimum_width` | 8 |
| `runner::profile_entry::loaded_config_ranges` | 4 |
| `print_validation::maximum_width` | 2 |
| `print_validation::bridge_width` | 1 |

Runtime Native stderr, copied from the test artifacts:

| Case | Native stderr | Ares | Test literal |
|---|---|---|---|
| layer height ≥ line width | `Line width too small\n` | `Too small line width` | `Too small line width` |
| width above 5× nozzle | `Line width too large\n` | `Too large line width` | `Too large line width` |
| `retraction_distances_when_cut` out of range | `…: 0 not in range [10, 18]` | `…: 0 not in range [10, 18]` | `…: 0 not in range [10.000000,18.000000]` |
| `extruder_printable_height` out of range | `…: 2100 not in range [0, 1000]` | `…: 2100 not in range [0, 1000]` | `…: 2100 not in range [0.000000,1000.000000]` |
| nil `filament_flush_temp` | `…: nil not in range [0, 1500]` | `…: nil not in range [0, 1500]` | `…: nil not in range [0.000000,1500.000000]` |

- **Line-width messages:** both Ares and the literals are behind Native.
- **Range message:** Ares already matches Native byte for byte, and only the literals are behind.

## Upstream boundary
- **`Print::validate`** (`Print.cpp:1969`):
  - The `validate_extrusion_width` lambda (`:2580-2592`) sets `err_msg = L("Line width too small")` when the minimum-nozzle width is at or below the layer height (`:2586`). It sets `L("Line width too large")` when the maximum-nozzle width exceeds `max_nozzle_diameter * MAX_LINE_WIDTH_MULTIPLIER` (`:2589`).
  - The bridge line-width check (`:2706-2715`) returns `L("Line width too small")` for a bridge width at or below the layer height without thin bridges (`:2711-2714`).
  - The English catalog leaves both untranslated (`localization/i18n/en/OrcaSlicer_en.po`, `msgstr ""`).
- **`DynamicPrintConfig::validate`** (`PrintConfig.cpp:10706`) formats an out-of-range value as `Slic3r::format(_u8L("%1% not in range [%2%, %3%]"), opt->serialize(), optdef->min, optdef->max)` (`:12689`). The bounds are doubles through `boost::format`. Native prints the integral bounds above as `10`, `18`, `0`, `1000`, `1500`.
- **The CLI** writes the map under `Param values in 3mf/config error: ` (`OrcaSlicer.cpp:4277-4284`).

## Ares destination
- **Production:** `crates/ares-core/src/print/validate/extrusion_width.rs`, the three messages that port `Print::validate`:
  - `width` → `:2586` `Line width too small` and `:2589` `Line width too large`;
  - the bridge check → `:2711-2714` `Line width too small`.
- **Unchanged:**
  - **The range message** (`options/validation/loaded.rs:243`). It already writes `[{min}, {max}]` and matches Native for every witnessed bound.
  - **The `PrintConfig.cpp:12641` "too large line width <value>" config message** (`options/validation/extrusion_width.rs:67`). It already uses the 2.5 text.

## Tests
The affected tests compare a literal against both sides: Native stderr when the Native runner is configured, and the Ares message. Ares is therefore held to the runtime Native bytes whenever Native runs, and to the 2.5 text when it does not. Every comparison stays exact; none becomes a `contains` or prefix match.

Each literal is replaced by the upstream 2.5 text:
- `Line width too small` (`Print.cpp:2586`, `:2711-2714`);
- `Line width too large` (`Print.cpp:2589`);
- `[0, 1500]`, `[10, 18]`, `[0, 1000]` (`PrintConfig.cpp:12689`, Native runtime bytes).

The harness classifier `runner/stages.rs`, which recognizes the skin/skeleton line-width rejection in Native stderr, and its `stage_tests.rs` cases switch to `Line width too small` the same way.

Files:
- `flow_errors.rs`
- `print_validation/{bed_compatibility_controls,bridge_width,bridge_width_controls,layer_nozzle_controls,maximum_width,maximum_width_controls,minimum_width,minimum_width_controls}.rs`
- `runner/{loaded_config_ranges,stages,stage_tests}.rs`
- `runner/stock_validation/cases.rs`

`orca_parity.rs` stays byte-identical.

## Included and deferred
- **Included:**
  - the `Print.cpp:2586`, `:2589` and `:2711-2714` texts;
  - the test expectations of the `PrintConfig.cpp:12689` range text for the witnessed bounds.
- **Deferred:**
  - **`boost::format` rendering of non-integral or very large range bounds** in `loaded.rs` (for example `1e+06`). No witness reaches it, and it lies outside `print/validate`.
  - **The other `Print::validate` messages.** They already match.
  - **The historical records under `docs/`** that quote the older wording. They describe earlier Native binaries and stay as written.

## RED
The RED commit changes only test literals.

- **Fail on main:**
  - The 19 line-width tests of the 37: stock_validation 8, `minimum_width` 8, `maximum_width` 2, `bridge_width` 1. The Native assertion now passes, and each then fails at the Ares assertion on `Too small/large line width`.
  - The Ares-only line-width controls, which pass on main today and fail once their literal carries the 2.5 text: `minimum_width_controls`, `maximum_width_controls`, `bridge_width_controls`, `layer_nozzle_controls`, `bed_compatibility_controls`, `flow_errors`.
- **Pass on main:** the other 18 of the 37, the 14 range-only stock cases and the 4 `loaded_config_ranges` cases. Ares already matches Native there; those tests were blocked only by the stale literal.
- **GREEN:** changes the three production strings, after which all of them pass.
