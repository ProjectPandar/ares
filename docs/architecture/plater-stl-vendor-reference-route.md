# STL plater vendor-preset reference lifecycle

## Upstream boundary

The source boundary is OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `Preset.cpp:1623–1626,1746–1754,1818–1824`: printer collection keys and creation defaults.
- `PresetBundle.cpp:7827–7839`: resolve a vendor preset over parent/collection defaults, includes, then authored values.
- `OrcaSlicer.cpp:2121–2123`: explicitly typed JSON without inheritance bypasses preset resolution.
- `GCode.cpp:8272–8274,8370–8382`: emit applied config members, excluding banned/nil options.
- `Plater.cpp:7056–7058,7111–7112,7149–7160`: session placement, grounding, and instance edits.

## Rust destination and invariant

`crates/ares-cli/tests/orca_parity/runner/plater_stl_session.rs` compares
`PlaterSession` slicing with an independently resolved Native vendor selection.
Both sides use the same catalog and selected machine/process/filament identities.
Native receives staged vendor leaves through `native_user_presets::resolved`,
not flattened typed/no-inherits files that skip the collection lifecycle.

The reference lifecycle must include collection-default membership, not merely
the effective values of explicitly authored options. Host, nozzle-variant, and
preset metadata are not globally removed from the G-code config block.

The Ares request, centering and move operations, Native model coordinates,
`--arrange 0`, and generator-only exact byte comparator remain unchanged.
The test adapter is a compatibility shell around the upstream preset/Plater
concepts, not an independently designed slicing pipeline.

## Included and deferred behavior

Included: the selected vendor inheritance/default lifecycle for both centered
and moved STL reference slices. The public Ares API remains unchanged.

Deferred: other test routes, UI behavior, placement algorithms, exporter policy,
and explicitly typed sparse-file semantics. Direct flat/flat controls continue
to exercise a distinct upstream loading route.
