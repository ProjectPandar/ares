# Task 22O.59 candidate boundary polylines implementation plan

## Status

Completed after independent source/specification review and final six-axis validation.

## Objective

Port pinned `PrintObject.cpp:3226-3233` as one private operation consuming O58 output, explicitly including the source empty gate and both ordered boundary expansions while deferring angle/anchor composition from line 3242 onward.

## Plan

1. **Review and oracle**
   - Independently verify ADR/spec/plan against PrintObject, Flow, Polygon rvalue conversion, ClipperUtils, accepted offset kernels, and O48/O58 provenance.
   - Build a removed actual-source driver for empty-gate suppression with invalid scalars/geometry, exact arithmetic bits, ordered closed polyline literals, topology, and error order; record source/archive/object/binary/output/link hashes and repeatability.

2. **Behavioral RED**
   - Register private `mod candidate_boundary_polylines;`, add ordinary production/test modules with a compiling `todo!()` seam, and freeze exact outputs plus complete allocation snapshots.
   - Keep every file at most 399 lines; prohibit `include!`, `include_bytes!`, and `include_str!` splitting.

3. **Minimal implementation**
   - Return `None` on the O58 survivor-empty gate before scalar or geometry work; otherwise compute the f64 total delta, offset once, consume into closed polylines, compute the promoted-f32 limiting delta, offset once, consume, and append by value.
   - Preserve source operation/output value order, consumed offset temporaries, first-error precedence, and borrowed inputs. Exclude final limiting allocation identity from parity; Rust may move rather than reproduce source iterator-copy storage. Add no composer, Flow resolution, angle, anchor, collision, commit, successor, or lifecycle wiring.

4. **Verify and review**
   - Kill gate/arithmetic/role/call-count/ownership/closure/append/error-order mutations and restore byte-exact.
   - Run focused/dependency/Linux-workspace Nextest, rustfmt, strict Clippy, wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS, diff/LOC/static/clean-Orca/no-staged gates.
   - Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis review; repair and re-review until unconditional approval.

## Exit criteria

- Exact empty behavior, delta bits, closed polylines, topology, and append order match pinned actual source.
- Every gate, arithmetic, call, role, temporary consumption, closure, append-value order, and error invariant is mutation-discriminated.
- The operation stays private, portable, lifecycle-neutral, ordinary-module based, and below 400 LOC per file.
- Oracle, mutation, runtime, portability, static, and review gates all pass.

## Completion record

The repeatable actual-source oracle records archive/driver/object/binary/output/link-command SHA-256 values `b643964e681e9435680b78fdd743dcb12c9c07cd16ef164e353d370add8132a1`, `577738d4c8f00879276ac7815afe6d9b0ca80e834fe67899df46cfe30ccbd532`, `a3b74f48e04fb41b61f6694b95b510571bdc78c978ed9d1bfbe79358690b7106`, `9762c5531a462913e3f0476ce42fb2a0ef4d2f8d7e94353b21ed60aa6abb1296`, `9fa3600b73ebe60ac821a2f9811b606aae85ad6f6a1ee797cc4c5ffd62bda320`, and `98f57cf5675fca7419790c1a5b16b5d4aa97ae208cbd6b76692bdc42c262333b`. Nineteen mutations, including explicit ascending output sorting, were killed; audit/source SHA-256 values are `acaea31285ee8e548af408ae65f3b08d9c08966d181140e67283bd7dfd4555d1` and `f803b04ba8db10fc611954883f34eef7cf11674871e9138ec5d2e016d0b4855a`.

Final gates pass focused 10/10, dependency 718/718, workspace 6,378/6,378 with two skipped, strict Clippy, wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS, rustfmt, diff/LOC/static, clean Orca, and no staged files.
