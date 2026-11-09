# Applied default-preset membership on partial project entry

Base: `bef86490057f6a0824a08c5c392e1c3ced4a2b7b`.
Upstream: `OrcaSlicer@345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Source-owned boundary

`Preset.cpp:1059-1089::Preset::load_external_config` constructs a dynamic
config from the default preset, overlays the project's values, removes
connection keys, and returns the default preset's complete keyset.
`OrcaSlicer.cpp:3167-3200::load_project_preset` inserts those keys into
`m_print_config` when that preset kind has not been replaced and its
project system identity is nonempty. Bookkeeping keys are excluded.
`PrintApply.cpp:1434,1676` preserves the applied dynamic config;
`GCode.cpp:8246,8338-8351` serializes its actual keys, not merely the static
`FullPrintConfig` default membership.

The existing filament-only whole-file failure shows eight such defaults
in Native but not Ares. Neither input 3MF nor filament JSON contains them;
Native's retained log records their insertion from the MK3S printer and
process default-preset path. A false `full_config_default` bit does not
mean a key cannot become an applied dynamic member.

## Rust destination and non-negotiable rule

Reuse `profiles/project_refresh{.rs,/preset.rs,/keys/}` — the same
source-cited seam already used by project-preset-catalog refresh.
Use the complete existing machine/process preset keysets and their
registry-backed `default_value`, not an eight-key list, a global registry
default expansion, or a membership-bit revert. Reuse the existing
system-name, connection, and bookkeeping rules.

In `project/domain/profile_input.rs::from_3mf_with_profiles`, seed the
embedded merge base with absent defaults for a retained, named machine or
process kind before profile overlays and typed deserialization. A loaded
kind is not implicitly defaulted as a retained project preset. Explicit
project values and loaded overlays win. Ordinary byte-only project entry
and STL entry do not acquire these applied preset members. The existing
catalog-refresh path already materializes its own default-preset keyset.

`full_config.rs` continues to require static default membership or actual
applied raw presence. The membership flags, canonical decoder, nil rules,
wire serializers, fixtures, assertions, and comparator remain unchanged.
System-preset value refresh and compatibility decisions beyond this
missing applied-presence boundary remain with their existing owners;
this is not a new Ares configuration pipeline.

## Test-first and exit gates

Reuse `load_settings_3mf_partial::load_settings_3mf_filament_only_whole_file`
as the failing test on this base; preserve its complete-output assertion.
Verify that the initial complete-output difference is the absence of
applied default-preset keys, not a decoder failure.

Then require the other load-settings 3MF cases and the four existing
print-host/default-bed/default-nozzle-volume/residual-membership witnesses
to remain PASS. Run the full `load_settings*` and `config_block*` test
families plus membership tests in official and seeded modes; retain
identical baseline/candidate IDs, raw exits, complete-output artifacts,
and zero PASS-to-FAIL transitions. Verify rustfmt, strict workspace/all-target
Clippy, and unchanged ordinary byte-only 3MF and STL behavior. Independently
owned differences must remain observable, not be hidden by key whitelists,
parser changes, or relaxed complete-output comparison.
