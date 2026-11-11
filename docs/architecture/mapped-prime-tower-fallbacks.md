# Mapped prime-tower colour fallback

## Upstream boundary

This slice ports the CLI colour guard in `disable_wipe_tower_after_mapping`
from OrcaSlicer revision `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `OrcaSlicer.cpp:3850-3891` reads `filament_colour` from CLI
  `m_extra_config`. The branch is entered only when the option exists; empty
  values are excluded before distinct-colour inspection. More than one
  distinct non-empty CLI colour cancels tower disabling. Resolved project and
  preset colours are not substitutes.
- `OrcaSlicer.cpp:4288-4290` force-disables wrapping detection for a BBL 3MF
  when `remove_wrapping_detect` is set. The wrapping branch is deferred because
  this slice does not yet carry that upstream removal flag.
- `OrcaSlicer.cpp:4309-4314` cancels tower disabling when a loaded filament
  slot is mixed. This branch is deferred pending a Native-valid CLI fixture.
  Native validates the parallel arrays in `FilamentMixer.cpp:799-855`: every
  array covers every `filament_is_mixed` slot, each component string parses to
  at least two one-based physical indices, and empty definitions are invalid.
  `PresetBundle.cpp:3885-3891` supplies per-slot defaults, while
  `PresetBundle.cpp:7183-7191` rejects incomplete definitions.

The included guard runs after the existing mapping predicate and before the
final `enable_prime_tower=false` assignment at `OrcaSlicer.cpp:4316-4325`.
It is an upstream behavior port, not a new Ares policy.

## Rust destination

`crates/ares-core/src/project/domain/profile_input/mapped_prime_tower.rs` is
the compatibility shell for the cited mapping decision. Extend its existing
`disable_after_mapping` function without changing the prior single-preset and
smooth-timelapse predicate or the later dirty-column append.

Carry the validated `ConfigOverrides` CLI extra-config boundary into the
mapping decision. Only when it contains `filament_colour` should the function
build the distinct non-empty CLI colour set from raw override tokens; never
inspect resolved project or preset colours. Preserve dirty-column deduplication. Wrapping removal and mixed-filament fallbacks are deferred.

## Verification boundary

Add one complete Ares-vs-Native E2E output comparison for the CLI extra-colour
fallback. Run it in official and seeded modes through the existing parity
runner and compare complete generated output. Wrapping and mixed-filament
witnesses remain deferred until their upstream-valid inputs are available.
Do not turn the fallback into a key-specific output filter.
