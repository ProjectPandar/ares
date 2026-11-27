# Legacy `zig-zag` project patterns

## Upstream boundary

In OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`_BBS_3MF_Importer::_extract_project_config_from_archive`
(`src/libslic3r/Format/bbs_3mf.cpp:2790–2811`) loads embedded project JSON
through `ConfigBase::load_from_json`. String options go through
`set_deserialize` (`src/libslic3r/Config.cpp:979–985`), which calls
`handle_legacy` before raw deserialization and substitution (`:600–621`).

`PrintConfigDef::handle_legacy` (`src/libslic3r/PrintConfig.cpp:9931–9937`)
rewrites the exact string `zig-zag` to `rectilinear` for these six keys:

- `sparse_infill_pattern`
- `top_surface_pattern`
- `bottom_surface_pattern`
- `internal_solid_infill_pattern`
- `ironing_pattern`
- `support_ironing_pattern`

The key does not change, and the recognized legacy token is not reported as
an unknown enum substitution. The canonical token `zigzag` is distinct and
must not be rewritten.

## Ares boundary

Add the rule to `crates/ares-core/src/options/legacy/token.rs`, at
`handle_legacy`. `ProjectSettingsRaw::handle_legacy` already calls this before
embedded enum substitution in `crates/ares-core/src/project/load.rs`. At
present the raw path can substitute an enum default instead of applying the
legacy rule; the typed legacy rule already covers all six keys and is unchanged.
No fill algorithm, option domain, other legacy rule, or export-only behavior
is included.

## Verification

Slice a synthetic 3MF whose embedded project settings use all six legacy
values. Check the six effective pattern values in the resulting G-code and
compare the public Ares slice output against the pinned Native CLI output
with the existing parity comparator in both reference lanes.
