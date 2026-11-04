# Object-wide AutoBrim adhesion frozen inputs

Dependency: the ears-EFC, AutoEFC and exact-group ports.

Source: tracked `tests/parity/auto-brim-object-speed/auto_brim-global-only.3mf`,
SHA256 `e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5`.
Settings, first mesh, and instance retained; add a second identical model part
translated30mm, extruder2, within the same object. Original extruder1 remains
PLA. Uniformly extend filament slot arrays. Factorial varies only brim type
Auto/NoBrim and slot2 material PLA/PETG. Both parts extrude first layer.
Classic, ByLayer, support/tower disabled, no skirts, combine_brims0, EFC flag0.
GUI path: add a part, assign filaments, choose brim type.

The frozen source generator, manifest and Native-first evidence are kept
externally. Before replay, resolve the tracked source above and verify its SHA. It copies archive entries, adds the
second component/part/relationship/mesh, and rewrites settings; stored ZIP CRC32.
All4 archives are referenced and SHA-asserted by `auto_brim_object_adhesion.rs`.
No Native output or probe is committed. Exploratory speed/height variants are
not frozen here.

Public tests reuse the existing shared Brim witness flow in
`auto_brim_object_speed::run`, with no copied harness. `NativeCase` invokes Native
first, public API and actual CLI, unchanged complete ordered generator-only
comparisons and raw API/CLI equality. Native Brim presence and positive XY
extrusion are checked first; active material change must differ from Native
PLA/PLA control. M73 remains strict: an M73-leading RED is accepted because
independent actual Brim geometry differs, not an M73-only witness.
