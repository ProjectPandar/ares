# Process compatibility for direct STL with explicit CLI settings

In OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`CLI::run` (`src/OrcaSlicer.cpp:2890–2894,2922–2927`) calls
`is_compatible_with_printer` with the separately loaded process and machine
configs and the printer's system name when both presets are supplied.
`Preset.cpp:956–971,974–996` gives a nonempty literal list precedence. With
no such list, it evaluates a nonempty `compatible_printers_condition` against
the machine config; no list and no condition means unrestricted. A false
expression rejects the pair; an expression evaluation error is accepted as in
`Preset.cpp:962–966`. The CLI adds `printer_settings_id` to the machine config
(`OrcaSlicer.cpp:2295–2303`); the preset helper separately supplies the system
name as `printer_preset` and the nozzle vector length as `num_extruders`
(`Preset.cpp:974–996`). Direct STL has no embedded printer or upward list to
trigger the printer-switch handling at `OrcaSlicer.cpp:2973–2998`. The final
rejection is `CLI_PROCESS_NOT_COMPATIBLE` (`:3000–3004`).

The [machine-only specification](2026-11-12-machine-only-load-settings.md)
lines 151–152 leave complete-list adjudication in place because that work
addresses an omitted process on an embedded project. Treating the retained
literal-only implementation as a rule for explicit direct STL settings
conflicts with `OrcaSlicer.cpp:2922–2927` and `Preset.cpp:956–971`: an empty list
is not an explicit denial. This slice supersedes that preservation decision
only for the empty or absent list of a supplied process on direct STL.

`crates/ares-core/src/profiles/load_settings.rs` takes the compatibility
condition from the supplied process and retains the supplied machine's own
options for this check. `project_slice/profile_entry.rs` dispatches at the
existing load-settings gate; `project_slice/compatibility.rs` reuses the
retained-process condition evaluator and typed machine option conversion. Do
not evaluate against the merged project, process or filament options, or user
overrides. The predicate does not change option composition or export. Keep
nonempty-list adjudication, embedded 3MF checks (including the machine-only
path), `--load-settings` input resolution, upward-machine reporting and
catalog/profile entry unchanged. Extending preset inheritance resolution or
porting the other CLI compatibility branches is outside this slice.

## Verification

Use the grounded Prusa CORE One 0.4 nozzle cube through the public CLI with
explicit machine, process and filament settings. The stock process has an
empty list and a regex/nozzle condition. Vary only process compatibility keys
for controls: omit both constraints, evaluate a false condition, allow a
literal match despite a false condition, and reject a literal miss despite a
true condition. Require real runtime Native comparisons in both modes,
keeping `--arrange 0` on Native. Compare exit and signed report return code;
for successful slices compare the whole ordered G-code after removing only the
generator banner. Rejections must produce no G-code. Retain the bytes, commands
and outputs, demonstrate the failing assertions before the production fix,
and compare identical scoped inventories across the fix in each mode.
