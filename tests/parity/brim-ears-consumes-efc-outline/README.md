# BrimEars EFC outline frozen inputs

Source: merged `tests/parity/per-object-skirt-brim-types/brim-ears.3mf`, SHA256
`ff8389789f3795235be0cc11666e5acb301e3f5797bbf915366a694495d31593`.
All four derived archives retain source mesh/model/instances/materials/speeds.
Disable support and prime tower uniformly; set compensation layers1; vary only
`elefant_foot_compensation=0|0.3` × `brim_use_efc_outline=0|1`.
Input SHA256s are asserted in `brim_ears_consumes_efc_outline.rs`; all inputs referenced.

Native-first evidence and replayable stored-ZIP settings generator:
`/home/indexyz/ares-coord/evidence/worker2-ears-efc-discovery-20261020T214059/`.
The generator copies archive entries, rewrites project-settings JSON only,
and writes per-entry CRC32. No Native output or probe is committed.
Replay against verified source hash into fresh external inputs.

Explicit dependency: approved AutoBrim EFC tip
`14d90d03f8e1188cf4305a1ac712239179a7678f`, including exact-group `fb70752b9`.
Public tests use merged `runner/native_api_cli.rs`, actual CLI + public API,
unchanged strict complete comparison and raw API/CLI equality; no M73 mask.
Native actual ear brim and skirt extrusion checked before comparison.
