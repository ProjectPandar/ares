# Template RNG seeded rewrite plan

Boundary and policy: [ARD](../../architecture/ard-template-rng-seeded.md) and
[spec](../specs/2026-10-04-template-rng-seeded.md). This replaces the existing
LCG compatibility shell around upstream PlaceholderParser::ContextData::rng;
no new Ares-owned slicing design.

1. Save paused adapter lint planning (fbbb5367); branch from main be2a2990.
2. Extend seed-only GCode.cpp initialization; private source from exact prior
   replay store snapshot, unchanged build.nix/toolchain. Build and immediately
   report new executable path/SHA. Preserve old producer and official controls.
3. Repeat six immutable inputs with new native seed0 twice and seed1 once;
   retain raw streams and explain all differences. Verify seed0 full ordered
   generator-only identity.
4. Before Rust changes, add full-output vendor E2E and type/distribution/lifecycle
   probes. Failure modes: wrong seed; sharing fuzzy stream; resetting across
   templates; double-as-integer dispatch; signed-range overflow; wrong rejection
   draw; wrong double low/high order; skipped branch consuming RNG; cross-job
   leakage; differing full output despite matching first coordinate. Observe RED.
5. Port exact selected libstdc++ distributions on existing Mt19937; thread
   independent template stream through existing context/config/emission state.
   Remove LCG. Keep modules below400 lines and no new Clippy warnings.
6. GREEN seeds0/1, repeatability and isolation; workspace Nextest, fmt, Clippy,
   wasm32 core+wasm build; full official/seeded coordinator controls no new FAIL.
7. Update coordinator control producer only after validated path exists. Push
   exact candidate, REVIEW-REQUEST and stop for approval; latest-main rebase,
   revalidation and ff-only integration afterward.
