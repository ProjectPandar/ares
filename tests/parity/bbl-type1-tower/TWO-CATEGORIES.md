# Two-category RED addendum

Stock X1 Carbon 0.4, unchanged two-object baseline, stock Bambu PLA Basic plus
Bambu Support For PLA/PETG. Filament adhesiveness categories100 and705; both
mapped to nozzle1, object slots remain distinct. Stock support filament assigned
to the second object is GUI-reachable; no category/temperature/template edits.

Initial PLA + Bambu PETG Basic attempt (categories100/300) was rejected by
Native's mixed-temperature validation (exit194/-62). It is NOT a qualifying RED.
No --allow-mix-temp/no-check flag or modified temperature range was used to bypass
it. Selecting stock Support For PLA/PETG instead succeeds official and seeded.

The candidate was produced by official Native --load-filaments against the
accepted baseline and --export-3mf. Variant-aware preset flattening mirrors
presets.rs; the retained preparation script records source selection. Evidence:
/home/indexyz/ares-coord/evidence/w5-type1-two-category-support/.

Input SHA25609d8037066427023093b02b068e7c29dd36acb27d9680d173f82925e6eb42f3a.
Separate test module uses native_stream_case::check, complete ordered streams and
Native replay before API/CLI. Existing frozen enabled/disabled cases unchanged.
No Native output constants stored. This witness adds a different material-category
and support-filament branch; it does not certify all arbitrary category orders.
