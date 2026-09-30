# Filament-printable range coverage

Follow ARD-0060's `PrintConfig`, `ConfigOptionInts` and
`Print::get_physical_unprintable_filaments` boundaries. Rust destination is
existing project option serialization/config output plus native test-adapter
domain generation, not a new pipeline.

Replace the single authored mask sample with serializable signed endpoints,
a deterministic full-interval interior and masks 0/1/2/3. Each legal sample
must survive strict requested/effective proof and compare the complete original
and Ares G-code streams, with paired artifacts retained. INT_MAX rejection is
separate negative evidence. Preserve the strict comparator.

This fixture exercises the source-owned single-extruder branch. Two-extruder
bit enforcement, grouping and map-conflict diagnostics remain explicitly
unverified. Neither finite range sampling nor a green focused gate closes
the full user objective.
