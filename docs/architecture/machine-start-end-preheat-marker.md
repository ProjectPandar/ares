# Preheat-gated machine-start end marker

## Source boundary

Base commit: `069a5fdd3d94ab12a18df8a4b38e0a680655be89`.
Upstream pin: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This slice ports a missing `libslic3r` G-code producer branch, not a new Ares
pipeline, marker convention or independent preheating system.

| File under `src/libslic3r/` | Port boundary |
| --- | --- |
| `GCode.cpp:4379-4386` | After writing rendered machine-start G-code, emit the machine-start end marker iff `m_config.enable_pre_heating.value` is true. |
| `GCode/GCodeProcessor.cpp:185` | `Machine_Start_GCode_End_Tag` is exactly `" MACHINE_START_GCODE_END"`; the generator supplies `;` and newline. |
| `PrintConfig.cpp:8837-8838` | Existing bool option `enable_pre_heating`, default false; do not add or change an option. |
| `GCode/GCodeProcessor.cpp:1687-1703` | Adjacent consumer records the machine-start boundary line for usage-block construction. Consumer/injector behavior is not included in this producer slice. |

Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/machine.rs::append_start`,
immediately after the rendered start output/newline block and before
`exhaust_fan::append_print_start` or the compatible-flavor preamble. The existing
startup emitter is a temporary compatibility shell around upstream G-code
export; add the missing upstream branch there, not a second emitter.

## First differing producer

The eight `smoke::sweep::prime_tower_export` cases cover H2D/X2D nozzle sizes
0.2/0.4/0.6/0.8. Retained official and seeded outputs each contain 8 FAILs.
Across all 48 retained oracle-attempt/Ares pairs, raw line comparison finds only
two differing regions: generator identity and the missing marker line. Both
config blocks state `enable_pre_heating = 1`. The marker is 26 bytes including
newline; it explains each normalized output's 26-byte length difference.

Representative `h2d_04` first mismatch: byte 32967, line 800, column 3, following
`M73 P6 R4`: reference `; MACHINE_START_GCODE_END`, Ares
`; filament start gcode`. Rust already has the typed flag
(`options/process_options/gcode_source.rs:17`, runtime field map
`options/gcode_fields.rs:8`) but does not consume it at this emission boundary.
This does not indicate a tower-motion or receiving-volume correction is needed.

These are retained `runner_cache` artifacts whose manifests state unknown
reference-producer identity; they are localization witnesses, not fresh Native
runtime credit or hypothetical PASS results. Their FAIL outcomes remain FAIL.

## Included and deferred behavior

- Emit exactly `; MACHINE_START_GCODE_END\n` when the existing effective bool is
  true; emit nothing when false or defaulted. No new gate, profile-name/vendor
  predicate, printer whitelist or `gcode_comments` predicate is permitted.
- The flag controls emission even when rendered machine-start G-code is empty.
  Keep template handling, custom-role markers, newlines, chamber/exhaust output,
  filament-start order and writer state otherwise unchanged.
- Do not filter this marker as a comment or normalize it away. Do not edit the
  comparator, reference, fixture, profile, option defaults or decoder to pass.
- Defer usage-block construction, preheat/pre-cool injection, machine-end
  markers, temperature scheduling, tool changes, tower geometry and purge
  planning. A producer-marker PASS does not establish preheating functionality.
  Stop and report a new first difference in any adjacent owner.
- The producer remains platform-neutral, without file I/O, UI, terminal or native
  runtime dependencies. WASM compilation does not establish browser behavior.

## Verification before completion

Existing eight complete-byte smoke cases provide the initial failing coverage;
retain them unchanged. Before implementation, bind a fresh matched-catalog
Native run to the selected H2D/X2D inputs and confirm the flag/marker/order, then
record public API and CLI full-byte failures. Add focused public E2E controls before
code for flag false/default and true with empty start; do not use isolated tests
or marker-only assertions as the completion gate. Use real geometry and the
existing generator-identity-only complete ordered-byte comparator.

After the source-owned branch, require all eight smoke cases plus focused
controls to pass complete Native/API/CLI comparison in official and seeded,
with identical adjacent-regression IDs and fresh baseline→candidate P→F 0.
Use separate worktrees and Cargo targets for different source trees. Retain any
new first difference without extending this slice or weakening the comparison.
