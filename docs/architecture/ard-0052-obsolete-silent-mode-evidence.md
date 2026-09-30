# ARD-0052: Prove obsolete silent-mode input normalization

`PrintConfig.cpp:8257-8280::PrintConfigDef::handle_legacy` unconditionally
consumes `silent_mode` as obsolete input. Its definition at 4440-4444
defaults to false. Firmware and machine-limit gates in `GCode.cpp:2210-2217`
and `GCodeProcessor.cpp:2453-2458` do not revive a consumed input.
The existing Rust destination is `ares-core::options::typed_legacy::{obsolete,
project}` and the typed printer default; this slice repairs its CLI
application-evidence shell, not a new time-estimation pipeline.

Both Boolean requests must still be injected into the original preset and
run through original export and full output comparison. Application proof
for this exact key must require the canonical exported false value, preserve
the raw request and effective value, and identify the source-owned obsolete
normalization. Do not count it as activated quiet-mode behavior. A missing,
wrong-shaped or true effective value fails even if it equals the request.
Unrelated options retain their existing equality rules.

This is not a general exemption for value mismatches and not permission to
replace rejected inputs with defaults. The unconditional source rule, the
literal default and live original behavior establish the expected projection.
The full byte comparator, domain cases and completion gate remain unchanged.

Include both Boolean values on the Ender-3 cube and negative proof checks.
Defer the internal enabled stealth estimator, other obsolete keys, direct
raw-3MF legacy combinations, and the older non-project SliceOptions seam.
The existing CLI and core scaffolds remain temporary compatibility shells
around their named upstream boundaries; global parity remains incomplete.
