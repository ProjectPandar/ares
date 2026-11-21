# Direct CLI `--load-custom-gcodes`

Upstream: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## Native contract

- `src/libslic3r/PrintConfig.cpp:13185–13190` defines a single string argument;
  `src/OrcaSlicer.cpp:1659–1662` extracts it and `:2049–2080` ignores an empty
  path and otherwise reads and parses one `CustomGCode::Info`, not a map of
  plate IDs. `src/libslic3r/CustomGCode.cpp:33–49` reads optional string `mode`
  (`SingleExtruder`, `MultiAsSingle`, `MultiExtruder`; absent/unrecognized
  maps to `Undef`) and required `gcodes` (normally an array, in input order).
  `:14–31` requires each item's string `type`, numeric `print_z`, string
  `color`, and numeric `extruder` (stored as an integer); string `extra` is
  optional and defaults to empty. The types are `ColorChange`, `PausePrint`,
  `ToolChange`, `Template`, `Custom`, and `Unknown` (also the fallback for
  unrecognized strings). See the item and mode types in
  `src/libslic3r/CustomGCode.hpp:17–71`.
- `src/OrcaSlicer.cpp:2053–2056` reports a missing file to stderr and uses
  `CLI_FILE_NOTFOUND` (`-3`, POSIX status `253`); `:2058–2079` reports read,
  syntax, or field conversion exceptions and uses `CLI_CONFIG_FILE_ERROR`
  (`-5`, POSIX status `251`). Both record plate index `0`, with the error
  strings from `:197–199` and report fields/destination from `:535–579`.
  Exit constants: `src/libslic3r/Utils.hpp:45–47`.
- `src/OrcaSlicer.cpp:2042–2046,2064–2073` targets zero-based plate `0` for
  `--slice 0` and non-project input, otherwise `--slice n` targets `n - 1`.
  `src/libslic3r/Format/bbs_3mf.cpp:3437–3465` loads archived events into
  `Model::plates_custom_gcodes`. `src/OrcaSlicer.cpp:4104–4115` replaces that
  whole map on the first model, with no merge even for plates not mentioned
  or an empty `gcodes` array. With `--slice 0`, other plates are still sliced
  but receive no events. `:6522–6531` selects the model's current plate;
  `:6986` passes the model to `Print::apply`.
- `src/libslic3r/Model.hpp:1742–1750` retrieves the current plate's info.
  `src/libslic3r/GCode.cpp:3646–3655` assigns events through `ToolOrdering`;
  `src/libslic3r/GCode/ToolOrdering.cpp:3263–3325` selects a layer using its
  adjacent Z midpoints in nonsequential mode. `GCode.cpp:6969–6972` consumes
  the layer's selected event; `:5697–5714,5755–5774` emits a pause, configured
  template, or literal extra text after the corresponding processor tag.

## Rust boundary and verification

The direct CLI decodes JSON into `CustomGCodeInfo` and its existing
`CustomGCodeItem` values. The slice setup replaces the map owned by `Project`
(`project/plate_custom_gcodes.rs`) before plate selection and print setup.
The existing tool ordering and G-code writer consume that same map; the
adapter does not create another layer-event representation, repackage XML,
normalize the JSON's mode as if it came from XML, or write G-code itself.

The public CLI comparisons use one synthetic STL and one saved synthetic
3MF with an existing layer event. A single `Custom` event in a one-filament,
by-layer print proves arrival in complete G-code and replacement rather than
appending in the saved project. An empty JSON file exercises the syntax error;
a nonexistent file exercises the separate missing-file branch. Compare exit,
stderr, recursive working directory inventory, complete result report, and
complete available G-code using the existing volatile metadata normalization.

Multi-filament event behavior, combinations with other actions, all-plate
saved-project CLI routing, non-array `gcodes` container quirks, and byte-exact
exception details for general JSON errors are outside this slice.
