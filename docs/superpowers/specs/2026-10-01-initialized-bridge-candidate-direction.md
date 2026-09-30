# Initialized bridge candidate direction rewrite slice

Rewrite boundary: `BridgeDetector.hpp::detect_bridging_direction(Lines, Polygons)`
and its candidate collection dependency, into `geometry/bridge_direction.rs`.
Existing external-surface integration is a temporary compatibility shell, not
an independent Ares pipeline goal. Preserve cost arithmetic and PCA behavior.
The initialized reference links GCC15.3.0 libstdc++; collection dependencies
are `libstdc++-v3/include/bits/{functional_hash,hashtable}.h`,
`libsupc++/hash_bytes.cc`, `src/c++11/hashtable_c++0x.cc` and
`src/shared/hashtable-aux.cc`. ARD-0075 records the actual linked library;
third-party notices and license texts ship with the Rust source rewrite.

Use the agreed project-byte to complete-G-code public seam. An existing small
overhang STL, Ender-3 profile, support disabled and narrow-fill detection disabled
isolates bridge direction from the separately diagnosed concentric branch.
Disable M73 in this minimized case only; original KSR complete output remains
an unmodified separate failing gate. Use the approved initialized reference,
retain native/project/Ares streams, and verify options actually applied.

Exit requires sensitive public RED/GREEN, original KSR bridge diagnostic
revalidation, workspace/rustfmt/clippy/Tier-1 gates, and independent six-axis
review. No new tolerance, output filter, source-token pin or legacy fallback.
Unrelated KSR full-stream failures are not waived by local bridge success.
