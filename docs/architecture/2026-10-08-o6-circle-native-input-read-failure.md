# O6 synthetic circle input: Native input-read failure

## Disposition

This is **NATIVE_INPUT_READ_FAILURE**, `parity_pass=false`, not the
O8-X2D-SCALAR selector-array family. Neither a successful Native slice nor
Native/Ares G-code equality is established for the original synthetic input.
Do not use this receipt to change the printable-area detector or imitate a crash.

## Exact input and reproduction

- Source: `crates/ares-core/tests/plateless_instances_hit_no_suitable_objects_gate.rs::circle_bed_cylinder_bbox_outside_vertices_inside`.
- SHA256: `82a5d8d342c8599251edf886aea67092610d3638781818d452ff14a718f1e225`.
- Producer ELF SHA256: `59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`;
  Build-ID `bdbbf4336095c2ce6283fc5441e1e56fbf595b99`.
- Fresh admitted execution: exit 139, no exported G-code.
- Symbol-matched stack: `std::string::_M_assign` → `CLI::run:1700` → `main:7543`.
  This is the input configuration/settings-string assignment region
  (`OrcaSlicer/src/OrcaSlicer.cpp:1699-1704`), before slicing.
- One nozzle (`[0.4]`), not the four/five-nozzle singleton-selector trigger;
  stack does not reach `get_extruder_variant_string` or CLI selector access at 3462.

The first launcher failed before Native execution because `ORCA_REPLAY_BIN`
was not declared; that attempt is not a Native verdict. The admitted run is separate; debugger executions are
stack diagnostics, not retries promoted to parity results.

## Read-only configuration comparison

The synthetic archive has eight project settings. All three IDs are present:
`printer_settings_id="Kobra 3"`, `print_settings_id="Std"`, and
`filament_settings_id=["PLA"]`. It lacks `printer_model`,
`printer_extruder_variant`, and `print_extruder_variant`.

The stock-exported control input has SHA256
`67a27bafcda838742db058b1edb812055c9470608e47dc83a314357a8e9b233d`,
635 settings, and all six compared fields. Its IDs name the Creality Ender-3
stock printer/process/filament; its two variant fields are
`["Direct Drive Standard"]`.

Raw JSON presence does not prove the Native importer populated its runtime
configuration. The stack alone does not establish which pointer/value caused
the failed assignment. Missing variant/model fields are observations, not a
proven root cause. This slice does not repair the synthetic fixture constructor.

## No-brim child and replacement-control outcome

The one-field child changes only `brim_type=no_brim`, SHA256
`f41bddbe88422d1991ccd905fc3e068f7a61a307de2509ab12e172cb610112ef`.
Its fresh admitted execution also exited 139 without G-code.
Neither sparse archive is an accepted Native slicing control. The original
constructor is not repaired by this slice.

The complete stock-template control is separately documented in
`../superpowers/specs/2026-10-08-stock-circle-print-volume.md`. Native returns
-50 at the Model/BuildVolume gate, not -102. The original cylinder placement
is debugger-confirmed; default stock brim remains unchanged.
