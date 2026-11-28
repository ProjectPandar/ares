# Legacy feature-filament selectors in project settings

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) loads
embedded project JSON through `_BBS_3MF_Importer::_extract_project_config_from_archive`
(`src/libslic3r/Format/bbs_3mf.cpp:2790–2811`). `ConfigBase::load_from_json`
visits distinct source keys in lexicographic order (`src/libslic3r/Config.cpp:56,
876–877,931–943`) and sends each JSON string to `set_deserialize` (`:979–985`).
`set_deserialize_nothrow` rewrites the source pair before raw deserialization
(`:600–615`); `set_deserialize_raw` writes the resolved destination (`:630–700`).
When two source keys resolve to the same destination, the later sorted source
key wins, regardless of the JSON text order or whether the winning key is an
alias.

`PrintConfigDef::handle_legacy` (`src/libslic3r/PrintConfig.cpp:9771–9799`)
renames feature selectors directly to their current destinations:

| Legacy keys | Current key |
|---|---|
| `infill_extruder`, `sparse_infill_filament` | `sparse_infill_filament_id` |
| `solid_infill_extruder`, `solid_infill_filament` | `internal_solid_filament_id` |
| `top_solid_infill_filament` | `top_surface_filament_id` |
| `bottom_solid_infill_filament` | `bottom_surface_filament_id` |
| `perimeter_extruder`, `wall_filament`, `wall_filament_id`, `outer_wall_filament` | `outer_wall_filament_id` |
| `inner_wall_filament` | `inner_wall_filament_id` |

Only the exact legacy scalar token `1` becomes `0` (inherit). Other tokens,
including explicit selectors greater than `1`, pass through to the destination
parser. A `1` written under a current key continues to select filament one.

For a scalar `different_settings_to_system` wire value, the same handler recurses
on its sorted, distinct semicolon-delimited names with an empty value and
applies `ReplaceString` to the original wire (`PrintConfig.cpp:77–84,9844–9854`).
A feature alias in this list must resolve directly to the same current name,
not to an intermediate alias. Keep the upstream replacement order and existing
quote and separator handling.

## Ares boundary

Apply these scalar and recursive-name rewrites in
`crates/ares-core/src/options/legacy/token.rs`. Use the existing feature selector
rules in `options/typed_legacy/actions/rules.rs` for the targets and inherit tokens
rather than adding a second alias table. The raw pass currently rewrites the
three `*_extruder` keys to intermediate names. The raw map can then retain a
canonical key separately from a legacy input that should overwrite it, even
when typed slicing chooses the right value. The saved-project exporter reads
that raw map (`crates/ares-core/src/project/export/saved_presets.rs`).
`ProjectSettingsRaw::handle_legacy` already processes raw entries in `BTreeMap`
order and inserts each result into its normalized map. Write the current key at that insertion and do not change
map ordering or add a later overwrite pass. `normalize_different_settings_to_system`
already recurses into the token handler, so it uses the same feature names.
The typed conversion stays available for inputs that bypass raw JSON load.

The empty value used for recursive name resolution is not the empty wire used
to discover an array's destination. Keep the existing non-scalar path for these
selectors, including its intermediate keys, until array dispatch is ported.
This slice does not change feature selector arrays such as `infill_extruder=["1"]`,
array-valued bookkeeping, malformed-array handling, typed feature application,
filament routing, or obsolete and unknown keys.

## Verification

Use settings-only derivatives of a synthetic saved project. Exercise the two
source ordering directions in Native and Ares saved `Metadata/project_settings.config`:
`perimeter_extruder` later than `outer_wall_filament_id` (legacy `1` and an explicit
selector), and `infill_extruder` earlier than `sparse_infill_filament_id` (current
`1` remains `1`). Write conflicting keys in reverse text order to distinguish
sorted precedence from archive order. Use a canonical control and a scalar
`different_settings_to_system` witness with multiple feature aliases; compare
the full bookkeeping value to Native and verify the old names are absent. Run
both reference lanes. No array cases are included.
