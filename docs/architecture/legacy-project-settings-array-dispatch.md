# Legacy string-array dispatch in embedded project settings

Upstream: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Owning source

`src/libslic3r/Format/bbs_3mf.cpp:2806` loads the embedded settings through
`ConfigBase::load_from_json`. In `src/libslic3r/Config.cpp:1008–1058` that
function dispatches a JSON array through `handle_legacy(key, wire)` with an
initially empty wire. It stops processing that entry if the key is cleared.
It then selects the wire format from the resulting key's option type, appends
array elements to the **same** wire, and calls `set_deserialize` with the
resulting key and wire. `parse_str_arr` (`Config.cpp:886–924`) uses `,` by
default, `;` with C-style escaped, quoted members for `coStrings`, and `#`
for nested subarrays and `coPointsGroups`. The first member at each depth
has no leading separator, even if the wire already contains text.

`ConfigBase::set_deserialize_nothrow` (`Config.cpp:600–615`) invokes the same
legacy handler again before `set_deserialize_raw` (`Config.cpp:635–710`)
parses the destination option and records any forward-compatibility
substitution. Scalar JSON strings go straight to `set_deserialize`; they do
not get the array's preliminary empty-wire call.

`PrintConfigDef::handle_legacy` (`src/libslic3r/PrintConfig.cpp:9917–9920`)
checks `top_one_wall_type` against `none`, not against the empty string. The
empty scalar binds `only_one_wall_top=1`. With an empty array the preliminary
call writes the same `1`, and there are no elements to append. With `["none"]`
the first call still writes `1`; the joined wire is `1none`, which fails bool
parsing (`Config.hpp:1828–1848`) and substitutes `0` with a warning. A scalar
`"none"` does not take the rename branch and must not enable top-wall mode.

Feature selector aliases use `PrintConfig.cpp:9771–9799` in both passes. For
`infill_extruder=["1"]`, the first call renames to `sparse_infill_filament_id`
without changing the empty wire. The second call sees the current key and
leaves the appended `1` explicit. The scalar `infill_extruder="1"` sees the
legacy key and the token together, so it still maps to inherit (`0`).

## Ares boundary

- Apply dispatch to embedded `Metadata/project_settings.config` entries in
  `crates/ares-core/src/project/raw_settings.rs` and `project/load.rs`, using
  the token rules in `options/legacy/token.rs` and the existing encoding,
  substitution, and per-kind wire decoding in `options/json_deserialize.rs`
  and its submodules. The main load and `project_config_substitutions` must
  see the same embedded rewrites before substitution. The raw map read by
  saved-project export and the typed load must see the same bound option.
- Finish both calls for each well-formed array before inserting its result in
  the normalized map. Resolve collisions in existing sorted **source** key
  order. Pick array encoding after the first rename; parse the post-legacy
  wire using the final destination kind. Do not reconstruct the wire from
  the original array after the prefix or joined wire has been changed. Keep
  the existing per-kind interpretation of unchanged input; do not repeat
  array legacy or vector decoding on already-bound values.
- With the empty-wire pass now retaining the final key and the prefix,
  remove the scalar-only `handle_legacy_scalar` split from the raw project
  path. Use one legacy rule set for scalar wires, both array calls, and
  recursive bookkeeping names, including the existing feature rule table.
  Keep `different_settings_to_system`'s sorted substring replacement
  (`PrintConfig.cpp:77–84,9844–9854`) and its quoted vector member boundaries.
- Keep scalar-only derived writes on their existing source-token path.
  Preserve current handling of other non-string input forms and keep
  caller overrides and independent typed/preset/model settings application
  outside the embedded JSON dispatch. Reuse existing wire handling rather
  than introducing a second option registry or per-option array rewrites.

Out of scope: malformed-array early break and skipping later JSON keys
(`Config.cpp:1051–1055`), global obsolete/unknown-key deletion, new alias or
shortcut lookup rules, and new generic substitution behavior unrelated to
the affected option kinds. Do not change top-level key retention or migrate
unrelated config loaders in this slice.

## Verification

Generate synthetic saved projects by changing only the settings member of a
synthetic 3MF. Check the pinned Native behavior against canonical cases,
then compare Ares with the live Native result in Official and Seeded:

| Input | Observable result |
| --- | --- |
| `top_one_wall_type=""` and `top_one_wall_type=[]` | Saved `only_one_wall_top="1"`, as in the canonical on case. |
| `top_one_wall_type=["none"]` | Saved off, and the slice reports substitution of `1none` by `0` for `only_one_wall_top`; scalar `"none"` remains off without that warning. |
| `infill_extruder=["1"]` | Saved `sparse_infill_filament_id="1"`; a scalar legacy `"1"` remains inherit. |
| `enable_arc_fitting=["0"]` | Saved scalar enum `"disabled"`. |
| `filament_type=["ASA-Aero"]` | Saved vector `["ASA-AERO"]`. |
| `extruder_type=["DirectDrive"]` | Saved `["Direct Drive"]`, without an enum substitution for this legacy token. |
| `different_settings_to_system=["bridge_fan_speed;layer_height","",""]` | Saved `["overhang_fan_speed;layer_height","",""]`, with the original slots preserved. |

Use saved `Metadata/project_settings.config` and structured CLI warnings as
public observations. Run scoped paired base/tip parity over project import,
saved export, substitutions, scalar-array handling and refresh. Retain the
preceding scalar feature-filament cases as compatibility coverage.
