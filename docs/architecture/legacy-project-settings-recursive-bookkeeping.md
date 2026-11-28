# Legacy names in project settings bookkeeping

Upstream: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Owning source

`_BBS_3MF_Importer` loads embedded `Metadata/project_settings.config` via
`ConfigBase::load_from_json` (`src/libslic3r/Format/bbs_3mf.cpp:2806`).
`ConfigBase::set_deserialize_nothrow` calls the legacy handler on the input
key and wire before binding the option (`src/libslic3r/Config.cpp:600–615`).

For `different_settings_to_system`, `PrintConfigDef::handle_legacy` removes
quotes from a copy of the wire, splits it at semicolons, and visits distinct
names in sorted order. It calls itself with each name and an empty value
(`src/libslic3r/PrintConfig.cpp:9844–9854`). When the name changes, including
when it is cleared, `ReplaceString` applies that replacement to the whole
current wire (`PrintConfig.cpp:55–84`). It does not remove a vector member or
rewrite separators. The handler clears obsolete keys before checking the
resulting key against `print_config_def`; an unregistered result is also
cleared (`PrintConfig.cpp:10005–10039`). The exporter serializes the bound
project config to the saved 3MF settings member
(`Format/bbs_3mf.cpp:8199–8205`).

## Ares boundary

Apply the terminal obsolete and registry checks to names visited by
`normalize_different_settings_to_system` in
`crates/ares-core/src/options/legacy/token.rs`. Keep recursive calls in the
shared token handler so aliases are resolved before membership is tested.
Extend the shared raw legacy rules to cover the remaining upstream retirements
for recursive names without copying the existing obsolete list. Use
`options/registry.rs::option_definition` for membership; do not add another
registry or a fallback.

Keep the current sorted substring replacements and preserve the quote and
separator structure consumed by the existing vector decoder. The correction
is at raw legacy dispatch, before `ProjectSettingsRaw` in
`crates/ares-core/src/project/raw_settings.rs` supplies the saved-project
export. It is not a second filter on serialized output.

### Already-bound bookkeeping

`CLI::run` resizes and edits the bound `ConfigOptionStrings` vector without
calling `handle_legacy` again (`src/OrcaSlicer.cpp:3202–3223`). The first
pass can produce an unregistered name by overlapping substring replacements.
A second pass misreads that result as input and erases it. The legacy pass
after the 3MF profile or override merge therefore preserves the bound
bookkeeping vector. An explicit override of the bookkeeping option is a new
input and still goes through legacy dispatch.

Out of scope: changes to which direct settings are retained, any top-level
obsolete or unknown-key filter, changes to typed legacy handling, and
changes to substitution or warning reporting. A nested name is bookkeeping
text, not a separate setting submitted to `set_deserialize_nothrow`.

## Verification

Export synthetic saved projects through the public CLI in both reference
lanes. Set the scalar bookkeeping wire to `;"silent_mode;layer_height";`
for a retired name, `;"probe_unregistered_config_key;layer_height";` for an
unregistered name, and `;";layer_height";` for the already-cleared control.
Compare the full saved `different_settings_to_system` member against live
Native output. Both witnesses must match the control's
`["", ";layer_height", ""]`: the registered name and all vector slots stay.
