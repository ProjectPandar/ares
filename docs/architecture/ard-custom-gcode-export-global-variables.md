# Custom G-code global variables live for one export

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) distinguishes an export's persistent variables from the variables local to a single template evaluation:

- `src/libslic3r/PlaceholderParser.hpp:20–32`, `PlaceholderParser::ContextData`, owns `global_config` outside the parser so multiple calls to `process` can share it. `PlaceholderParser.cpp:2688–2699` constructs a fresh `MyContext` per call and attaches the supplied `ContextData`.
- `src/libslic3r/GCode.cpp:3407,3581–3588`, `GCode::_do_export`, initializes a fresh `m_placeholder_parser_integration.context.global_config` for the export. `GCode::placeholder_parser_process` passes the same `&ppi.context` to `ppi.parser.process` at `:5385–5389`. Machine start (`:4381`), layer change (`GCode::process_layer_change_gcode`, `:6106,6140–6142`), and machine end (`:4878`) use that wrapper. A later export starts with an empty global dictionary.
- `PlaceholderParser.cpp:992–1022,1127–1137` resolves a brace identifier against the read-only dictionaries first, then writable system outputs, globals, and locals; it stores a new global in the shared dictionary and a local in `config_local`. `new_old_variable` (`:1529–1553`) prevents shadowing system names or crossing local/global scope; a declaration in the same scope reuses the variable, with typed assignment as in `:1567–1599`. Sharing the dictionary changes lifetime, not grammar, typing, or those conflict rules.

## Diagnosis

In the two Prusa CORE One INDX HF profiles, `machine_start_gcode` declares `global used_tools = 0` and increments it using `is_extruder_used`. `layer_change_gcode` then reads `{if used_tools > 1}`. The symbol is declared by the preset; it is not a built-in tool count. Both unchanged stock projects slice successfully in Native; Ares reports `Not a variable name` in `layer_change_gcode`. A reduced probe with an unrelated identifier shows the same difference, including updates from one layer template being visible to the following layer and to machine end in Native. Changing only the probe's declaration from `global` to `local` makes that cross-template read fail in Native too.

In `crates/ares-core/src/project_slice/gcode_emit/template.rs:16–49`, `render_for_filament` gives each render a new journal and drops every journaled name when it returns, irrespective of scope. `template/macro_block/evaluate.rs:116–155` already distinguishes declaration scopes within one render, but its lifetime is limited to the current render configuration. `machine.rs:194` and `layer_gcode.rs:153–203` render with different configurations, so the later template cannot read the startup declaration.

## Rust destination and scope

Port the export-owned portion of `PlaceholderParser::ContextData` to the G-code emitter's per-export state (`crates/ares-core/src/project_slice/gcode_emit/motion/state.rs::EmitState`, which already carries the template RNG). Make the `template.rs` renderer and `template/macro_block` declaration/assignment path distinguish the shared global dictionary from the per-render locals. The existing journal is a compatibility shell for render-local declarations, not a source of truth for global lifetime. Preserve declaration scope and type metadata needed by assignments and same-scope redeclarations; do not convert global values into preset keys, writer-output fields, or read-only template inputs.

Share that dictionary across the emitter calls corresponding to `GCode::placeholder_parser_process`, including machine start, before/layer change, and machine end. Audit other template calls against upstream before passing the context: a call corresponding to a separate parser context must not inherit this dictionary just because it renders G-code text. Reads and assignments to an existing global from later calls use the same value; locals are discarded after their call; exports and concurrently emitted jobs never share variables. Preserve the existing lookup precedence, type checks, and collision diagnostics. Do not synthesize `used_tools`, change the INDX profiles, or infer the value from the tool list.

This slice does not change preset loading, legacy bracket/suffix lookup (`legacy_variable_expansion`, `:1042–1076`, uses the read-only resolver), inactive-branch validation mode, random generation, system output write-back, or vector grammar/coercion. Already-supported types use the same ownership rule; no new assignment forms or types are introduced. Existing custom G-code error reporting remains in place.

## Verification

1. First add a public CLI E2E using a synthetic printable project and a non-stock identifier. Machine start declares and mutates it, the first two layer calls each read and increment it, and machine end reads the final value. Compare scoped marker order and values to official and seeded Native, retaining outputs and statuses. The current Ares failure must occur at the later read, not at the declaration or preset validation.
2. Use a local-declaration control (same project except for the declaration keyword); startup can read its local but the layer read must produce the same unknown-variable failure as Native. Add a scope-collision or typed reassignment witness to ensure persistent names remain in the writable global dictionary rather than crossing into another namespace.
3. Use the public Rust slicing API for two exports in one process: after a successful export that declared the name, a new export that reads it without declaring it must fail. This verifies export isolation rather than just process isolation.
4. Replay both unmodified stock INDX projects with `--slice 1` and the same arguments on each executable as a diagnostic control. Record statuses and whether export reaches G-code; do not claim full stream parity if an unrelated stock difference remains. Do not add stock-derived or Native output fixtures to the repository.
