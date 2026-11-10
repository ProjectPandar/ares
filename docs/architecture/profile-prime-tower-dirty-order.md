# Selected-profile override and mapped prime-tower order

## Upstream boundary

Source revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

`src/OrcaSlicer.cpp:4138-4221` snapshots supplied CLI option values, applies
`m_extra_config`, and appends changed option keys to the owning
`different_settings_to_system` columns. Existing column order is retained and
existing keys are not duplicated.

Only afterward, `src/OrcaSlicer.cpp:4303-4342` resolves smooth timelapse and the
mapping-derived prime-tower disable flag. Its disable branch sets
`enable_prime_tower=false` and appends that key to the process dirty column,
including when the option was already false. It preserves existing dirty keys
and does not append a second copy.

## Rust destination

`project/domain/profile_input/overlay.rs` currently performs the mapping-derived
prime-tower edit while assembling the profile overlay. The caller in
`project/domain/profile_input.rs` then applies user overrides. This reverses
the cited Native order: a newly recorded CLI override must precede the later
mapping-derived prime-tower dirty key.

Extract the existing mapping-derived operation into a small module under
`project/domain/profile_input/`. Call it after `ConfigOverrides::apply` on the
merged raw configuration and before typed deserialization. Keep ordinary
preset dirty-column construction in the overlay. Do not sort or rebuild dirty
keys from a set; retain their upstream operation order.

The included slice is the existing same-selected-preset mapping behavior, its
smooth-timelapse check, the false assignment, and process-column append without
duplication. Other dirty columns and supplied values remain untouched except
for the cited mapping-derived assignment. The existing mapping predicate is a
temporary compatibility shell, not the complete Native mapping implementation;
mixed-filament and wrapping-detection exceptions, belt-printer handling, and
other mapping behavior are deferred. This change does not extend that predicate
or alter sparse STL view membership.

## Verification contract

Use the existing profile-settings selected-slice comparisons against Native on
the same original input, replacement files, and CLI overrides. Keep their
ordered-byte comparator and assertions. Include the load-settings, replacement
whole-file, palette-growth, adjacent-color, adaptive-spacing, native-jerk, and
no-override STL comparisons. All comparisons use the same official and seeded
Native application contracts; no exporter key filtering or expectation changes
are part of this port.
