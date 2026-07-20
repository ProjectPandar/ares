# Task 22M Package 6 Encoder Accessor Amendment

## Authority And Compile RED

This amendment supplements the approved Task 22M specification/plan and the
approved Package 6 core-allocation specification/plan (SHA-256
`3ae1edb039b8b25062b069c982c862cc11515d762d6ae9674e20a6d6df3d6f1d` /
`9054fa1da17a2cb0a5295507f854c7bf22b84c3ebdbbf7f510a275bca1710c43`).
Every earlier Package 4/5 contract remains authoritative.

After the approved feature and module gates were applied, the genuine Rust
1.91 non-test M-feature build reached the unchanged M encoder and failed with
E0599 at `project_slice/task22m_oracle.rs:18`. The encoder reads
`PostCompensationPrintObject::as_parts`, but that accessor is still compiled
only under `cfg(test)` in `project_slice/compensation.rs:88-91`.

A complete static audit of the unchanged encoder found no other feature-gated
accessor: post-region, sidecar, region, layer, surface, ExPolygon, Polygon, and
Point reads are already available in normal core builds. This amendment closes
only the one observed compile boundary. It does not authorize changing the M
encoder, wrapper, geometry, wire, Option behavior, or default public API.

## Authorized Delta

Package 6 may additionally modify exactly:

- `crates/ares-core/src/project_slice/compensation.rs`.

Change only the `PostCompensationPrintObject::as_parts` attribute from
`cfg(test)` to
`cfg(any(test, feature = "task22m-browser-oracle"))`. The method body,
visibility, signature, `into_parts`, and all other bytes remain unchanged. Do
not remove the gate or expose the accessor in default non-test builds.

The file is frozen before this repair at 268 LOC / SHA-256
`45d12c34292b765e23d599904afaefcfe66c24fdae5fa3543f84f6bafe10d208`.
It already belongs to the original Task 22M manifest. This specification and
its companion plan are the only new paths, replacing the approved exact
66-path frame with an exact 68-path frame.

## Acceptance

The E0599 Rust 1.91 M-feature build is the RED. GREEN requires:

- core and adapter default/M-feature wasm32 checks pass;
- the old L feature remains unknown;
- default bindings expose no Task 22 hook and M bindings expose exactly two;
- Task 22M remains 81/81 and Task 22L remains 53/53;
- synthetic and KSR L/M checkpoint identities remain exact;
- strict clippy, rustfmt, LOC, macro/unsafe, stale-L, and diff gates pass; and
- the complete Package 6 browser matrix passes twice.

No new test is required because this is a non-test feature compilation seam
whose direct RED is the compiler error. Any additional gated accessor, changed
checkpoint byte, or wider `compensation.rs` edit requires another approved
amendment.
