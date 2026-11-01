# Raw coBools CONFIG_BLOCK serialization

Source: OrcaSlicer345efec64c7b95a05ce5cf96df0dde66a158835c,
Config.hpp:1994-2003 ConfigOptionBoolsTempl::serialize and:2016-2041
deserialize_with_substitutions. Boolean vectors serialize0/1 elements with
commas; a semicolon token is not a legal bool-vector wire value.
PrintConfig.cpp:3545-3555 defines filament_is_mixed as coBools and
filament_mixed_components as coStrings;:8955-8961 defines
filament_prime_volume as coFloats. GCode.cpp:8287 bans prime_volume_mode.

Rust boundary: project/raw_settings.rs RawConfigValue::option_token, the
existing type-owned raw CONFIG_BLOCK export adapter. The newly known mixed
flag lacks a typed ProjectSettings field, so raw export must follow its Bools
registry kind rather than the generic internal semicolon-list representation.
Add the Bools list serializer here; do not replace semicolons in arbitrary
strings or special-case one key. Keep typed collection, definitions, inputs,
Native producer, comparator and processor algorithms unchanged.

Audit of the four newly added definitions:
- filament_is_mixed: Bools requires comma0,0; current generic raw path is wrong.
- filament_mixed_components: Strings correctly uses escaped semicolon slots.
- filament_prime_volume: Floats already uses per-element numeric formatting
  and comma join in this same adapter.
- prime_volume_mode: scalar enum and explicitly banned from CONFIG_BLOCK.

The existing three vg1_x1c global-relative, origin/inches and virtual-state
public processor witnesses are the regression seam. Each is RED on54b2fff56:
Ares emits filament_is_mixed=0;0; qualified beta merged Native aborts134 with
BadOptionValueException before move CSV creation. GREEN must produce0,0,
allow Native processing to finish, and retain all existing exact move/state/
time-bit assertions. No test replacement, masks, outputs or ROOT registration.
Native artifacts and before/after captures remain external. Other beta config
shape migrations and active slicing algorithm changes are deferred.
