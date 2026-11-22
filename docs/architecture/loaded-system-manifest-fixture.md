# Manifest-backed system preset parity fixture

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) resolves a typed system file with a nonempty `inherits` through `OrcaSlicer.cpp:2118–2157`. `libslic3r/PresetBundle.cpp:555–650` matches the file against instantiated vendor records by source file identity; `:653–695` and `:7627–7666` load the manifest and complete the inherited preset. Detached copies of the positive files are not equivalent inputs. Native rejects the first such file in `OrcaSlicer.cpp:2238–2240,2281–2286` before slicing.

The source-status contract is separate from completion. `OrcaSlicer.cpp:2296–2306,2331–2345,2445–2452` records the system source for machine, process and filament. `:3203–3231,3374–3381,3592–3604` writes their empty parent slots to `inherits_group`. A typed file with a blank parent bypasses resolution at `:2121–2123`.

## Fixture boundary

- Keep the input project and existing positive leaf bytes. Stage them at the original manifest paths in private copies of the supplied `WEMAKE3D` and `OrcaFilamentLibrary` sources. Use a fresh Native run for each leg and repeat the same positive input.
- The neutral control changes each leaf's raw parent name to a distinct name for an otherwise identical, non-instantiated parent, registered in the same manifest. The typed source stays `system`. Native must produce the same complete stream for all three resolved legs, with `inherits_group = ;;`. No output-derived preset values are used.
- Keep the original detached blank-parent fixture as a separate bypass control with complete Native/API/CLI stream equality. Bypassing resolution is not equivalent to resolving a vendor preset, so it is not the neutrality control.
- Pass the corresponding staged leaf bytes to the public Ares load-settings API and the same leaf files to the public CLI. Require raw Ares API/CLI equality, a raw Ares repeat, and ordered full-stream Native/Ares equality after generator identity only. Record the complete effective config and the first stream difference; do not filter configuration rows.

## Deferred rewrite boundary

This change repairs the inputs to `runner/loaded_system_inherits_group.rs`, not Rust preset resolution. The current direct-import catalog in `ares-core/src/profiles/load_settings.rs` and `profiles/catalog/store.rs` is the temporary passthrough for this witness: the valid resolved input can now show an applied-config difference rather than a Native admission failure. A later rewrite may bring the cited `PresetBundle` manifest identity and completion semantics to the existing catalog and CLI adapter. It must not introduce a second composition or alter the source-status contract. User inheritance, missing-type probing, and installed bundle discovery are outside this fixture change.
