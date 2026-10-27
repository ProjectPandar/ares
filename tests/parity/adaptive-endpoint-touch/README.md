# Adaptive endpoint-touch parity input

Original A1/cube-with-hole/adaptivecubic40 discovery archive, copied without
re-export or parameter changes. SHA256 and source boundary are in manifest.json.
The9 ZIP entries are input geometry/configuration only; no embedded G-code or
Native-generated output is committed.

`FillAdaptive.cpp:815-880,939-954,1074-1110` collects endpoint-touch pairs,
excludes them from T-joints, and merges them with source integer midpoints.
The existing public Native replay/validated API/real CLI seam compares complete
ordered bytes, normalizing only the generator line. M73/F/E/XYZ/comments/time/
footer/config echo remain checked; API==CLI raw equality is separate.

Run via `cargo nextest run -p ares-cli --test orca_parity -E
'test(orca_parity_adaptive_endpoint_touch_a1)'` under the canonical official
and seeded environments with CPU18, threads1, persistent shared Native cache
and `ARES_REQUIRE_ORCA=1`. Retained outputs and replay/comparison JSON are the
repeatable external artifacts. See `docs/architecture/ard-adaptive-endpoint-touch.md`.
