# Authored IDs for explicitly loaded user filaments

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) distinguishes a
loaded collection record from the metadata of a file passed to `--load-filaments`:

- `PresetBundle.cpp:902–904,1511–1512` loads the selected user's filament
  collection. `Preset.cpp:1900–1901` reads the file's ID, but when
  `find_preset2` resolves its parent (`:1907–1917`), the canonical record copies
  the parent's config and ID (`:1925–1932`).
- `OrcaSlicer.cpp:2118–2157` resolves inherited files through the bundle.
  `PresetBundle.cpp:582–606` matches by same-file identity and returns the loaded
  **config**. It does not replace the CLI's original JSON metadata map.
- `Config.cpp:962–963` records the authored ID in that map. The CLI starts with
  an empty per-file ID (`OrcaSlicer.cpp:2419–2424`) and reads only the original
  map for its ID (`:2205–2218,2250–2255`). It carries that value alongside the
  resolved config (`:2449–2453`) and writes it to the loaded logical slot
  (`:3616–3622`). A missing or explicitly empty ID yields an empty element;
  a nonempty authored ID survives even if it differs from the parent's.
- `Config.hpp:1207–1210` and `Config.cpp:109–140` serialize a sole empty element
  as `""`. This is one filament slot with an empty ID, not a missing slot.

## Rust destination

`profiles/catalog/source_records.rs:85–102` builds the flattened user record,
including fields inherited from its parent. `profiles/load_settings.rs:142–148`
replaces the request document with that canonical record before inserting the
slot. `profiles/catalog/compose.rs:116–127` already takes the ID from the selected
record rather than from merged metadata, but for this request the selected
record is now the canonical one. Keep the authored ID separate across that
admission boundary and apply it to the composed loaded slot in
`profiles/load_settings.rs`. Use the same slot order as the existing composition;
skipped input positions and retained project slots are not new ID entries.

This extends the authored-ID rule to explicitly loaded inherited user files.
Catalog selections (including the complete STL witness), canonical metadata,
standalone typed files, system sources, and physical option inheritance are
unchanged. Admission still requires an independently loaded record with matching
kind and opaque host identity. No public API, filesystem acquisition, or
serialization change is needed.

## Verification

Use the isolated CLI datadir witness with a user filament inheriting a parent
with nonempty ID. Check omitted, explicit empty, and distinct nonempty authored
IDs against Native. Both slicers must succeed; the expected Native ID line and
complete banner-normalized G-code are checked independently. The unmodified
branch emits the parent's ID for omission, despite matching all other lines.
Keep the existing catalog/STL and user producer witnesses unchanged in meaning.
