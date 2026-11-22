# Numeric suffixes in legacy G-code placeholders

## OrcaSlicer boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) routes standalone
`[identifier]` to `MyContext::legacy_variable_expansion`
(`src/libslic3r/PlaceholderParser.cpp:2364-2375`). It resolves the full identifier
first (`:994-1005`, `:1042-1045`). Only on a miss does it split at the last
underscore and resolve the prefix as a vector (`:1046-1057`). A numeric suffix
selects that zero-based element. A nonempty vector whose index is past the end
uses element zero; empty vectors and nil selected elements produce errors
(`:1068-1076`). `GCode::_do_export` renders `machine_start_gcode` through this
parser (`src/libslic3r/GCode.cpp:4381`).

The KINARB 2HB stock startup references `[nozzle_temperature_initial_layer_1]`
with only one configured filament temperature. OrcaSlicer renders the first
temperature for both `_0` and `_1`. This is a parser rule, not a second configured
temperature or a printer-specific startup symbol.

## Rust destination and scope

The destination is the legacy bracket path from `Renderer::render_text` in
`crates/ares-core/src/project_slice/gcode_emit/template.rs` through
`expression.rs` and `expression/parser/variable.rs`. Keep exact identifier
resolution as the first step. On a miss for standalone `[identifier]`, recognize
a nonempty decimal suffix after the last underscore and resolve the prefix in
the same template config. Select the requested vector element, reusing the
first element for an index at or beyond the end, including an index too large
to represent. Keep legacy serialization and route errors through the template's
line/caret reporting. A missing prefix retains the unknown-variable error; a
scalar prefix, empty vector, or nil selected element is not a successful read.

The existing `Config::insert` expansion of vector entries into `key_N` names is
a temporary compatibility shell, not the upstream indexing rule. Leave those
entries and their existing readers untouched in this slice. Do not add synthetic
keys for missing indices or pad the source option. This preserves the existing
full-name lookup, including names whose final segment happens to be numeric.

Defer the nested `[vector[index_variable]]` production
(`legacy_variable_expansion2`), brace expressions, inactive-branch name checks,
non-decimal or empty suffix parsing and malformed-suffix diagnostic offsets,
upstream symbol-store separation, and removal of synthesized names. The two
other printer-class errors and vendor profile loading are separate from this
parser behavior. No CLI selection or preset changes are needed.

## Verification

Start with a synthetic one-plate project using a known matching slicing setup.
Exercise the public slicing API through the existing native stream test harness.
Change only `machine_start_gcode` to emit markers for a singleton temperature
using `_0`, `_1`, and a larger index. Include an in-range read from a vector with
distinct elements (a leading-zero suffix such as `_01` exercises prefix lookup
rather than a synthesized `key_1` entry). Compare complete G-code with the
existing strict comparator in official and seeded modes, and assert the rendered
markers. A control using only exact names must succeed before the rewrite; the
suffix case must reproduce the startup error before the rewrite. Use the existing
paired diagnostic harness for a numeric suffix on a scalar, if that diagnostic
is not covered by the success test's companion cases.

Repeat the original unmodified KINARB 2HB 0.4 project with identical explicit
plate-selection arguments after the rewrite. Require the startup-template error
to disappear and both programs to produce plate G-code. Report any unrelated
whole-stream mismatch separately; successful export alone is not byte parity.
