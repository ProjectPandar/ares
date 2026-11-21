# Type1 wipe tower template strings

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/libslic3r/GCode.cpp:1102,1593–1613,1624–1631`: `WipeTowerIntegration::append_tcr` stores four already-rendered G-code fragments as `ConfigOptionString` values before expanding the rotated tower G-code. Legacy placeholder expansion calls `serialize()` (`PlaceholderParser.cpp:1064–1067`, `Config.hpp:1172–1178`). `Config.cpp:76–96,150–170` defines the matching C-style encoding and decoding: CR, LF, double quotes, and backslashes are escaped. The outer template processor observes that escaped expansion; the downstream tower stream receives the unescaped result.

## Rust boundary

In `crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/emit.rs`, serialize the `filament_end_gcode`, `restore_layer_z_before_toolchange`, `change_filament_gcode`, and `filament_start_gcode` values before passing them to the existing outer template renderer. Retain the final unescape and the existing writer-state update order. Ordinary Rust strings in the existing template call are a compatibility shell for OrcaSlicer's config-option boundary; no change to the generic Rust template value model is required.

Preset input decoding, `CONFIG_BLOCK` export, Type2 substitution, slot ordering, and tower geometry are unchanged.

## Public behavior

On a two-filament X1 Carbon project, put a literal backslash followed by `n`, a quoted string, and a second backslash sequence in the second filament's start script. They must reach the tower G-code in one comment line, not become a line break or lose backslashes. Native replay, the Ares slice API, and the Ares CLI must agree on the entire ordered G-code stream with only generator identity replaced.
