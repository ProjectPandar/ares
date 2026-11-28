# Legacy IMEX project settings

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) loads
embedded project JSON through `_BBS_3MF_Importer::_extract_project_config_from_archive`
(`src/libslic3r/Format/bbs_3mf.cpp:2790–2811`). `ConfigBase::load_from_json`
passes strings to `set_deserialize` and resolves array keys before assembling
their wire values (`src/libslic3r/Config.cpp:979–985,1008–1058`).
`set_deserialize_nothrow` rewrites the pair before applying it (`:600–615`).

`PrintConfigDef::handle_legacy` (`src/libslic3r/PrintConfig.cpp:9995–10003`)
renames `is_ixex` to `is_imex` and changes the `ixex_` prefix to `imex_`.
The former `ixex_carriage_width_x` and `ixex_carriage_width_y` go directly
to `imex_nozzle_clearance_x` and `imex_nozzle_clearance_y`, respectively.
These rewrites change keys, not values; the selected IMEX mode table still
contains the authored strings. `GCode::do_export` consumes the enable flag
and the selected table row to render the startup script and mode placeholders
(`src/libslic3r/GCode.cpp:4131–4227`). The saved-project writer serializes
the resolved configuration (`src/libslic3r/Format/bbs_3mf.cpp:8196–8205`).

## Ares boundary

Port the key rewrites in `handle_legacy` in
`crates/ares-core/src/options/legacy/token.rs`, where
`ProjectSettingsRaw::handle_legacy` already runs before the embedded settings
are deserialized and used for slicing or saved export. The JSON reader
(`Config.cpp:56,876–877,931–943`) uses `nlohmann::json`'s map-backed objects:
literal duplicate keys retain the last parsed value, then distinct keys are
visited in sorted key order, not file order. Ares already deserializes the
raw document into a `BTreeMap` and iterates it when inserting normalized
keys. Keep that ordering: `ixex_*` sorts after `imex_*` and `is_ixex` after
`is_imex`, so the legacy value wins if both spellings occur. Do not rewrite
canonical `imex_*` keys or change their values.

No change to plate metadata aliases, IMEX routing or clearance calculations,
G-code template evaluation, export serialization, general unknown-key handling,
or other legacy rules is included.

## Verification

Use synthetic saved projects with equivalent legacy and canonical IMEX settings,
plus a disabled control. In both reference lanes, compare the emitted G-code
against Native with the existing parity comparison and verify that the selected
non-default mode's script and placeholders are emitted. Compare the saved
`Metadata/project_settings.config` member against Native, including the enable
flag and distinct X/Y clearance values. In the export witness, place
conflicting canonical keys after the legacy keys in the JSON text and
verify Native still chooses the legacy values. Verify Native resolves them
to the canonical keys before using its output as the reference.
